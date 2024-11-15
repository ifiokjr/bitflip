use anchor_spl::token_2022;
use bitflip_legacy_program::FlipBitsProps;
use bitflip_legacy_program::ID_CONST;
use bitflip_legacy_program::InitializeTokenProps;
use bitflip_legacy_program::SetBitsVariant;
use bitflip_legacy_program::accounts;
use bitflip_legacy_program::accounts::InitializeToken;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::hash;
use solana_sdk::nonce;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
#[cfg(not(feature = "test_banks_client"))]
use solana_sdk::system_instruction::advance_nonce_account;
use solana_sdk::system_program;
use solana_sdk::sysvar;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use wallet_standard::prelude::*;
use wasm_client_anchor::AnchorClientError;
use wasm_client_anchor::AnchorClientResult;
use wasm_client_anchor::create_program_client;
use wasm_client_anchor::create_program_client_macro;
use wasm_client_anchor::prelude::*;
use wasm_client_solana::SolanaRpcClient;
use wasm_client_solana::nonce_utils;

use crate::get_derived_player_token_account;
use crate::get_pda_config;
use crate::get_pda_derived_player;
use crate::get_pda_game;
use crate::get_pda_game_nonce;
use crate::get_pda_mint;
use crate::get_pda_section_data;
use crate::get_pda_section_state;
use crate::get_pda_treasury;
use crate::get_player_token_account;
use crate::get_section_token_account;
use crate::get_treasury_token_account;

create_program_client!(ID_CONST, BitflipLegacyProgramClient);
create_program_client_macro!(bitflip_legacy_program, BitflipLegacyProgramClient);

bitflip_legacy_program_client_request_builder!(InitializeConfig, "optional:args");
bitflip_legacy_program_client_request_builder!(InitializeToken);
bitflip_legacy_program_client_request_builder!(InitializeGame, "optional:args");
bitflip_legacy_program_client_request_builder!(StartGame, "optional:args");
bitflip_legacy_program_client_request_builder!(UnlockSection, "optional:args");
bitflip_legacy_program_client_request_builder!(FlipBits);
bitflip_legacy_program_client_request_builder!(CreateDerivedPlayer, "optional:args");
bitflip_legacy_program_client_request_builder!(UpdateAuthority, "optional:args");
bitflip_legacy_program_client_request_builder!(RefreshAccessSigner, "optional:args");

/// Initialize the config for this program.
///
/// The program client must use the `authority` as the payer. The `admin` will
/// need to be added as a signer to the generated transaction.
pub fn initialize_config<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
	admin: Pubkey,
) -> InitializeConfigRequest<'_, W> {
	let config = get_pda_config().0;
	let treasury = get_pda_treasury().0;
	let authority = program_client.payer();
	let system_program = system_program::ID;

	let request = program_client
		.initialize_config()
		.accounts(accounts::InitializeConfig {
			config,
			admin,
			treasury,
			authority,
			system_program,
		})
		.build();

	request
}

/// Initialize the token request.
///
/// The payer is the authority and is provided by the program client.
pub fn initialize_token_request<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
) -> InitializeTokenRequest<'_, W> {
	let authority = program_client.payer();
	let (config, _) = get_pda_config();
	let (mint, _) = get_pda_mint();
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);
	let associated_token_program = spl_associated_token_account::ID;
	let system_program = system_program::ID;
	let bitflip_legacy_program = ID_CONST;

	program_client
		.initialize_token()
		.args(InitializeTokenProps {
			name: "Bitflip".into(),
			symbol: "BIT".into(),
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
		})
		.build()
}

pub fn initialize_game_request<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
	access_signer: Pubkey,
	refresh_signer: Pubkey,
	game_index: u8,
) -> InitializeGameRequest<'_, W> {
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let game_nonce = get_pda_game_nonce(game_index).0;
	let authority = program_client.payer();
	let recent_blockhashes = sysvar::recent_blockhashes::ID;
	let rent = sysvar::rent::ID;
	let system_program = system_program::ID;

	program_client
		.initialize_game()
		.accounts(accounts::InitializeGame {
			config,
			game,
			game_nonce,
			authority,
			access_signer,
			refresh_signer,
			recent_blockhashes,
			rent,
			system_program,
		})
		.build()
}
/// Unlock a section using a `DurableNonce` tranaction.
///
/// The request that is returned from this method still needs to be signed by
/// the backend wih the `access_signer`.
pub fn unlock_section_request<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
	access_signer: Pubkey,
	game_index: u8,
	section_index: u8,
	_nonce_blockhash: hash::Hash,
) -> UnlockSectionRequest<'_, W> {
	let config = get_pda_config().0;
	let mint = get_pda_mint().0;
	let treasury = get_pda_treasury().0;
	let treasury_token_account = get_treasury_token_account();
	let player = program_client.payer();
	let player_token_account = get_player_token_account(&player);
	let associated_token_program = spl_associated_token_account::ID;
	let token_program = token_2022::ID;
	#[cfg(not(feature = "test_banks_client"))]
	let game_nonce = get_pda_game_nonce(game_index).0;
	let game = get_pda_game(0).0;
	let system_program = system_program::ID;
	let section = get_pda_section_state(game_index, section_index).0;
	let section_data = get_pda_section_data(game_index, section_index).0;
	let section_token_account = get_section_token_account(game_index, section_index);
	log::info!("game_index: {game_index}, section_index: {section_index}");
	let previous_section =
		{ (section_index > 0).then(|| get_pda_section_state(game_index, section_index - 1).0) };
	#[cfg(not(feature = "test_banks_client"))]
	let advance_nonce_instruction = advance_nonce_account(&game_nonce, &access_signer);
	let request = program_client.unlock_section();
	#[cfg(not(feature = "test_banks_client"))]
	let request = request
		.blockhash(_nonce_blockhash)
		.instruction(advance_nonce_instruction);
	let request = request
		.accounts(accounts::UnlockSection {
			config,
			mint,
			treasury,
			treasury_token_account,
			game,
			section,
			section_data,
			section_token_account,
			previous_section,
			player,
			player_token_account,
			associated_token_program,
			token_program,
			system_program,
			access_signer,
		})
		.build();

	request
}

