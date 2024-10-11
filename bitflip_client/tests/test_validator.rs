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
use bitflip_client::set_bits_request;
use bitflip_client::start_bits_session_request;
use bitflip_program::BITS_DATA_SECTION_LENGTH;
use bitflip_program::BITS_DATA_SECTIONS;
use bitflip_program::BitsDataSectionState;
use bitflip_program::BitsMetaState;
use bitflip_program::ConfigState;
use bitflip_program::InitializeConfigProps;
use bitflip_program::SetBitsVariant;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::accounts::InitializeConfig;
use futures::future::try_join;
use futures::future::try_join_all;
use shared::IntoAccountSharedData;
use shared::TestBitflipProgramClient;
use shared::create_admin_keypair;
use shared::create_authority_keypair;
use shared::create_bits_meta_state;
use shared::create_bits_section_state;
use shared::create_config_state;
use shared::create_runner;
use shared::create_runner_with_accounts;
use shared::create_section_bumps;
use shared::get_admin_program;
use solana_sdk::account::ReadableAccount;
use solana_sdk::clock::Clock;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use test_log::test;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wallet_standard_wallets::MemoryWallet;
use wasm_client_solana::solana_account_decoder::UiAccount;
use wasm_client_solana::solana_account_decoder::UiAccountEncoding;
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
	let program_client: TestBitflipProgramClient = BitflipProgramClient::builder()
		.rpc(rpc.clone())
		.wallet(wallet)
		.build()
		.into();

	let request = program_client
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

	let simulation = request.simulate_transaction().await?;

	log::info!("units_consumed: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 75_000);

	request.sign_and_send_transaction().await?;

	let config_state_account = program_client.account::<ConfigState>(&config).await?;

	let authority_redaction = create_insta_redaction(authority, "authority:pubkey");
	insta::assert_yaml_snapshot!(config_state_account,{
		".authority" => insta::dynamic_redaction(authority_redaction),
	}, @r#"
 authority: "[authority:pubkey]"
 lamportsPerBit: 100000
 bump: 254
 treasuryBump: 255
 mintBump: 0
 bitsIndex: 0
 "#);

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
	let program_client: TestBitflipProgramClient = BitflipProgramClient::builder()
		.rpc(rpc.clone())
		.wallet(wallet)
		.build()
		.into();

	let request = initialize_token_request(&program_client, &authority_keypair);

	let simulation = request.simulate_transaction().await?;

	log::info!("units_consumed: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 200_000);

	let signature = request.sign_and_send_transaction().await?;
	rpc.confirm_transaction(&signature).await?;

	let authority_redaction = create_insta_redaction(treasury, "authority:pubkey");
	let mint_redaction = create_insta_redaction(mint, "mint:pubkey");
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
	  ".info.extensions[2].state.updateAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[2].state.mint" =>	insta::dynamic_redaction(mint_redaction.clone())
	}, @r#"
 type: mint
 info:
   mintAuthority: "[authority:pubkey]"
   supply: "1000000000000000"
   decimals: 6
   isInitialized: true
   freezeAuthority: "[authority:pubkey]"
   extensions:
     - extension: metadataPointer
       state:
         authority: "[authority:pubkey]"
         metadataAddress: "[mint:pubkey]"
     - extension: mintCloseAuthority
       state:
         closeAuthority: "[authority:pubkey]"
     - extension: tokenMetadata
       state:
         updateAuthority: "[authority:pubkey]"
         mint: "[mint:pubkey]"
         name: Bitflip
         symbol: BITFLIP
         uri: "https://bitflip.art/token.json"
         additionalMetadata: []
 "#);

	let treasury_token_account_data = rpc.get_account_data(&treasury_token_account).await?;
	let token_account = parse_token_v2(
		&treasury_token_account_data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("token account: {token_account:#?}");
	insta::assert_yaml_snapshot!(token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction),
		".info.owner" => insta::dynamic_redaction(authority_redaction),
	}, @r#"
 type: account
 info:
   mint: "[mint:pubkey]"
   owner: "[authority:pubkey]"
   tokenAmount:
     uiAmount: 1000000000
     decimals: 6
     amount: "1000000000000000"
     uiAmountString: "1000000000"
   state: initialized
   isNative: false
   extensions:
     - extension: immutableOwner
 "#);

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
	let program_client = get_admin_program(rpc);

	let request = program_client
		.initialize_bits_meta()
		.accounts(bitflip_program::accounts::InitializeBitsMeta {
			config,
			bits_meta,
			authority,
			system_program,
		})
		.signers(vec![&authority_keypair])
		.build();

	let simulation = request.simulate_transaction().await?;
	log::info!("simulation: {simulation:#?}");

	request.sign_and_send_transaction().await?;

	let bits_meta_account: BitsMetaState = program_client.account(&bits_meta).await?;

	insta::assert_yaml_snapshot!(bits_meta_account, @r#"
 startTime: 0
 flips: 0
 "on": 0
 "off": 1048576
 index: 0
 bump: 254
 sectionBumps: []
 "#);

	Ok(())
}

#[test(tokio::test)]
async fn initialize_bits_data_sections() -> anyhow::Result<()> {
	let game_index = 0;
	let (config, _) = get_pda_config();
	let authority_keypair = create_authority_keypair();
	let (bits_meta, _) = get_pda_bits_meta(game_index);
	let mut accounts = HashMap::new();
	accounts.insert(config, create_config_state(None));
	accounts.insert(bits_meta, create_bits_meta_state(game_index, None));

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_admin_program(rpc);
	let program_client_ref = &program_client;

	let requests =
		initialize_bits_data_sections_request(&program_client, &authority_keypair, game_index)
			.await?;

	for request in &requests {
		request.sign_and_send_transaction().await?;
	}

	let range = 0u8..16u8;
	let futures = range.into_iter().map(|section| {
		async move {
			let pubkey = get_pda_bits_data_section(game_index, section).0;
			program_client_ref
				.account::<BitsDataSectionState>(&pubkey)
				.await
		}
	});
	let accounts = try_join_all(futures).await?;
	let bits_meta_account: BitsMetaState = program_client.account(&bits_meta).await?;

	check!(bits_meta_account.section_bumps.len() == BITS_DATA_SECTIONS);

	for (section, account) in accounts.iter().enumerate() {
		let bump = get_pda_bits_data_section(game_index, section as u8).1;
		check!(
			bits_meta_account
				.section_bumps
				.get(section)
				.copied()
				.unwrap() == bump,
			"check bump for section:  {section}"
		);
		let expected_data = [0u16; BITS_DATA_SECTION_LENGTH];
		check!(
			&account.data == &expected_data,
			"check data length for section:  {section}"
		);
	}

	Ok(())
}

#[test(tokio::test)]
async fn start_bits_session() -> anyhow::Result<()> {
	let (config, _) = get_pda_config();

	let authority_keypair = create_authority_keypair();
	let game_index = 0;
	let (bits_meta, _) = get_pda_bits_meta(game_index);
	let mut accounts = HashMap::new();
	accounts.insert(config, create_config_state(None));
	accounts.insert(bits_meta, create_bits_meta_state(game_index, Some(16)));

	for (pubkey, account) in create_bits_section_state(game_index) {
		accounts.insert(pubkey, account);
	}

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_admin_program(rpc);
	initialize_token_request(&program_client, &authority_keypair)
		.sign_and_send_transaction()
		.await?;

	let request = start_bits_session_request(&program_client, &authority_keypair, game_index);
	let simulation = request.simulate_transaction().await?;
	log::info!("simulation: {simulation:#?}");
	request.sign_and_send_transaction().await?;

	let bits_meta_account: BitsMetaState = program_client.account(&bits_meta).await?;
	insta::assert_yaml_snapshot!(bits_meta_account, {
		".startTime" => "[timestamp]"
	}, @r#"
 startTime: "[timestamp]"
 flips: 0
 "on": 0
 "off": 1048576
 index: 0
 bump: 254
 sectionBumps:
   - 252
   - 253
   - 254
   - 254
   - 252
   - 252
   - 255
   - 255
   - 255
   - 253
   - 255
   - 255
   - 254
   - 250
   - 253
   - 255
 "#);
	check!(bits_meta_account.started());

	Ok(())
}

#[test(tokio::test)]
async fn set_bits_on() -> anyhow::Result<()> {
	let (config, _) = get_pda_config();
	let (mint, mint_bump) = get_pda_mint();
	let authority_keypair = create_authority_keypair();
	let (bits_meta, bits_meta_bump) = get_pda_bits_meta(0);
	let game_index = 0;
	let section = 0;
	let index = 0;
	let mut accounts = HashMap::new();
	accounts.insert(config, create_config_state(Some(mint_bump)));
	let bits_meta_state = BitsMetaState::builder()
		.start_time(Clock::default().unix_timestamp)
		.index(game_index)
		.section_bumps(create_section_bumps(game_index))
		.bump(bits_meta_bump)
		.build();
	accounts.insert(bits_meta, bits_meta_state.into_account_shared_data());

	for (pubkey, account) in create_bits_section_state(game_index) {
		accounts.insert(pubkey, account);
	}

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_admin_program(rpc);
	let bits_data_section = get_pda_bits_data_section(game_index, section).0;
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let player = program_client.payer();
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);
	let player_token_account =
		get_associated_token_address_with_program_id(&player, &mint, &token_program);

	let program_client_ref = &program_client;
	let authority_keypair_ref = &authority_keypair;
	let pair = try_join(
		async move {
			let signature = initialize_token_request(program_client_ref, authority_keypair_ref)
				.sign_and_send_transaction()
				.await?;
			rpc.confirm_transaction(&signature).await?;
			anyhow::Ok(())
		},
		async move {
			let signature =
				start_bits_session_request(program_client_ref, authority_keypair_ref, game_index)
					.sign_and_send_transaction()
					.await?;
			rpc.confirm_transaction(&signature).await?;
			anyhow::Ok(())
		},
	);

	pair.await?;
	//

	let offset = 12;
	let bits = 1 << offset;
	let request = set_bits_request(
		&program_client,
		game_index,
		section,
		index,
		SetBitsVariant::On(offset),
	);
	let simulation = request.simulate_transaction().await?;
	log::info!("simulation: {simulation:#?}");
	let signature = request.sign_and_send_transaction().await?;
	rpc.confirm_transaction(&signature).await?;

	let account: BitsDataSectionState = program_client.account(&bits_data_section).await?;
	check!(account.data[0] == bits);

	let player_redaction = create_insta_redaction(player, "player:pubkey");
	let mint_redaction = create_insta_redaction(mint, "mint:pubkey");
	let treasury_redaction = create_insta_redaction(treasury, "treasury:pubkey");
	let raw_player_token_account = rpc.get_account(&player_token_account).await?;
	let parsed_player_token_account = parse_token_v2(
		raw_player_token_account.data(),
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;
	log::info!("player token account: {parsed_player_token_account:#?}");
	insta::assert_yaml_snapshot!(parsed_player_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction.clone()),
		".info.owner" => insta::dynamic_redaction(player_redaction),
	}, @r#"
 type: account
 info:
   mint: "[mint:pubkey]"
   owner: "[player:pubkey]"
   tokenAmount:
     uiAmount: 1
     decimals: 6
     amount: "1000000"
     uiAmountString: "1"
   state: initialized
   isNative: false
   extensions:
     - extension: immutableOwner
 "#);

	let raw_treasury_token_account = rpc.get_account(&treasury_token_account).await?;
	let parsed_treasury_token_account = parse_token_v2(
		raw_treasury_token_account.data(),
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;
	log::info!("treasury token account: {parsed_treasury_token_account:#?}");
	insta::assert_yaml_snapshot!(parsed_treasury_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction),
		".info.owner" => insta::dynamic_redaction(treasury_redaction),
	}, @r#"
 type: account
 info:
   mint: "[mint:pubkey]"
   owner: "[treasury:pubkey]"
   tokenAmount:
     uiAmount: 999999999
     decimals: 6
     amount: "999999999000000"
     uiAmountString: "999999999"
   state: initialized
   isNative: false
   extensions:
     - extension: immutableOwner
 "#);

	let bits_meta_account: BitsMetaState = program_client.account(&bits_meta).await?;
	check!(bits_meta_account.started());
	insta::assert_yaml_snapshot!(bits_meta_account, {
		".startTime" => "[timestamp]"
	}, @r#"
 startTime: "[timestamp]"
 flips: 1
 "on": 1
 "off": 1048575
 index: 0
 bump: 254
 sectionBumps:
   - 252
   - 253
   - 254
   - 254
   - 252
   - 252
   - 255
   - 255
   - 255
   - 253
   - 255
   - 255
   - 254
   - 250
   - 253
   - 255
 "#);

	let treasury_account = rpc.get_account(&treasury).await?;
	let treasury_ui_account = UiAccount::encode(
		&treasury,
		&treasury_account,
		UiAccountEncoding::Base64,
		None,
		None,
	);
	log::info!("treasury account: {treasury_ui_account:#?}");
	insta::assert_yaml_snapshot!(treasury_ui_account, @r#"
 lamports: 990880
 data:
   - ""
   - base64
 owner: "11111111111111111111111111111111"
 executable: false
 rentEpoch: 18446744073709551615
 space: 0
 "#);

	Ok(())
}
