use spl_pod::primitives::PodU64;
use steel::*;

use crate::seeds_config;
use crate::seeds_game;
use crate::seeds_section;
use crate::seeds_treasury;
use crate::BitflipError;
use crate::BitflipInstruction;
use crate::ConfigState;
use crate::GameState;
use crate::SectionState;
use crate::ID;

/// This instruction is used to unlock a section. It will use a nonce
/// transaction to help make each bid private.
pub fn process_section_unlock(accounts: &[AccountInfo], data: &[u8]) -> Result<(), ProgramError> {
	// parse the instruction data.
	let args = SectionUnlock::try_from_bytes(data)?;

	// load accounts
	let [owner_info, temp_signer_info, config_info, game_info, section_info, treasury_info, system_program_info] =
		accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let game = game_info.as_account_mut::<GameState>(&ID)?;
	let config_seeds_with_bump = seeds_config!(config.bump);
	let treasury_seeds_with_bump = seeds_treasury!(config.treasury_bump);
	let game_seeds_with_bump = seeds_game!(game.game_index, game.bump);
	let section_seeds = seeds_section!(game.game_index, game.section_index);
	let section_bump = section_info.assert_canonical_bump(section_seeds, &ID)?;

	owner_info.assert_signer()?.assert_writable()?;
	temp_signer_info
		.assert_empty()?
		.assert_signer()?
		.assert_owner(&system_program::ID)?;
	config_info
		.assert_type::<ConfigState>(&ID)?
		.assert_seeds_with_bump(config_seeds_with_bump, &ID)?;
	game_info
		.assert_type::<GameState>(&ID)?
		.assert_writable()?
		.assert_seeds_with_bump(game_seeds_with_bump, &ID)?;
	section_info.assert_empty()?.assert_writable()?;
	treasury_info
		.assert_writable()?
		.assert_seeds_with_bump(treasury_seeds_with_bump, &ID)?;
	system_program_info.assert_program(&system_program::ID)?;

	game.assert_err(
		|game| game.temp_signer.eq(temp_signer_info.key),
		BitflipError::GameSignerInvalid,
	)?;

	let clock = Clock::get()?;
	game.assert_err(
		|game| game.running(clock.unix_timestamp),
		BitflipError::GameNotRunning,
	)?;

	// create the section account
	create_account_with_bump::<SectionState>(
		section_info,
		system_program_info,
		owner_info,
		&ID,
		section_seeds,
		section_bump,
	)?;

	let section = section_info.as_account_mut::<SectionState>(&ID)?;
	section.init(
		*owner_info.key,
		game.game_index,
		game.section_index,
		section_bump,
	);

	msg!("transferring lamports from owner to treasury");
	treasury_info.collect(args.lamports.into(), owner_info)?;

	msg!("incrementing section index");
	game.increment_section();

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct SectionUnlock {
	/// The amount of lamports bid to unlock the section. Since the bid uses
	/// DurableNonce 's it will be a private bid and the backend will determine
	/// which is the winner and unlock the section accordingly.
	pub lamports: PodU64,
}

impl Eq for SectionUnlock {}

instruction!(BitflipInstruction, SectionUnlock);

#[cfg(test)]
mod tests {
	// use super::*;
}
