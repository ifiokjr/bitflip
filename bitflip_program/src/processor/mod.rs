mod config_initialize;
mod config_update_authority;
mod event_cpi;
mod flip_bit;
mod game_initialize;
mod game_refresh_signer;
use steel::*;

pub use self::config_initialize::*;
pub use self::config_update_authority::*;
pub use self::event_cpi::*;
pub use self::flip_bit::*;
pub use self::game_initialize::*;
pub use self::game_refresh_signer::*;
use crate::ID;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive)]
pub enum BitflipInstruction {
	EventCpi = 0,
	ConfigInitialize = 1,
	ConfigUpdateAuthority = 2,
	GameInitialize = 3,
	GameRefreshSigner = 4,
	FlipBit = 10,
}

pub fn process_instruction(
	program_id: &Pubkey,
	accounts: &[AccountInfo],
	data: &[u8],
) -> ProgramResult {
	let (ix, data) = parse_instruction(&ID, program_id, data)?;

	match ix {
		BitflipInstruction::EventCpi => process_event_cpi(accounts, data)?,
		BitflipInstruction::ConfigInitialize => process_config_initialize(accounts)?,
		BitflipInstruction::GameInitialize => process_game_initialize(accounts)?,
		BitflipInstruction::ConfigUpdateAuthority => process_config_update_authority(accounts)?,
		BitflipInstruction::GameRefreshSigner => process_game_refresh_signer(accounts)?,
		BitflipInstruction::FlipBit => process_flip_bit(accounts, data)?,
	}

	Ok(())
}
