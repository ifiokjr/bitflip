use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use steel::ProgramError;

use crate::ID;
use crate::SEED_CONFIG;
use crate::SEED_EVENT;
use crate::SEED_GAME;
use crate::SEED_NONCE;
use crate::SEED_PLAYER;
use crate::SEED_PREFIX;
use crate::SEED_SECTION;
use crate::SEED_TREASURY;
use crate::TokenMember;

pub fn get_token_account(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
	get_associated_token_address_with_program_id(wallet, mint, &spl_token_2022::ID)
}

pub fn get_pda_config() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_CONFIG], &ID)
}

pub fn create_pda_config(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_CONFIG, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_event() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_EVENT], &ID)
}

pub fn create_pda_event(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_EVENT, &[bump]], &ID)?;
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

pub fn get_pda_nonce(address: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_NONCE, address.as_ref()], &ID)
}

pub fn create_pda_nonce(address: &Pubkey, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey =
		Pubkey::create_program_address(&[SEED_PREFIX, SEED_NONCE, address.as_ref(), &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_derived_player_token_account(player: &Pubkey, member: TokenMember) -> Pubkey {
	let player_pda = get_pda_derived_player(player).0;
	let mint = get_pda_mint(member).0;

	get_token_account(&player_pda, &mint)
}

pub fn get_pda_treasury() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_TREASURY], &ID)
}

pub fn create_pda_treasury(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_TREASURY, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_treasury_token_account(member: TokenMember) -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint = get_pda_mint(member).0;

	get_token_account(&treasury, &mint)
}

pub fn get_pda_mint(member: TokenMember) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, member.seed()], &ID)
}

pub fn create_pda_mint(member: TokenMember, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, member.seed(), &[bump]], &ID)?;
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
			SEED_NONCE,
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
			SEED_NONCE,
			&[bump],
		],
		&ID,
	)?;
	Ok(pubkey)
}

pub fn get_pda_section(game_index: u8, section_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_SECTION,
			&section_index.to_le_bytes(),
		],
		&ID,
	)
}

pub fn create_pda_section(
	game_index: u8,
	section_index: u8,
	bump: u8,
) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(
		&[
			SEED_PREFIX,
			SEED_GAME,
			&game_index.to_le_bytes(),
			SEED_SECTION,
			&section_index.to_le_bytes(),
			&[bump],
		],
		&ID,
	)?;
	Ok(pubkey)
}

pub fn get_section_token_account(game_index: u8, section_index: u8, member: TokenMember) -> Pubkey {
	let section = get_pda_section(game_index, section_index).0;
	let mint = get_pda_mint(member).0;

	get_token_account(&section, &mint)
}

pub fn get_player_token_account(player: &Pubkey, member: TokenMember) -> Pubkey {
	let mint = get_pda_mint(member).0;

	get_token_account(player, &mint)
}
