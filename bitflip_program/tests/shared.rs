#![cfg(feature = "client")]

use std::collections::HashMap;
use std::hash::RandomState;

use anyhow::Result;
use bitflip_program::ConfigState;
use bitflip_program::EARNED_TOKENS_PER_SECTION;
use bitflip_program::GameState;
use bitflip_program::ID;
use bitflip_program::MINIMUM_FLIPS_PER_SECTION;
use bitflip_program::SectionState;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::TOTAL_BIT_TOKENS;
use bitflip_program::get_mint_space;
use bitflip_program::get_pda_config;
use bitflip_program::get_pda_game;
use bitflip_program::get_pda_mint_bit;
use bitflip_program::get_pda_mint_gibibit;
use bitflip_program::get_pda_mint_kibibit;
use bitflip_program::get_pda_mint_mebibit;
use bitflip_program::get_pda_section;
use bitflip_program::get_pda_treasury;
use bitflip_program::get_section_bit_token_account;
use bitflip_program::get_token_amount;
use bitflip_program::get_treasury_bit_token_account;
use solana_sdk::account::AccountSharedData;
use solana_sdk::account::WritableAccount;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use spl_pod::primitives::PodBool;
use spl_token_2022::extension::BaseStateWithExtensionsMut;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::extension::PodStateWithExtensionsMut;
use spl_token_2022::extension::group_pointer::GroupPointer;
use spl_token_2022::extension::metadata_pointer::MetadataPointer;
use spl_token_2022::extension::mint_close_authority::MintCloseAuthority;
use spl_token_2022::pod::PodAccount;
use spl_token_2022::pod::PodCOption;
use spl_token_2022::pod::PodMint;
use steel::*;
use test_utils::SECRET_KEY_ADMIN;
use test_utils::SECRET_KEY_AUTHORITY;
use test_utils::SECRET_KEY_TREASURY;
use test_utils::SECRET_KEY_WALLET;
use test_utils_solana::ProgramTest;
use test_utils_solana::TestRpcProvider;
use test_utils_solana::processor;
use test_utils_solana::solana_sdk::account::Account;
use wallet_standard_wallets::MemoryWallet;
use wasm_client_solana::SolanaRpcClient;

#[cfg(feature = "test_validator")]
const DEVENV_ROOT: &str = env!("DEVENV_ROOT");

pub trait ToRpcClient {
	fn to_rpc(&self) -> SolanaRpcClient;
}

impl ToRpcClient for TestRpcProvider {
	fn to_rpc(&self) -> SolanaRpcClient {
		self.to_rpc_client()
	}
}

#[cfg(feature = "test_validator")]
impl ToRpcClient for test_utils_solana::TestValidatorRunner {
	fn to_rpc(&self) -> SolanaRpcClient {
		self.rpc().clone()
	}
}

