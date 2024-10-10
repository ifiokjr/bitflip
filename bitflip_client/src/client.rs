use std::ops::Div;
use std::ops::Mul;

use anchor_spl::token_2022;
use bitflip_program::ID_CONST;
use bitflip_program::InitializeTokenProps;
use bitflip_program::SetBitsProps;
use bitflip_program::SetBitsVariant;
use bitflip_program::accounts;
use bitflip_program::accounts::InitializeToken;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::signature::Keypair;
use solana_sdk::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use wallet_standard::prelude::*;
use wasm_client_anchor::AnchorClientError;
use wasm_client_anchor::AnchorClientResult;
use wasm_client_anchor::EmptyAnchorRequest;
use wasm_client_anchor::create_program_client;
use wasm_client_anchor::create_program_client_macro;
use wasm_client_anchor::prelude::*;

use crate::get_pda_bits_data_section;
use crate::get_pda_bits_meta;
use crate::get_pda_config;
use crate::get_pda_mint;
use crate::get_pda_treasury;

create_program_client!(ID_CONST, BitflipProgramClient);
create_program_client_macro!(bitflip_program, BitflipProgramClient);

bitflip_program_client_request_builder!(InitializeConfig);
bitflip_program_client_request_builder!(InitializeToken);
bitflip_program_client_request_builder!(InitializeBitsMeta, "optional:args");
bitflip_program_client_request_builder!(InitializeBitsDataSection);
bitflip_program_client_request_builder!(StartBitsSession);
bitflip_program_client_request_builder!(SetBits);

/// Initialize the token request.
pub fn initialize_token_request<'a, W: WalletAnchor>(
	program_client: &'a BitflipProgramClient<W>,
	signer: &'a Keypair,
) -> InitializeTokenRequest<'a, W> {
	let authority = signer.pubkey();
	let (config, _) = get_pda_config();
	let (mint, _) = get_pda_mint();
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);
	let associated_token_program = spl_associated_token_account::ID;
	let system_program = system_program::ID;
	let bitflip_program = ID_CONST;

	program_client
		.initialize_token()
		.args(InitializeTokenProps {
			name: "Bitflip".into(),
			symbol: "BITFLIP".into(),
			uri: "https://bitflip.art/token.json".into(),
		})
		.accounts(InitializeToken {
			config,
			authority,
			mint,
			treasury,
			treasury_token_account,
			associated_token_program,
			token_program,
			system_program,
			bitflip_program,
		})
		.signer(signer)
		.build()
}

/// Initialize all the sections for the bitflip game.
pub async fn initialize_bits_data_sections_request<'a, W: WalletAnchor>(
	program_client: &'a BitflipProgramClient<W>,
	signer: &'a Keypair,
	game_index: u8,
) -> AnchorClientResult<Vec<EmptyAnchorRequest<'a, W>>> {
	let (config, _) = get_pda_config();
	let (bits_meta, _) = get_pda_bits_meta(0);
	let system_program = system_program::ID;

	let create_request = move |section: u8| {
		let bits_data_section = get_pda_bits_data_section(game_index, section).0;

		program_client
			.initialize_bits_data_section()
			.args(section)
			.accounts(accounts::InitializeBitsDataSection {
				config,
				bits_meta,
				bits_data_section,
				authority: signer.pubkey(),
				system_program,
			})
			.signer(signer)
			.build()
	};

	let Some(compute_units) = create_request(0)
		.simulate_transaction()
		.await?
		.value
		.units_consumed
		.map(|v| v.div(100).mul(110) as u32)
	else {
		return Err(AnchorClientError::Custom(
			"Could not simulate the transaction".into(),
		));
	};

	log::info!("These are the calculated computed units: {compute_units}");

	let mut requests = vec![];
	let steps = MAX_COMPUTE_UNIT_LIMIT / compute_units;

	for chunk in range_chunks(0..16, steps as usize) {
		log::info!("creating sections {chunk:#?} with len: {}", chunk.len());
		let mut instructions = vec![];
		let compute_limit_instruction =
			ComputeBudgetInstruction::set_compute_unit_limit(compute_units.mul(chunk.len() as u32));

		instructions.push(compute_limit_instruction);

		for section in chunk {
			instructions.append(&mut create_request(section as u8).instructions());
		}

		let request = program_client
			.empty_request()
			.instructions(instructions)
			.sync_signers(vec![signer])
			.build();

		requests.push(request);
	}

	Ok(requests)
}

pub fn set_bits_request<W: WalletAnchor>(
	program_client: &BitflipProgramClient<W>,
	game_index: u8,
	section: u8,
	index: u16,
	variant: SetBitsVariant,
) -> SetBitsRequest<'_, W> {
	let (config, _) = get_pda_config();
	let player = WalletSolanaPubkey::pubkey(program_client.wallet());

	let (bits_meta, _) = get_pda_bits_meta(0);
	let (mint, _) = get_pda_mint();
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);
	let associated_token_program = spl_associated_token_account::ID;
	let player_token_account =
		get_associated_token_address_with_program_id(&player, &mint, &token_program);
	let system_program = system_program::ID;

	program_client
		.set_bits()
		.args(SetBitsProps {
			section,
			index,
			variant,
		})
		.accounts(accounts::SetBits {
			config,
			mint,
			treasury,
			treasury_token_account,
			bits_meta,
			bits_data_section: get_pda_bits_data_section(game_index, section).0,
			player,
			player_token_account,
			associated_token_program,
			token_program,
			system_program,
		})
		.build()
}

pub const MAX_COMPUTE_UNIT_LIMIT: u32 = 1_400_000;
pub(crate) struct RangeChunks {
	start: usize,
	end: usize,
	chunk_size: usize,
}

impl Iterator for RangeChunks {
	type Item = std::ops::Range<usize>;

	fn next(&mut self) -> Option<Self::Item> {
		if self.start >= self.end {
			None
		} else {
			let chunk_end = (self.start + self.chunk_size).min(self.end);
			let chunk = self.start..chunk_end;
			self.start = chunk_end;
			Some(chunk)
		}
	}
}

fn range_chunks(range: std::ops::Range<usize>, chunk_size: usize) -> RangeChunks {
	RangeChunks {
		start: range.start,
		end: range.end,
		chunk_size,
	}
}
