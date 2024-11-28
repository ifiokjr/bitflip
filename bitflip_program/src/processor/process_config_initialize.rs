use solana_program::msg;
use steel::*;
use sysvar::rent::Rent;

use crate::get_pda_mint;
use crate::seeds_config;
use crate::seeds_treasury;
use crate::BitflipError;
use crate::BitflipInstruction;
use crate::ConfigState;
use crate::TokenMember;
use crate::ADMIN_PUBKEY;
use crate::ID;

/// Initialize the program.
///
/// This creates the config account and the treasury account.
///
/// It also initializes the mint accounts for each token type.
///
/// TODO: [`crate::cpi::token_group_initialize`] is failing in the tests. Find a
/// way to fix.
pub fn process_config_initialize(accounts: &[AccountInfo<'_>]) -> ProgramResult {
	// load accounts
	let [admin_info, authority_info, config_info, treasury_info, system_program_info] = accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	if admin_info.key.ne(&ADMIN_PUBKEY) {
		msg!(
			"UnauthorizedAdmin: {}, expected: {}",
			admin_info.key,
			ADMIN_PUBKEY
		);
		return Err(BitflipError::UnauthorizedAdmin.into());
	}

	if authority_info.key.eq(&ADMIN_PUBKEY) {
		msg!("DuplicateAuthority: {}", authority_info.key);
		return Err(BitflipError::DuplicateAuthority.into());
	}

	let config_seeds = seeds_config!();
	let treasury_seeds = seeds_treasury!();
	let config_bump = config_info.find_canonical_bump(config_seeds, &ID)?;
	let treasury_bump = treasury_info.find_canonical_bump(treasury_seeds, &ID)?;
	let mint_bit_bump = get_pda_mint(TokenMember::Bit).1;
	let mint_kibibit_bump = get_pda_mint(TokenMember::Kibibit).1;
	let mint_mebibit_bump = get_pda_mint(TokenMember::Mebibit).1;
	let mint_gibibit_bump = get_pda_mint(TokenMember::Gibibit).1;

	admin_info.is_signer()?;
	authority_info.is_signer()?.is_writable()?;
	config_info.is_empty()?.is_writable()?;
	treasury_info
		.is_empty()?
		.is_writable()?
		.has_owner(&system_program::ID)?;
	system_program_info.is_program(&system_program::ID)?;

	// initialize config
	create_account_with_bump::<ConfigState>(
		config_info,
		system_program_info,
		authority_info,
		&ID,
		config_seeds,
		config_bump,
	)?;

	// initialize config
	let config = config_info.as_account_mut::<ConfigState>(&ID)?;
	*config = ConfigState::new(
		*authority_info.key,
		config_bump,
		treasury_bump,
		mint_bit_bump,
		mint_kibibit_bump,
		mint_mebibit_bump,
		mint_gibibit_bump,
	);

	msg!("transfer sol to treasury for rent exemption");
	let rent_sysvar = Rent::get()?;
	let extra_lamports = rent_sysvar
		.minimum_balance(treasury_info.data_len())
		.checked_sub(treasury_info.lamports())
		.ok_or(ProgramError::ArithmeticOverflow)?;
	treasury_info.collect(extra_lamports, authority_info)?;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct ConfigInitialize {}

instruction!(BitflipInstruction, ConfigInitialize);

#[cfg(test)]
mod tests {
	use assert2::check;
	use solana_sdk::native_loader;

	use super::*;
	use crate::get_pda_config;
	use crate::get_pda_treasury;
	use crate::leak;

	#[test_log::test]
	fn should_pass_validation() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_config_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::UnsupportedSysvar);

		Ok(())
	}

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_config_initialize(&accounts[0..3]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn admin_should_be_hardcoded() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let admin_info = &mut accounts[0];
		admin_info.key = leak(Pubkey::new_unique());

		let result = process_config_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::Custom(BitflipError::UnauthorizedAdmin.into()));

		Ok(())
	}

	#[test_log::test]
	fn admin_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let admin_info = &mut accounts[0];
		admin_info.is_signer = false;

		let result = process_config_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_not_be_admin() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.key = leak(ADMIN_PUBKEY);

		let result = process_config_initialize(&accounts);
		check!(
			result.unwrap_err() == ProgramError::Custom(BitflipError::DuplicateAuthority.into())
		);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_signer = false;

		let result = process_config_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_writable = false;

		let result = process_config_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[2];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_config_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[2];
		config_info.is_writable = false;

		let result = process_config_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 5] {
		let admin_lamports = leak(0);
		let admin_data = leak(vec![]);
		let authority_key = leak(Pubkey::new_unique());
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = leak(vec![]);
		let treasury_key = leak(get_pda_treasury().0);
		let treasury_lamports = leak(0);
		let treasury_data = leak(vec![]);
		let system_program_lamports = leak(1_000_000_000);
		let system_program_data = leak(vec![]);

		let admin_info = AccountInfo::new(
			&ADMIN_PUBKEY,
			true,
			false,
			admin_lamports,
			admin_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let authority_info = AccountInfo::new(
			authority_key,
			true,
			true,
			authority_lamports,
			authority_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let config_info = AccountInfo::new(
			config_key,
			false,
			true,
			config_lamports,
			config_data,
			&ID,
			false,
			u64::MAX,
		);
		let treasury_info = AccountInfo::new(
			treasury_key,
			false,
			true,
			treasury_lamports,
			treasury_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let system_program_info = AccountInfo::new(
			&system_program::ID,
			false,
			false,
			system_program_lamports,
			system_program_data,
			&native_loader::ID,
			true,
			u64::MAX,
		);

		[
			admin_info,
			authority_info,
			config_info,
			treasury_info,
			system_program_info,
		]
	}
}
