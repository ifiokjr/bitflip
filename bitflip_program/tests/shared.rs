#![cfg(feature = "client")]

use std::collections::HashMap;
use std::fs;
use std::hash::RandomState;
use std::path::Path;

use anyhow::Context;
use bitflip_program::get_pda_config;
use bitflip_program::get_pda_game;
use bitflip_program::get_pda_mint;
use bitflip_program::get_pda_section;
use bitflip_program::get_pda_treasury;
use bitflip_program::get_section_token_account;
use bitflip_program::get_token_amount;
use bitflip_program::get_treasury_token_account;
use bitflip_program::ConfigState;
use bitflip_program::GameState;
use bitflip_program::SectionState;
use bitflip_program::TokenMember;
use bitflip_program::EARNED_TOKENS_PER_SECTION;
use bitflip_program::ID;
use bitflip_program::MINIMUM_FLIPS_PER_SECTION;
use rstest::fixture;
use serde::Deserialize;
use serde::Serialize;
use solana_sdk::account::AccountSharedData;
use solana_sdk::account::WritableAccount;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::native_token::sol_to_lamports;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::rent::Rent;
use solana_sdk::signature::Keypair;
use solana_sdk::signature::Signer;
use spl_pod::bytemuck::pod_get_packed_len;
use spl_pod::primitives::PodBool;
use spl_token_2022::extension::group_member_pointer::GroupMemberPointer;
use spl_token_2022::extension::group_pointer::GroupPointer;
use spl_token_2022::extension::metadata_pointer::MetadataPointer;
use spl_token_2022::extension::mint_close_authority::MintCloseAuthority;
use spl_token_2022::extension::BaseStateWithExtensions;
use spl_token_2022::extension::BaseStateWithExtensionsMut;
use spl_token_2022::extension::ExtensionType;
use spl_token_2022::extension::PodStateWithExtensionsMut;
use spl_token_2022::pod::PodAccount;
use spl_token_2022::pod::PodCOption;
use spl_token_2022::pod::PodMint;
use spl_token_group_interface::state::TokenGroup;
use spl_token_group_interface::state::TokenGroupMember;
use spl_token_metadata_interface::state::TokenMetadata;
use spl_type_length_value::variable_len_pack::VariableLenPack;
use steel::*;
use test_utils::SECRET_KEY_ADMIN;
use test_utils::SECRET_KEY_AUTHORITY;
use test_utils::SECRET_KEY_TREASURY;
use test_utils::SECRET_KEY_WALLET;
use test_utils_solana::processor;
use test_utils_solana::solana_sdk::account::Account;
use test_utils_solana::ProgramTest;
use test_utils_solana::TestRpcProvider;
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

pub(crate) async fn create_program_context_with_factory<
	F: Fn(&mut ProgramTest) -> anyhow::Result<()>,
>(
	factory: F,
) -> anyhow::Result<TestRpcProvider> {
	let mut program_test = create_program_test();

	factory(&mut program_test)?;
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
	let (_, mint_bit_bump) = get_pda_mint(TokenMember::Bit);
	let (_, mint_kibibit_bump) = get_pda_mint(TokenMember::Kibibit);
	let (_, mint_mebibit_bump) = get_pda_mint(TokenMember::Mebibit);
	let (_, mint_gibibit_bump) = get_pda_mint(TokenMember::Gibibit);
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

	map.insert(
		treasury,
		AccountSharedData::new(Rent::default().minimum_balance(0), 0, &system_program::ID),
	);
	map.insert(config, config_state_account);

	map
}

