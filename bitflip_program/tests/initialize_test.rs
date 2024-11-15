#![cfg(feature = "client")]

use std::future::Future;

use assert2::check;
use bitflip_program::BitflipError;
use bitflip_program::ConfigState;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::get_pda_config;
use bitflip_program::get_pda_mint_bit;
use bitflip_program::get_pda_treasury;
use bitflip_program::get_treasury_bit_token_account;
use bitflip_program::initialize;
use shared::ToRpcClient;
use shared::create_admin_keypair;
use shared::create_authority_keypair;
use shared::create_program_context_with_factory;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::TransactionError;
use solana_sdk::transaction::VersionedTransaction;
use steel::*;
use sysvar::rent::Rent;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wasm_client_solana::solana_account_decoder::parse_account_data::SplTokenAdditionalData;
use wasm_client_solana::solana_account_decoder::parse_token::parse_token_v2;

mod shared;

#[test_log::test(tokio::test)]
async fn initialize_config_test() -> anyhow::Result<()> {
	shared_initialize_config_test(create_banks_client_rpc).await?;
	Ok(())
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn initialize_config_test_validator() -> anyhow::Result<()> {
	let compute_units = shared_initialize_config_test(create_validator_rpc).await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units <= 400_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));

	Ok(())
}

#[test_log::test(tokio::test)]
async fn invalid_admin_initialize_config_test() -> anyhow::Result<()> {
	shared_invalid_admin_initialize_config_test(create_banks_client_rpc).await
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn invalid_admin_initialize_config_test_validator() -> anyhow::Result<()> {
	shared_invalid_admin_initialize_config_test(create_validator_rpc).await
}

#[test_log::test(tokio::test)]
async fn duplicate_authority_initialize_config_test() -> anyhow::Result<()> {
	shared_duplicate_authority_initialize_config_test(create_banks_client_rpc).await
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn duplicate_authority_initialize_config_test_validator() -> anyhow::Result<()> {
	shared_duplicate_authority_initialize_config_test(create_validator_rpc).await
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc() -> anyhow::Result<impl ToRpcClient> {
	let runner = shared::create_runner().await;

	Ok(runner)
}

async fn create_banks_client_rpc() -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|_| {}).await?;
	Ok(provider)
}

async fn shared_initialize_config_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
) -> anyhow::Result<u64> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let admin_keypair = create_admin_keypair();
	let admin = admin_keypair.pubkey();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let config = get_pda_config().0;
	let treasury = get_pda_treasury().0;
	let mint_bit = get_pda_mint_bit().0;
	let treasury_bit_token_account = get_treasury_bit_token_account();
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let instruction = initialize(&admin, &authority);
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

	transaction
		.try_sign(&[&admin_keypair], None)?
		.try_sign(&[&authority_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let config_state_data = rpc.get_account_data(&config).await?;
	let config_state_account = ConfigState::try_from_bytes(&config_state_data)?;
	let authority_redaction = create_insta_redaction(authority, "treasury");
	insta::assert_compact_json_snapshot!(config_state_account,{
		".authority" => insta::dynamic_redaction(authority_redaction),
	}, @r#"
 {
   "authority": "[treasury]",
   "bump": 254,
   "treasuryBump": 255,
   "mintBitBump": 255,
   "mintKibibitBump": 255,
   "mintMebibitBump": 255,
   "mintGibibitBump": 255,
   "gameIndex": 0
 }
 "#);

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
     "supply": "8589934592",
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
       "uiAmount": 8589934592.0,
       "decimals": 0,
       "amount": "8589934592",
       "uiAmountString": "8589934592"
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

async fn shared_invalid_admin_initialize_config_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
) -> anyhow::Result<()> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let admin_keypair = Keypair::new();
	let admin = admin_keypair.pubkey();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = initialize(&admin, &authority);
	let transaction =
		VersionedTransaction::new_unsigned_v0(&authority, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");

	check!(
		simulation.value.err.unwrap()
			== TransactionError::InstructionError(
				0,
				InstructionError::Custom(BitflipError::UnauthorizedAdmin.into())
			)
	);

	Ok(())
}

async fn shared_duplicate_authority_initialize_config_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
) -> anyhow::Result<()> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let admin_keypair = create_admin_keypair();
	let admin = admin_keypair.pubkey();
	let authority_keypair = create_admin_keypair();
	let authority = authority_keypair.pubkey();
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = initialize(&admin, &authority);
	let transaction =
		VersionedTransaction::new_unsigned_v0(&authority, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");

	check!(
		simulation.value.err.unwrap()
			== TransactionError::InstructionError(
				0,
				InstructionError::Custom(BitflipError::DuplicateAuthority.into())
			),
	);

	Ok(())
}
