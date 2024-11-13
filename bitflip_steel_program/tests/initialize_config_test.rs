use std::future::Future;

use assert2::check;
use bitflip_steel_program::BitflipError;
use bitflip_steel_program::ConfigState;
use bitflip_steel_program::get_pda_config;
use bitflip_steel_program::get_pda_treasury;
use bitflip_steel_program::initialize_config;
use shared::ToRpcClient;
use shared::create_admin_keypair;
use shared::create_authority_keypair;
use shared::create_program_context_with_factory;
#[cfg(feature = "test_validator")]
use shared::create_runner;
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
async fn initialize_config_test() -> anyhow::Result<()> {
	shared_initialize_config_test(create_banks_client_rpc).await
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn initialize_config_test_validator() -> anyhow::Result<()> {
	shared_initialize_config_test(create_validator_rpc).await
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
	let runner = create_runner().await;

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
) -> anyhow::Result<()> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let admin_keypair = create_admin_keypair();
	let admin = admin_keypair.pubkey();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let (config, _) = get_pda_config();
	let (treasury, _) = get_pda_treasury();
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = initialize_config(&admin, &authority);
	let mut transaction =
		VersionedTransaction::new_unsigned_v0(&authority, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 100_000);

	transaction
		.try_sign(&[&admin_keypair], None)?
		.try_sign(&[&authority_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let config_state_data = rpc.get_account_data(&config).await?;
	let config_state_account = ConfigState::try_from_bytes(&config_state_data)?;
	let authority_redaction = create_insta_redaction(authority, "authority:pubkey");
	insta::assert_compact_json_snapshot!(config_state_account,{
		".authority" => insta::dynamic_redaction(authority_redaction),
	}, @r#"{"authority": "[authority:pubkey]", "bump": 254, "treasuryBump": 255, "mintBump": 0, "gameIndex": 0}"#);

	let treasury_lamports = rpc.get_balance(&treasury).await?;
	check!(treasury_lamports == Rent::default().minimum_balance(0));

	Ok(())
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
	let ix = initialize_config(&admin, &authority);
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
	let ix = initialize_config(&admin, &authority);
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
