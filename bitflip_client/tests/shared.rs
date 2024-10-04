use std::collections::HashMap;

use anyhow::Result;
use bitflip_client::BitflipProgramClient;
use bitflip_program::ID_CONST;
use solana_sdk::account::AccountSharedData;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use test_utils::SECRET_KEY_ADMIN;
use test_utils::SECRET_KEY_AUTHORITY;
use test_utils::SECRET_KEY_TREASURY;
use test_utils_solana::ProgramTest;
use test_utils_solana::ProgramTestContext;
use test_utils_solana::TestProgramInfo;
use test_utils_solana::TestValidatorRunner;
use test_utils_solana::TestValidatorRunnerProps;
use test_utils_solana::anchor_processor;
use test_utils_solana::solana_sdk::account::Account;
use wallet_standard_wallets::MemoryWallet;
use wasm_client_solana::LOCALNET;
use wasm_client_solana::SolanaRpcClient;

const DEVENV_ROOT: &str = env!("DEVENV_ROOT");

/// Add the anchor program to the project.
pub fn create_program_test() -> ProgramTest {
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

pub async fn create_runner() -> TestValidatorRunner {
	create_runner_with_accounts(HashMap::new()).await
}

pub async fn create_runner_with_accounts(
	accounts: HashMap<Pubkey, AccountSharedData>,
) -> TestValidatorRunner {
	let launchpad_program = TestProgramInfo::builder()
		.program_id(ID_CONST)
		.program_path(format!("{DEVENV_ROOT}/target/deploy/bitflip_program.so"))
		.build();
	let props = TestValidatorRunnerProps::builder()
		.programs(vec![launchpad_program])
		.pubkeys(vec![
			create_admin_keypair().pubkey(),
			create_authority_keypair().pubkey(),
			create_treasury_keypair().pubkey(),
		])
		.commitment(CommitmentLevel::Finalized)
		.accounts(accounts)
		.build();
	let runner = TestValidatorRunner::run(props).await;

	runner
}

pub struct BitflipProgramClientTest {
	pub rpc: SolanaRpcClient,
}

impl BitflipProgramClientTest {
	pub async fn run_with_program_factory<F: Fn(&BitflipProgramClientTest) -> ProgramTest>(
		factory: F,
	) -> Result<(Self, ProgramTestContext)> {
		let admin_keypair = create_admin_keypair();
		let rpc = SolanaRpcClient::new_with_commitment(LOCALNET, CommitmentConfig {
			commitment: CommitmentLevel::Finalized,
		});
		let data = Self { rpc };
		let mut program_test = factory(&data);

		program_test.add_account(admin_keypair.pubkey(), Account {
			lamports: 1_000_000_000_000,
			..Account::default()
		});

		let context = program_test.start_with_context().await;

		Ok((data, context))
	}

	pub async fn run() -> Result<(Self, ProgramTestContext)> {
		Self::run_with_program_factory(|_| create_program_test()).await
	}

	/// The program using the admin wallet account
	pub fn admin_program(&self) -> TestBitflipProgramClient {
		self.program(&create_admin_keypair())
	}

	/// A program using a custom payer.
	pub fn program(&self, payer: &Keypair) -> TestBitflipProgramClient {
		let wallet = MemoryWallet::new(self.rpc.clone(), &[payer.insecure_clone()]);

		TestBitflipProgramClient::builder()
			.wallet(wallet)
			.rpc(self.rpc.clone())
			.build()
			.into()
	}
}

pub type TestBitflipProgramClient = BitflipProgramClient<MemoryWallet>;
