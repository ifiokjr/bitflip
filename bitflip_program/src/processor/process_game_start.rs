use steel::*;

use crate::BitflipError;
use crate::BitflipInstruction;
use crate::ConfigState;
use crate::GameState;
use crate::ID;
use crate::SEED_CONFIG;
use crate::SEED_GAME;
use crate::SEED_PREFIX;

pub fn process_game_start(accounts: &[AccountInfo]) -> Result<(), ProgramError> {
	let [funded_signer_info, temp_signer_info, config_info, game_info, system_program_info] =
		accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let game = game_info.as_account_mut::<GameState>(&ID)?;
	let config_seeds = &[SEED_PREFIX, SEED_CONFIG, &[config.bump]];
	let game_seeds = &[
		SEED_PREFIX,
		SEED_GAME,
		&config.game_index.to_le_bytes(),
		&[game.bump],
	];

	funded_signer_info.assert_signer()?;
	temp_signer_info.assert_signer()?;
	config_info
		.assert_type::<ConfigState>(&ID)?
		.assert_seeds_with_bump(config_seeds, &ID)?;
	game_info
		.assert_type::<GameState>(&ID)?
		.assert_writable()?
		.assert_seeds_with_bump(game_seeds, &ID)?;
	system_program_info.assert_program(&system_program::ID)?;
	game.assert_err(
		|game| {
			game.temp_signer.eq(temp_signer_info.key)
				&& game.funded_signer.eq(funded_signer_info.key)
		},
		BitflipError::GameSignerInvalid,
	)?;
	game.assert_err(
		|game| game.section_index == 0,
		BitflipError::InvalidSectionIndex,
	)?;

	let current_timestamp = Clock::get()?.unix_timestamp;
	game.assert_err(
		|game| !game.running(current_timestamp),
		BitflipError::GameAlreadyStarted,
	)?;

	game.start(current_timestamp);

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct GameStart {}

instruction!(BitflipInstruction, GameStart);

#[cfg(test)]
mod tests {
	// use super::*;
}
