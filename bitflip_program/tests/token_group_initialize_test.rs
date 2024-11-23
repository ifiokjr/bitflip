#![cfg(feature = "client")]

use std::future::Future;

use assert2::check;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::get_pda_mint_bit;
use bitflip_program::get_pda_treasury;
use bitflip_program::get_treasury_bit_token_account;
use bitflip_program::token_group_initialize;
use shared::ToRpcClient;
use shared::create_authority_keypair;
use shared::create_config_accounts;
use shared::create_program_context_with_factory;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::transaction::VersionedTransaction;
use steel::*;
use sysvar::rent::Rent;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wasm_client_solana::solana_account_decoder::parse_account_data::SplTokenAdditionalData;
use wasm_client_solana::solana_account_decoder::parse_token::parse_token_v2;

mod shared;

#[test_log::test(tokio::test)]
async fn token_group_initialize_test() -> anyhow::Result<()> {
	shared_token_group_initialize_test(create_banks_client_rpc).await?;
	Ok(())
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn token_group_initialize_test_validator() -> anyhow::Result<()> {
	let compute_units = shared_token_group_initialize_test(create_validator_rpc).await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units <= 90_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));
	shared::save_compute_units(
		"token_group_initialize",
		compute_units,
		"Initialize the token group",
	)?;

	Ok(())
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc() -> anyhow::Result<impl ToRpcClient> {
	let accounts = create_config_accounts();
	let runner = shared::create_runner_with_accounts(accounts).await;

	Ok(runner)
}

async fn create_banks_client_rpc() -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|p| {
		let accounts = create_config_accounts();

		for (key, account) in accounts {
			p.add_account(key, account.into());
		}
	})
	.await?;
	Ok(provider)
}

async fn shared_token_group_initialize_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
) -> anyhow::Result<u64> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let treasury = get_pda_treasury().0;
	let mint_bit = get_pda_mint_bit().0;
	let treasury_bit_token_account = get_treasury_bit_token_account();
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let instruction = token_group_initialize(&authority);
	let compute_limit_instruction = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);
	let mut transaction = VersionedTransaction::new_unsigned_v0(
		&authority,
		&[compute_limit_instruction, instruction],
		&[],
		recent_blockhash,
	)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");

	let compute_units = simulation.value.units_consumed.unwrap();

	transaction.try_sign(&[&authority_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let treasury_lamports = rpc.get_balance(&treasury).await?;
	check!(treasury_lamports == Rent::default().minimum_balance(0));

	let authority_redaction = create_insta_redaction(treasury, "treasury:pubkey");
	let mint_bit_redaction = create_insta_redaction(mint_bit, "mint_bit:pubkey");
	let data = rpc.get_account_data(&mint_bit).await?;
	let mint_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("mint_bit account: {mint_account:#?}");
	insta::assert_compact_json_snapshot!(mint_account, {
		".info.mintAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.freezeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.metadataAddress" =>	insta::dynamic_redaction(mint_bit_redaction.clone()),
	  ".info.extensions[1].state.closeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[2].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[2].state.groupAddress" => insta::dynamic_redaction(mint_bit_redaction.clone()),
	  ".info.extensions[3].state.updateAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[3].state.mint" =>	insta::dynamic_redaction(mint_bit_redaction.clone())
	}, @r#"
 {
   "type": "mint",
   "info": {
     "mintAuthority": "[treasury:pubkey]",
     "supply": "1073741824",
     "decimals": 0,
     "isInitialized": true,
     "freezeAuthority": "[treasury:pubkey]",
     "extensions": [
       {
         "extension": "metadataPointer",
         "state": {
           "authority": "[treasury:pubkey]",
           "metadataAddress": "[mint_bit:pubkey]"
         }
       },
       {
         "extension": "mintCloseAuthority",
         "state": {
           "closeAuthority": "[treasury:pubkey]"
         }
       },
       {
         "extension": "groupPointer",
         "state": {
           "authority": "[treasury:pubkey]",
           "groupAddress": "[mint_bit:pubkey]"
         }
       },
       {
         "extension": "tokenMetadata",
         "state": {
           "updateAuthority": "[treasury:pubkey]",
           "mint": "[mint_bit:pubkey]",
           "name": "Bit",
           "symbol": "B",
           "uri": "https://bitflip.art/bit-meta.json",
           "additionalMetadata": []
         }
       }
     ]
   }
 }
 "#);

	let data = rpc.get_account_data(&treasury_bit_token_account).await?;
	let parsed_treasury_bit_token_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("token account: {parsed_treasury_bit_token_account:#?}");
	insta::assert_compact_json_snapshot!(parsed_treasury_bit_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_bit_redaction),
		".info.owner" => insta::dynamic_redaction(authority_redaction),
	}, @r#"
 {
   "type": "account",
   "info": {
     "mint": "[mint_bit:pubkey]",
     "owner": "[treasury:pubkey]",
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

	Ok(compute_units)
}
