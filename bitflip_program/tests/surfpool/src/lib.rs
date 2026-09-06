#![cfg(test)]

use pina_test::AccountMeta;
use pina_test::Keypair;
use pina_test::ProgramTest;
use pina_test::Pubkey;
use pina_test::Rent;
use pina_test::Signer;
use pina_test::TestError;
use program_under_test::BIT_GAME_COUNT;
use program_under_test::BIT_MINT_DECIMALS;
use program_under_test::BIT_SECTION_ALLOCATION_TOKENS;
use program_under_test::BIT_TOTAL_SUPPLY_TOKENS;
use program_under_test::BitflipAccountType;
use program_under_test::BitflipError;
use program_under_test::BitflipInstruction;
use program_under_test::CONFIG_VERSION;
use program_under_test::ConfigState;
use program_under_test::DEFAULT_CLAIM_PRICE_LAMPORTS;
use program_under_test::DEFAULT_EARLY_UNLOCK_FLIPS;
use program_under_test::DEFAULT_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_MAX_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_MIN_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_UNLOCK_INTERVAL_SECONDS;
use program_under_test::ECONOMY_VERSION;
use program_under_test::GAME_STATUS_LIVE;
use program_under_test::ID;
use program_under_test::SECTION_BYTES;
use program_under_test::SECTION_STATUS_ACTIVE;
use program_under_test::SECTION_STATUS_MINTED;
use program_under_test::SECTION_STATUS_SEALED;
use program_under_test::SectionState;
use program_under_test::pricing::DEFAULT_BURST_ELASTICITY;
use program_under_test::pricing::DEFAULT_CHANGE_DENOMINATOR;
use program_under_test::pricing::DEFAULT_EMISSION_DURATION_SECONDS;
use program_under_test::pricing::DEFAULT_END_FLOOR_PRICE_LAMPORTS;
use program_under_test::pricing::DEFAULT_MAX_PRICE_LAMPORTS;
use program_under_test::pricing::DEFAULT_MIN_PRICE_LAMPORTS;
use program_under_test::pricing::DEFAULT_OWNER_SHARE_BASIS_POINTS;
use program_under_test::pricing::DEFAULT_START_PRICE_LAMPORTS;
use program_under_test::pricing::DEFAULT_TARGET_TOKENS_PER_WINDOW;
use program_under_test::pricing::DEFAULT_WINDOW_SECONDS;
use solana_program_pack::Pack;
use solana_system_interface::instruction as system_instruction;
use spl_associated_token_account_interface::address::get_associated_token_address_with_program_id;
use spl_associated_token_account_interface::instruction::create_associated_token_account;
use spl_associated_token_account_interface::program as associated_token_program;
use spl_token_2022_interface::extension::StateWithExtensions;
use spl_token_2022_interface::instruction as token_instruction;
use spl_token_2022_interface::instruction::AuthorityType;
use spl_token_2022_interface::state::Account as TokenAccount;
use spl_token_2022_interface::state::Mint;

const CONFIG_SEED: &[u8] = b"config";
const GAME_SEED: &[u8] = b"game";
const SECTION_SEED: &[u8] = b"section";
const TEST_AUTHORITY_SECRET: [u8; 32] = [
	182, 10, 64, 122, 65, 236, 229, 117, 171, 47, 35, 220, 225, 174, 225, 180, 186, 105, 196, 216,
	166, 207, 46, 39, 227, 107, 186, 154, 107, 88, 177, 103,
];
const TEST_AUTHORITY_ADDRESS: &str = "HMvYWLX41QFw8C3umdL1mbcRDyhGgLWKJK5Zf1dDvFm9";

fn config_address(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[CONFIG_SEED], program_id)
}

fn game_address(program_id: &Pubkey, game_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[GAME_SEED, &[game_index]], program_id)
}

fn section_address(program_id: &Pubkey, game_index: u8, section_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SECTION_SEED, &[game_index], &[section_index]], program_id)
}

fn test_authority() -> Keypair {
	let authority = Keypair::new_from_array(TEST_AUTHORITY_SECRET);
	assert_eq!(authority.pubkey().to_string(), TEST_AUTHORITY_ADDRESS);
	authority
}

async fn start_config() -> (ProgramTest, Keypair, Pubkey, Pubkey) {
	let program_id = Pubkey::new_from_array(ID.to_bytes());
	let program = ProgramTest::start(program_id)
		.await
		.expect("start isolated program test");
	let authority = test_authority();
	program
		.fund(&authority.pubkey(), 1_000_000_000)
		.expect("fund test authority");
	let (config, config_bump) = config_address(&program_id);
	program
		.send_instruction(initialize_config_instruction(
			&program,
			&program.payer(),
			&config,
			config_bump,
		))
		.expect("initialize configuration");
	(program, authority, config, program_id)
}

async fn start_game(early_unlock_flips: u32) -> (ProgramTest, Keypair, Pubkey, Pubkey) {
	let (program, authority, config, program_id) = start_config().await;
	if early_unlock_flips != DEFAULT_EARLY_UNLOCK_FLIPS {
		program
			.send_with_signers(
				update_config_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&authority.pubkey(),
					&authority.pubkey(),
					early_unlock_flips,
				),
				&[&authority],
			)
			.expect("configure test progression");
	}
	let (game, game_bump) = game_address(&program_id, 0);
	let (initial_section, section_bump) = section_address(&program_id, 0, 0);
	program
		.send_with_signers(
			initialize_game_instruction(
				&program,
				[&authority.pubkey(), &config, &game, &initial_section],
				0,
				game_bump,
				section_bump,
			),
			&[&authority],
		)
		.expect("initialize game with test authority");
	(program, authority, config, game)
}

struct CustodyGame {
	program: ProgramTest,
	authority: Keypair,
	config: Pubkey,
	game: Pubkey,
	bit_mint: Keypair,
	bit_reserve: Pubkey,
	initial_section_vault: Pubkey,
}

async fn start_game_with_custody(early_unlock_flips: u32) -> CustodyGame {
	let (program, authority, config, game) = start_game(early_unlock_flips).await;
	let (initial_section, _) = section_address(&program.program_id(), 0, 0);
	let (bit_mint, bit_reserve) = create_bit_mint_and_reserve(
		&program,
		&authority,
		&config,
		BIT_MINT_DECIMALS,
		BIT_TOTAL_SUPPLY_TOKENS,
		true,
	);
	program
		.send_with_signers(
			configure_bit_custody_instruction(
				&program,
				&authority.pubkey(),
				&config,
				&bit_mint.pubkey(),
				&bit_reserve,
			),
			&[&authority],
		)
		.expect("configure test BIT custody");
	let initial_section_vault = get_associated_token_address_with_program_id(
		&initial_section,
		&bit_mint.pubkey(),
		&spl_token_2022_interface::id(),
	);
	program
		.send_instruction(fund_section_vault_instruction(
			&program,
			&program.payer(),
			&config,
			&initial_section,
			&bit_mint.pubkey(),
			&bit_reserve,
			&initial_section_vault,
			0,
		))
		.expect("fund initial test section vault");

	CustodyGame {
		program,
		authority,
		config,
		game,
		bit_mint,
		bit_reserve,
		initial_section_vault,
	}
}

