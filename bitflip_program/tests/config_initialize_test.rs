#![cfg(feature = "client")]

use std::future::Future;

use assert2::check;
use bitflip_program::BitflipError;
use bitflip_program::ConfigState;
use bitflip_program::config_initialize;
use bitflip_program::get_pda_config;
use bitflip_program::get_pda_treasury;
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

mod shared;

#[test_log::test(tokio::test)]
async fn config_initialize_test() -> anyhow::Result<()> {
	shared_config_initialize_test(create_banks_client_rpc).await?;
	Ok(())
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn config_initialize_test_validator() -> anyhow::Result<()> {
	let compute_units = shared_config_initialize_test(create_validator_rpc).await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units <= 70_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));
	shared::save_compute_units("config_initialize", compute_units, "Initialize the config")?;

	Ok(())
}

#[test_log::test(tokio::test)]
async fn invalid_admin_config_initialize_test() -> anyhow::Result<()> {
	shared_invalid_admin_config_initialize_test(create_banks_client_rpc).await
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn invalid_admin_config_initialize_test_validator() -> anyhow::Result<()> {
	shared_invalid_admin_config_initialize_test(create_validator_rpc).await
}

#[test_log::test(tokio::test)]
async fn duplicate_authority_config_initialize_test() -> anyhow::Result<()> {
	shared_duplicate_authority_config_initialize_test(create_banks_client_rpc).await
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn duplicate_authority_config_initialize_test_validator() -> anyhow::Result<()> {
	shared_duplicate_authority_config_initialize_test(create_validator_rpc).await
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc() -> anyhow::Result<impl ToRpcClient> {
	let runner = shared::create_runner().await;

	Ok(runner)
}

async fn create_banks_client_rpc() -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|_| Ok(())).await?;
	Ok(provider)
}

async fn shared_config_initialize_test<
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
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let instruction = config_initialize(&admin, &authority);
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

	let treasury_lamports = rpc.get_balance(&treasury).await?;
	check!(treasury_lamports == Rent::default().minimum_balance(0));

	let config_state_data = rpc.get_account_data(&config).await?;
	let config_state_account = ConfigState::try_from_bytes(&config_state_data)?;
	let authority_redaction = create_insta_redaction(authority, "treasury");
	insta::assert_compact_json_snapshot!(config_state_account,{
		".authority" => insta::dynamic_redaction(authority_redaction),
	});

	Ok(compute_units)
}

async fn shared_invalid_admin_config_initialize_test<
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
	let ix = config_initialize(&admin, &authority);
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

async fn shared_duplicate_authority_config_initialize_test<
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
	let ix = config_initialize(&admin, &authority);
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
