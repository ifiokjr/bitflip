use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use steel::ProgramError;

use crate::ID;
use crate::SEED_BIT_MINT;
use crate::SEED_BYTE_MINT;
use crate::SEED_CONFIG;
use crate::SEED_GAME;
use crate::SEED_GAME_NONCE;
use crate::SEED_GIGA_BIT_MINT;
use crate::SEED_KILO_BIT_MINT;
use crate::SEED_MEGA_BIT_MINT;
use crate::SEED_PLAYER;
use crate::SEED_PREFIX;
use crate::SEED_SECTION;
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
	let mint = get_pda_mint_bit().0;
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

pub fn get_treasury_bit_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let bit_mint = get_pda_mint_bit().0;

	get_treasury_token_account(&treasury, &bit_mint)
}

pub fn get_treasury_kilo_bit_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let bit_kilo_bit = get_pda_bit_kilo_bit().0;

	get_treasury_token_account(&treasury, &bit_kilo_bit)
}

pub fn get_treasury_mega_bit_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint_mega_bit = get_pda_mint_mega_bit().0;

	get_treasury_token_account(&treasury, &mint_mega_bit)
}

pub fn get_treasury_giga_bit_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint_giga_bit = get_pda_mint_giga_bit().0;

	get_treasury_token_account(&treasury, &mint_giga_bit)
}

pub fn get_treasury_token_account(treasury: &Pubkey, mint: &Pubkey) -> Pubkey {
	get_associated_token_address_with_program_id(treasury, mint, &spl_token_2022::ID)
}

pub fn get_pda_mint_bit() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_BIT_MINT], &ID)
}

pub fn create_pda_mint_bit(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_BIT_MINT, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_mint_byte() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_BYTE_MINT], &ID)
}

pub fn create_pda_mint_byte(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_BYTE_MINT, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_bit_kilo_bit() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_KILO_BIT_MINT], &ID)
}

pub fn create_pda_bit_kilo_bit(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_KILO_BIT_MINT, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_mint_mega_bit() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_MEGA_BIT_MINT], &ID)
}

pub fn create_pda_mint_mega_bit(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_MEGA_BIT_MINT, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_mint_giga_bit() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_GIGA_BIT_MINT], &ID)
}

pub fn create_pda_mint_giga_bit(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_GIGA_BIT_MINT, &[bump]], &ID)?;
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

pub fn get_section_bit_token_account(game_index: u8, section_index: u8) -> Pubkey {
	let section = get_pda_section(game_index, section_index).0;
	let mint = get_pda_mint_bit().0;

	get_section_token_account(&section, &mint)
}

pub fn get_section_token_account(section: &Pubkey, mint: &Pubkey) -> Pubkey {
	get_associated_token_address_with_program_id(section, mint, &spl_token_2022::ID)
}

pub fn get_player_bit_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint_bit().0;

	get_player_token_account(player, &mint)
}

pub fn get_player_kilo_bit_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_bit_kilo_bit().0;

	get_player_token_account(player, &mint)
}

pub fn get_player_mega_bit_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint_mega_bit().0;

	get_player_token_account(player, &mint)
}

pub fn get_player_giga_bit_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint_giga_bit().0;

	get_player_token_account(player, &mint)
}

pub fn get_player_token_account(player: &Pubkey, mint: &Pubkey) -> Pubkey {
	get_associated_token_address_with_program_id(player, mint, &spl_token_2022::ID)
}
