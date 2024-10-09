#![cfg(feature = "test_validator")]
use std::collections::HashMap;

use anchor_spl::token_2022;
use assert2::check;
use bitflip_client::BitflipProgramClient;
use bitflip_client::get_pda_bits_data_section;
use bitflip_client::get_pda_bits_meta;
use bitflip_client::get_pda_config;
use bitflip_client::get_pda_mint;
use bitflip_client::get_pda_treasury;
use bitflip_client::initialize_bits_data_sections_request;
use bitflip_client::initialize_token_request;
use bitflip_program::BITS_DATA_SECTION_LENGTH;
use bitflip_program::BitsDataSectionState;
use bitflip_program::BitsMetaState;
use bitflip_program::ConfigState;
use bitflip_program::ID_CONST;
use bitflip_program::InitializeConfigProps;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::accounts::InitializeConfig;
use futures::future::try_join_all;
use shared::TestBitflipProgramClient;
use shared::create_admin_keypair;
use shared::create_authority_keypair;
use shared::create_bits_meta_state;
use shared::create_config_state;
use shared::create_runner;
use shared::create_runner_with_accounts;
use shared::get_admin_program;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use test_log::test;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wallet_standard_wallets::MemoryWallet;
use wasm_client_solana::solana_account_decoder::parse_account_data::SplTokenAdditionalData;
use wasm_client_solana::solana_account_decoder::parse_token::parse_token_v2;

mod shared;

#[test(tokio::test)]
async fn initialize_config_test() -> anyhow::Result<()> {
	let admin_keypair = create_admin_keypair();
	let admin = admin_keypair.pubkey();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let system_program = system_program::id();
	let (config, _) = get_pda_config();
	let (treasury, _) = get_pda_treasury();
	let runner = create_runner().await;
	let rpc = runner.rpc();
	let wallet = MemoryWallet::new(rpc.clone(), &[admin_keypair.insecure_clone()]);
	let bitflip_program_client: TestBitflipProgramClient = BitflipProgramClient::builder()
		.rpc(rpc.clone())
		.wallet(wallet)
		.build()
		.into();

	let initialize_config_request = bitflip_program_client
		.initialize_config()
		.args(InitializeConfigProps { authority })
		.accounts(InitializeConfig {
			config,
			admin,
			treasury,
			system_program,
		})
		.signers(vec![&admin_keypair])
		.build();

	let simulation = initialize_config_request.simulate_transaction().await?;

	log::info!("units_consumed: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 75_000);

	initialize_config_request
		.sign_and_send_transaction()
		.await?;

	let config_state_account = bitflip_program_client
		.account::<ConfigState>(&config)
		.await?;

	insta::assert_yaml_snapshot!(config_state_account);

	Ok(())
}

#[test(tokio::test)]
async fn initialize_token() -> anyhow::Result<()> {
	let mut accounts = HashMap::new();
	let admin_keypair = create_admin_keypair();
	let (config, _) = get_pda_config();
	let authority_keypair = create_authority_keypair();
	let (mint, _) = get_pda_mint();
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);

	accounts.insert(config, create_config_state(None));

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();

	let wallet = MemoryWallet::new(rpc.clone(), &[admin_keypair.insecure_clone()]);
	let bitflip_program_client: TestBitflipProgramClient = BitflipProgramClient::builder()
		.rpc(rpc.clone())
		.wallet(wallet)
		.build()
		.into();

	let request = initialize_token_request(&bitflip_program_client, &authority_keypair);

	let simulation = request.simulate_transaction().await?;

	log::info!("units_consumed: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 200_000);

	let signature = request.sign_and_send_transaction().await?;
	rpc.confirm_transaction(&signature).await?;

	let authority_redaction = create_insta_redaction(treasury, "authority:pubkey");
	let mint_redaction = create_insta_redaction(mint, "mint:pubkey");
	let program_redaction = create_insta_redaction(ID_CONST, "program:pubkey");
	let mint_data = rpc.get_account_data(&mint).await?;
	let mint_account = parse_token_v2(
		&mint_data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("mint account: {mint_account:#?}");

	insta::assert_yaml_snapshot!(mint_account, {
		".info.mintAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.freezeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.metadataAddress" =>	insta::dynamic_redaction(mint_redaction.clone()),
	  ".info.extensions[1].state.closeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[2].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[2].state.programId" => insta::dynamic_redaction(program_redaction),
		".info.extensions[3].state.delegate" =>	insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[4].state.updateAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[4].state.mint" =>	insta::dynamic_redaction(mint_redaction.clone())
	});

	let treasury_token_account_data = rpc.get_account_data(&treasury_token_account).await?;
	let token_account = parse_token_v2(
		&treasury_token_account_data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("token account: {token_account:#?}");
	insta::assert_yaml_snapshot!(token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction),
		".info.owner" => insta::dynamic_redaction(authority_redaction),
	});

	Ok(())
}

#[test(tokio::test)]
async fn initialize_bits_meta() -> anyhow::Result<()> {
	let mut accounts = HashMap::new();
	let (config, _) = get_pda_config();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let (bits_meta, _) = get_pda_bits_meta(0);
	let system_program = system_program::ID;

	accounts.insert(config, create_config_state(None));

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let bitflip_program_client = get_admin_program(rpc);

	let initialize_bits_meta_request = bitflip_program_client
		.initialize_bits_meta()
		.accounts(bitflip_program::accounts::InitializeBitsMeta {
			config,
			bits_meta,
			authority,
			system_program,
		})
		.signers(vec![&authority_keypair])
		.build();

	let simulation = initialize_bits_meta_request.simulate_transaction().await?;
	log::info!("simulation: {simulation:#?}");

	initialize_bits_meta_request
		.sign_and_send_transaction()
		.await?;

	let bits_meta_account: BitsMetaState = bitflip_program_client.account(&bits_meta).await?;

	insta::assert_yaml_snapshot!(bits_meta_account);

	Ok(())
}

#[test(tokio::test)]
async fn initialize_bits_data_sections() -> anyhow::Result<()> {
	let index = 0;
	let (config, _) = get_pda_config();
	let mut accounts = HashMap::new();
	let authority_keypair = create_authority_keypair();
	let (bits_meta, _) = get_pda_bits_meta(index);
	accounts.insert(config, create_config_state(None));
	accounts.insert(bits_meta, create_bits_meta_state(index));

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_admin_program(rpc);
	let program_client_ref = &program_client;

	let requests =
		initialize_bits_data_sections_request(&program_client, &authority_keypair, index).await?;

	for request in &requests {
		request.sign_and_send_transaction().await?;
	}

	let range = 0u8..16u8;
	let futures = range.into_iter().map(|section| {
		async move {
			let pubkey = get_pda_bits_data_section(index, section).0;
			program_client_ref
				.account::<BitsDataSectionState>(&pubkey)
				.await
		}
	});
	let accounts = try_join_all(futures).await?;

	for (section, account) in accounts.iter().enumerate() {
		let bump = get_pda_bits_data_section(index, section as u8).1;
		check!(
			account.data.len() == BITS_DATA_SECTION_LENGTH,
			"check data length for section:  {section}"
		);
		check!(
			account.section == section as u8,
			"check section:  {section}"
		);
		check!(account.bump == bump, "check bump for section:  {section}");
	}

	Ok(())
}
