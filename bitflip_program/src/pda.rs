use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use steel::ProgramError;

use crate::ID;
use crate::SEED_BIT_MINT;
use crate::SEED_CONFIG;
use crate::SEED_GAME;
use crate::SEED_GIBIBIT_MINT;
use crate::SEED_KIBIBIT_MINT;
use crate::SEED_MEBIBIT_MINT;
use crate::SEED_NONCE;
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

pub fn get_pda_nonce(address: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_NONCE, address.as_ref()], &ID)
}

pub fn create_pda_nonce(address: &Pubkey, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey =
		Pubkey::create_program_address(&[SEED_PREFIX, SEED_NONCE, address.as_ref(), &[bump]], &ID)?;
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
	let mint_bit = get_pda_mint_bit().0;

	get_treasury_token_account(&treasury, &mint_bit)
}

pub fn get_treasury_kibibit_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let kibibit = get_pda_mint_kibibit().0;

	get_treasury_token_account(&treasury, &kibibit)
}

pub fn get_treasury_mebibit_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint_mebibit = get_pda_mint_mebibit().0;

	get_treasury_token_account(&treasury, &mint_mebibit)
}

pub fn get_treasury_gibibit_token_account() -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint_gibibit = get_pda_mint_gibibit().0;

	get_treasury_token_account(&treasury, &mint_gibibit)
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

pub fn get_pda_mint_kibibit() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_KIBIBIT_MINT], &ID)
}

pub fn create_pda_mint_kibibit(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_KIBIBIT_MINT, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_mint_mebibit() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_MEBIBIT_MINT], &ID)
}

pub fn create_pda_mint_mebibit(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_MEBIBIT_MINT, &[bump]], &ID)?;
	Ok(pubkey)
}

pub fn get_pda_mint_gibibit() -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_GIBIBIT_MINT], &ID)
}

pub fn create_pda_mint_gibibit(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(&[SEED_PREFIX, SEED_GIBIBIT_MINT, &[bump]], &ID)?;
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

pub fn get_player_kibibit_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint_kibibit().0;

	get_player_token_account(player, &mint)
}

pub fn get_player_mebibit_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint_mebibit().0;

	get_player_token_account(player, &mint)
}

pub fn get_player_gibibit_token_account(player: &Pubkey) -> Pubkey {
	let mint = get_pda_mint_gibibit().0;

	get_player_token_account(player, &mint)
}

pub fn get_player_token_account(player: &Pubkey, mint: &Pubkey) -> Pubkey {
	get_associated_token_address_with_program_id(player, mint, &spl_token_2022::ID)
}
