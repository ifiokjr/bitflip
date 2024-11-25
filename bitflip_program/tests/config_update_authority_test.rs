#![cfg(feature = "client")]

use std::future::Future;

use assert2::check;
use bitflip_program::config_update_authority;
use bitflip_program::get_pda_config;
use bitflip_program::BitflipError;
use bitflip_program::ConfigState;
use shared::create_authority_keypair;
use shared::create_config_accounts;
use shared::create_program_context_with_factory;
use shared::create_token_accounts;
use shared::ToRpcClient;
use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::TransactionError;
use solana_sdk::transaction::VersionedTransaction;
use steel::*;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;

mod shared;

#[test_log::test(tokio::test)]
async fn config_update_authority_test() -> anyhow::Result<()> {
	shared_config_update_authority_test(create_banks_client_rpc).await?;
	Ok(())
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn config_update_authority_test_validator() -> anyhow::Result<()> {
	let compute_units = shared_config_update_authority_test(create_validator_rpc).await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units == 10_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));
	shared::save_compute_units(
		"config_update_authority",
		compute_units,
		"Update the authority of the config",
	)?;

	Ok(())
}

#[test_log::test(tokio::test)]
async fn authority_must_change_test() -> anyhow::Result<()> {
	shared_authority_must_change_test(create_banks_client_rpc).await
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn authority_must_change_test_validator() -> anyhow::Result<()> {
	shared_authority_must_change_test(create_validator_rpc).await
}

async fn create_banks_client_rpc() -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|p| {
		let mut accounts = create_config_accounts();
		accounts.extend(create_token_accounts(false)?);

		for (key, account) in accounts {
			p.add_account(key, account.into());
		}

		Ok(())
	})
	.await?;

	Ok(provider)
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc() -> anyhow::Result<impl ToRpcClient> {
	let mut config_state_accounts = create_config_accounts();
	config_state_accounts.extend(create_token_accounts(false)?);

	let runner = shared::create_runner_with_accounts(config_state_accounts).await;

	Ok(runner)
}

async fn shared_config_update_authority_test<
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
	let new_authority_keypair = Keypair::new();
	let new_authority = new_authority_keypair.pubkey();
	let config = get_pda_config().0;
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = config_update_authority(&authority, &new_authority);
	let mut transaction =
		VersionedTransaction::new_unsigned_v0(&authority, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");
	let compute_units = simulation.value.units_consumed.unwrap();

	transaction
		.try_sign(&[&new_authority_keypair], None)?
		.try_sign(&[&authority_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let config_state_data = rpc.get_account_data(&config).await?;
	let config_state_account = ConfigState::try_from_bytes(&config_state_data)?;
	let authority_redaction = create_insta_redaction(new_authority, "new_authority:pubkey");
	insta::assert_compact_json_snapshot!(config_state_account,{
		".authority" => insta::dynamic_redaction(authority_redaction),
	});

	Ok(compute_units)
}

async fn shared_authority_must_change_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
) -> anyhow::Result<()> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = config_update_authority(&authority, &authority);
	let transaction =
		VersionedTransaction::new_unsigned_v0(&authority, &[ix], &[], recent_blockhash)?;
	let simulation = rpc.simulate_transaction(&transaction).await?;

	log::info!("simulation: {simulation:#?}");
	check!(
		simulation.value.err.unwrap()
			== TransactionError::InstructionError(
				0,
				InstructionError::Custom(BitflipError::DuplicateAuthority.into())
			)
	);

	Ok(())
}
