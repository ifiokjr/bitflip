use std::collections::HashMap;
use std::hash::RandomState;

use anchor_lang::system_program;
use anchor_lang::AnchorSerialize;
use anchor_lang::Discriminator;
use anyhow::Result;
use bitflip_legacy_client::get_pda_config;
use bitflip_legacy_client::get_pda_game;
use bitflip_legacy_client::get_pda_game_nonce;
use bitflip_legacy_client::get_pda_mint;
use bitflip_legacy_client::get_pda_section_data;
use bitflip_legacy_client::get_pda_section_state;
use bitflip_legacy_client::get_pda_treasury;
use bitflip_legacy_client::get_section_token_account;
use bitflip_legacy_client::BitflipLegacyProgramClient;
use bitflip_legacy_program::get_token_amount;
use bitflip_legacy_program::ConfigState;
use bitflip_legacy_program::GameState;
use bitflip_legacy_program::SectionData;
use bitflip_legacy_program::SectionState;
use bitflip_legacy_program::BITFLIP_SECTION_LENGTH;
use bitflip_legacy_program::ID_CONST;
use bitflip_legacy_program::MINIMUM_FLIPS_PER_SECTION;
use bitflip_legacy_program::TOKENS_PER_SECTION;
use bitflip_legacy_program::TOKEN_DECIMALS;
use solana_sdk::account::AccountSharedData;
use solana_sdk::account::WritableAccount;
use solana_sdk::account_utils::StateMut;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::hash::Hash;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::nonce;
use solana_sdk::nonce::state::DurableNonce;
use solana_sdk::program_pack::Pack;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use test_utils::SECRET_KEY_ADMIN;
use test_utils::SECRET_KEY_AUTHORITY;
use test_utils::SECRET_KEY_TREASURY;
use test_utils::SECRET_KEY_WALLET;
use test_utils_solana::anchor_processor;
use test_utils_solana::solana_sdk::account::Account;
use test_utils_solana::FromAnchorData;
use test_utils_solana::ProgramTest;
use test_utils_solana::TestRpcProvider;
use wallet_standard_wallets::MemoryWallet;
use wasm_client_anchor::prelude::*;
use wasm_client_solana::SolanaRpcClient;

#[cfg(feature = "test_validator")]
const DEVENV_ROOT: &str = env!("DEVENV_ROOT");

/// Add the anchor program to the project.
pub(crate) fn create_program_test() -> ProgramTest {
	ProgramTest::new(
		"bitflip",
		ID_CONST,
		anchor_processor!(bitflip_legacy_program),
	)
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
		.program_path(format!(
			"{DEVENV_ROOT}/target/deploy/bitflip_legacy_program.so"
		))
		.build();
	let props = test_utils_solana::TestValidatorRunnerProps::builder()
		.programs(vec![launchpad_program])
		.pubkeys(vec![
			create_admin_keypair().pubkey(),
			create_authority_keypair().pubkey(),
			create_treasury_keypair().pubkey(),
			create_wallet_keypair().pubkey(),
		])
		.commitment(CommitmentLevel::Finalized)
		.accounts(accounts)
		.build();

	test_utils_solana::TestValidatorRunner::run(props).await
}

pub(crate) async fn create_program_context_with_factory<F: Fn(&mut ProgramTest)>(
	factory: F,
) -> Result<TestRpcProvider> {
	let mut program_test = create_program_test();

	factory(&mut program_test);
	program_test.add_account(
		create_admin_keypair().pubkey(),
		Account {
			lamports: sol_to_lamports(10.0),
			..Account::default()
		},
	);
	program_test.add_account(
		create_authority_keypair().pubkey(),
		Account {
			lamports: sol_to_lamports(10.0),
			..Account::default()
		},
	);
	program_test.add_account(
		create_wallet_keypair().pubkey(),
		Account {
			lamports: sol_to_lamports(10.0),
			..Account::default()
		},
	);

	let context = program_test.start_with_context().await;

	Ok(context.into())
}

/// The program client using the admin wallet account
pub fn get_admin_program(rpc: &SolanaRpcClient) -> TestBitflipLegacyProgramClient {
	get_program(rpc, &create_admin_keypair())
}

pub fn get_authority_program(rpc: &SolanaRpcClient) -> TestBitflipLegacyProgramClient {
	get_program(rpc, &create_authority_keypair())
}

pub fn get_wallet_program(rpc: &SolanaRpcClient) -> TestBitflipLegacyProgramClient {
	get_program(rpc, &create_wallet_keypair())
}

/// A program client using a custom payer.
pub fn get_program(rpc: &SolanaRpcClient, payer: &Keypair) -> TestBitflipLegacyProgramClient {
	let wallet = MemoryWallet::new(rpc.clone(), &[payer.insecure_clone()]);

	TestBitflipLegacyProgramClient::builder()
		.wallet(wallet)
		.rpc(rpc.clone())
		.build()
		.into()
}

