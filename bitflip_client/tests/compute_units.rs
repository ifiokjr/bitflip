use assert2::check;
use bitflip_client::BitflipProgramClient;
use bitflip_client::get_pda_config;
use bitflip_client::get_pda_mint;
use bitflip_client::get_pda_treasury;
use bitflip_program::ConfigState;
use bitflip_program::ID_CONST;
use bitflip_program::InitializeConfigProps;
use bitflip_program::accounts::InitializeConfig;
use shared::TestBitflipProgramClient;
use shared::create_admin_keypair;
use shared::create_authority_keypair;
use shared::create_runner;
use shared::create_runner_with_accounts;
use shared::create_treasury_keypair;
use solana_sdk::account::AccountSharedData;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use test_log::test;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wallet_standard_wallets::MemoryWallet;

mod shared;

#[test(tokio::test)]
async fn initialize_config_test() -> anyhow::Result<()> {
	let admin_keypair = create_admin_keypair();
	let admin = admin_keypair.pubkey();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let system_program = system_program::id();
	let (config, config_bump) = get_pda_config();
	let (treasury, treasury_bump) = get_pda_treasury();
	let mut runner = create_runner().await;
	let rpc = runner.rpc();
	let wallet = MemoryWallet::new(rpc.clone(), &[admin_keypair.insecure_clone()]);
	let bitflip_program_client: TestBitflipProgramClient = BitflipProgramClient::builder()
		.rpc(rpc.clone())
		.wallet(wallet)
		.build()
		.into();

	let initialize_config_request = bitflip_program_client
		.initialize_config()
		.args(InitializeConfigProps { authority })
		.accounts(InitializeConfig {
			config,
			admin,
			treasury,
			system_program,
		})
		.signers(vec![&admin_keypair])
		.build();

	let simulation = initialize_config_request
		.sign_and_simulate_transaction()
		.await?;

	log::info!("units_consumed: {simulation:#?}");
	check!(simulation.value.units_consumed.unwrap() < 75_000);

	let signature = initialize_config_request
		.sign_and_send_transaction()
		.await?;

	let config_state_account = bitflip_program_client
		.account::<ConfigState>(&config)
		.await?;

	insta::assert_yaml_snapshot!(config_state_account);

	Ok(())
}

fn create_config_state() -> AccountSharedData {
	let authority = create_authority_keypair().pubkey();
	let (_, bump) = get_pda_config();
	let (_, treasury_bump) = get_pda_treasury();
	let state = InitializeConfigProps { authority }.into_launchpad_state(bump, treasury_bump);

	AccountSharedData::from_anchor_data(state, ID_CONST)
}
