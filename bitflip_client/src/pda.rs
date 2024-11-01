use anchor_spl::token_2022;
use bitflip_program::ID_CONST;
use bitflip_program::SEED_CONFIG;
use bitflip_program::SEED_GAME;
use bitflip_program::SEED_GAME_NONCE;
use bitflip_program::SEED_MINT;
use bitflip_program::SEED_PLAYER;
use bitflip_program::SEED_PREFIX;
use bitflip_program::SEED_SECTION;
use bitflip_program::SEED_TREASURY;
use solana_sdk::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;

pub fn get_pda_config() -> (Pubkey, u8) {
	get_pda_config_with_program(&ID_CONST)
}

pub fn get_pda_config_with_program(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_CONFIG], program_id)
}

pub fn get_pda_derived_player(player: &Pubkey) -> (Pubkey, u8) {
	get_pda_derived_player_with_program(player, &ID_CONST)
}

pub fn get_pda_derived_player_with_program(player: &Pubkey, program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_PLAYER, player.as_ref()], program_id)
}

pub fn get_derived_player_token_account(player: &Pubkey) -> Pubkey {
	let player_pda = get_pda_derived_player(player).0;
	let mint = get_pda_mint().0;
	let token_program = token_2022::ID;

	get_associated_token_address_with_program_id(&player_pda, &mint, &token_program)
}

pub fn get_pda_treasury() -> (Pubkey, u8) {
	get_pda_treasury_with_program(&ID_CONST)
}

pub fn get_pda_treasury_with_program(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_TREASURY], program_id)
}

pub fn get_treasury_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint = get_pda_mint().0;
	let token_program = token_2022::ID;

	get_associated_token_address_with_program_id(&treasury, &mint, &token_program)
}

pub fn get_pda_mint() -> (Pubkey, u8) {
	get_pda_mint_with_program(&ID_CONST)
}

pub fn get_pda_mint_with_program(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_MINT], program_id)
}

pub fn get_pda_game(game_index: u8) -> (Pubkey, u8) {
	get_pda_game_with_program(game_index, &ID_CONST)
}

pub fn get_pda_game_with_program(game_index: u8, program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[SEED_PREFIX, SEED_GAME, &game_index.to_le_bytes()],
		program_id,
	)
}

pub fn get_pda_game_nonce(game_index: u8) -> (Pubkey, u8) {
	get_pda_game_nonce_with_program(game_index, &ID_CONST)
}

pub fn get_pda_game_nonce_with_program(game_index: u8, program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_GAME_NONCE,
		],
		program_id,
	)
}

pub fn get_pda_section(game_index: u8, section_index: u8) -> (Pubkey, u8) {
	get_pda_section_with_program(game_index, section_index, &ID_CONST)
}

pub fn get_pda_section_with_program(
	game_index: u8,
	section_index: u8,
	program_id: &Pubkey,
) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_SECTION,
			&section_index.to_le_bytes(),
		],
		program_id,
	)
}

pub fn get_section_token_account(game_index: u8, section_index: u8) -> Pubkey {
	let section = get_pda_section(game_index, section_index).0;
	let mint = get_pda_mint().0;
	let token_program = token_2022::ID;

	get_associated_token_address_with_program_id(&section, &mint, &token_program)
}

pub fn get_player_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint().0;
	let token_program = token_2022::ID;

	get_associated_token_address_with_program_id(player, &mint, &token_program)
}
