use steel::*;

use super::BitflipInstruction;
use crate::ADMIN_PUBKEY;
use crate::BitflipError;
use crate::ConfigState;
use crate::ID;
use crate::constants::*;
use crate::get_pda_config;
use crate::get_pda_mint_bit;
use crate::get_pda_mint_gibibit;
use crate::get_pda_mint_kibibit;
use crate::get_pda_mint_mebibit;
use crate::get_pda_treasury;

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
	let [admin_info, authority_info, config_info, system_program_info] = accounts else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	if admin_info.key.ne(&ADMIN_PUBKEY) {
		return Err(BitflipError::UnauthorizedAdmin.into());
	}

	if authority_info.key.eq(&ADMIN_PUBKEY) {
		return Err(BitflipError::DuplicateAuthority.into());
	}

	config_info.is_empty()?.is_writable()?;
	admin_info.is_signer()?;
	authority_info.is_signer()?.is_writable()?;
	system_program_info.is_program(&system_program::ID)?;

	let (config_key, config_bump) = get_pda_config();
	let treasury_bump = get_pda_treasury().1;
	let mint_bit_bump = get_pda_mint_bit().1;
	let mint_kibibit_bump = get_pda_mint_kibibit().1;
	let mint_mebibit_bump = get_pda_mint_mebibit().1;
	let mint_gibibit_bump = get_pda_mint_gibibit().1;

	if config_info.key.ne(&config_key) {
		return Err(ProgramError::InvalidSeeds);
	}

	// initialize config
	create_account_with_bump::<ConfigState>(
		config_info,
		system_program_info,
		authority_info,
		&ID,
		&[SEED_PREFIX, SEED_CONFIG],
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
	use crate::leak;

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

	fn create_account_infos() -> [AccountInfo<'static>; 4] {
		let admin_lamports = leak(0);
		let admin_data = leak(vec![]);
		let authority_key = leak(Pubkey::new_unique());
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = leak(vec![]);
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

		[admin_info, authority_info, config_info, system_program_info]
	}
}
