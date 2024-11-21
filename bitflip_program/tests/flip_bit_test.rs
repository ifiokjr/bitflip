#![cfg(feature = "client")]

use std::future::Future;
use std::time::SystemTime;

use assert2::check;
use bitflip_program::ACCESS_SIGNER_DURATION;
use bitflip_program::BITFLIP_SECTION_LENGTH;
use bitflip_program::SectionState;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::flip_bit;
use bitflip_program::get_pda_game;
use bitflip_program::get_pda_mint_bit;
use bitflip_program::get_pda_section;
use bitflip_program::get_player_bit_token_account;
use bitflip_program::get_section_bit_token_account;
use shared::ToRpcClient;
use shared::create_config_accounts;
use shared::create_game_state;
use shared::create_program_context_with_factory;
use shared::create_section_state;
use shared::create_wallet_keypair;
use solana_sdk::account::ReadableAccount;
use solana_sdk::transaction::VersionedTransaction;
use steel::*;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wasm_client_solana::solana_account_decoder::parse_account_data::SplTokenAdditionalData;
use wasm_client_solana::solana_account_decoder::parse_token::parse_token_v2;

mod shared;

#[test_log::test(tokio::test)]
async fn flip_bit_test() -> anyhow::Result<()> {
	let game_index = 0;
	let section_index = 0;
	shared_flip_bit_test(
		|| create_banks_client_rpc(game_index, section_index),
		game_index,
		section_index,
	)
	.await?;
	Ok(())
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn flip_bit_test_validator() -> anyhow::Result<()> {
	let game_index = 0;
	let section_index = 0;
	let compute_units = shared_flip_bit_test(
		|| create_validator_rpc(game_index, section_index),
		game_index,
		section_index,
	)
	.await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units <= 100_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));

	Ok(())
}

async fn create_banks_client_rpc(
	game_index: u8,
	section_index: u8,
) -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|p| {
		let config_state_accounts = create_config_accounts();

		for (key, account) in config_state_accounts {
			p.add_account(key, account.into());
		}

		let now = SystemTime::now()
			.duration_since(SystemTime::UNIX_EPOCH)
			.unwrap()
			.as_secs() as i64;
		let game = get_pda_game(game_index).0;
		let create_game_state = create_game_state(0, 0, now - 3600, now + ACCESS_SIGNER_DURATION);
		p.add_account(game, create_game_state.game_state_account.into());

		let section_accounts = create_section_state(
			Pubkey::new_unique(),
			game_index,
			section_index.saturating_add(1),
			false,
		);

		for (section, section_account) in section_accounts {
			p.add_account(section, section_account.into());
		}
	})
	.await?;

	Ok(provider)
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc(
	game_index: u8,
	section_index: u8,
) -> anyhow::Result<impl ToRpcClient> {
	let mut accounts = create_config_accounts();

	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs() as i64;
	let game = get_pda_game(game_index).0;
	let create_game_state = create_game_state(0, 0, now - 3600, now + ACCESS_SIGNER_DURATION);
	accounts.insert(game, create_game_state.game_state_account);

	let section_accounts = create_section_state(
		Pubkey::new_unique(),
		game_index,
		section_index.saturating_add(1),
		false,
	);
	accounts.extend(section_accounts);

	let runner = shared::create_runner_with_accounts(accounts).await;

	Ok(runner)
}

async fn shared_flip_bit_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
	game_index: u8,
	section_index: u8,
) -> anyhow::Result<u64> {
	let array_index = 0;
	let offset = 0;
	let value = 1;
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let wallet_keypair = create_wallet_keypair();
	let player = wallet_keypair.pubkey();
	let mint_bit = get_pda_mint_bit().0;
	let player_bit_token_account = get_player_bit_token_account(&player);
	let section_bit_token_account = get_section_bit_token_account(game_index, section_index);
	let section = get_pda_section(game_index, section_index).0;
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = flip_bit(
		&player,
		game_index,
		section_index,
		array_index,
		offset,
		value,
	);
	let mut transaction =
		VersionedTransaction::new_unsigned_v0(&player, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");
	let compute_units = simulation.value.units_consumed.unwrap();
	transaction.try_sign(&[&wallet_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let section_data = rpc.get_account_data(&section).await?;
	let section_state = SectionState::try_from_bytes(&section_data)?;
	let mut expected_data = [0u16; BITFLIP_SECTION_LENGTH];
	expected_data[0] = 1 << offset;
	// check!(&section_state.data == &expected_data);
	insta::assert_compact_json_snapshot!(section_state, {
		".data" => "[data]",
		".owner" => "[owner:pubkey]",
		".startTime" => "[timestamp]",
	}, @r#"
 {
   "data": "[data]",
   "owner": "[owner:pubkey]",
   "flips": 1,
   "on": 1,
   "off": 4095,
   "gameIndex": 0,
   "sectionIndex": 0,
   "bump": 254
 }
 "#);

	let mint_redaction = create_insta_redaction(mint_bit, "mint:pubkey");
	let section_redaction = create_insta_redaction(section, "section:pubkey");
	let raw_player_token_account = rpc.get_account(&player_bit_token_account).await?;
	let parsed_player_token_account = parse_token_v2(
		raw_player_token_account.data(),
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("player token account: {parsed_player_token_account:#?}");
	insta::assert_compact_json_snapshot!(parsed_player_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction.clone()),
		".info.owner" => "[owner:pubkey]",
	}, @r#"
 {
   "type": "account",
   "info": {
     "mint": "[mint:pubkey]",
     "owner": "[owner:pubkey]",
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

	let raw_section_token_account = rpc.get_account(&section_bit_token_account).await?;
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
     "closeAuthority": "EkxzLviVZsDkDLvNT4v8q1WPNYdzJLJgH9ehwLzdPV7P",
     "extensions": [
       {
         "extension": "immutableOwner"
       }
     ]
   }
 }
 "#);

	Ok(compute_units)
}
