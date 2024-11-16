mod config_initialize;
mod config_update_authority;
mod game_initialize;

use steel::*;

pub use self::config_initialize::*;
pub use self::config_update_authority::*;
pub use self::game_initialize::*;
use crate::ID;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum BitflipInstruction {
	ConfigInitialize = 0,
	GameInitialize = 1,
	ConfigUpdateAuthority = 2,
}

pub fn process_instruction(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	data: &[u8],
) -> ProgramResult {
	let (ix, _data) = parse_instruction(&ID, program_id, data)?;

	match ix {
		BitflipInstruction::ConfigInitialize => process_config_initialize(accounts)?,
		BitflipInstruction::GameInitialize => process_game_initialize(accounts)?,
		BitflipInstruction::ConfigUpdateAuthority => process_config_update_authority(accounts)?,
	}

	Ok(())
}
