#![cfg(test)]

use pina_test::AccountMeta;
use pina_test::Keypair;
use pina_test::ProgramTest;
use pina_test::Pubkey;
use pina_test::Signer;
use pina_test::TestError;
use program_under_test::BitflipAccountType;
use program_under_test::BitflipError;
use program_under_test::BitflipInstruction;
use program_under_test::ConfigState;
use program_under_test::DEFAULT_CLAIM_PRICE_LAMPORTS;
use program_under_test::DEFAULT_EARLY_UNLOCK_FLIPS;
use program_under_test::DEFAULT_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_MAX_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_MIN_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_UNLOCK_INTERVAL_SECONDS;
use program_under_test::GAME_STATUS_LIVE;
use program_under_test::ID;
use program_under_test::SECTION_STATUS_ACTIVE;
use program_under_test::SECTION_STATUS_MINTED;
use program_under_test::SECTION_STATUS_SEALED;
use program_under_test::SectionState;

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
				&authority.pubkey(),
				&config,
				&game,
				&initial_section,
				0,
				game_bump,
				section_bump,
			),
			&[&authority],
		)
		.expect("initialize game with test authority");
	(program, authority, config, game)
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
	payer: &Pubkey,
	config: &Pubkey,
	game: &Pubkey,
	initial_section: &Pubkey,
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