fn initialize_config_instruction(
	program: &ProgramTest,
	payer: &Pubkey,
	config: &Pubkey,
	bump: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[BitflipInstruction::InitializeConfig as u8, bump],
		vec![
			AccountMeta::new(*payer, true),
			AccountMeta::new(*config, false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
}

fn initialize_game_instruction(
	program: &ProgramTest,
	[payer, config, game, initial_section]: [&Pubkey; 4],
	game_index: u8,
	game_bump: u8,
	section_bump: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[
			BitflipInstruction::InitializeGame as u8,
			game_index,
			0,
			game_bump,
			section_bump,
		],
		vec![
			AccountMeta::new(*payer, true),
			AccountMeta::new(*config, false),
			AccountMeta::new(*game, false),
			AccountMeta::new(*initial_section, false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
}

fn update_config_instruction(
	program: &ProgramTest,
	authority: &Pubkey,
	config: &Pubkey,
	treasury: &Pubkey,
	collection_authority: &Pubkey,
	early_unlock_flips: u32,
) -> pina_test::Instruction {
	let mut data = Vec::with_capacity(105);
	data.push(BitflipInstruction::UpdateConfig as u8);
	data.extend_from_slice(&treasury.to_bytes());
	data.extend_from_slice(&collection_authority.to_bytes());
	data.extend_from_slice(&DEFAULT_CLAIM_PRICE_LAMPORTS.to_le_bytes());
	data.extend_from_slice(&DEFAULT_FLIP_FEE_LAMPORTS.to_le_bytes());
	data.extend_from_slice(&DEFAULT_MIN_FLIP_FEE_LAMPORTS.to_le_bytes());
	data.extend_from_slice(&DEFAULT_MAX_FLIP_FEE_LAMPORTS.to_le_bytes());
	data.extend_from_slice(&DEFAULT_UNLOCK_INTERVAL_SECONDS.to_le_bytes());
	data.extend_from_slice(&early_unlock_flips.to_le_bytes());
	program.instruction(
		&data,
		vec![
			AccountMeta::new_readonly(*authority, true),
			AccountMeta::new(*config, false),
		],
	)
}

fn propose_authority_instruction(
	program: &ProgramTest,
	authority: &Pubkey,
	config: &Pubkey,
	pending_authority: &Pubkey,
) -> pina_test::Instruction {
	let mut data = Vec::with_capacity(33);
	data.push(BitflipInstruction::ProposeAuthority as u8);
	data.extend_from_slice(&pending_authority.to_bytes());
	program.instruction(
		&data,
		vec![
			AccountMeta::new_readonly(*authority, true),
			AccountMeta::new(*config, false),
		],
	)
}

fn accept_authority_instruction(
	program: &ProgramTest,
	pending_authority: &Pubkey,
	config: &Pubkey,
) -> pina_test::Instruction {
	program.instruction(
		&[BitflipInstruction::AcceptAuthority as u8],
		vec![
			AccountMeta::new_readonly(*pending_authority, true),
			AccountMeta::new(*config, false),
		],
	)
}

fn configure_bit_custody_instruction(
	program: &ProgramTest,
	authority: &Pubkey,
	config: &Pubkey,
	bit_mint: &Pubkey,
	bit_reserve: &Pubkey,
) -> pina_test::Instruction {
	program.instruction(
		&[BitflipInstruction::ConfigureBitCustody as u8],
		vec![
			AccountMeta::new_readonly(*authority, true),
			AccountMeta::new(*config, false),
			AccountMeta::new_readonly(*bit_mint, false),
			AccountMeta::new_readonly(*bit_reserve, false),
			AccountMeta::new_readonly(spl_token_2022_interface::id(), false),
		],
	)
}

#[allow(clippy::too_many_arguments)]
fn fund_section_vault_instruction(
	program: &ProgramTest,
	funder: &Pubkey,
	config: &Pubkey,
	section: &Pubkey,
	bit_mint: &Pubkey,
	bit_reserve: &Pubkey,
	section_vault: &Pubkey,
	section_index: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[BitflipInstruction::FundSectionVault as u8, 0, section_index],
		vec![
			AccountMeta::new(*funder, true),
			AccountMeta::new_readonly(*config, false),
			AccountMeta::new(*section, false),
			AccountMeta::new_readonly(*bit_mint, false),
			AccountMeta::new(*bit_reserve, false),
			AccountMeta::new(*section_vault, false),
			AccountMeta::new_readonly(associated_token_program::id(), false),
			AccountMeta::new_readonly(spl_token_2022_interface::id(), false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
}

fn create_bit_mint_and_reserve(
	program: &ProgramTest,
	authority: &Keypair,
	config: &Pubkey,
	decimals: u8,
	supply: u64,
	revoke_mint_authority: bool,
) -> (Keypair, Pubkey) {
	let token_program = spl_token_2022_interface::id();
	let bit_mint = Keypair::new();
	let create_mint = system_instruction::create_account(
		&program.payer(),
		&bit_mint.pubkey(),
		Rent::default().minimum_balance(Mint::LEN),
		u64::try_from(Mint::LEN).expect("mint length"),
		&token_program,
	);
	program
		.send_with_signers(create_mint, &[&bit_mint])
		.expect("create Token-2022 BIT mint account");
	program
		.send_instruction(
			token_instruction::initialize_mint2(
				&token_program,
				&bit_mint.pubkey(),
				&authority.pubkey(),
				None,
				decimals,
			)
			.expect("build initialize mint instruction"),
		)
		.expect("initialize zero-decimal BIT mint");

	let bit_reserve =
		get_associated_token_address_with_program_id(config, &bit_mint.pubkey(), &token_program);
	program
		.send_instruction(create_associated_token_account(
			&program.payer(),
			config,
			&bit_mint.pubkey(),
			&token_program,
		))
		.expect("create config-owned BIT reserve");
	program
		.send_with_signers(
			token_instruction::mint_to_checked(
				&token_program,
				&bit_mint.pubkey(),
				&bit_reserve,
				&authority.pubkey(),
				&[],
				supply,
				decimals,
			)
			.expect("build fixed-supply mint instruction"),
			&[authority],
		)
		.expect("mint the fixed BIT supply once");

	if revoke_mint_authority {
		revoke_bit_mint_authority(program, authority, &bit_mint.pubkey());
	}

	(bit_mint, bit_reserve)
}

fn revoke_bit_mint_authority(program: &ProgramTest, authority: &Keypair, bit_mint: &Pubkey) {
	program
		.send_with_signers(
			token_instruction::set_authority(
				&spl_token_2022_interface::id(),
				bit_mint,
				None,
				AuthorityType::MintTokens,
				&authority.pubkey(),
				&[],
			)
			.expect("build mint-authority revocation"),
			&[authority],
		)
		.expect("permanently revoke BIT mint authority");
}

fn token_amount(program: &ProgramTest, token_account: &Pubkey) -> u64 {
	let account = program
		.account(token_account)
		.expect("fetch Token-2022 account");
	StateWithExtensions::<TokenAccount>::unpack(&account.data)
		.expect("decode Token-2022 account")
		.base
		.amount
}

fn create_player_bit_account(program: &ProgramTest, owner: &Pubkey, bit_mint: &Pubkey) -> Pubkey {
	let token_program = spl_token_2022_interface::id();
	let token_account =
		get_associated_token_address_with_program_id(owner, bit_mint, &token_program);
	program
		.send_instruction(create_associated_token_account(
			&program.payer(),
			owner,
			bit_mint,
			&token_program,
		))
		.expect("create player BIT account");
	token_account
}

#[allow(clippy::too_many_arguments)]
fn claim_section_instruction(
	program: &ProgramTest,
	owner: &Pubkey,
	config: &Pubkey,
	game: &Pubkey,
	previous_section: &Pubkey,
	section: &Pubkey,
	treasury: &Pubkey,
	section_index: u8,
	bump: u8,
	maximum_price_lamports: u64,
) -> pina_test::Instruction {
	let mut data = Vec::with_capacity(12);
	data.extend_from_slice(&[
		BitflipInstruction::ClaimSection as u8,
		0,
		section_index,
		bump,
	]);
	data.extend_from_slice(&maximum_price_lamports.to_le_bytes());
	program.instruction(
		&data,
		vec![
			AccountMeta::new(*owner, true),
			AccountMeta::new_readonly(*config, false),
			AccountMeta::new(*game, false),
			AccountMeta::new_readonly(*previous_section, false),
			AccountMeta::new(*section, false),
			AccountMeta::new(*treasury, false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
}

#[derive(Clone, Copy)]
struct TestFlipLimits {
	game_index: u8,
	section_index: u8,
	expected_window_id: u64,
	maximum_unit_price_lamports: u64,
	maximum_total_price_lamports: u64,
	minimum_reward_tokens: u64,
}

impl TestFlipLimits {
	fn full_reward(coordinate_count: usize) -> Self {
		let reward_tokens = u64::try_from(coordinate_count).expect("coordinate count fits u64");
		Self {
			game_index: 0,
			section_index: 0,
			expected_window_id: 0,
			maximum_unit_price_lamports: DEFAULT_START_PRICE_LAMPORTS,
			maximum_total_price_lamports: DEFAULT_START_PRICE_LAMPORTS * reward_tokens,
			minimum_reward_tokens: reward_tokens,
		}
	}
}

fn flip_pixels_instruction(
	program: &ProgramTest,
	[player, config, game, section, bit_mint]: [&Pubkey; 5],
	coordinates: &[(u8, u8)],
	limits: TestFlipLimits,
) -> pina_test::Instruction {
	let mut packed_coordinates = [0; 32];
	for (index, (x, y)) in coordinates.iter().enumerate() {
		packed_coordinates[index * 2] = *x;
		packed_coordinates[index * 2 + 1] = *y;
	}
	let mut data = Vec::with_capacity(68);
	data.extend_from_slice(&[
		BitflipInstruction::FlipPixels as u8,
		limits.game_index,
		limits.section_index,
		coordinates.len() as u8,
	]);
	data.extend_from_slice(&packed_coordinates);
	data.extend_from_slice(&limits.expected_window_id.to_le_bytes());
	data.extend_from_slice(&limits.maximum_unit_price_lamports.to_le_bytes());
	data.extend_from_slice(&limits.maximum_total_price_lamports.to_le_bytes());
	data.extend_from_slice(&limits.minimum_reward_tokens.to_le_bytes());
	let token_program = spl_token_2022_interface::id();
	let section_vault =
		get_associated_token_address_with_program_id(section, bit_mint, &token_program);
	let player_bit_account =
		get_associated_token_address_with_program_id(player, bit_mint, &token_program);
	program.instruction(
		&data,
		vec![
			AccountMeta::new(*player, true),
			AccountMeta::new_readonly(*config, false),
			AccountMeta::new_readonly(*game, false),
			AccountMeta::new(*section, false),
			AccountMeta::new_readonly(*bit_mint, false),
			AccountMeta::new(section_vault, false),
			AccountMeta::new(player_bit_account, false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
}

fn settle_section_economy_instruction(
	program: &ProgramTest,
	game: &Pubkey,
	section: &Pubkey,
	section_index: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[
			BitflipInstruction::SettleSectionEconomy as u8,
			0,
			section_index,
		],
		vec![
			AccountMeta::new_readonly(*game, false),
			AccountMeta::new(*section, false),
		],
	)
}

fn withdraw_section_owner_fees_instruction(
	program: &ProgramTest,
	owner: &Pubkey,
	section: &Pubkey,
	section_index: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[
			BitflipInstruction::WithdrawSectionOwnerFees as u8,
			0,
			section_index,
		],
		vec![
			AccountMeta::new(*owner, true),
			AccountMeta::new(*section, false),
		],
	)
}

async fn claim_first_user_section(
	program: &mut ProgramTest,
	authority: &Keypair,
	config: &Pubkey,
	game: &Pubkey,
	owner: &Keypair,
	bit_mint: &Pubkey,
	bit_reserve: &Pubkey,
) -> Pubkey {
	let (initial_section, _) = section_address(&program.program_id(), 0, 0);
	let _ = create_player_bit_account(program, &owner.pubkey(), bit_mint);
	program
		.send_with_signers(
			flip_pixels_instruction(
				program,
				[&owner.pubkey(), config, game, &initial_section, bit_mint],
				&[(0, 0)],
				TestFlipLimits::full_reward(1),
			),
			&[owner],
		)
		.expect("unlock first purchasable section");
	let (section, bump) = section_address(&program.program_id(), 0, 1);
	program
		.send_with_signers(
			claim_section_instruction(
				program,
				&owner.pubkey(),
				config,
				game,
				&initial_section,
				&section,
				&authority.pubkey(),
				1,
				bump,
				DEFAULT_CLAIM_PRICE_LAMPORTS,
			),
			&[owner],
		)
		.expect("claim first purchasable section");
	let section_vault = get_associated_token_address_with_program_id(
		&section,
		bit_mint,
		&spl_token_2022_interface::id(),
	);
	program
		.send_instruction(fund_section_vault_instruction(
			program,
			&program.payer(),
			config,
			&section,
			bit_mint,
			bit_reserve,
			&section_vault,
			1,
		))
		.expect("fund claimed section vault");
	section
}

fn seal_section_instruction(
	program: &ProgramTest,
	owner: &Pubkey,
	game: &Pubkey,
	section: &Pubkey,
	section_index: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[BitflipInstruction::SealSection as u8, 0, section_index],
		vec![
			AccountMeta::new(*owner, true),
			AccountMeta::new_readonly(*game, false),
			AccountMeta::new(*section, false),
		],
	)
}

fn list_section_instruction(
	program: &ProgramTest,
	owner: &Pubkey,
	game: &Pubkey,
	section: &Pubkey,
	section_index: u8,
	price_lamports: u64,
) -> pina_test::Instruction {
	let mut data = Vec::with_capacity(11);
	data.extend_from_slice(&[BitflipInstruction::ListSection as u8, 0, section_index]);
	data.extend_from_slice(&price_lamports.to_le_bytes());
	program.instruction(
		&data,
		vec![
			AccountMeta::new_readonly(*owner, true),
			AccountMeta::new_readonly(*game, false),
			AccountMeta::new(*section, false),
		],
	)
}

fn cancel_section_listing_instruction(
	program: &ProgramTest,
	owner: &Pubkey,
	game: &Pubkey,
	section: &Pubkey,
	section_index: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[
			BitflipInstruction::CancelSectionListing as u8,
			0,
			section_index,
		],
		vec![
			AccountMeta::new_readonly(*owner, true),
			AccountMeta::new_readonly(*game, false),
			AccountMeta::new(*section, false),
		],
	)
}

#[allow(clippy::too_many_arguments)]
fn purchase_section_instruction(
	program: &ProgramTest,
	buyer: &Pubkey,
	seller: &Pubkey,
	game: &Pubkey,
	section: &Pubkey,
	section_index: u8,
	maximum_price_lamports: u64,
) -> pina_test::Instruction {
	let mut data = Vec::with_capacity(11);
	data.extend_from_slice(&[BitflipInstruction::PurchaseSection as u8, 0, section_index]);
	data.extend_from_slice(&maximum_price_lamports.to_le_bytes());
	program.instruction(
		&data,
		vec![
			AccountMeta::new(*buyer, true),
			AccountMeta::new(*seller, false),
			AccountMeta::new_readonly(*game, false),
			AccountMeta::new(*section, false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
}

#[allow(clippy::too_many_arguments)]
fn record_mint_instruction(
	program: &ProgramTest,
	collection_authority: &Pubkey,
	config: &Pubkey,
	game: &Pubkey,
	section: &Pubkey,
	section_index: u8,
	expected_owner: &Pubkey,
	asset_id: &Pubkey,
	merkle_tree: &Pubkey,
	leaf_index: u32,
) -> pina_test::Instruction {
	let mut data = Vec::with_capacity(103);
	data.extend_from_slice(&[
		BitflipInstruction::RecordSectionMint as u8,
		0,
		section_index,
	]);
	data.extend_from_slice(&expected_owner.to_bytes());
	data.extend_from_slice(&asset_id.to_bytes());
	data.extend_from_slice(&merkle_tree.to_bytes());
	data.extend_from_slice(&leaf_index.to_le_bytes());
	program.instruction(
		&data,
		vec![
			AccountMeta::new_readonly(*collection_authority, true),
			AccountMeta::new_readonly(*config, false),
			AccountMeta::new(*game, false),
			AccountMeta::new(*section, false),
		],
	)
}

fn assert_custom_error(error: &TestError, expected: BitflipError) {
	let code = expected as u32;
	let message = error.message();
	assert!(
		message.contains(&format!("custom program error: 0x{code:x}"))
			|| message.contains(&format!("Custom({code})")),
		"expected {expected:?} ({code}), got: {message}"
	);
}

fn u64_at(data: &[u8], offset: usize) -> u64 {
	u64::from_le_bytes(data[offset..offset + 8].try_into().expect("u64 field"))
}

fn u32_at(data: &[u8], offset: usize) -> u32 {
	u32::from_le_bytes(data[offset..offset + 4].try_into().expect("u32 field"))
}

fn coordinates_for_batch(batch_index: usize) -> [(u8, u8); 16] {
	core::array::from_fn(|coordinate_index| {
		let pixel_index = (batch_index * 16 + coordinate_index) % 4_096;
		(
			u8::try_from(pixel_index % 64).expect("x coordinate"),
			u8::try_from(pixel_index / 64).expect("y coordinate"),
		)
	})
}

fn section_pixel_is_on(data: &[u8], x: u8, y: u8) -> bool {
	let pixel_index = usize::from(y) * 64 + usize::from(x);
	let pixels_offset = SectionState::SIZE - SECTION_BYTES;
	data[pixels_offset + pixel_index / 8] & (1 << (pixel_index % 8)) != 0
}

#[test]
#[ignore = "run with pina test"]
fn permissionless_sponsor_initializes_safe_fixed_configuration() {
	pina_test::run(async {
		let program_id = Pubkey::new_from_array(ID.to_bytes());
		let mut program = ProgramTest::start(program_id)
			.await
			.expect("start isolated program test");
		let payer = program.payer();
		let (config, bump) = config_address(&program_id);

		program
			.send_instruction(initialize_config_instruction(
				&program, &payer, &config, bump,
			))
			.expect("initialize fixed Bitflip configuration");

		let account = program.account(&config).expect("fetch config account");
		assert_eq!(account.owner, program_id);
		assert_eq!(account.data.len(), ConfigState::SIZE);
		assert_eq!(account.data[0], BitflipAccountType::ConfigState as u8);
		assert_eq!(account.data[1], CONFIG_VERSION, "config ABI version");
		assert_eq!(&account.data[130..194], &[0; 64], "custody unset");
		assert_eq!(u64_at(&account.data, 194), DEFAULT_CLAIM_PRICE_LAMPORTS);
		assert_eq!(u64_at(&account.data, 202), DEFAULT_FLIP_FEE_LAMPORTS);
		assert_eq!(u64_at(&account.data, 210), DEFAULT_MIN_FLIP_FEE_LAMPORTS);
		assert_eq!(u64_at(&account.data, 218), DEFAULT_MAX_FLIP_FEE_LAMPORTS);
		assert_eq!(u32_at(&account.data, 226), DEFAULT_UNLOCK_INTERVAL_SECONDS);
		assert_eq!(u32_at(&account.data, 230), DEFAULT_EARLY_UNLOCK_FLIPS);
		assert_eq!(account.data[236], bump);

		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with pina test"]
fn config_cannot_be_initialized_twice() {
	pina_test::run(async {
		let program_id = Pubkey::new_from_array(ID.to_bytes());
		let mut program = ProgramTest::start(program_id)
			.await
			.expect("start isolated program test");
		let payer = program.payer();
		let (config, bump) = config_address(&program_id);
		let initialize = || initialize_config_instruction(&program, &payer, &config, bump);

		program
			.send_instruction(initialize())
			.expect("first initialization succeeds");
		let before = program.account(&config).expect("fetch initialized config");
		let error = program
			.send_instruction(initialize())
			.expect_err("second initialization must fail");
		let after = program
			.account(&config)
			.expect("fetch config after failure");

		assert_eq!(before.data, after.data, "failed transaction is atomic");
		assert!(!error.message().is_empty());
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn fixed_token_2022_custody_funds_each_section_vault_once() {
	pina_test::run(async {
		let (mut program, authority, config, _) = start_game(DEFAULT_EARLY_UNLOCK_FLIPS).await;
		let (section, _) = section_address(&program.program_id(), 0, 0);
		let (decimal_mint, decimal_reserve) = create_bit_mint_and_reserve(
			&program,
			&authority,
			&config,
			1,
			BIT_TOTAL_SUPPLY_TOKENS,
			true,
		);
		let wrong_decimals = program
			.send_with_signers(
				configure_bit_custody_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&decimal_mint.pubkey(),
					&decimal_reserve,
				),
				&[&authority],
			)
			.expect_err("custody rejects fractional BIT");
		assert_custom_error(&wrong_decimals, BitflipError::InvalidBitMint);

		let (supply_mint, supply_reserve) = create_bit_mint_and_reserve(
			&program,
			&authority,
			&config,
			BIT_MINT_DECIMALS,
			BIT_TOTAL_SUPPLY_TOKENS - 1,
			true,
		);
		let wrong_supply = program
			.send_with_signers(
				configure_bit_custody_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&supply_mint.pubkey(),
					&supply_reserve,
				),
				&[&authority],
			)
			.expect_err("custody rejects a supply below the fixed cap");
		assert_custom_error(&wrong_supply, BitflipError::InvalidBitMint);

		let (bit_mint, bit_reserve) = create_bit_mint_and_reserve(
			&program,
			&authority,
			&config,
			BIT_MINT_DECIMALS,
			BIT_TOTAL_SUPPLY_TOKENS,
			false,
		);

		let active_authority = program
			.send_with_signers(
				configure_bit_custody_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&bit_mint.pubkey(),
					&bit_reserve,
				),
				&[&authority],
			)
			.expect_err("custody rejects a mint with live issuance authority");
		assert_custom_error(&active_authority, BitflipError::InvalidBitMint);
		assert_eq!(
			&program.account(&config).expect("fetch config").data[130..194],
			&[0; 64],
			"failed configuration is atomic"
		);

		revoke_bit_mint_authority(&program, &authority, &bit_mint.pubkey());
		program
			.send_with_signers(
				configure_bit_custody_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&bit_mint.pubkey(),
					&bit_reserve,
				),
				&[&authority],
			)
			.expect("register immutable fixed BIT custody");

		let mint_account = program.account(&bit_mint.pubkey()).expect("fetch BIT mint");
		let mint = StateWithExtensions::<Mint>::unpack(&mint_account.data)
			.expect("decode BIT mint")
			.base;
		assert_eq!(mint.decimals, 0);
		assert_eq!(mint.supply, BIT_TOTAL_SUPPLY_TOKENS);
		let config_account = program.account(&config).expect("fetch configured config");
		assert_eq!(
			&config_account.data[130..162],
			bit_mint.pubkey().to_bytes().as_slice()
		);
		assert_eq!(
			&config_account.data[162..194],
			bit_reserve.to_bytes().as_slice()
		);

		let section_vault = get_associated_token_address_with_program_id(
			&section,
			&bit_mint.pubkey(),
			&spl_token_2022_interface::id(),
		);
		program
			.send_instruction(fund_section_vault_instruction(
				&program,
				&program.payer(),
				&config,
				&section,
				&bit_mint.pubkey(),
				&bit_reserve,
				&section_vault,
				0,
			))
			.expect("funder pays rent and moves one section allocation");

		assert_eq!(
			token_amount(&program, &section_vault),
			BIT_SECTION_ALLOCATION_TOKENS
		);
		assert_eq!(
			token_amount(&program, &bit_reserve),
			BIT_TOTAL_SUPPLY_TOKENS - BIT_SECTION_ALLOCATION_TOKENS
		);
		let section_account = program.account(&section).expect("fetch funded section");
		assert_eq!(
			&section_account.data[97..129],
			section_vault.to_bytes().as_slice()
		);

		let reserve_before_duplicate = token_amount(&program, &bit_reserve);
		let vault_before_duplicate = token_amount(&program, &section_vault);
		let duplicate = program
			.send_instruction(fund_section_vault_instruction(
				&program,
				&program.payer(),
				&config,
				&section,
				&bit_mint.pubkey(),
				&bit_reserve,
				&section_vault,
				0,
			))
			.expect_err("a section allocation cannot be transferred twice");
		assert_custom_error(&duplicate, BitflipError::SectionVaultAlreadyFunded);
		assert_eq!(
			token_amount(&program, &bit_reserve),
			reserve_before_duplicate
		);
		assert_eq!(
			token_amount(&program, &section_vault),
			vault_before_duplicate
		);

		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with pina test"]
fn non_canonical_config_bump_is_rejected() {
	pina_test::run(async {
		let program_id = Pubkey::new_from_array(ID.to_bytes());
		let mut program = ProgramTest::start(program_id)
			.await
			.expect("start isolated program test");
		let payer = program.payer();
		let (config, bump) = config_address(&program_id);

		let error = program
			.send_instruction(initialize_config_instruction(
				&program,
				&payer,
				&config,
				bump.wrapping_sub(1),
			))
			.expect_err("non-canonical bump must fail");

		assert!(!error.message().is_empty());
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with pina test"]
fn untrusted_sponsor_cannot_start_the_unlock_clock() {
	pina_test::run(async {
		let program_id = Pubkey::new_from_array(ID.to_bytes());
		let mut program = ProgramTest::start(program_id)
			.await
			.expect("start isolated program test");
		let payer = program.payer();
		let (config, config_bump) = config_address(&program_id);
		program
			.send_instruction(initialize_config_instruction(
				&program,
				&payer,
				&config,
				config_bump,
			))
			.expect("initialize configuration");

		let (game, game_bump) = game_address(&program_id, 0);
		let (initial_section, section_bump) = section_address(&program_id, 0, 0);
		let error = program
			.send_instruction(initialize_game_instruction(
				&program,
				[&payer, &config, &game, &initial_section],
				0,
				game_bump,
				section_bump,
			))
			.expect_err("only the configured authority can start a game");

		assert!(!error.message().is_empty());
		assert!(
			program.account(&game).is_err(),
			"failed start creates no PDA"
		);
		assert!(
			program.account(&initial_section).is_err(),
			"failed start creates no section PDA"
		);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn game_bootstraps_one_program_owned_section() {
	pina_test::run(async {
		let (mut program, _, _, game) = start_game(DEFAULT_EARLY_UNLOCK_FLIPS).await;
		let (initial_section, _) = section_address(&program.program_id(), 0, 0);

		let game_account = program.account(&game).expect("fetch game account");
		assert_eq!(game_account.data.len(), program_under_test::GameState::SIZE);
		assert_eq!(game_account.data[4], ECONOMY_VERSION);
		assert_eq!(
			u16::from_le_bytes([game_account.data[13], game_account.data[14]]),
			1,
			"only the bootstrapped section exists"
		);
		assert_eq!(
			u64_at(&game_account.data, 33),
			BIT_SECTION_ALLOCATION_TOKENS
		);
		assert_eq!(
			u64_at(&game_account.data, 41),
			DEFAULT_EMISSION_DURATION_SECONDS
		);
		assert_eq!(u64_at(&game_account.data, 49), DEFAULT_WINDOW_SECONDS);
		assert_eq!(
			u64_at(&game_account.data, 57),
			DEFAULT_TARGET_TOKENS_PER_WINDOW
		);
		assert_eq!(u64_at(&game_account.data, 65), DEFAULT_START_PRICE_LAMPORTS);
		assert_eq!(u64_at(&game_account.data, 73), DEFAULT_MIN_PRICE_LAMPORTS);
		assert_eq!(u64_at(&game_account.data, 81), DEFAULT_MAX_PRICE_LAMPORTS);
		assert_eq!(u64_at(&game_account.data, 89), DEFAULT_MIN_PRICE_LAMPORTS);
		assert_eq!(
			u64_at(&game_account.data, 97),
			DEFAULT_END_FLOOR_PRICE_LAMPORTS
		);
		assert_eq!(u64_at(&game_account.data, 105), DEFAULT_CHANGE_DENOMINATOR);
		assert_eq!(u64_at(&game_account.data, 113), DEFAULT_BURST_ELASTICITY);
		assert_eq!(
			u16::from_le_bytes([game_account.data[121], game_account.data[122]]),
			DEFAULT_OWNER_SHARE_BASIS_POINTS
		);
		let section_account = program
			.account(&initial_section)
			.expect("fetch initial section");
		assert_eq!(section_account.owner, program.program_id());
		assert_eq!(section_account.data.len(), SectionState::SIZE);
		assert_eq!(&section_account.data[1..33], game.to_bytes().as_slice());
		assert_eq!(&section_account.data[97..129], &[0; 32], "vault unset");
		assert_eq!(section_account.data[130], 0, "initial section index");
		assert_eq!(section_account.data[131], SECTION_STATUS_ACTIVE);
		let launched_at = u64_at(&section_account.data, 171);
		assert!(launched_at > 0);
		assert_eq!(u64_at(&section_account.data, 179), launched_at);
		assert_eq!(u64_at(&section_account.data, 187), launched_at);
		assert_eq!(u64_at(&section_account.data, 195), 0);
		assert_eq!(
			u64_at(&section_account.data, 203),
			DEFAULT_TARGET_TOKENS_PER_WINDOW
		);
		assert_eq!(u64_at(&section_account.data, 211), 0);
		assert_eq!(u64_at(&section_account.data, 219), 0);
		assert_eq!(u64_at(&section_account.data, 227), 0);
		assert_eq!(
			u64_at(&section_account.data, 235),
			DEFAULT_START_PRICE_LAMPORTS
		);
		assert_eq!(
			u64_at(&section_account.data, 243),
			DEFAULT_START_PRICE_LAMPORTS
		);

		program
			.send_instruction(settle_section_economy_instruction(
				&program,
				&game,
				&initial_section,
				0,
			))
			.expect("permissionless settlement succeeds");
		let settled = program
			.account(&initial_section)
			.expect("fetch settled section");
		assert_eq!(u64_at(&settled.data, 219), 0);
		assert_eq!(u64_at(&settled.data, 227), 0);

		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn fixed_supply_prevents_a_fifth_game() {
	pina_test::run(async {
		let (mut program, authority, config, program_id) = start_config().await;
		let game_index = BIT_GAME_COUNT;
		let (game, game_bump) = game_address(&program_id, game_index);
		let (initial_section, section_bump) = section_address(&program_id, game_index, 0);
		let error = program
			.send_with_signers(
				initialize_game_instruction(
					&program,
					[&authority.pubkey(), &config, &game, &initial_section],
					game_index,
					game_bump,
					section_bump,
				),
				&[&authority],
			)
			.expect_err("the fixed supply cannot fund a fifth game");

		assert_custom_error(&error, BitflipError::InvalidGameIndex);
		assert!(program.account(&game).is_err());
		assert!(program.account(&initial_section).is_err());
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn claims_enforce_order_and_activity_unlocks() {
	pina_test::run(async {
		let CustodyGame {
			mut program,
			authority,
			config,
			game,
			bit_mint,
			..
		} = start_game_with_custody(1).await;
		let program_id = program.program_id();
		let player = Keypair::new();
		let owner_one = Keypair::new();
		program
			.fund(&player.pubkey(), 100_000_000)
			.expect("fund initial section player");
		program
			.fund(&owner_one.pubkey(), 100_000_000)
			.expect("fund second section owner");
		let (section_zero, _) = section_address(&program_id, 0, 0);
		let (section_one, section_one_bump) = section_address(&program_id, 0, 1);
		let (section_two, section_two_bump) = section_address(&program_id, 0, 2);
		let _ = create_player_bit_account(&program, &player.pubkey(), &bit_mint.pubkey());

		let out_of_order = program
			.send_with_signers(
				claim_section_instruction(
					&program,
					&owner_one.pubkey(),
					&config,
					&game,
					&section_zero,
					&section_two,
					&authority.pubkey(),
					2,
					section_two_bump,
					DEFAULT_CLAIM_PRICE_LAMPORTS,
				),
				&[&owner_one],
			)
			.expect_err("sections cannot be claimed out of order");
		assert_custom_error(&out_of_order, BitflipError::GameNotLive);

		let locked = program
			.send_with_signers(
				claim_section_instruction(
					&program,
					&owner_one.pubkey(),
					&config,
					&game,
					&section_zero,
					&section_one,
					&authority.pubkey(),
					1,
					section_one_bump,
					DEFAULT_CLAIM_PRICE_LAMPORTS,
				),
				&[&owner_one],
			)
			.expect_err("next section stays locked before activity threshold");
		assert_custom_error(&locked, BitflipError::SectionLocked);

		program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&player.pubkey(),
						&config,
						&game,
						&section_zero,
						&bit_mint.pubkey(),
					],
					&[(4, 9)],
					TestFlipLimits::full_reward(1),
				),
				&[&player],
			)
			.expect("one paid flip reaches the configured activity threshold");
		program
			.send_with_signers(
				claim_section_instruction(
					&program,
					&owner_one.pubkey(),
					&config,
					&game,
					&section_zero,
					&section_one,
					&authority.pubkey(),
					1,
					section_one_bump,
					DEFAULT_CLAIM_PRICE_LAMPORTS,
				),
				&[&owner_one],
			)
			.expect("activity unlocks the next section");

		let game_account = program.account(&game).expect("fetch game account");
		assert_eq!(game_account.data[2], GAME_STATUS_LIVE);
		assert_eq!(
			u16::from_le_bytes([game_account.data[13], game_account.data[14]]),
			2
		);
		let claimed_section = program
			.account(&section_one)
			.expect("fetch newly claimed section");
		let launched_at = u64_at(&claimed_section.data, 171);
		assert!(launched_at > 0);
		assert_eq!(u64_at(&claimed_section.data, 179), launched_at);
		assert_eq!(u64_at(&claimed_section.data, 187), launched_at);
		assert_eq!(
			u64_at(&claimed_section.data, 203),
			DEFAULT_TARGET_TOKENS_PER_WINDOW
		);
		assert_eq!(u64_at(&claimed_section.data, 219), 0);
		assert_eq!(u64_at(&claimed_section.data, 227), 0);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn claim_price_slippage_is_atomic() {
	pina_test::run(async {
		let CustodyGame {
			mut program,
			authority,
			config,
			game,
			bit_mint,
			..
		} = start_game_with_custody(1).await;
		let owner = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section owner");
		let (previous_section, _) = section_address(&program.program_id(), 0, 0);
		let (section, bump) = section_address(&program.program_id(), 0, 1);
		let _ = create_player_bit_account(&program, &owner.pubkey(), &bit_mint.pubkey());
		program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&owner.pubkey(),
						&config,
						&game,
						&previous_section,
						&bit_mint.pubkey(),
					],
					&[(0, 0)],
					TestFlipLimits::full_reward(1),
				),
				&[&owner],
			)
			.expect("unlock first purchasable section");
		let before_owner = program.balance(&owner.pubkey()).expect("owner balance");
		let before_treasury = program
			.balance(&authority.pubkey())
			.expect("treasury balance");

		let error = program
			.send_with_signers(
				claim_section_instruction(
					&program,
					&owner.pubkey(),
					&config,
					&game,
					&previous_section,
					&section,
					&authority.pubkey(),
					1,
					bump,
					DEFAULT_CLAIM_PRICE_LAMPORTS - 1,
				),
				&[&owner],
			)
			.expect_err("claim must reject a price above the signed maximum");
		assert_custom_error(&error, BitflipError::PriceSlippage);
		assert!(program.account(&section).is_err());
		assert_eq!(
			program.balance(&owner.pubkey()).expect("owner balance"),
			before_owner
		);
		assert_eq!(
			program
				.balance(&authority.pubkey())
				.expect("treasury balance"),
			before_treasury
		);

		program
			.send_with_signers(
				claim_section_instruction(
					&program,
					&owner.pubkey(),
					&config,
					&game,
					&previous_section,
					&section,
					&authority.pubkey(),
					1,
					bump,
					DEFAULT_CLAIM_PRICE_LAMPORTS,
				),
				&[&owner],
			)
			.expect("exact maximum price succeeds");
		assert!(
			program.balance(&owner.pubkey()).expect("owner balance")
				< before_owner - DEFAULT_CLAIM_PRICE_LAMPORTS,
			"owner also funds rent for the new section account"
		);
		assert_eq!(
			program
				.balance(&authority.pubkey())
				.expect("treasury balance"),
			before_treasury + DEFAULT_CLAIM_PRICE_LAMPORTS
		);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn duplicate_and_underpriced_flips_are_atomic() {
	pina_test::run(async {
		let CustodyGame {
			mut program,
			config,
			game,
			bit_mint,
			initial_section_vault,
			..
		} = start_game_with_custody(DEFAULT_EARLY_UNLOCK_FLIPS).await;
		let owner = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section player");
		let (section, _) = section_address(&program.program_id(), 0, 0);
		let player_bit_account =
			create_player_bit_account(&program, &owner.pubkey(), &bit_mint.pubkey());
		let before = program.account(&section).expect("fetch section");
		let before_vault = token_amount(&program, &initial_section_vault);
		let before_player_bit = token_amount(&program, &player_bit_account);

		let duplicate = program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&owner.pubkey(),
						&config,
						&game,
						&section,
						&bit_mint.pubkey(),
					],
					&[(4, 9), (4, 9)],
					TestFlipLimits::full_reward(2),
				),
				&[&owner],
			)
			.expect_err("duplicate coordinates must fail");
		assert_custom_error(&duplicate, BitflipError::DuplicateCoordinate);

		let slippage = program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&owner.pubkey(),
						&config,
						&game,
						&section,
						&bit_mint.pubkey(),
					],
					&[(4, 9)],
					TestFlipLimits {
						maximum_total_price_lamports: DEFAULT_START_PRICE_LAMPORTS - 1,
						..TestFlipLimits::full_reward(1)
					},
				),
				&[&owner],
			)
			.expect_err("flip must reject a fee above the signed maximum");
		assert_custom_error(&slippage, BitflipError::PriceSlippage);

		let stale_window = program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&owner.pubkey(),
						&config,
						&game,
						&section,
						&bit_mint.pubkey(),
					],
					&[(4, 9)],
					TestFlipLimits {
						expected_window_id: 1,
						..TestFlipLimits::full_reward(1)
					},
				),
				&[&owner],
			)
			.expect_err("flip must reject a stale signed reward window");
		assert_custom_error(&stale_window, BitflipError::StalePriceWindow);

		let insufficient_reward = program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&owner.pubkey(),
						&config,
						&game,
						&section,
						&bit_mint.pubkey(),
					],
					&[(4, 9)],
					TestFlipLimits {
						minimum_reward_tokens: 2,
						..TestFlipLimits::full_reward(1)
					},
				),
				&[&owner],
			)
			.expect_err("flip must reject fewer rewards than the signed minimum");
		assert_custom_error(&insufficient_reward, BitflipError::InsufficientReward);

		assert_eq!(
			program.account(&section).expect("fetch section").data,
			before.data
		);
		assert_eq!(token_amount(&program, &initial_section_vault), before_vault);
		assert_eq!(
			token_amount(&program, &player_bit_account),
			before_player_bit
		);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn user_owned_section_receives_fixed_fee_share_atomically() {
	pina_test::run(async {
		let CustodyGame {
			mut program,
			authority,
			config,
			game,
			bit_mint,
			bit_reserve,
			..
		} = start_game_with_custody(1).await;
		let owner = Keypair::new();
		let player = Keypair::new();
		program
			.fund(&owner.pubkey(), 200_000_000)
			.expect("fund section owner");
		program
			.fund(&player.pubkey(), 100_000_000)
			.expect("fund section player");
		let section = claim_first_user_section(
			&mut program,
			&authority,
			&config,
			&game,
			&owner,
			&bit_mint.pubkey(),
			&bit_reserve,
		)
		.await;
		let player_bits = create_player_bit_account(&program, &player.pubkey(), &bit_mint.pubkey());
		let owner_bits = get_associated_token_address_with_program_id(
			&owner.pubkey(),
			&bit_mint.pubkey(),
			&spl_token_2022_interface::id(),
		);
		let split = program_under_test::pricing::split_fee(
			DEFAULT_START_PRICE_LAMPORTS,
			DEFAULT_OWNER_SHARE_BASIS_POINTS,
		)
		.expect("valid fixed owner share");

		let owner_before = program.balance(&owner.pubkey()).expect("owner before flip");
		let player_before = program
			.balance(&player.pubkey())
			.expect("player before flip");

		let section_lamports_before = program.balance(&section).expect("section before paid flip");
		program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&player.pubkey(),
						&config,
						&game,
						&section,
						&bit_mint.pubkey(),
					],
					&[(2, 2)],
					TestFlipLimits {
						section_index: 1,
						..TestFlipLimits::full_reward(1)
					},
				),
				&[&player],
			)
			.expect("pay a non-owner flip fee");
		assert_eq!(
			program
				.balance(&owner.pubkey())
				.expect("owner before withdrawal"),
			owner_before,
		);
		assert_eq!(
			program
				.balance(&player.pubkey())
				.expect("player after paid flip"),
			player_before - DEFAULT_START_PRICE_LAMPORTS,
		);
		assert_eq!(
			program.balance(&section).expect("section after paid flip"),
			section_lamports_before + DEFAULT_START_PRICE_LAMPORTS,
		);
		assert_eq!(token_amount(&program, &player_bits), 1);
		let accrued = program.account(&section).expect("section fee ledgers");
		assert_eq!(u64_at(&accrued.data, 251), split.protocol_lamports);
		assert_eq!(u64_at(&accrued.data, 259), split.owner_lamports);
		let attacker = Keypair::new();
		program
			.fund(&attacker.pubkey(), 1_000_000)
			.expect("fund false owner");
		program
			.send_with_signers(
				withdraw_section_owner_fees_instruction(&program, &attacker.pubkey(), &section, 1),
				&[&attacker],
			)
			.expect_err("a false owner cannot withdraw accrued fees");
		assert_eq!(
			program
				.account(&section)
				.expect("section after false owner")
				.data,
			accrued.data,
		);

		program
			.send_with_signers(
				withdraw_section_owner_fees_instruction(&program, &owner.pubkey(), &section, 1),
				&[&owner],
			)
			.expect("owner withdraws their accrued share");
		assert_eq!(
			program
				.balance(&owner.pubkey())
				.expect("owner after withdrawal"),
			owner_before + split.owner_lamports,
		);
		assert_eq!(
			program.balance(&section).expect("section after withdrawal"),
			section_lamports_before + split.protocol_lamports,
		);
		let withdrawn = program.account(&section).expect("withdrawn fee ledgers");
		assert_eq!(u64_at(&withdrawn.data, 251), split.protocol_lamports);
		assert_eq!(u64_at(&withdrawn.data, 259), 0);
		let empty = program
			.send_with_signers(
				withdraw_section_owner_fees_instruction(&program, &owner.pubkey(), &section, 1),
				&[&owner],
			)
			.expect_err("an owner cannot withdraw the same share twice");
		assert_custom_error(&empty, BitflipError::NoOwnerFees);

		let owner_lamports_before = program
			.balance(&owner.pubkey())
			.expect("owner before self flip");
		let section_lamports_before = program.balance(&section).expect("section before self flip");
		let owner_tokens_before = token_amount(&program, &owner_bits);
		program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&owner.pubkey(),
						&config,
						&game,
						&section,
						&bit_mint.pubkey(),
					],
					&[(3, 3)],
					TestFlipLimits {
						section_index: 1,
						..TestFlipLimits::full_reward(1)
					},
				),
				&[&owner],
			)
			.expect("an owner can flip but retains only their fixed share");
		assert_eq!(
			program
				.balance(&owner.pubkey())
				.expect("owner after self flip"),
			owner_lamports_before - DEFAULT_START_PRICE_LAMPORTS,
		);
		assert_eq!(
			program.balance(&section).expect("section after self flip"),
			section_lamports_before + DEFAULT_START_PRICE_LAMPORTS,
		);
		assert_eq!(token_amount(&program, &owner_bits), owner_tokens_before + 1);
		let self_flip = program.account(&section).expect("self-flip fee ledgers");
		assert_eq!(u64_at(&self_flip.data, 251), split.protocol_lamports * 2);
		assert_eq!(u64_at(&self_flip.data, 259), split.owner_lamports);
		program
			.send_with_signers(
				withdraw_section_owner_fees_instruction(&program, &owner.pubkey(), &section, 1),
				&[&owner],
			)
			.expect("owner withdraws their self-flip share");
		assert_eq!(
			program
				.balance(&owner.pubkey())
				.expect("owner after self withdrawal"),
			owner_lamports_before - split.protocol_lamports,
		);

		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn owner_can_list_cancel_and_sell_a_section_atomically() {
	pina_test::run(async {
		const SALE_PRICE: u64 = 25_000_000;
		let CustodyGame {
			mut program,
			authority,
			config,
			game,
			bit_mint,
			bit_reserve,
			..
		} = start_game_with_custody(1).await;
		let seller = Keypair::new();
		let buyer = Keypair::new();
		program
			.fund(&seller.pubkey(), 100_000_000)
			.expect("fund seller");
		program
			.fund(&buyer.pubkey(), 100_000_000)
			.expect("fund buyer");
		let section = claim_first_user_section(
			&mut program,
			&authority,
			&config,
			&game,
			&seller,
			&bit_mint.pubkey(),
			&bit_reserve,
		)
		.await;
		program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					[
						&seller.pubkey(),
						&config,
						&game,
						&section,
						&bit_mint.pubkey(),
					],
					&[(4, 4)],
					TestFlipLimits {
						section_index: 1,
						..TestFlipLimits::full_reward(1)
					},
				),
				&[&seller],
			)
			.expect("seller accrues a fee share before listing");
		let owner_share = program_under_test::pricing::split_fee(
			DEFAULT_START_PRICE_LAMPORTS,
			DEFAULT_OWNER_SHARE_BASIS_POINTS,
		)
		.expect("valid owner share")
		.owner_lamports;

		let list =
			|| list_section_instruction(&program, &seller.pubkey(), &game, &section, 1, SALE_PRICE);
		program
			.send_with_signers(list(), &[&seller])
			.expect("owner lists the section");
		assert_eq!(
			u64_at(&program.account(&section).expect("fetch listing").data, 163,),
			SALE_PRICE
		);

		program
			.send_with_signers(
				cancel_section_listing_instruction(&program, &seller.pubkey(), &game, &section, 1),
				&[&seller],
			)
			.expect("owner cancels the listing");
		let not_for_sale = program
			.send_with_signers(
				purchase_section_instruction(
					&program,
					&buyer.pubkey(),
					&seller.pubkey(),
					&game,
					&section,
					1,
					SALE_PRICE,
				),
				&[&buyer],
			)
			.expect_err("a cancelled listing cannot be purchased");
		assert_custom_error(&not_for_sale, BitflipError::SectionNotForSale);

		program
			.send_with_signers(list(), &[&seller])
			.expect("owner relists the section");
		let before_seller = program.balance(&seller.pubkey()).expect("seller balance");
		let before_buyer = program.balance(&buyer.pubkey()).expect("buyer balance");
		let before_section = program.balance(&section).expect("section balance");
		let slippage = program
			.send_with_signers(
				purchase_section_instruction(
					&program,
					&buyer.pubkey(),
					&seller.pubkey(),
					&game,
					&section,
					1,
					SALE_PRICE - 1,
				),
				&[&buyer],
			)
			.expect_err("buyer maximum protects against a changed listing price");
		assert_custom_error(&slippage, BitflipError::PriceSlippage);
		assert_eq!(
			program.balance(&seller.pubkey()).expect("seller balance"),
			before_seller
		);
		assert_eq!(
			program.balance(&buyer.pubkey()).expect("buyer balance"),
			before_buyer
		);

		program
			.send_with_signers(
				purchase_section_instruction(
					&program,
					&buyer.pubkey(),
					&seller.pubkey(),
					&game,
					&section,
					1,
					SALE_PRICE,
				),
				&[&buyer],
			)
			.expect("buyer purchases at the listed price");
		let section_account = program.account(&section).expect("fetch sold section");
		assert_eq!(
			&section_account.data[1..33],
			buyer.pubkey().to_bytes().as_slice()
		);
		assert_eq!(u64_at(&section_account.data, 163), 0);
		assert_eq!(u64_at(&section_account.data, 259), 0);
		assert_eq!(
			program.balance(&seller.pubkey()).expect("seller balance"),
			before_seller + SALE_PRICE + owner_share
		);
		assert_eq!(
			program.balance(&buyer.pubkey()).expect("buyer balance"),
			before_buyer - SALE_PRICE
		);
		assert_eq!(
			program.balance(&section).expect("section balance"),
			before_section - owner_share
		);

		let old_owner = program
			.send_with_signers(
				seal_section_instruction(&program, &seller.pubkey(), &game, &section, 1),
				&[&seller],
			)
			.expect_err("seller loses owner authority immediately");
		assert!(!old_owner.message().is_empty());
		program
			.send_with_signers(
				seal_section_instruction(&program, &buyer.pubkey(), &game, &section, 1),
				&[&buyer],
			)
			.expect("buyer receives owner authority");

		let stale_mint = program
			.send_with_signers(
				record_mint_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&game,
					&section,
					1,
					&seller.pubkey(),
					&Keypair::new().pubkey(),
					&Keypair::new().pubkey(),
					0,
				),
				&[&authority],
			)
			.expect_err("a stale mint cannot target the seller after purchase");
		assert_custom_error(&stale_mint, BitflipError::OwnerChanged);

		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn authority_rotation_requires_both_signers_and_revokes_the_old_authority() {
	pina_test::run(async {
		let (mut program, authority, config, _) = start_config().await;
		let pending_authority = Keypair::new();
		let outsider = Keypair::new();
		program
			.fund(&pending_authority.pubkey(), 1_000_000)
			.expect("fund pending authority");
		program
			.fund(&outsider.pubkey(), 1_000_000)
			.expect("fund outsider");

		program
			.send_with_signers(
				propose_authority_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&pending_authority.pubkey(),
				),
				&[&authority],
			)
			.expect("current authority proposes a successor");
		let wrong_acceptance = program
			.send_with_signers(
				accept_authority_instruction(&program, &outsider.pubkey(), &config),
				&[&outsider],
			)
			.expect_err("an unrelated signer cannot accept authority");
		assert!(!wrong_acceptance.message().is_empty());

		program
			.send_with_signers(
				accept_authority_instruction(&program, &pending_authority.pubkey(), &config),
				&[&pending_authority],
			)
			.expect("the proposed signer accepts authority");
		let config_account = program.account(&config).expect("fetch config");
		assert_eq!(
			&config_account.data[2..34],
			pending_authority.pubkey().to_bytes().as_slice()
		);
		assert_eq!(&config_account.data[34..66], &[0; 32]);

		let old_authority = program
			.send_with_signers(
				update_config_instruction(
					&program,
					&authority.pubkey(),
					&config,
					&authority.pubkey(),
					&authority.pubkey(),
					DEFAULT_EARLY_UNLOCK_FLIPS,
				),
				&[&authority],
			)
			.expect_err("old authority loses control immediately");
		assert!(!old_authority.message().is_empty());
		program
			.send_with_signers(
				update_config_instruction(
					&program,
					&pending_authority.pubkey(),
					&config,
					&pending_authority.pubkey(),
					&pending_authority.pubkey(),
					DEFAULT_EARLY_UNLOCK_FLIPS,
				),
				&[&pending_authority],
			)
			.expect("new authority controls configuration");
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn only_the_owner_can_seal_an_active_section() {
	pina_test::run(async {
		let CustodyGame {
			mut program,
			authority,
			config,
			game,
			bit_mint,
			bit_reserve,
			..
		} = start_game_with_custody(1).await;
		let owner = Keypair::new();
		let outsider = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section owner");
		program
			.fund(&outsider.pubkey(), 1_000_000)
			.expect("fund outsider");
		let section = claim_first_user_section(
			&mut program,
			&authority,
			&config,
			&game,
			&owner,
			&bit_mint.pubkey(),
			&bit_reserve,
		)
		.await;

		let unauthorized = program
			.send_with_signers(
				seal_section_instruction(&program, &outsider.pubkey(), &game, &section, 1),
				&[&outsider],
			)
			.expect_err("non-owner cannot seal a section");
		assert!(!unauthorized.message().is_empty());
		assert_eq!(
			program.account(&section).expect("fetch section").data[131],
			SECTION_STATUS_ACTIVE
		);

		program
			.send_with_signers(
				seal_section_instruction(&program, &owner.pubkey(), &game, &section, 1),
				&[&owner],
			)
			.expect("owner seals the section");
		assert_eq!(
			program.account(&section).expect("fetch section").data[131],
			SECTION_STATUS_SEALED
		);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn mint_recording_requires_sealed_state_and_collection_authority() {
	pina_test::run(async {
		let CustodyGame {
			mut program,
			authority,
			config,
			game,
			bit_mint,
			bit_reserve,
			..
		} = start_game_with_custody(1).await;
		let owner = Keypair::new();
		let outsider = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section owner");
		program
			.fund(&outsider.pubkey(), 1_000_000)
			.expect("fund outsider");
		let section = claim_first_user_section(
			&mut program,
			&authority,
			&config,
			&game,
			&owner,
			&bit_mint.pubkey(),
			&bit_reserve,
		)
		.await;
		let asset_id = Keypair::new().pubkey();
		let merkle_tree = Keypair::new().pubkey();
		let record = |collection_authority: &Pubkey| {
			record_mint_instruction(
				&program,
				collection_authority,
				&config,
				&game,
				&section,
				1,
				&owner.pubkey(),
				&asset_id,
				&merkle_tree,
				42,
			)
		};

		let active = program
			.send_with_signers(record(&authority.pubkey()), &[&authority])
			.expect_err("an active section cannot record a mint");
		assert_custom_error(&active, BitflipError::SectionNotSealed);
		program
			.send_with_signers(
				seal_section_instruction(&program, &owner.pubkey(), &game, &section, 1),
				&[&owner],
			)
			.expect("seal section");

		let unauthorized = program
			.send_with_signers(record(&outsider.pubkey()), &[&outsider])
			.expect_err("an unrelated signer cannot record a mint");
		assert!(!unauthorized.message().is_empty());
		program
			.send_with_signers(record(&authority.pubkey()), &[&authority])
			.expect("collection authority records the mint");

		let section_account = program.account(&section).expect("fetch section");
		assert_eq!(section_account.data[131], SECTION_STATUS_MINTED);
		assert_eq!(
			&section_account.data[33..65],
			asset_id.to_bytes().as_slice()
		);
		assert_eq!(
			&section_account.data[65..97],
			merkle_tree.to_bytes().as_slice()
		);
		assert_eq!(u32_at(&section_account.data, 135), 42);
		let game_account = program.account(&game).expect("fetch game");
		assert_eq!(
			u16::from_le_bytes([game_account.data[15], game_account.data[16]]),
			1
		);

		let duplicate = program
			.send_with_signers(record(&authority.pubkey()), &[&authority])
			.expect_err("a recorded mint cannot be overwritten");
		assert_custom_error(&duplicate, BitflipError::SectionAlreadyMinted);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn burst_traffic_preserves_real_sbf_accounting() {
	const TRANSACTION_COUNT: usize = 128;
	const PIXELS_PER_TRANSACTION: u64 = 16;

	pina_test::run(async {
		let CustodyGame {
			mut program,
			config,
			game,
			bit_mint,
			initial_section_vault,
			..
		} = start_game_with_custody(DEFAULT_EARLY_UNLOCK_FLIPS).await;
		let (section, _) = section_address(&program.program_id(), 0, 0);
		let player = program.payer();
		let player_bit_account = create_player_bit_account(&program, &player, &bit_mint.pubkey());
		let section_lamports_before = program
			.balance(&section)
			.expect("section balance before burst");
		let ambiguous_batches = std::thread::scope(|scope| {
			let program_ref = &program;
			let handles: Vec<_> = (0..TRANSACTION_COUNT)
				.map(|batch_index| {
					let coordinates = coordinates_for_batch(batch_index);
					let instruction = flip_pixels_instruction(
						program_ref,
						[&player, &config, &game, &section, &bit_mint.pubkey()],
						&coordinates,
						TestFlipLimits::full_reward(PIXELS_PER_TRANSACTION as usize),
					);

					(
						batch_index,
						scope.spawn(move || program_ref.send_instruction(instruction)),
					)
				})
				.collect();

			handles
				.into_iter()
				.filter_map(|(batch_index, handle)| {
					handle
						.join()
						.expect("burst worker does not panic")
						.err()
						.map(|_| batch_index)
				})
				.collect::<Vec<_>>()
		});

		let burst_snapshot = program.account(&section).expect("section after burst");
		for batch_index in ambiguous_batches {
			let coordinates = coordinates_for_batch(batch_index);
			let first_landed =
				section_pixel_is_on(&burst_snapshot.data, coordinates[0].0, coordinates[0].1);
			assert!(
				coordinates
					.iter()
					.all(|(x, y)| section_pixel_is_on(&burst_snapshot.data, *x, *y) == first_landed),
				"an atomic batch can never leave partially toggled pixels"
			);
			if !first_landed {
				program
					.send_instruction(flip_pixels_instruction(
						&program,
						[&player, &config, &game, &section, &bit_mint.pubkey()],
						&coordinates,
						TestFlipLimits::full_reward(PIXELS_PER_TRANSACTION as usize),
					))
					.expect("resubmit a confirmed-missing contended batch");
			}
		}

		let expected_flips =
			u64::try_from(TRANSACTION_COUNT).expect("transaction count") * PIXELS_PER_TRANSACTION;
		let section_account = program.account(&section).expect("section after burst");
		let game_account = program.account(&game).expect("game after burst");
		assert_eq!(u64_at(&section_account.data, 139), expected_flips);
		assert_eq!(
			u64_at(&section_account.data, 147),
			u64::try_from(TRANSACTION_COUNT).expect("transaction count")
		);
		assert_eq!(u64_at(&game_account.data, 25), 0);
		assert_eq!(u64_at(&section_account.data, 211), expected_flips);
		assert_eq!(u64_at(&section_account.data, 219), expected_flips);
		assert_eq!(
			u64_at(&section_account.data, 251),
			expected_flips * DEFAULT_START_PRICE_LAMPORTS
		);
		assert_eq!(token_amount(&program, &player_bit_account), expected_flips);
		assert_eq!(
			token_amount(&program, &initial_section_vault),
			BIT_SECTION_ALLOCATION_TOKENS - expected_flips
		);
		assert_eq!(
			program
				.balance(&section)
				.expect("section balance after burst"),
			section_lamports_before + expected_flips * DEFAULT_START_PRICE_LAMPORTS
		);

		let exhausted_section = section_account.data.clone();
		let exhausted = program
			.send_instruction(flip_pixels_instruction(
				&program,
				[&player, &config, &game, &section, &bit_mint.pubkey()],
				&[(0, 0)],
				TestFlipLimits {
					minimum_reward_tokens: 0,
					..TestFlipLimits::full_reward(1)
				},
			))
			.expect_err("a zero-minimum custom client cannot bypass the reward cap");
		assert_custom_error(&exhausted, BitflipError::InsufficientReward);
		assert_eq!(
			program
				.account(&section)
				.expect("section after rejected overflow")
				.data,
			exhausted_section,
		);
		assert_eq!(token_amount(&program, &player_bit_account), expected_flips);
		assert_eq!(
			token_amount(&program, &initial_section_vault),
			BIT_SECTION_ALLOCATION_TOKENS - expected_flips
		);

		let other_section = Pubkey::new_unique();
		let other_player = Pubkey::new_unique();
		let first = flip_pixels_instruction(
			&program,
			[&player, &config, &game, &section, &bit_mint.pubkey()],
			&[(0, 0)],
			TestFlipLimits::full_reward(1),
		);
		let second = flip_pixels_instruction(
			&program,
			[
				&other_player,
				&config,
				&game,
				&other_section,
				&bit_mint.pubkey(),
			],
			&[(0, 0)],
			TestFlipLimits {
				section_index: 1,
				..TestFlipLimits::full_reward(1)
			},
		);
		let shared_writable: Vec<Pubkey> = first
			.accounts
			.iter()
			.filter(|account| account.is_writable)
			.filter(|account| {
				second
					.accounts
					.iter()
					.any(|candidate| candidate.is_writable && candidate.pubkey == account.pubkey)
			})
			.map(|account| account.pubkey)
			.collect();
		assert!(
			shared_writable.is_empty(),
			"independent players and sections share no writable accounts"
		);

		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn independent_sections_process_concurrent_reward_traffic() {
	const TRANSACTIONS_PER_SECTION: usize = 64;

	pina_test::run(async {
		let CustodyGame {
			mut program,
			authority,
			config,
			game,
			bit_mint,
			bit_reserve,
			initial_section_vault,
		} = start_game_with_custody(1).await;
		let section_one_player = Keypair::new();
		program
			.fund(&section_one_player.pubkey(), 200_000_000)
			.expect("fund second section player");
		let section_one = claim_first_user_section(
			&mut program,
			&authority,
			&config,
			&game,
			&section_one_player,
			&bit_mint.pubkey(),
			&bit_reserve,
		)
		.await;
		let (section_zero, _) = section_address(&program.program_id(), 0, 0);
		let section_zero_player = program.payer();
		let section_zero_player_bits =
			create_player_bit_account(&program, &section_zero_player, &bit_mint.pubkey());
		let section_one_player_bits = get_associated_token_address_with_program_id(
			&section_one_player.pubkey(),
			&bit_mint.pubkey(),
			&spl_token_2022_interface::id(),
		);
		let section_one_vault = get_associated_token_address_with_program_id(
			&section_one,
			&bit_mint.pubkey(),
			&spl_token_2022_interface::id(),
		);

		let ambiguous = std::thread::scope(|scope| {
			let program_ref = &program;
			let owner_ref = &section_one_player;
			let handles: Vec<_> = (0..TRANSACTIONS_PER_SECTION * 2)
				.map(|transaction_index| {
					let section_index = u8::try_from(transaction_index % 2).expect("section index");
					let batch_index = transaction_index / 2;
					let (player, section, coordinates) = if section_index == 0 {
						(
							section_zero_player,
							section_zero,
							coordinates_for_batch(batch_index + 1),
						)
					} else {
						(
							owner_ref.pubkey(),
							section_one,
							coordinates_for_batch(batch_index),
						)
					};
					let instruction = flip_pixels_instruction(
						program_ref,
						[&player, &config, &game, &section, &bit_mint.pubkey()],
						&coordinates,
						TestFlipLimits {
							section_index,
							..TestFlipLimits::full_reward(16)
						},
					);
					(
						(section_index, batch_index),
						scope.spawn(move || {
							if section_index == 0 {
								program_ref.send_instruction(instruction)
							} else {
								program_ref.send_with_signers(instruction, &[owner_ref])
							}
						}),
					)
				})
				.collect();

			handles
				.into_iter()
				.filter_map(|(batch, handle)| {
					handle
						.join()
						.expect("multi-section worker does not panic")
						.err()
						.map(|_| batch)
				})
				.collect::<Vec<_>>()
		});

		let section_zero_snapshot = program
			.account(&section_zero)
			.expect("section zero after burst");
		let section_one_snapshot = program
			.account(&section_one)
			.expect("section one after burst");
		for (section_index, batch_index) in ambiguous {
			let (player, section, coordinates, snapshot) = if section_index == 0 {
				(
					section_zero_player,
					section_zero,
					coordinates_for_batch(batch_index + 1),
					&section_zero_snapshot,
				)
			} else {
				(
					section_one_player.pubkey(),
					section_one,
					coordinates_for_batch(batch_index),
					&section_one_snapshot,
				)
			};
			if !section_pixel_is_on(&snapshot.data, coordinates[0].0, coordinates[0].1) {
				let instruction = flip_pixels_instruction(
					&program,
					[&player, &config, &game, &section, &bit_mint.pubkey()],
					&coordinates,
					TestFlipLimits {
						section_index,
						..TestFlipLimits::full_reward(16)
					},
				);
				if section_index == 0 {
					program.send_instruction(instruction)
				} else {
					program.send_with_signers(instruction, &[&section_one_player])
				}
				.expect("resubmit a confirmed-missing sharded batch");
			}
		}

		let section_reward_tokens =
			u64::try_from(TRANSACTIONS_PER_SECTION * 16).expect("section reward token count");
		let section_zero_account = program.account(&section_zero).expect("final section zero");
		let section_one_account = program.account(&section_one).expect("final section one");
		assert_eq!(
			u64_at(&section_zero_account.data, 211),
			section_reward_tokens + 1
		);
		assert_eq!(
			u64_at(&section_one_account.data, 211),
			section_reward_tokens
		);
		assert_eq!(
			token_amount(&program, &section_zero_player_bits),
			section_reward_tokens
		);
		assert_eq!(
			token_amount(&program, &section_one_player_bits),
			section_reward_tokens + 1,
		);
		assert_eq!(
			token_amount(&program, &initial_section_vault),
			BIT_SECTION_ALLOCATION_TOKENS - section_reward_tokens - 1,
		);
		assert_eq!(
			token_amount(&program, &section_one_vault),
			BIT_SECTION_ALLOCATION_TOKENS - section_reward_tokens,
		);

		program.stop().expect("stop isolated program test");
	});
}
