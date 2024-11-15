#![cfg(feature = "test_validator")]
use std::collections::HashMap;
use std::time::SystemTime;

use anchor_spl::token_2022;
use assert2::check;
use bitflip_legacy_client::flip_bits_request;
use bitflip_legacy_client::get_game_nonce_account;
use bitflip_legacy_client::get_pda_config;
use bitflip_legacy_client::get_pda_game;
use bitflip_legacy_client::get_pda_game_nonce;
use bitflip_legacy_client::get_pda_mint;
use bitflip_legacy_client::get_pda_section_data;
use bitflip_legacy_client::get_pda_section_state;
use bitflip_legacy_client::get_pda_treasury;
use bitflip_legacy_client::get_player_token_account;
use bitflip_legacy_client::get_section_token_account;
use bitflip_legacy_client::initialize_game_request;
use bitflip_legacy_client::initialize_token_request;
use bitflip_legacy_client::start_game_request;
use bitflip_legacy_client::unlock_section_request;
use bitflip_legacy_program::ACCESS_SIGNER_DURATION;
use bitflip_legacy_program::BITFLIP_SECTION_LENGTH;
use bitflip_legacy_program::ConfigState;
use bitflip_legacy_program::GameState;
use bitflip_legacy_program::SectionData;
use bitflip_legacy_program::SectionState;
use bitflip_legacy_program::SetBitsVariant;
use bitflip_legacy_program::TOKEN_DECIMALS;
use bitflip_legacy_program::accounts::InitializeConfig;
use insta::internals::Content;
use insta::internals::ContentPath;
use shared::create_admin_keypair;
use shared::create_authority_keypair;
use shared::create_config_state;
use shared::create_game_state;
use shared::create_runner_with_accounts;
use shared::create_wallet_keypair;
use shared::get_authority_program;
use shared::get_wallet_program;
use solana_sdk::account::ReadableAccount;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use test_log::test;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
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
	log::info!("config: {config}");
	let (treasury, _) = get_pda_treasury();
	let runner = create_runner_with_accounts(HashMap::new()).await;
	let rpc = runner.rpc();

	let program_client = get_authority_program(rpc);

	let request = program_client
		.initialize_config()
		.accounts(InitializeConfig {
			config,
			admin,
			treasury,
			system_program,
			authority,
		})
		.signers(vec![&admin_keypair])
		.build();

	let simulation = request.simulate().await?;
	log::info!("simulation: {simulation:#?}");
	request.sign_and_send_transaction().await?;

	let config_state_account: ConfigState = program_client.account(&config).await?;
	let authority_redaction = create_insta_redaction(authority, "authority:pubkey");
	insta::assert_compact_json_snapshot!(config_state_account,{
		".authority" => insta::dynamic_redaction(authority_redaction),
	}, @r#"{"authority": "[authority:pubkey]", "bump": 254, "treasuryBump": 255, "mintBump": 0, "gameIndex": 0}"#);

	Ok(())
}