pub fn create_token_accounts(
	with_group: bool,
) -> anyhow::Result<HashMap<Pubkey, AccountSharedData>> {
	let mut map = HashMap::new();
	let treasury = get_pda_treasury().0;
	for ii in 0..4 {
		let member = TokenMember::try_from(ii)?;
		let mint = get_pda_mint(member).0;
		let mint_data = create_mint_data(member, treasury, with_group)?;
		map.insert(
			mint,
			AccountSharedData::create(
				Rent::default().minimum_balance(mint_data.len()),
				mint_data,
				spl_token_2022::ID,
				false,
				u64::MAX,
			),
		);

		let token_amount = get_token_amount(member.supply(), member.decimals())?;
		let treasury_token_account = get_treasury_token_account(member);
		let treasury_token_account_data =
			create_token_account_data(member, treasury, treasury, token_amount)?;
		map.insert(
			treasury_token_account,
			AccountSharedData::create(
				Rent::default().minimum_balance(treasury_token_account_data.len()),
				treasury_token_account_data,
				spl_token_2022::ID,
				false,
				u64::MAX,
			),
		);
	}

	Ok(map)
}

fn create_mint_data(
	member: TokenMember,
	treasury: Pubkey,
	with_group: bool,
) -> anyhow::Result<Vec<u8>> {
	let mint = get_pda_mint(member).0;
	let token_metadata = TokenMetadata {
		name: member.name().into(),
		symbol: member.symbol().into(),
		uri: member.uri().into(),
		update_authority: Some(treasury).try_into()?,
		mint,
		additional_metadata: vec![],
	};
	let mut mint_space = member.initial_mint_space()? + 4 + token_metadata.get_packed_len()?;

	if with_group {
		mint_space += if member.parent().is_none() {
			pod_get_packed_len::<TokenGroup>()
		} else {
			pod_get_packed_len::<TokenGroupMember>()
		};
	}

	let mut mint_data = vec![0u8; mint_space];
	let mut mint_state =
		PodStateWithExtensionsMut::<PodMint>::unpack_uninitialized(&mut mint_data)?;
	let metadata_pointer = mint_state.init_extension::<MetadataPointer>(true)?;
	metadata_pointer.metadata_address = Some(mint).try_into()?;
	metadata_pointer.authority = Some(treasury).try_into()?;

	let mint_close_pointer = mint_state.init_extension::<MintCloseAuthority>(true)?;
	mint_close_pointer.close_authority = Some(treasury).try_into()?;

	if member.parent().is_none() {
		let group_pointer = mint_state.init_extension::<GroupPointer>(true)?;
		group_pointer.group_address = Some(mint).try_into()?;
		group_pointer.authority = Some(treasury).try_into()?;
	} else {
		let member_pointer = mint_state.init_extension::<GroupMemberPointer>(true)?;
		member_pointer.member_address = Some(mint).try_into()?;
		member_pointer.authority = Some(treasury).try_into()?;
	}

	// token metadata
	mint_state.init_variable_len_extension(&token_metadata, false)?;

	if with_group {
		if let Some(parent) = member.parent() {
			// token group member
			let parent_mint = get_pda_mint(parent).0;
			let member_number: u64 = ((member as u8) - 1) as u64;
			let new_extension = TokenGroupMember::new(&mint, &parent_mint, member_number);
			let extension = mint_state.init_extension::<TokenGroupMember>(true)?;
			*extension = new_extension;
		} else {
			// token group
			let new_extension = TokenGroup::new(&mint, Some(treasury).try_into()?, 8);
			let extension = mint_state.init_extension::<TokenGroup>(true)?;
			*extension = new_extension;
		}
	}

	*mint_state.base = PodMint {
		mint_authority: PodCOption::some(treasury),
		supply: member.supply().into(),
		decimals: member.decimals(),
		is_initialized: PodBool::from_bool(true),
		freeze_authority: PodCOption::some(treasury),
	};
	mint_state.init_account_type()?;

	Ok(mint_data)
}

