use steel::*;

use crate::Initialize;
use crate::UpdateAuthority;
use crate::get_pda_config;
use crate::get_pda_mint_bit;
use crate::get_pda_mint_gibibit;
use crate::get_pda_mint_kibibit;
use crate::get_pda_mint_mebibit;
use crate::get_pda_treasury;
use crate::get_treasury_token_account;

/// Create an instruction to initialize the mint, treasury and [`ConfigState`].
///
/// ### Arguments
///
/// * `admin` - The admin account which protects this instruction from
///   outsiders: must be a signer.
/// * `authority` - The authority account: must be a signer.
///
/// When using this instruction in a transaction you will need to make sure both
/// the admin and authority are signers for the transaction.
pub fn initialize(admin: &Pubkey, authority: &Pubkey) -> Instruction {
	let config = get_pda_config().0;
	let treasury = get_pda_treasury().0;
	let mint_bit = get_pda_mint_bit().0;
	let treasury_bit_token_account = get_treasury_token_account(&treasury, &mint_bit);
	let mint_kibibit = get_pda_mint_kibibit().0;
	let treasury_kibibit_token_account = get_treasury_token_account(&treasury, &mint_kibibit);
	let mint_mebibit = get_pda_mint_mebibit().0;
	let treasury_mebibit_token_account = get_treasury_token_account(&treasury, &mint_mebibit);
	let mint_gibibit = get_pda_mint_gibibit().0;
	let treasury_gibibit_token_account = get_treasury_token_account(&treasury, &mint_gibibit);
	let associated_token_program = spl_associated_token_account::ID;
	let token_program = spl_token_2022::ID;
	let system_program = system_program::ID;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new_readonly(*admin, true),
			AccountMeta::new(*authority, true),
			AccountMeta::new(config, false),
			AccountMeta::new(treasury, false),
			AccountMeta::new(mint_bit, false),
			AccountMeta::new(treasury_bit_token_account, false),
			AccountMeta::new(mint_kibibit, false),
			AccountMeta::new(treasury_kibibit_token_account, false),
			AccountMeta::new(mint_mebibit, false),
			AccountMeta::new(treasury_mebibit_token_account, false),
			AccountMeta::new(mint_gibibit, false),
			AccountMeta::new(treasury_gibibit_token_account, false),
			AccountMeta::new_readonly(associated_token_program, false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(system_program, false),
		],
		data: Initialize {}.to_bytes(),
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
