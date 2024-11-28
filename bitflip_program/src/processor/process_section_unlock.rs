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

	let config_state = config_info.as_account::<ConfigState>(&ID)?;
	let game_state = game_info.as_account_mut::<GameState>(&ID)?;
	let config_seeds_with_bump = seeds_config!(config_state.bump);
	let treasury_seeds_with_bump = seeds_treasury!(config_state.treasury_bump);
	let game_seeds_with_bump = seeds_game!(game_state.game_index, game_state.bump);
	let section_seeds = seeds_section!(game_state.game_index, game_state.section_index);
	let section_bump = section_info.find_canonical_bump(section_seeds, &ID)?;

	owner_info.is_signer()?.is_writable()?;
	temp_signer_info
		.is_empty()?
		.is_signer()?
		.has_owner(&system_program::ID)?;
	config_info
		.is_type::<ConfigState>(&ID)?
		.has_seeds_with_bump(config_seeds_with_bump, &ID)?;
	game_info
		.is_type::<GameState>(&ID)?
		.is_writable()?
		.has_seeds_with_bump(game_seeds_with_bump, &ID)?;
	section_info.is_empty()?.is_writable()?;
	treasury_info
		.is_writable()?
		.has_seeds_with_bump(treasury_seeds_with_bump, &ID)?;
	system_program_info.is_program(&system_program::ID)?;

	game_state.assert_err(
		|game| game.temp_signer.eq(temp_signer_info.key),
		BitflipError::GameSignerInvalid,
	)?;

	let clock = Clock::get()?;
	game_state.assert_err(
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
	section.init(*owner_info.key, game_state.section_index, section_bump);

	msg!("transferring lamports from owner to treasury");
	treasury_info.collect(args.lamports.into(), owner_info)?;

	msg!("incrementing section index");
	game_state.increment_section();

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