#[allow(clippy::too_many_arguments)]
fn flip_pixels_instruction(
	program: &ProgramTest,
	player: &Pubkey,
	config: &Pubkey,
	game: &Pubkey,
	section: &Pubkey,
	treasury: &Pubkey,
	coordinates: &[(u8, u8)],
	maximum_total_fee_lamports: u64,
) -> pina_test::Instruction {
	let mut packed_coordinates = [0; 32];
	for (index, (x, y)) in coordinates.iter().enumerate() {
		packed_coordinates[index * 2] = *x;
		packed_coordinates[index * 2 + 1] = *y;
	}
	let mut data = Vec::with_capacity(44);
	data.extend_from_slice(&[
		BitflipInstruction::FlipPixels as u8,
		0,
		0,
		coordinates.len() as u8,
	]);
	data.extend_from_slice(&packed_coordinates);
	data.extend_from_slice(&maximum_total_fee_lamports.to_le_bytes());
	program.instruction(
		&data,
		vec![
			AccountMeta::new(*player, true),
			AccountMeta::new_readonly(*config, false),
			AccountMeta::new(*game, false),
			AccountMeta::new(*section, false),
			AccountMeta::new(*treasury, false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
}

async fn claim_first_user_section(
	program: &mut ProgramTest,
	authority: &Keypair,
	config: &Pubkey,
	game: &Pubkey,
	owner: &Keypair,
) -> Pubkey {
	let (initial_section, _) = section_address(&program.program_id(), 0, 0);
	program
		.send_with_signers(
			flip_pixels_instruction(
				program,
				&owner.pubkey(),
				config,
				game,
				&initial_section,
				&authority.pubkey(),
				&[(0, 0)],
				DEFAULT_FLIP_FEE_LAMPORTS,
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
			AccountMeta::new_readonly(*owner, true),
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
		assert_eq!(account.data[1], 1, "config ABI version");
		assert_eq!(u64_at(&account.data, 130), DEFAULT_CLAIM_PRICE_LAMPORTS);
		assert_eq!(u64_at(&account.data, 138), DEFAULT_FLIP_FEE_LAMPORTS);
		assert_eq!(u64_at(&account.data, 146), DEFAULT_MIN_FLIP_FEE_LAMPORTS);
		assert_eq!(u64_at(&account.data, 154), DEFAULT_MAX_FLIP_FEE_LAMPORTS);
		assert_eq!(u32_at(&account.data, 162), DEFAULT_UNLOCK_INTERVAL_SECONDS);
		assert_eq!(u32_at(&account.data, 166), DEFAULT_EARLY_UNLOCK_FLIPS);
		assert_eq!(account.data[172], bump);

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
				&payer,
				&config,
				&game,
				&initial_section,
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
		assert_eq!(
			u16::from_le_bytes([game_account.data[12], game_account.data[13]]),
			1,
			"only the bootstrapped section exists"
		);
		let section_account = program
			.account(&initial_section)
			.expect("fetch initial section");
		assert_eq!(section_account.owner, program.program_id());
		assert_eq!(section_account.data.len(), SectionState::SIZE);
		assert_eq!(&section_account.data[1..33], game.to_bytes().as_slice());
		assert_eq!(section_account.data[98], 0, "initial section index");
		assert_eq!(section_account.data[99], SECTION_STATUS_ACTIVE);

		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn claims_enforce_order_and_activity_unlocks() {
	pina_test::run(async {
		let (mut program, authority, config, game) = start_game(1).await;
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
					&player.pubkey(),
					&config,
					&game,
					&section_zero,
					&authority.pubkey(),
					&[(4, 9)],
					DEFAULT_FLIP_FEE_LAMPORTS,
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
			u16::from_le_bytes([game_account.data[12], game_account.data[13]]),
			2
		);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn claim_price_slippage_is_atomic() {
	pina_test::run(async {
		let (mut program, authority, config, game) = start_game(1).await;
		let owner = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section owner");
		let (previous_section, _) = section_address(&program.program_id(), 0, 0);
		let (section, bump) = section_address(&program.program_id(), 0, 1);
		program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					&owner.pubkey(),
					&config,
					&game,
					&previous_section,
					&authority.pubkey(),
					&[(0, 0)],
					DEFAULT_FLIP_FEE_LAMPORTS,
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
		let (mut program, authority, config, game) = start_game(DEFAULT_EARLY_UNLOCK_FLIPS).await;
		let owner = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section player");
		let (section, _) = section_address(&program.program_id(), 0, 0);
		let before = program.account(&section).expect("fetch section");
		let before_treasury = program
			.balance(&authority.pubkey())
			.expect("treasury balance");

		let duplicate = program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					&owner.pubkey(),
					&config,
					&game,
					&section,
					&authority.pubkey(),
					&[(4, 9), (4, 9)],
					DEFAULT_FLIP_FEE_LAMPORTS * 2,
				),
				&[&owner],
			)
			.expect_err("duplicate coordinates must fail");
		assert_custom_error(&duplicate, BitflipError::DuplicateCoordinate);

		let slippage = program
			.send_with_signers(
				flip_pixels_instruction(
					&program,
					&owner.pubkey(),
					&config,
					&game,
					&section,
					&authority.pubkey(),
					&[(4, 9)],
					DEFAULT_FLIP_FEE_LAMPORTS - 1,
				),
				&[&owner],
			)
			.expect_err("flip must reject a fee above the signed maximum");
		assert_custom_error(&slippage, BitflipError::PriceSlippage);

		assert_eq!(
			program.account(&section).expect("fetch section").data,
			before.data
		);
		assert_eq!(
			program
				.balance(&authority.pubkey())
				.expect("treasury balance"),
			before_treasury
		);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn owner_can_list_cancel_and_sell_a_section_atomically() {
	pina_test::run(async {
		const SALE_PRICE: u64 = 25_000_000;
		let (mut program, authority, config, game) = start_game(1).await;
		let seller = Keypair::new();
		let buyer = Keypair::new();
		program
			.fund(&seller.pubkey(), 100_000_000)
			.expect("fund seller");
		program
			.fund(&buyer.pubkey(), 100_000_000)
			.expect("fund buyer");
		let section =
			claim_first_user_section(&mut program, &authority, &config, &game, &seller).await;

		let list =
			|| list_section_instruction(&program, &seller.pubkey(), &game, &section, 1, SALE_PRICE);
		program
			.send_with_signers(list(), &[&seller])
			.expect("owner lists the section");
		assert_eq!(
			u64_at(&program.account(&section).expect("fetch listing").data, 131,),
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
		assert_eq!(u64_at(&section_account.data, 131), 0);
		assert_eq!(
			program.balance(&seller.pubkey()).expect("seller balance"),
			before_seller + SALE_PRICE
		);
		assert_eq!(
			program.balance(&buyer.pubkey()).expect("buyer balance"),
			before_buyer - SALE_PRICE
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
		let (mut program, authority, config, game) = start_game(1).await;
		let owner = Keypair::new();
		let outsider = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section owner");
		program
			.fund(&outsider.pubkey(), 1_000_000)
			.expect("fund outsider");
		let section =
			claim_first_user_section(&mut program, &authority, &config, &game, &owner).await;

		let unauthorized = program
			.send_with_signers(
				seal_section_instruction(&program, &outsider.pubkey(), &game, &section, 1),
				&[&outsider],
			)
			.expect_err("non-owner cannot seal a section");
		assert!(!unauthorized.message().is_empty());
		assert_eq!(
			program.account(&section).expect("fetch section").data[99],
			SECTION_STATUS_ACTIVE
		);

		program
			.send_with_signers(
				seal_section_instruction(&program, &owner.pubkey(), &game, &section, 1),
				&[&owner],
			)
			.expect("owner seals the section");
		assert_eq!(
			program.account(&section).expect("fetch section").data[99],
			SECTION_STATUS_SEALED
		);
		program.stop().expect("stop isolated program test");
	});
}

#[test]
#[ignore = "run with test:surfpool"]
fn mint_recording_requires_sealed_state_and_collection_authority() {
	pina_test::run(async {
		let (mut program, authority, config, game) = start_game(1).await;
		let owner = Keypair::new();
		let outsider = Keypair::new();
		program
			.fund(&owner.pubkey(), 100_000_000)
			.expect("fund section owner");
		program
			.fund(&outsider.pubkey(), 1_000_000)
			.expect("fund outsider");
		let section =
			claim_first_user_section(&mut program, &authority, &config, &game, &owner).await;
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
		assert_eq!(section_account.data[99], SECTION_STATUS_MINTED);
		assert_eq!(
			&section_account.data[33..65],
			asset_id.to_bytes().as_slice()
		);
		assert_eq!(
			&section_account.data[65..97],
			merkle_tree.to_bytes().as_slice()
		);
		assert_eq!(u32_at(&section_account.data, 103), 42);
		let game_account = program.account(&game).expect("fetch game");
		assert_eq!(
			u16::from_le_bytes([game_account.data[14], game_account.data[15]]),
			1
		);

		let duplicate = program
			.send_with_signers(record(&authority.pubkey()), &[&authority])
			.expect_err("a recorded mint cannot be overwritten");
		assert_custom_error(&duplicate, BitflipError::SectionAlreadyMinted);
		program.stop().expect("stop isolated program test");
	});
}