#[test(tokio::test)]
async fn initialize_token() -> anyhow::Result<()> {
	let mut accounts = HashMap::new();
	let (config, _) = get_pda_config();
	let (mint, _) = get_pda_mint();
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);

	accounts.insert(config, create_config_state(None));
	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_authority_program(rpc);
	let request = initialize_token_request(&program_client);
	let simulation = request.simulate().await?;
	log::info!("simulation: {simulation:#?}");
	let signature = request.sign_and_send_transaction().await?;
	rpc.confirm_transaction(&signature).await?;

	let authority_redaction = create_insta_redaction(treasury, "authority:pubkey");
	let mint_redaction = create_insta_redaction(mint, "mint:pubkey");
	let data = rpc.get_account_data(&mint).await?;
	let mint_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("mint account: {mint_account:#?}");
	insta::assert_compact_json_snapshot!(mint_account, {
		".info.mintAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.freezeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.metadataAddress" =>	insta::dynamic_redaction(mint_redaction.clone()),
	  ".info.extensions[1].state.closeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[2].state.updateAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[2].state.mint" =>	insta::dynamic_redaction(mint_redaction.clone())
	}, @r#"
 {
   "type": "mint",
   "info": {
     "mintAuthority": "[authority:pubkey]",
     "supply": "1073741824",
     "decimals": 0,
     "isInitialized": true,
     "freezeAuthority": "[authority:pubkey]",
     "extensions": [
       {
         "extension": "metadataPointer",
         "state": {
           "authority": "[authority:pubkey]",
           "metadataAddress": "[mint:pubkey]"
         }
       },
       {
         "extension": "mintCloseAuthority",
         "state": {
           "closeAuthority": "[authority:pubkey]"
         }
       },
       {
         "extension": "tokenMetadata",
         "state": {
           "updateAuthority": "[authority:pubkey]",
           "mint": "[mint:pubkey]",
           "name": "Bitflip",
           "symbol": "BIT",
           "uri": "https://bitflip.art/token.json",
           "additionalMetadata": []
         }
       }
     ]
   }
 }
 "#);

	let data = rpc.get_account_data(&treasury_token_account).await?;
	let parsed_treasury_token_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("token account: {parsed_treasury_token_account:#?}");
	insta::assert_compact_json_snapshot!(parsed_treasury_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction),
		".info.owner" => insta::dynamic_redaction(authority_redaction),
	}, @r#"
 {
   "type": "account",
   "info": {
     "mint": "[mint:pubkey]",
     "owner": "[authority:pubkey]",
     "tokenAmount": {
       "uiAmount": 1073741824.0,
       "decimals": 0,
       "amount": "1073741824",
       "uiAmountString": "1073741824"
     },
     "state": "initialized",
     "isNative": false,
     "extensions": [
       {
         "extension": "immutableOwner"
       }
     ]
   }
 }
 "#);

	Ok(())
}

