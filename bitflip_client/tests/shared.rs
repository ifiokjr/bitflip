use std::collections::HashMap;
use std::hash::RandomState;

use anyhow::Result;
use bitflip_client::BitflipProgramClient;
use bitflip_client::get_pda_config;
use bitflip_client::get_pda_treasury;
use bitflip_program::ID_CONST;
use bitflip_program::InitializeConfigProps;
use solana_sdk::account::AccountSharedData;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use test_utils::SECRET_KEY_ADMIN;
use test_utils::SECRET_KEY_AUTHORITY;
use test_utils::SECRET_KEY_TREASURY;
use test_utils::SECRET_KEY_WALLET;
use test_utils_solana::FromAnchorData;
use test_utils_solana::ProgramTest;
use test_utils_solana::ProgramTestContext;
use test_utils_solana::anchor_processor;
use test_utils_solana::solana_sdk::account::Account;
use wallet_standard_wallets::MemoryWallet;
use wasm_client_solana::LOCALNET;
use wasm_client_solana::SimulateTransactionResponse;
use wasm_client_solana::SolanaRpcClient;

#[cfg(feature = "test_validator")]
const DEVENV_ROOT: &str = env!("DEVENV_ROOT");

/// Add the anchor program to the project.
pub(crate) fn create_program_test() -> ProgramTest {
	ProgramTest::new("bitflip", ID_CONST, anchor_processor!(bitflip_program))
}

pub fn create_admin_keypair() -> Keypair {
	Keypair::from_bytes(&SECRET_KEY_ADMIN).unwrap()
}

pub fn create_authority_keypair() -> Keypair {
	Keypair::from_bytes(&SECRET_KEY_AUTHORITY).unwrap()
}

pub fn create_treasury_keypair() -> Keypair {
	Keypair::from_bytes(&SECRET_KEY_TREASURY).unwrap()
}

pub fn create_wallet_keypair() -> Keypair {
	Keypair::from_bytes(&SECRET_KEY_WALLET).unwrap()
}

#[cfg(feature = "test_validator")]
pub async fn create_runner() -> test_utils_solana::TestValidatorRunner {
	create_runner_with_accounts(HashMap::new()).await
}

#[cfg(feature = "test_validator")]
pub async fn create_runner_with_accounts(
	accounts: HashMap<Pubkey, AccountSharedData, RandomState>,
) -> test_utils_solana::TestValidatorRunner {
	let launchpad_program = test_utils_solana::TestProgramInfo::builder()
		.program_id(ID_CONST)
		.program_path(format!("{DEVENV_ROOT}/target/deploy/bitflip_program.so"))
		.build();
	let props = test_utils_solana::TestValidatorRunnerProps::builder()
		.programs(vec![launchpad_program])
		.pubkeys(vec![
			create_admin_keypair().pubkey(),
			create_authority_keypair().pubkey(),
			create_treasury_keypair().pubkey(),
		])
		.commitment(CommitmentLevel::Finalized)
		.accounts(accounts)
		.build();

	test_utils_solana::TestValidatorRunner::run(props).await
}

pub(crate) fn create_rpc() -> SolanaRpcClient {
	SolanaRpcClient::new(LOCALNET)
}

pub(crate) async fn create_program_context_with_factory<F: Fn(&mut ProgramTest)>(
	factory: F,
) -> Result<ProgramTestContext> {
	let mut program_test = create_program_test();

	factory(&mut program_test);

	program_test.add_account(create_admin_keypair().pubkey(), Account {
		lamports: sol_to_lamports(10.0),
		..Account::default()
	});

	program_test.add_account(create_authority_keypair().pubkey(), Account {
		lamports: sol_to_lamports(10.0),
		..Account::default()
	});

	let context = program_test.start_with_context().await;

	Ok(context)
}

pub async fn create_program_context() -> Result<ProgramTestContext> {
	create_program_context_with_factory(|_| {}).await
}

/// The program client using the admin wallet account
pub fn get_admin_program(rpc: &SolanaRpcClient) -> TestBitflipProgramClient {
	get_program(rpc, &create_admin_keypair())
}

/// A program client using a custom payer.
pub fn get_program(rpc: &SolanaRpcClient, payer: &Keypair) -> TestBitflipProgramClient {
	let wallet = MemoryWallet::new(rpc.clone(), &[payer.insecure_clone()]);

	TestBitflipProgramClient::builder()
		.wallet(wallet)
		.rpc(rpc.clone())
		.build()
		.into()
}

pub fn create_config_state(mint_bump: Option<u8>) -> AccountSharedData {
	let authority = create_authority_keypair().pubkey();
	let (_, bump) = get_pda_config();
	let (_, treasury_bump) = get_pda_treasury();
	let mut state = InitializeConfigProps { authority }.into_launchpad_state(bump, treasury_bump);

	if let Some(mint_bump) = mint_bump {
		state.mint_bump = mint_bump;
	}

	AccountSharedData::from_anchor_data(state, ID_CONST)
}

pub type TestBitflipProgramClient = BitflipProgramClient<MemoryWallet>;
