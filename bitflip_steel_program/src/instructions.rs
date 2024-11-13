use steel::*;

use crate::InitializeConfig;
use crate::UpdateAuthority;
use crate::get_pda_config;
use crate::get_pda_treasury;

/// Create an instruction to initialize the config.
///
/// ### Arguments
///
/// * `admin` - The admin account: must be a signer.
/// * `authority` - The authority account: must be a signer.
///
/// When using this instruction in a transaction you will need to make sure both
/// the admin and authority are signers for the transaction.
pub fn initialize_config(admin: &Pubkey, authority: &Pubkey) -> Instruction {
	let config = get_pda_config().0;
	let treasury = get_pda_treasury().0;
	let system_program = system_program::ID;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(config, false),
			AccountMeta::new_readonly(*admin, true),
			AccountMeta::new(treasury, false),
			AccountMeta::new(*authority, true),
			AccountMeta::new_readonly(system_program, false),
		],
		data: InitializeConfig {}.to_bytes(),
	}
}
/// Create an instruction to update the authority of the config.
///
/// ### Arguments
///
/// * `authority` - The current authority: must be a signer.
/// * `new_authority` - The new authority: must be a signer.
pub fn update_authority(authority: &Pubkey, new_authority: &Pubkey) -> Instruction {
	let config = get_pda_config().0;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(config, false),
			AccountMeta::new(*authority, true),
			AccountMeta::new(*new_authority, true),
		],
		data: UpdateAuthority {}.to_bytes(),
	}
}