pub async fn get_game_nonce_account(
	rpc: &SolanaRpcClient,
	game_index: u8,
) -> AnchorClientResult<nonce::state::Data> {
	let game_nonce = get_pda_game_nonce(game_index).0;
	let nonce_account = nonce_utils::get_account(rpc, &game_nonce)
		.await
		.map_err(|e| AnchorClientError::Custom(e.to_string()))?;
	let nonce_data = nonce_utils::data_from_account(&nonce_account)
		.map_err(|e| AnchorClientError::Custom(e.to_string()))?;

	Ok(nonce_data)
}

/// Start the game using the `authority` as the payer for the transaction.
///
/// The request will still need to be signed on the backend by the
/// `access_signer`.
pub fn start_game_request<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
	access_signer: Pubkey,
	game_index: u8,
) -> StartGameRequest<'_, W> {
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let authority = program_client.payer();
	let system_program = system_program::ID;

	program_client
		.start_game()
		.accounts(accounts::StartGame {
			config,
			game,
			authority,
			access_signer,
			system_program,
		})
		.build()
}

pub fn flip_bits_request<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
	game_index: u8,
	section_index: u8,
	array_index: u16,
	variant: SetBitsVariant,
) -> FlipBitsRequest<'_, W> {
	let (config, _) = get_pda_config();
	let player = program_client.wallet().solana_pubkey();
	let game = get_pda_game(game_index).0;
	let mint = get_pda_mint().0;
	let section = get_pda_section_state(game_index, section_index).0;
	let section_data = get_pda_section_data(game_index, section_index).0;
	let section_token_account = get_section_token_account(game_index, section_index);
	let token_program = token_2022::ID;
	let associated_token_program = spl_associated_token_account::ID;
	let player_token_account = get_player_token_account(&player);
	let system_program = system_program::ID;
	let instruction = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);

	log::info!(
		"\nconfig: {config}
player: {player}
mint: {mint}
game: {game}
section: {section}
section_token_account: {section_token_account}
player_token_account: {player_token_account}"
	);

	program_client
		.flip_bits()
		.instruction(instruction)
		.args(FlipBitsProps {
			section_index,
			array_index,
			variant,
		})
		.accounts(accounts::FlipBits {
			config,
			mint,
			game,
			section,
			section_data,
			section_token_account,
			player,
			player_token_account,
			associated_token_program,
			token_program,
			system_program,
		})
		.build()
}

/// The player should be the wallet's pubkey included in the `program_client`.
pub fn create_derived_player_request<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
) -> CreateDerivedPlayerRequest<'_, W> {
	let config = get_pda_config().0;
	let mint = get_pda_mint().0;
	let player = program_client.wallet().solana_pubkey();
	let derived_player = get_pda_derived_player(&player).0;
	let derived_player_token_account = get_derived_player_token_account(&derived_player);
	let associated_token_program = spl_associated_token_account::ID;
	let token_program = token_2022::ID;
	let system_program = system_program::ID;

	program_client
		.create_derived_player()
		.accounts(accounts::CreateDerivedPlayer {
			config,
			mint,
			derived_player,
			derived_player_token_account,
			player,
			associated_token_program,
			token_program,
			system_program,
		})
		.build()
}

/// Refresh the access signer for the game.
///
/// The refresh signer must be included as a signer in the transaction.
pub fn refresh_access_signer_request<'a, W: WalletAnchor>(
	program_client: &'a BitflipLegacyProgramClient<W>,
	game_index: u8,
	access_signer_keypair: &'a Keypair,
) -> RefreshAccessSignerRequest<'a, W> {
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let refresh_signer = program_client.payer();
	let access_signer = access_signer_keypair.pubkey();

	program_client
		.refresh_access_signer()
		.signer(access_signer_keypair)
		.accounts(accounts::RefreshAccessSigner {
			config,
			game,
			access_signer,
			refresh_signer,
		})
		.build()
}

/// Update the authority of the config. The new admin must be included as a
/// signer in the transaction when submitted. Submitting the transaction without
/// manually signing will not succeed.
pub fn update_authority_request<W: WalletAnchor>(
	program_client: &BitflipLegacyProgramClient<W>,
	new_authority: Pubkey,
) -> UpdateAuthorityRequest<'_, W> {
	let config = get_pda_config().0;
	let authority = program_client.payer();

	program_client
		.update_authority()
		.accounts(accounts::UpdateAuthority {
			config,
			authority,
			new_authority,
		})
		.build()
}
