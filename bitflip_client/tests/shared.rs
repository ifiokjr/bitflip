use anyhow::Result;
use bitflip_client::BitflipProgram;
use bitflip_program::ID_CONST;
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use test_utils::SECRET_KEY_ADMIN;
use test_utils::SECRET_KEY_AUTHORITY;
use test_utils::SECRET_KEY_TREASURY;
use test_utils_solana::anchor_processor;
use test_utils_solana::solana_sdk::account::Account;
use test_utils_solana::ProgramTest;
use test_utils_solana::ProgramTestContext;
use wallet_standard_wallets::MemoryWallet;
use wasm_client_solana::SolanaRpcClient;
use wasm_client_solana::LOCALNET;

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

pub struct BitflipProgramTest {
	pub rpc: SolanaRpcClient,
}

impl BitflipProgramTest {
	pub async fn run_with_program_factory<F: Fn(&BitflipProgramTest) -> ProgramTest>(
		factory: F,
	) -> Result<(Self, ProgramTestContext)> {
		let admin_keypair = create_admin_keypair();
		let rpc = SolanaRpcClient::new_with_commitment(
			LOCALNET,
			CommitmentConfig {
				commitment: CommitmentLevel::Finalized,
			},
		);
		let data = Self { rpc };
		let mut program_test = factory(&data);

		program_test.add_account(
			admin_keypair.pubkey(),
			Account {
				lamports: 1_000_000_000_000,
				..Account::default()
			},
		);

		let context = program_test.start_with_context().await;

		Ok((data, context))
	}

	pub async fn run() -> Result<(Self, ProgramTestContext)> {
		Self::run_with_program_factory(|_| create_program_test()).await
	}

	/// The program using the admin wallet account
	pub fn admin_program(&self) -> TestBitBitflipProgram {
		self.program(&create_admin_keypair())
	}

	/// A program using a custom payer.
	pub fn program(&self, payer: &Keypair) -> TestBitBitflipProgram {
		let wallet = MemoryWallet::new(self.rpc.clone(), &[payer.insecure_clone()]);

		TestBitBitflipProgram::builder()
			.wallet(wallet)
			.rpc(self.rpc.clone())
			.build()
			.into()
	}
}

pub type TestBitBitflipProgram = BitflipProgram<MemoryWallet>;

#[macro_export]
macro_rules! assert_simulated_error {
	($result:ident, $expected_error:path) => {{
		let error_name = $expected_error.name();

		::assert2::check!(
			$result.result.unwrap().is_err(),
			"the simulation was expected to error"
		);
		::assert2::check!(
			$result
				.simulation_details
				.unwrap()
				.logs
				.iter()
				.any(|log| log.contains(format!("Error Code: {error_name}").as_str())),
			"`{:#?}` not found in logs",
			$expected_error,
		);
	}};
}

#[macro_export]
macro_rules! assert_metadata_error {
	($result:ident, $expected_error:path) => {{
		let error_name = $expected_error.name();

		::assert2::check!(
			$result.result.is_err(),
			"the simulation was expected to error"
		);
		::assert2::check!(
			$result
				.metadata
				.unwrap()
				.log_messages
				.iter()
				.any(|log| log.contains(format!("Error Code: {error_name}").as_str())),
			"`{:#?}` not found in logs",
			$expected_error,
		);
	}};
}
