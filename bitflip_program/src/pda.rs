use solana_program::pubkey::Pubkey;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use steel::ProgramError;

use crate::TokenMember;
use crate::ID;

pub fn get_token_account(wallet: &Pubkey, mint: &Pubkey) -> Pubkey {
	get_associated_token_address_with_program_id(wallet, mint, &spl_token_2022::ID)
}

macro_rules! seeds_config {
	() => {
		&[crate::SEED_PREFIX, crate::SEED_CONFIG]
	};
	($bump:expr) => {
		&[crate::SEED_PREFIX, crate::SEED_CONFIG, &[$bump]]
	};
}

pub(crate) use seeds_config;

pub fn get_pda_config() -> (Pubkey, u8) {
	Pubkey::find_program_address(seeds_config!(), &ID)
}

pub fn create_pda_config(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(seeds_config!(bump), &ID)?;
	Ok(pubkey)
}

macro_rules! seeds_event {
	() => {
		&[crate::SEED_PREFIX, crate::SEED_EVENT]
	};
	($bump:expr) => {
		&[crate::SEED_PREFIX, crate::SEED_EVENT, &[$bump]]
	};
}

pub(crate) use seeds_event;

pub fn get_pda_event() -> (Pubkey, u8) {
	Pubkey::find_program_address(seeds_event!(), &ID)
}

pub fn create_pda_event(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(seeds_event!(bump), &ID)?;
	Ok(pubkey)
}

macro_rules! seeds_treasury {
	() => {
		&[crate::SEED_PREFIX, crate::SEED_TREASURY]
	};
	($bump:expr) => {
		&[crate::SEED_PREFIX, crate::SEED_TREASURY, &[$bump]]
	};
}

pub(crate) use seeds_treasury;

pub fn get_pda_treasury() -> (Pubkey, u8) {
	Pubkey::find_program_address(seeds_treasury!(), &ID)
}

pub fn create_pda_treasury(bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(seeds_treasury!(bump), &ID)?;
	Ok(pubkey)
}

pub fn get_treasury_token_account(member: TokenMember) -> Pubkey {
	let treasury = get_pda_treasury().0;
	let mint = get_pda_mint(member).0;

	get_token_account(&treasury, &mint)
}

macro_rules! seeds_mint {
	($member:expr) => {
		&[crate::SEED_PREFIX, $member.seed()]
	};
	($member:expr, $bump:expr) => {
		&[crate::SEED_PREFIX, $member.seed(), &[$bump]]
	};
}

pub(crate) use seeds_mint;

pub fn get_pda_mint(member: TokenMember) -> (Pubkey, u8) {
	Pubkey::find_program_address(seeds_mint!(member), &ID)
}

pub fn create_pda_mint(member: TokenMember, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(seeds_mint!(member, bump), &ID)?;
	Ok(pubkey)
}

macro_rules! seeds_game {
	($game_index:expr) => {
		&[
			crate::SEED_PREFIX,
			crate::SEED_GAME,
			&$game_index.to_le_bytes(),
		]
	};
	($game_index:expr, $bump:expr) => {
		&[
			crate::SEED_PREFIX,
			crate::SEED_GAME,
			&$game_index.to_le_bytes(),
			&[$bump],
		]
	};
}

pub(crate) use seeds_game;

pub fn get_pda_game(game_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(seeds_game!(game_index), &ID)
}

pub fn create_pda_game(game_index: u8, bump: u8) -> Result<Pubkey, ProgramError> {
	let pubkey = Pubkey::create_program_address(seeds_game!(game_index, bump), &ID)?;
	Ok(pubkey)
}

macro_rules! seeds_section {
	($game_index:expr, $section_index:expr) => {
		&[
			crate::SEED_PREFIX,
			crate::SEED_GAME,
			&$game_index.to_le_bytes(),
			crate::SEED_SECTION,
			&$section_index.to_le_bytes(),
		]
	};
	($game_index:expr, $section_index:expr, $bump:expr) => {
		&[
			crate::SEED_PREFIX,
			crate::SEED_GAME,
			&$game_index.to_le_bytes(),
			crate::SEED_SECTION,
			&$section_index.to_le_bytes(),
			&[$bump],
		]
	};
}

pub(crate) use seeds_section;

pub fn get_pda_section(game_index: u8, section_index: u8) -> (Pubkey, u8) {
	Pubkey::find_program_address(seeds_section!(game_index, section_index), &ID)
}

pub fn create_pda_section(
	game_index: u8,
	section_index: u8,
	bump: u8,
) -> Result<Pubkey, ProgramError> {
	let pubkey =
		Pubkey::create_program_address(seeds_section!(game_index, section_index, bump), &ID)?;
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
