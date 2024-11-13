mod initialize_config;
mod update_authority;

use steel::*;

pub use self::initialize_config::*;
pub use self::update_authority::*;
use crate::ID;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum BitflipInstruction {
	InitializeConfig = 0,
	UpdateAuthority = 1,
}

pub fn process_instruction(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	data: &[u8],
) -> ProgramResult {
	let (ix, _data) = parse_instruction(&ID, program_id, data)?;

	match ix {
		BitflipInstruction::InitializeConfig => process_initialize_config(accounts)?,
		BitflipInstruction::UpdateAuthority => process_update_authority(accounts)?,
	}

	Ok(())
}
