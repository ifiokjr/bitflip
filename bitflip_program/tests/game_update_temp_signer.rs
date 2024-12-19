#![cfg(feature = "client")]

use std::future::Future;

use assert2::check;
use bitflip_program::game_update_temp_signer;
use bitflip_program::get_pda_game;
use bitflip_program::GameState;
use shared::create_config_accounts;
use shared::create_game_state;
use shared::create_program_context_with_factory;
use shared::create_token_accounts;
use shared::CreatedGameState;
use shared::ToRpcClient;
use solana_sdk::signature::Keypair;
use solana_sdk::transaction::VersionedTransaction;
use steel::*;
use test_utils_insta::create_insta_redaction;
use test_utils_solana::prelude::*;

mod shared;

#[test_log::test(tokio::test)]
async fn game_update_temp_signer_test() -> anyhow::Result<()> {
	let created_game_state = create_game_state(0, 0, 0);
	shared_game_update_temp_signer_test(
		|| create_banks_client_rpc(&created_game_state),
		&created_game_state,
	)
	.await?;

	Ok(())
}

#[cfg(feature = "test_validator")]
#[test_log::test(tokio::test)]
async fn game_update_temp_signer_test_validator() -> anyhow::Result<()> {
	let created_game_state = create_game_state(0, 0, 0);
	let compute_units = shared_game_update_temp_signer_test(
		|| create_validator_rpc(&created_game_state),
		&created_game_state,
	)
	.await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units == 10_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));
	shared::save_compute_units(
		"game_update_temp_signer",
		compute_units,
		"Refresh the signer",
	)?;
	Ok(())
}

async fn create_banks_client_rpc(
	CreatedGameState {
		ref game_state_account,
		ref funded_signer,
		ref funded_signer_account,
		..
	}: &CreatedGameState,
) -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|p| {
		let mut config_state_accounts = create_config_accounts();
		config_state_accounts.extend(create_token_accounts(false)?);

		for (key, account) in config_state_accounts {
			p.add_account(key, account.into());
		}

		let game = get_pda_game(0).0;

		p.add_account(game, game_state_account.clone().into());
		p.add_account(funded_signer.pubkey(), funded_signer_account.clone().into());

		Ok(())
	})
	.await?;

	Ok(provider)
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc(
	CreatedGameState {
		ref game_state_account,
		ref funded_signer,
		ref funded_signer_account,
		..
	}: &CreatedGameState,
) -> anyhow::Result<impl ToRpcClient> {
	let mut accounts = create_config_accounts();
	let game = get_pda_game(0).0;
	accounts.insert(game, game_state_account.clone());
	accounts.insert(funded_signer.pubkey(), funded_signer_account.clone());

	let runner = shared::create_runner_with_accounts(accounts).await;

	Ok(runner)
}

async fn shared_game_update_temp_signer_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
	CreatedGameState {
		funded_signer: ref funded_signer_keypair,
		..
	}: &CreatedGameState,
) -> anyhow::Result<u64> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let temp_signer_keypair = Keypair::new();
	let temp_signer = temp_signer_keypair.pubkey();
	let funded_signer = funded_signer_keypair.pubkey();
	let game_index = 0;
	let game = get_pda_game(game_index).0;

	// Create transaction
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let ix = game_update_temp_signer(&funded_signer, &temp_signer, game_index);
	let mut transaction =
		VersionedTransaction::new_unsigned_v0(&funded_signer, &[ix], &[], recent_blockhash)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");
	let compute_units = simulation.value.units_consumed.unwrap();

	// Sign and send transaction
	transaction
		.try_sign(&[&temp_signer_keypair], None)?
		.try_sign(&[funded_signer_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	// Verify the game state was updated
	let game_state_data = rpc.get_account_data(&game).await?;
	let game_state_account = GameState::try_from_bytes(&game_state_data)?;
	let temp_signer_redaction = create_insta_redaction(temp_signer, "temp_signer:pubkey");
	let funded_signer_redaction = create_insta_redaction(funded_signer, "funded_signer:pubkey");
	insta::assert_compact_json_snapshot!(game_state_account,{
		".tempSigner" => insta::dynamic_redaction(temp_signer_redaction),
		".fundedSigner" => insta::dynamic_redaction(funded_signer_redaction),
	});

	Ok(compute_units)
}
