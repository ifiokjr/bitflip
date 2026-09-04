#![cfg(test)]

use pina_test::AccountMeta;
use pina_test::ProgramTest;
use pina_test::Pubkey;
use program_under_test::BitflipAccountType;
use program_under_test::BitflipInstruction;
use program_under_test::ConfigState;
use program_under_test::DEFAULT_CLAIM_PRICE_LAMPORTS;
use program_under_test::DEFAULT_EARLY_UNLOCK_FLIPS;
use program_under_test::DEFAULT_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_MAX_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_MIN_FLIP_FEE_LAMPORTS;
use program_under_test::DEFAULT_UNLOCK_INTERVAL_SECONDS;
use program_under_test::ID;

const CONFIG_SEED: &[u8] = b"config";
const GAME_SEED: &[u8] = b"game";

fn config_address(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[CONFIG_SEED], program_id)
}

fn game_address(program_id: &Pubkey, game_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[GAME_SEED, &[game_index]], program_id)
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
	game_index: u8,
	bump: u8,
) -> pina_test::Instruction {
	program.instruction(
		&[BitflipInstruction::InitializeGame as u8, game_index, bump],
		vec![
			AccountMeta::new(*payer, true),
			AccountMeta::new(*config, false),
			AccountMeta::new(*game, false),
			AccountMeta::new_readonly(Pubkey::default(), false),
		],
	)
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
		let error = program
			.send_instruction(initialize_game_instruction(
				&program, &payer, &config, &game, 0, game_bump,
			))
			.expect_err("only the configured authority can start a game");

		assert!(!error.message().is_empty());
		assert!(
			program.account(&game).is_err(),
			"failed start creates no PDA"
		);
		program.stop().expect("stop isolated program test");
	});
}