#[test(tokio::test)]
async fn initialize_game() -> anyhow::Result<()> {
	let game_index = 0;
	let (config, _) = get_pda_config();
	let (game, _) = get_pda_game(0);
	let mut accounts = HashMap::new();
	accounts.insert(config, create_config_state(None));
	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();

	let program_client = get_authority_program(rpc);
	let (access_signer, refresh_signer) = (Keypair::new(), Keypair::new());
	let request = initialize_game_request(
		&program_client,
		access_signer.pubkey(),
		refresh_signer.pubkey(),
		game_index,
	);

	log::info!("about to simulate");
	let mut transaction = request.sign_transaction().await?;
	transaction.try_sign(&[&access_signer, &refresh_signer], None)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");

	rpc.send_and_confirm_transaction(&transaction).await?;

	let access_signer_redaction =
		create_insta_redaction(access_signer.pubkey(), "access_signer:pubkey");
	let refresh_signer_redaction =
		create_insta_redaction(refresh_signer.pubkey(), "refresh_signer:pubkey");
	let game_state_account: GameState = program_client.account(&game).await?;
	insta::assert_compact_json_snapshot!(game_state_account, {
			".accessSigner" => insta::dynamic_redaction(access_signer_redaction),
			".refreshSigner" => insta::dynamic_redaction(refresh_signer_redaction),
		}, @r#"
 {
   "refreshSigner": "[refresh_signer:pubkey]",
   "accessSigner": "[access_signer:pubkey]",
   "accessExpiry": 0,
   "startTime": 0,
   "gameIndex": 0,
   "sectionIndex": 0,
   "bump": 253,
   "nonceBump": 253
 }
 "#);

	Ok(())
}

#[test(tokio::test)]
async fn start_game() -> anyhow::Result<()> {
	let game_index = 0;
	let section_index = 0;
	let config = get_pda_config().0;
	let game = get_pda_game(0).0;
	let (game_account_data, _, access_signer, refresh_signer) =
		create_game_state(game_index, section_index, 0, 0);

	let mut accounts = HashMap::new();
	accounts.insert(config, create_config_state(None));
	accounts.insert(config, create_config_state(None));
	accounts.insert(game, game_account_data.clone());

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_authority_program(rpc);

	initialize_token_request(&program_client)
		.sign_and_send_transaction()
		.await?;

	let request = start_game_request(&program_client, access_signer.pubkey(), game_index);
	let simulation = request.simulate().await?;
	log::info!("simulation: {simulation:#?}");

	let mut transaction = request.sign_transaction().await?;
	transaction.try_sign(&[&access_signer], None)?;
	rpc.send_and_confirm_transaction(&transaction).await?;

	let access_signer_redaction =
		create_insta_redaction(access_signer.pubkey(), "access_signer:pubkey");
	let refresh_signer_redaction =
		create_insta_redaction(refresh_signer.pubkey(), "refresh_signer:pubkey");
	let game_account: GameState = program_client.account(&game).await?;
	insta::assert_compact_json_snapshot!(game_account, {
			".accessSigner" => insta::dynamic_redaction(access_signer_redaction),
			".refreshSigner" => insta::dynamic_redaction(refresh_signer_redaction),
			".accessExpiry" => "[timestamp]",
			".startTime" => "[timestamp]"
	}, @r#"
 {
   "refreshSigner": "[refresh_signer:pubkey]",
   "accessSigner": "[access_signer:pubkey]",
   "accessExpiry": "[timestamp]",
   "startTime": "[timestamp]",
   "gameIndex": 0,
   "sectionIndex": 0,
   "bump": 253,
   "nonceBump": 253
 }
 "#);
	check!(game_account.started());

	Ok(())
}

#[test(tokio::test)]
async fn unlock_first_section() -> anyhow::Result<()> {
	let game_index = 0;
	let section_index = 0;
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let game_nonce = get_pda_game_nonce(game_index).0;
	let section = get_pda_section_state(game_index, section_index).0;
	let section_token_account = get_section_token_account(game_index, section_index);
	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)?
		.as_secs() as i64;
	log::info!("now: {now}");

	let (game_account_data, game_nonce_account_data, access_signer, _) = create_game_state(
		game_index,
		section_index,
		now - 3600,
		now + ACCESS_SIGNER_DURATION,
	);
	log::info!("game_nonce: {game_nonce}\ngame: {game}");
	let mut accounts = HashMap::new();
	accounts.insert(config, create_config_state(None));
	accounts.insert(game, game_account_data.clone());
	accounts.insert(game_nonce, game_nonce_account_data.clone());

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_wallet_program(rpc);
	let owner = create_wallet_keypair().pubkey();
	let mint = get_pda_mint().0;

	log::warn!("the running game: {game}");
	let game_state_account: GameState = rpc.get_anchor_account(&game).await?;
	log::info!("onchain game_state: {game_state_account:#?}");

	let authority_program_client = get_authority_program(rpc);
	// let start_request = start_game_request(
	// 	&authority_program_client,
	// 	access_signer.pubkey(),
	// 	game_index,
	// );
	// let simulation = start_request.simulate().await?;
	// log::info!("token simulation: {simulation:#?}");
	// let signature = start_request.sign_and_send_transaction().await?;
	// rpc.confirm_transaction(&signature).await?;

	let token_request = initialize_token_request(&authority_program_client);
	let simulation = token_request.simulate().await?;
	log::info!("token simulation: {simulation:#?}");
	let signature = token_request.sign_and_send_transaction().await?;
	rpc.confirm_transaction(&signature).await?;

	let blockhash = get_game_nonce_account(rpc, game_index).await?.blockhash();
	let request = unlock_section_request(
		&program_client,
		access_signer.pubkey(),
		game_index,
		section_index,
		blockhash,
	);

	log::info!("about to simulate section request!");
	// ensure that the transaction is valid even though it hasn't yet been signed by
	// the `access_signer`
	let simulation = request.simulate().await?;
	log::info!("simulation: {simulation:#?}");

	let mut transaction = request.sign_transaction().await?;
	transaction.sign(&[&access_signer], None);
	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let data_redaction = |content: Content, _: ContentPath| {
		let slice = content.as_slice().unwrap();
		check!(
			slice.len() == BITFLIP_SECTION_LENGTH,
			"there should be {BITFLIP_SECTION_LENGTH} elements in the array"
		);

		for item in slice {
			check!(item.as_u64().unwrap() == 0);
		}

		format!("[u16; {BITFLIP_SECTION_LENGTH}]")
	};
	let owner_redaction = create_insta_redaction(&owner, "owner:pubkey");
	let section_state: SectionState = program_client.account(&section).await?;

	insta::assert_compact_json_snapshot!(section_state, {
		".data" => insta::dynamic_redaction(data_redaction),
		".owner" => insta::dynamic_redaction(owner_redaction),
	}, @r#"{"owner": "[owner:pubkey]", "flips": 0, "on": 0, "off": 4096, "index": 0, "bump": 255, "dataBump": 255}"#);

	let data = rpc.get_account_data(&section_token_account).await?;
	let parsed_section_token_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;
	log::info!("token account: {parsed_section_token_account:#?}");

	let mint_redaction = create_insta_redaction(mint, "mint:pubkey");
	let owner_redaction = create_insta_redaction(section, "owner:pubkey");
	insta::assert_compact_json_snapshot!(parsed_section_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction),
		".info.owner" => insta::dynamic_redaction(owner_redaction),
	}, @r#"
 {
   "type": "account",
   "info": {
     "mint": "[mint:pubkey]",
     "owner": "[owner:pubkey]",
     "tokenAmount": {
       "uiAmount": 262144.0,
       "decimals": 0,
       "amount": "262144",
       "uiAmountString": "262144"
     },
     "state": "initialized",
     "isNative": false,
     "extensions": [
       {
         "extension": "immutableOwner"
       }
     ]
   }
 }
 "#);

	Ok(())
}

#[test(tokio::test)]
async fn toggle_bit() -> anyhow::Result<()> {
	let game_index = 0;
	let section_index = 0;
	let next_section_index = 0;
	// let next_section_index = 10;
	let bit_index = 0;
	let config = get_pda_config().0;
	let (mint, mint_bump) = get_pda_mint();
	let game = get_pda_game(game_index).0;
	let game_nonce = get_pda_game_nonce(game_index).0;
	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)?
		.as_secs() as i64;
	let (game_account_data, game_nonce_account_data, access_signer, ..) = create_game_state(
		game_index,
		next_section_index,
		now - 3600,
		now + ACCESS_SIGNER_DURATION,
	);
	let mut accounts = HashMap::new();

	accounts.insert(config, create_config_state(Some(mint_bump)));
	accounts.insert(game, game_account_data.clone());
	accounts.insert(game_nonce, game_nonce_account_data.clone());

	// let section_accounts = create_section_state(game_index,
	// next_section_index); for (pubkey, data) in section_accounts {
	// 	accounts.insert(pubkey, data);
	// }

	let runner = create_runner_with_accounts(accounts).await;
	let rpc = runner.rpc();
	let program_client = get_wallet_program(rpc);
	let section_state = get_pda_section_state(game_index, next_section_index).0;
	let section_data = get_pda_section_data(game_index, next_section_index).0;
	let player = program_client.payer();
	let section = get_pda_section_state(game_index, section_index).0;
	let section_token_account = get_section_token_account(game_index, section_index);
	let player_token_account = get_player_token_account(&player);
	let signature = initialize_token_request(&get_authority_program(rpc))
		.sign_and_send_transaction()
		.await?;
	rpc.confirm_transaction(&signature).await?;
	log::info!("setting up section: {section}, section_token_account: {section_token_account}");

	let blockhash = get_game_nonce_account(rpc, game_index).await?.blockhash();
	let unlock_request = unlock_section_request(
		&program_client,
		access_signer.pubkey(),
		game_index,
		section_index,
		blockhash,
	);
	let simulation = unlock_request.simulate().await?;
	log::info!("simulation: {simulation:#?}");

	let mut unlock_section_transaction = unlock_request.sign_transaction().await?;
	unlock_section_transaction.try_sign(&[&access_signer], None)?;
	let signature = rpc
		.send_and_confirm_transaction(&unlock_section_transaction)
		.await?;
	rpc.confirm_transaction(&signature).await?;
	log::info!("done setting section");

	let offset = 12;
	let bits = 1 << offset;
	let request = flip_bits_request(
		&program_client,
		game_index,
		section_index,
		bit_index,
		SetBitsVariant::On(offset),
	);
	let simulation = request.simulate().await?;
	log::info!("simulation: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 200_000);

	let signature = request.sign_and_send_transaction().await?;
	rpc.confirm_transaction(&signature).await?;

	let account: SectionData = program_client.account(&section_data).await?;
	check!(account.data[0] == bits);

	let player_redaction = create_insta_redaction(player, "player:pubkey");
	let mint_redaction = create_insta_redaction(mint, "mint:pubkey");
	let section_redaction = create_insta_redaction(section, "section:pubkey");
	let raw_player_token_account = rpc.get_account(&player_token_account).await?;
	let parsed_player_token_account = parse_token_v2(
		raw_player_token_account.data(),
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;
	log::info!("player token account: {parsed_player_token_account:#?}");
	insta::assert_compact_json_snapshot!(parsed_player_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction.clone()),
		".info.owner" => insta::dynamic_redaction(player_redaction),
	}, @r#"
 {
   "type": "account",
   "info": {
     "mint": "[mint:pubkey]",
     "owner": "[player:pubkey]",
     "tokenAmount": {
       "uiAmount": 1.0,
       "decimals": 0,
       "amount": "1",
       "uiAmountString": "1"
     },
     "state": "initialized",
     "isNative": false,
     "extensions": [
       {
         "extension": "immutableOwner"
       }
     ]
   }
 }
 "#);

	let raw_section_token_account = rpc.get_account(&section_token_account).await?;
	let parsed_section_token_account = parse_token_v2(
		raw_section_token_account.data(),
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;
	log::info!("section token account: {parsed_section_token_account:#?}");
	insta::assert_compact_json_snapshot!(parsed_section_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction),
		".info.owner" => insta::dynamic_redaction(section_redaction),
	}, @r#"
 {
   "type": "account",
   "info": {
     "mint": "[mint:pubkey]",
     "owner": "[section:pubkey]",
     "tokenAmount": {
       "uiAmount": 262143.0,
       "decimals": 0,
       "amount": "262143",
       "uiAmountString": "262143"
     },
     "state": "initialized",
     "isNative": false,
     "extensions": [
       {
         "extension": "immutableOwner"
       }
     ]
   }
 }
 "#);

	let section_state_account: SectionState = program_client.account(&section_state).await?;
	let section_data_account: SectionData = program_client.account(&section_data).await?;
	for (ii, item) in section_data_account.data.iter().enumerate() {
		if ii == bit_index as usize {
			check!(*item == bits, "item at index {ii} should be {bits}");
		} else {
			check!(*item == 0, "item at index {ii} should be 0");
		}
	}

	let owner_redaction = create_insta_redaction(&player, "player:pubkey");

	insta::assert_compact_json_snapshot!(section_state_account, {
	 ".owner" => insta::dynamic_redaction(owner_redaction),
	 ".startTime" => "[timestamp]",
 }, @r#"{"owner": "[player:pubkey]", "flips": 1, "on": 1, "off": 4095, "index": 0, "bump": 255, "dataBump": 255}"#);

	Ok(())
}