fn create_token_account_data(
	member: TokenMember,
	owner: Pubkey,
	close_authority: Pubkey,
	amount: u64,
) -> anyhow::Result<Vec<u8>> {
	let mint = get_pda_mint(member).0;
	let treasury_bit_token_account_state = PodAccount {
		mint,
		owner,
		amount: amount.into(),
		delegate: PodCOption::none(),
		state: spl_token_2022::state::AccountState::Initialized.into(),
		is_native: PodCOption::none(),
		delegated_amount: 0.into(),
		close_authority: PodCOption::some(close_authority),
	};
	let account_space =
		ExtensionType::try_calculate_account_len::<spl_token_2022::state::Account>(&[
			ExtensionType::ImmutableOwner,
		])?;
	let mut account_data = vec![0u8; account_space];
	let mut account =
		PodStateWithExtensionsMut::<PodAccount>::unpack_uninitialized(&mut account_data)?;

	account.init_account_extension_from_type(ExtensionType::ImmutableOwner)?;

	*account.base = treasury_bit_token_account_state;
	account.init_account_type()?;

	Ok(account_data)
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
) -> anyhow::Result<HashMap<Pubkey, AccountSharedData>> {
	let mut map = HashMap::new();
	let game = get_pda_game(game_index).0;

	for section_index in 0..next_section_index {
		let (section, section_bump) = get_pda_section(game_index, section_index);
		let section_token_account =
			get_section_token_account(game_index, section_index, TokenMember::Bit);
		let mut section_state = SectionState::new(owner, game_index, section_index, section_bump);

		if set_minimum_flips {
			section_state.flips = MINIMUM_FLIPS_PER_SECTION.into();
		}

		map.insert(section, section_state.to_account_shared_data());

		let member = TokenMember::Bit;
		let token_amount = get_token_amount(EARNED_TOKENS_PER_SECTION, member.decimals()).unwrap();
		let section_token_account_data =
			create_token_account_data(member, section, game, token_amount)?;
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

	Ok(map)
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

#[cfg(feature = "test_validator")]
#[derive(Debug, Serialize, Deserialize)]
struct ComputeUnits {
	instructions: indexmap::IndexMap<String, InstructionMetrics>,
	version: String,
}

#[cfg(feature = "test_validator")]
#[derive(Debug, Serialize, Deserialize)]
struct InstructionMetrics {
	compute_units: u64,
	rounded_compute_units: u64,
	description: String,
}

#[cfg(feature = "test_validator")]
pub fn save_compute_units(name: &str, compute_units: u64, description: &str) -> anyhow::Result<()> {
	let path = "compute_units.json";

	// Read existing or create new
	let mut data: ComputeUnits = fs::read_to_string(path)
		.context("could not read file")
		.and_then(|s| serde_json::from_str(&s).context("could not parse JSON"))
		.unwrap_or(ComputeUnits {
			instructions: indexmap::indexmap! {},
			version: "0.0.0".to_string(),
		});

	data.instructions.insert(
		name.to_string(),
		InstructionMetrics {
			compute_units,
			rounded_compute_units: bitflip_program::round_compute_units_up(compute_units),
			description: description.to_string(),
		},
	);

	data.instructions.sort_unstable_keys();

	// Write back to file
	fs::write(path, serde_json::to_string_pretty(&data)?)?;

	// Format the JSON file using dprint
	std::process::Command::new("dprint")
		.arg("fmt")
		.arg("compute_units.json")
		.output()
		.context("Failed to run dprint fmt command")?;

	Ok(())
}

#[cfg(feature = "test_validator")]
pub fn generate_compute_constants() -> std::io::Result<()> {
	let data: ComputeUnits = serde_json::from_str(&fs::read_to_string("compute_units.json")?)?;

	let mut output = String::from("// Auto-generated compute unit constants\n\n");

	for (name, metrics) in data.instructions {
		let const_name = name.to_uppercase();
		output.push_str(&format!(
			"pub const {}_COMPUTE_UNITS: u64 = {};\n",
			const_name, metrics.rounded_compute_units
		));
	}

	fs::write(
		Path::new(&std::env::var("OUT_DIR").unwrap()).join("compute_units.rs"),
		output,
	)?;

	Ok(())
}

#[macro_export]
macro_rules! set_snapshot_suffix {
	($($expr:expr),*) => {
			let mut settings = insta::Settings::clone_current();
			settings.set_snapshot_suffix(format!($($expr,)*));
			let _guard = settings.bind_to_scope();
	}
}

#[fixture]
pub fn testname() -> String {
	std::thread::current()
		.name()
		.unwrap()
		.split("::")
		.last()
		.unwrap()
		.split("_")
		.skip(2)
		.collect::<Vec<&str>>()
		.join("_")
		.to_string()
}
