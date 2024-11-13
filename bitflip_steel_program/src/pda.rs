use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use steel::ProgramError;

use crate::ID;
use crate::SEED_CONFIG;
use crate::SEED_GAME;
use crate::SEED_GAME_NONCE;
use crate::SEED_MINT;
use crate::SEED_PLAYER;
use crate::SEED_PREFIX;
use crate::SEED_SECTION_DATA;
use crate::SEED_SECTION_STATE;
use crate::SEED_TREASURY;

pub fn get_pda_config() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_CONFIG], &ID)
}

pub fn create_pda_config(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_CONFIG, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_derived_player(player: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_PLAYER, player.as_ref()], &ID)
}

pub fn create_pda_derived_player(player: &Pubkey, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey =
		Pubkey::create_program_address(&[SEED_PREFIX, SEED_PLAYER, player.as_ref(), &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_derived_player_token_account(player: &Pubkey) -> Pubkey {
	let player_pda = get_pda_derived_player(player).0;
	let mint = get_pda_mint().0;
	let token_program = spl_token_2022::ID;

	get_associated_token_address_with_program_id(&player_pda, &mint, &token_program)
}

pub fn get_pda_treasury() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_TREASURY], &ID)
}

pub fn create_pda_treasury(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_TREASURY, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_treasury_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint = get_pda_mint().0;
	let token_program = spl_token_2022::ID;

	get_associated_token_address_with_program_id(&treasury, &mint, &token_program)
}

pub fn get_pda_mint() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_MINT], &ID)
}

pub fn create_pda_mint(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_MINT, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_game(game_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_GAME, &game_index.to_le_bytes()], &ID)
}

pub fn create_pda_game(game_index: u8, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(
		&[SEED_PREFIX, SEED_GAME, &game_index.to_le_bytes(), &[bump]],
		&ID,
	)?;
	Ok(pubkey)
}

pub fn get_pda_game_nonce(game_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_GAME_NONCE,
		],
		&ID,
	)
}

pub fn create_pda_game_nonce(game_index: u8, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_GAME_NONCE,
			&[bump],
		],
		&ID,
	)?;
	Ok(pubkey)
}

pub fn get_pda_section_state(game_index: u8, section_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_SECTION_STATE,
			&section_index.to_le_bytes(),
		],
		&ID,
	)
}

pub fn create_pda_section_state(
	game_index: u8,
	section_index: u8,
	bump: u8,
) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_SECTION_STATE,
			&section_index.to_le_bytes(),
			&[bump],
		],
		&ID,
	)?;
	Ok(pubkey)
}

pub fn get_pda_section_data(game_index: u8, section_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_SECTION_DATA,
			&section_index.to_le_bytes(),
		],
		&ID,
	)
}

pub fn create_pda_section_data(
	game_index: u8,
	section_index: u8,
	bump: u8,
) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_SECTION_DATA,
			&section_index.to_le_bytes(),
			&[bump],
		],
		&ID,
	)?;
	Ok(pubkey)
}

pub fn get_section_token_account(game_index: u8, section_index: u8) -> Pubkey {
	let section = get_pda_section_state(game_index, section_index).0;
	let mint = get_pda_mint().0;
	let token_program = spl_token_2022::ID;

	get_associated_token_address_with_program_id(&section, &mint, &token_program)
}

pub fn get_player_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint().0;
	let token_program = spl_token_2022::ID;

	get_associated_token_address_with_program_id(player, &mint, &token_program)
}
