mod process_config_initialize;
mod process_config_update_authority;

mod process_flip_bit;
mod process_game_initialize;
mod process_game_reset_signers;
mod process_game_start;
mod process_game_update_temp_signer;
mod process_section_unlock;
mod process_token_group_initialize;
mod process_token_initialize;

use steel::*;

pub use self::process_config_initialize::*;
pub use self::process_config_update_authority::*;
pub use self::process_flip_bit::*;
pub use self::process_game_initialize::*;
pub use self::process_game_reset_signers::*;
pub use self::process_game_start::*;
pub use self::process_game_update_temp_signer::*;
pub use self::process_section_unlock::*;
pub use self::process_token_group_initialize::*;
pub use self::process_token_initialize::*;
use crate::ID;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum BitflipInstruction {
	ConfigInitialize = 0,
	ConfigUpdateAuthority = 1,
	TokenInitialize = 2,
	TokenGroupInitialize = 3,
	GameInitialize = 4,
	GameStart = 5,
	GameUpdateTempSigner = 6,
	GameResetSigners = 7,
	SectionUnlock = 8,
	FlipBit = 9,
}

pub fn process_instruction(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	data: &[u8],
) -> ProgramResult {
	let (ix, data) = parse_instruction(&ID, program_id, data)?;

	match ix {
		BitflipInstruction::ConfigInitialize => process_config_initialize(accounts)?,
		BitflipInstruction::ConfigUpdateAuthority => process_config_update_authority(accounts)?,
		BitflipInstruction::TokenGroupInitialize => process_token_group_initialize(accounts)?,
		BitflipInstruction::TokenInitialize => process_token_initialize(accounts, data)?,
		BitflipInstruction::GameInitialize => process_game_initialize(accounts)?,
		BitflipInstruction::GameStart => process_game_start(accounts)?,
		BitflipInstruction::GameUpdateTempSigner => process_game_update_temp_signer(accounts)?,
		BitflipInstruction::GameResetSigners => process_game_reset_signers(accounts)?,
		BitflipInstruction::SectionUnlock => process_section_unlock(accounts, data)?,
		BitflipInstruction::FlipBit => process_flip_bit(accounts, data)?,
	}

	Ok(())
}
