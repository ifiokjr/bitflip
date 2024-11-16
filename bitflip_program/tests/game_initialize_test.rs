#![cfg(feature = "client")]

use std::future::Future;

use assert2::check;
use bitflip_program::BitflipError;
use bitflip_program::ConfigState;
use bitflip_program::GameState;
use bitflip_program::game_initialize;
use bitflip_program::get_pda_config;
use bitflip_program::get_pda_game;
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
async fn game_initialize_test() -> anyhow::Result<()> {
	shared_game_initialize_test(create_banks_client_rpc).await?;
	Ok(())
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn game_initialize_test_validator() -> anyhow::Result<()> {
	let compute_units = shared_game_initialize_test(create_validator_rpc).await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units == 20_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));

	Ok(())
}

async fn create_banks_client_rpc() -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|p| {
		let config = get_pda_config().0;
		let config_state_account = create_config_state();
		p.add_account(config, config_state_account.into());
	})
	.await?;

	Ok(provider)
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc() -> anyhow::Result<impl ToRpcClient> {
	let mut accounts = std::collections::HashMap::new();
	let config = get_pda_config().0;
	let config_state_account = create_config_state();
	accounts.insert(config, config_state_account);

	let runner = shared::create_runner_with_accounts(accounts).await;

	Ok(runner)
}

async fn shared_game_initialize_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
) -> anyhow::Result<u64> {
	let game_index = 0;
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let access_signer_keypair = Keypair::new();
	let refresh_signer_keypair = Keypair::new();
	let access_signer = access_signer_keypair.pubkey();
	let refresh_signer = refresh_signer_keypair.pubkey();
	let game = get_pda_game(game_index).0;
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = game_initialize(game_index, &authority, &access_signer, &refresh_signer);
	let mut transaction =
		VersionedTransaction::new_unsigned_v0(&authority, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");
	let compute_units = simulation.value.units_consumed.unwrap();

	transaction
		.try_sign(&[&authority_keypair], None)?
		.try_sign(&[&access_signer_keypair], None)?
		.try_sign(&[&refresh_signer_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let game_state_data = rpc.get_account_data(&game).await?;
	let game_state_account = GameState::try_from_bytes(&game_state_data)?;
	let authority_redaction = create_insta_redaction(authority, "authority:pubkey");
	let access_signer_redaction = create_insta_redaction(access_signer, "access_signer:pubkey");
	let refresh_signer_redaction = create_insta_redaction(refresh_signer, "refresh_signer:pubkey");
	insta::assert_compact_json_snapshot!(game_state_account,{
		".authority" => insta::dynamic_redaction(authority_redaction),
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
   "padding": [
     0,
     0,
     0,
     0,
     0
   ]
 }
 "#);

	Ok(compute_units)
}