/// Add the anchor program to the project.
pub(crate) fn create_program_test() -> ProgramTest {
	ProgramTest::new(
		"bitflip",
		ID,
		processor!(bitflip_program::process_instruction),
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
		.program_id(ID)
		.program_path(format!("{DEVENV_ROOT}/target/deploy/bitflip_program.so"))
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
	program_test.add_account(create_admin_keypair().pubkey(), Account {
		lamports: sol_to_lamports(10.0),
		..Account::default()
	});
	program_test.add_account(create_authority_keypair().pubkey(), Account {
		lamports: sol_to_lamports(10.0),
		..Account::default()
	});
	program_test.add_account(create_wallet_keypair().pubkey(), Account {
		lamports: sol_to_lamports(10.0),
		..Account::default()
	});

	let context = program_test.start_with_context().await;

	Ok(context.into())
}

pub async fn create_program_context() -> Result<TestRpcProvider> {
	create_program_context_with_factory(|_| {}).await
}

/// The program client using the admin wallet account
pub fn get_admin_wallet(rpc: &SolanaRpcClient) -> MemoryWallet {
	get_wallet(rpc, &create_admin_keypair())
}

pub fn get_authority_wallet(rpc: &SolanaRpcClient) -> MemoryWallet {
	get_wallet(rpc, &create_authority_keypair())
}

pub fn get_wallet_wallet(rpc: &SolanaRpcClient) -> MemoryWallet {
	get_wallet(rpc, &create_wallet_keypair())
}

/// A program client using a custom payer.
pub fn get_wallet(rpc: &SolanaRpcClient, payer: &Keypair) -> MemoryWallet {
	MemoryWallet::new(rpc.clone(), &[payer.insecure_clone()])
}

pub fn create_config_accounts() -> HashMap<Pubkey, AccountSharedData> {
	let mut map = HashMap::new();
	let authority = create_authority_keypair().pubkey();
	let config_bump = get_pda_config().1;
	let (treasury, treasury_bump) = get_pda_treasury();
	let (mint_bit, mint_bit_bump) = get_pda_mint_bit();
	let mint_kibibit_bump = get_pda_mint_kibibit().1;
	let mint_mebibit_bump = get_pda_mint_mebibit().1;
	let mint_gibibit_bump = get_pda_mint_gibibit().1;
	let config = get_pda_config().0;
	let config_state_account = ConfigState::new(
		authority,
		config_bump,
		treasury_bump,
		mint_bit_bump,
		mint_kibibit_bump,
		mint_mebibit_bump,
		mint_gibibit_bump,
	)
	.to_account_shared_data();

	let mint_bit_data = create_bit_mint_data(treasury, mint_bit);
	let token_amount = get_token_amount(TOTAL_BIT_TOKENS, TOKEN_DECIMALS).unwrap();
	let treasury_bit_token_account = get_treasury_bit_token_account();

	let treasury_bit_token_account_data =
		// create_associated_token_account_data(mint_bit, treasury, token_amount, &mint_bit_data);
		create_associated_token_account_data(mint_bit, treasury, token_amount);

	map.insert(config, config_state_account);
	map.insert(
		mint_bit,
		AccountSharedData::create(
			Rent::default().minimum_balance(mint_bit_data.len()),
			mint_bit_data,
			spl_token_2022::ID,
			false,
			u64::MAX,
		),
	);
	map.insert(
		treasury_bit_token_account,
		AccountSharedData::create(
			Rent::default().minimum_balance(treasury_bit_token_account_data.len()),
			treasury_bit_token_account_data,
			spl_token_2022::ID,
			false,
			u64::MAX,
		),
	);

	map
}

fn create_associated_token_account_data(
	mint_bit: Pubkey,
	treasury: Pubkey,
	token_amount: u64,
	// mint_bit_data: &[u8],
) -> Vec<u8> {
	let treasury_bit_token_account_state = PodAccount {
		mint: mint_bit,
		owner: treasury,
		amount: token_amount.into(),
		delegate: PodCOption::none(),
		state: spl_token_2022::state::AccountState::Initialized.into(),
		is_native: PodCOption::none(),
		delegated_amount: 0.into(),
		close_authority: PodCOption::some(treasury),
	};
	// let mint = PodStateWithExtensions::<PodMint>::unpack(mint_bit_data).unwrap();
	// let mint_extensions = mint.get_extension_types().unwrap();
	// let mut account_extensions =
	// 	ExtensionType::get_required_init_account_extensions(&mint_extensions);
	// account_extensions.extend_from_slice(&[ExtensionType::ImmutableOwner]);
	// let account_space =
	// ExtensionType::try_calculate_account_len::<spl_token_2022::state::Account>(
	// 	&account_extensions,
	// )
	// .unwrap();
	let account_space =
		ExtensionType::try_calculate_account_len::<spl_token_2022::state::Account>(&[
			ExtensionType::ImmutableOwner,
		])
		.unwrap();
	let mut account_data = vec![0u8; account_space];
	let mut account =
		PodStateWithExtensionsMut::<PodAccount>::unpack_uninitialized(&mut account_data).unwrap();

	// for extension in account_extensions {
	// 	log::info!("extension: {:?}", extension);
	// 	account.init_account_extension_from_type(extension).unwrap();
	// }
	account
		.init_account_extension_from_type(ExtensionType::ImmutableOwner)
		.unwrap();

	*account.base = treasury_bit_token_account_state;
	account.init_account_type().unwrap();
	account_data
}

fn create_bit_mint_data(treasury: Pubkey, mint_bit: Pubkey) -> Vec<u8> {
	let mint_bit_state = PodMint {
		mint_authority: PodCOption::some(treasury),
		supply: TOTAL_BIT_TOKENS.into(),
		decimals: TOKEN_DECIMALS,
		is_initialized: PodBool::from_bool(true),
		freeze_authority: PodCOption::some(treasury),
	};
	let mut mint_data = vec![0u8; get_mint_space().unwrap()];
	let mut mint =
		PodStateWithExtensionsMut::<PodMint>::unpack_uninitialized(&mut mint_data).unwrap();
	let metadata_pointer = mint.init_extension::<MetadataPointer>(true).unwrap();
	metadata_pointer.metadata_address = Some(mint_bit).try_into().unwrap();
	metadata_pointer.authority = Some(treasury).try_into().unwrap();

	let mint_close_pointer = mint.init_extension::<MintCloseAuthority>(true).unwrap();
	mint_close_pointer.close_authority = Some(treasury).try_into().unwrap();

	let group_pointer = mint.init_extension::<GroupPointer>(true).unwrap();
	group_pointer.group_address = Some(mint_bit).try_into().unwrap();
	group_pointer.authority = Some(treasury).try_into().unwrap();

	*mint.base = mint_bit_state;

	mint.init_account_type().unwrap();
	mint_data
}

pub struct CreatedGameState {
	pub game_state_account: AccountSharedData,
	pub access_signer: Keypair,
	pub refresh_signer: Keypair,
	pub refresh_signer_account: AccountSharedData,
}

pub fn create_game_state(
	game_index: u8,
	section_index: u8,
	start_time: i64,
	access_expiry: i64,
) -> CreatedGameState {
	let game_bump = get_pda_game(game_index).1;
	let (access_signer, refresh_signer) = (Keypair::new(), Keypair::new());
	let game_state = GameState::builder()
		.access_signer(access_signer.pubkey())
		.refresh_signer(refresh_signer.pubkey())
		.start_time(start_time)
		.game_index(game_index)
		.bump(game_bump)
		.access_expiry(access_expiry)
		.section_index(section_index)
		.build();

	let lamports = Rent::default().minimum_balance(0) + 5_000_000;
	let refresh_signer_account = AccountSharedData::new(lamports, 0, &system_program::ID);

	CreatedGameState {
		game_state_account: game_state.to_account_shared_data(),
		access_signer,
		refresh_signer,
		refresh_signer_account,
	}
}

pub fn create_section_state(
	owner: Pubkey,
	game_index: u8,
	next_section_index: u8,
	set_minimum_flips: bool,
) -> HashMap<Pubkey, AccountSharedData> {
	let mint_bit = get_pda_mint_bit().0;
	let mut map = HashMap::new();

	for section_index in 0..next_section_index {
		let (section, section_bump) = get_pda_section(game_index, section_index);
		let section_token_account = get_section_bit_token_account(game_index, section_index);
		let mut section_state = SectionState::new(owner, game_index, section_index, section_bump);

		if set_minimum_flips {
			section_state.flips = MINIMUM_FLIPS_PER_SECTION.into();
		}

		map.insert(section, section_state.to_account_shared_data());

		let token_amount = get_token_amount(EARNED_TOKENS_PER_SECTION, TOKEN_DECIMALS).unwrap();
		let section_token_account_data =
			create_associated_token_account_data(mint_bit, section, token_amount);

		let lamports = Rent::default().minimum_balance(section_token_account_data.len());
		map.insert(
			section_token_account,
			AccountSharedData::create(
				lamports,
				section_token_account_data,
				spl_token_2022::ID,
				false,
				u64::MAX,
			),
		);
	}

	map
}

pub trait IntoAccountSharedData: Pod + Discriminator {
	fn to_account_shared_data(&self) -> AccountSharedData;
	fn to_account(&self) -> Account;
}

impl<T: Pod + Discriminator> IntoAccountSharedData for T {
	fn to_account_shared_data(&self) -> AccountSharedData {
		let mut bytes = Vec::new();
		let mut initial_bytes = [0u8; 8];
		initial_bytes[0] = T::discriminator();

		bytes.extend_from_slice(&initial_bytes);
		bytes.extend_from_slice(bytemuck::bytes_of(self));

		let rent = Rent::default().minimum_balance(bytes.len());

		AccountSharedData::create(rent, bytes, ID, false, u64::MAX)
	}

	fn to_account(&self) -> Account {
		self.to_account_shared_data().into()
	}
}
