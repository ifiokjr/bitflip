mod initialize;
mod update_authority;

use steel::*;

pub use self::initialize::*;
pub use self::update_authority::*;
use crate::ID;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum BitflipInstruction {
	Initialize = 0,
	UpdateAuthority = 1,
}

pub fn process_instruction(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	data: &[u8],
) -> ProgramResult {
	let (ix, _data) = parse_instruction(&ID, program_id, data)?;

	match ix {
		BitflipInstruction::Initialize => process_initialize(accounts)?,
		BitflipInstruction::UpdateAuthority => process_update_authority(accounts)?,
	}

	Ok(())
}
