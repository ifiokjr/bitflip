use std::future::Future;

use assert2::check;
use bitflip_steel_program::BitflipError;
use bitflip_steel_program::ConfigState;
use bitflip_steel_program::get_pda_config;
use bitflip_steel_program::update_authority;
use shared::ToRpcClient;
use shared::create_authority_keypair;
use shared::create_config_state;
use shared::create_program_context_with_factory;
use solana_sdk::instruction::InstructionError;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::TransactionError;
use solana_sdk::transaction::VersionedTransaction;
use steel::*;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;

mod shared;

#[test_log::test(tokio::test)]
async fn update_authority_test() -> anyhow::Result<()> {
	shared_update_authority_test(create_banks_client_rpc).await
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn update_authority_test_validator() -> anyhow::Result<()> {
	shared_update_authority_test(create_validator_rpc).await
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
		let config = get_pda_config().0;
		let config_state_account = create_config_state(None);
		p.add_account(config, config_state_account.into());
	})
	.await?;

	Ok(provider)
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc() -> anyhow::Result<impl ToRpcClient> {
	use std::collections::HashMap;

	use shared::create_runner_with_accounts;
	let mut accounts = HashMap::new();
	let config = get_pda_config().0;
	let config_state_account = create_config_state(None);

	accounts.insert(config, config_state_account);

	let runner = create_runner_with_accounts(accounts).await;

	Ok(runner)
}

async fn shared_update_authority_test<
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
	let new_authority_keypair = Keypair::new();
	let new_authority = new_authority_keypair.pubkey();
	let config = get_pda_config().0;
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = update_authority(&authority, &new_authority);
	let mut transaction =
		VersionedTransaction::new_unsigned_v0(&authority, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 100_000);

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
	}, @r#"{"authority": "[new_authority:pubkey]", "bump": 254, "treasuryBump": 255, "mintBump": 0, "gameIndex": 0}"#);

	Ok(())
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
	let ix = update_authority(&authority, &authority);
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