pub fn create_config_state(mint_bump: Option<u8>) -> AccountSharedData {
	let authority = create_authority_keypair().pubkey();
	let config_bump = get_pda_config().1;
	let treasury_bump = get_pda_treasury().1;
	let mut state = ConfigState::new(authority, config_bump, treasury_bump);

	if let Some(mint_bump) = mint_bump {
		state.mint_bump = mint_bump;
	}

	state.into_account_shared_data()
}

pub fn create_game_state(
	game_index: u8,
	section_index: u8,
	start_time: i64,
	access_expiry: i64,
) -> (AccountSharedData, AccountSharedData, Keypair, Keypair) {
	let game_bump = get_pda_game(game_index).1;
	let game_nonce_bump = get_pda_game_nonce(game_index).1;
	let (access_signer, refresh_signer) = (Keypair::new(), Keypair::new());
	let game_state = GameState::builder()
		.access_signer(access_signer.pubkey())
		.refresh_signer(refresh_signer.pubkey())
		.start_time(start_time)
		.game_index(game_index)
		.bump(game_bump)
		.nonce_bump(game_nonce_bump)
		.access_expiry(access_expiry)
		.section_index(section_index)
		.build();

	let state = nonce::State::new_initialized(
		&access_signer.pubkey(),
		DurableNonce::from_blockhash(&Hash::default()),
		100_000,
	);
	log::info!("game_state: {game_state:#?}");
	let versioned_state = nonce::state::Versions::new(state);
	let space = nonce::State::size();
	let rent_sysvar = Rent::default();
	let lamports = rent_sysvar.minimum_balance(space);
	let mut game_nonce_account = AccountSharedData::new(lamports, space, &system_program::ID);
	game_nonce_account.set_state(&versioned_state).unwrap();

	(
		game_state.into_account_shared_data(),
		game_nonce_account,
		access_signer,
		refresh_signer,
	)
}

pub fn create_section_state(
	game_index: u8,
	next_section_index: u8,
) -> HashMap<Pubkey, AccountSharedData> {
	let mint = get_pda_mint().0;
	let mut map = HashMap::new();

	for section_index in 0..next_section_index {
		let (section, section_bump) = get_pda_section_state(game_index, section_index);
		let (section_data, section_data_bump) = get_pda_section_data(game_index, section_index);
		let section_token_account = get_section_token_account(game_index, section_index);

		let mut section_state = SectionState::new(
			Pubkey::new_unique(),
			section_index,
			section_bump,
			section_data_bump,
		);
		section_state.flips = MINIMUM_FLIPS_PER_SECTION;
		map.insert(section, section_state.into_account_shared_data());

		let section_data_account = {
			let section_data = SectionData {
				data: [0; BITFLIP_SECTION_LENGTH],
			};
			let mut data = vec![];
			data.append(&mut SectionData::DISCRIMINATOR.to_vec());
			data.append(&mut section_data.to_bytes());
			let rent = Rent::default().minimum_balance(SectionData::space());
			AccountSharedData::create(rent, data, ID_CONST, false, u64::MAX)
		};
		map.insert(section_data, section_data_account);

		let amount = get_token_amount(TOKENS_PER_SECTION, TOKEN_DECIMALS).unwrap();
		let token_account_state = spl_token_2022::state::Account {
			mint,
			owner: section,
			amount,
			delegate: None.into(),
			state: spl_token_2022::state::AccountState::Initialized,
			is_native: None.into(),
			delegated_amount: 0,
			close_authority: None.into(),
		};
		let token_account_data = {
			let mut buf = vec![0u8; spl_token_2022::state::Account::LEN];
			token_account_state.pack_into_slice(&mut buf[..]);
			buf
		};
		let lamports = Rent::default().minimum_balance(token_account_data.len());
		map.insert(
			section_token_account,
			AccountSharedData::create(
				lamports,
				token_account_data,
				spl_token_2022::ID,
				false,
				u64::MAX,
			),
		);
	}

	map
}

pub type TestBitflipLegacyProgramClient = BitflipLegacyProgramClient<MemoryWallet>;

pub trait IntoAccountSharedData: AnchorSerialize + Discriminator {
	fn into_account_shared_data(self) -> AccountSharedData;
	fn into_account(self) -> Account;
}

impl<T: AnchorSerialize + Discriminator> IntoAccountSharedData for T {
	fn into_account_shared_data(self) -> AccountSharedData {
		AccountSharedData::from_anchor_data(self, ID_CONST)
	}

	fn into_account(self) -> Account {
		self.into_account_shared_data().into()
	}
}
