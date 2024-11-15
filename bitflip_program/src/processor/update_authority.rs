use steel::*;

use super::BitflipInstruction;
use crate::BitflipError;
use crate::ConfigState;
use crate::ID;
use crate::create_pda_config;

pub fn process_update_authority(accounts: &[AccountInfo]) -> ProgramResult {
	let [config_info, authority_info, new_authority_info] = accounts else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	// validate accounts
	config_info.is_writable()?.has_owner(&ID)?;
	authority_info
		.is_signer()?
		.is_writable()?
		.has_owner(&system_program::ID)?;
	new_authority_info
		.is_signer()?
		.has_owner(&system_program::ID)?;

	if config_info.data_len() < ConfigState::space() {
		return Err(ProgramError::AccountDataTooSmall);
	}

	let config = config_info.as_account_mut::<ConfigState>(&ID)?;
	let config_key = create_pda_config(config.bump)?;

	if config_info.key.ne(&config_key) {
		return Err(ProgramError::InvalidSeeds);
	}

	if authority_info.key.ne(&config.authority) {
		return Err(BitflipError::Unauthorized.into());
	}

	if new_authority_info.key.eq(&config.authority) {
		return Err(BitflipError::DuplicateAuthority.into());
	}

	config.authority = *new_authority_info.key;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct UpdateAuthority {}

instruction!(BitflipInstruction, UpdateAuthority);

#[cfg(test)]
mod tests {
	use assert2::check;
	use solana_sdk::clock::Epoch;
	use solana_sdk::sysvar::rent::Rent;

	use super::*;
	use crate::get_pda_config;
	use crate::get_pda_mint_bit;
	use crate::get_pda_treasury;
	use crate::leak;

	#[test_log::test]
	fn should_pass() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		process_update_authority(&accounts)?;

		let config_info = &accounts[0];
		let config_state = config_info.as_account::<ConfigState>(&ID)?;
		check!(config_state.authority == *accounts[2].key);

		Ok(())
	}

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_update_authority(&accounts[0..2]);

		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn config_must_not_be_empty() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		accounts[0].realloc(0, true)?;

		let result = process_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::AccountDataTooSmall);

		Ok(())
	}

	#[test_log::test]
	fn config_must_be_writeable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[0];
		config_info.is_writable = false;

		let result = process_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn config_must_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[0];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn authority_must_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_signer = false;

		let result = process_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_must_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_writable = false;

		let result = process_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_must_be_from_config() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.key = leak(Pubkey::new_unique());

		let result = process_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::Custom(BitflipError::Unauthorized.into()));

		Ok(())
	}
	#[test_log::test]
	fn new_authority_must_change() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_key = *accounts[1].key;
		let new_authority_info = &mut accounts[2];
		new_authority_info.key = leak(authority_key);

		let result = process_update_authority(&accounts);
		check!(
			result.unwrap_err() == ProgramError::Custom(BitflipError::DuplicateAuthority.into())
		);

		Ok(())
	}

	#[test_log::test]
	fn new_authority_must_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let new_authority_info = &mut accounts[2];
		new_authority_info.is_signer = false;

		let result = process_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 3] {
		let (config_key, config_bump) = leak(get_pda_config());
		let treasury_bump = get_pda_treasury().1;
		let mint_bump = get_pda_mint_bit().1;
		let authority_lamports = leak(1_000_000_000);
		let new_authority_lamports = leak(1_000_000_000);
		let authority_key = leak(Pubkey::new_unique());
		let new_authority_key = leak(Pubkey::new_unique());
		let rent_sysvar = Rent::default();
		let minimum_balance = leak(rent_sysvar.minimum_balance(ConfigState::space()));
		let mut data = vec![0u8; 8];
		data[0] = ConfigState::discriminator();
		data.append(
			&mut ConfigState::new(*authority_key, *config_bump, treasury_bump, mint_bump)
				.to_bytes()
				.to_vec(),
		);
		let authority_data = leak(vec![]);
		let new_authority_data = leak(vec![]);

		let config_info = AccountInfo::new(
			config_key,
			false,
			true,
			minimum_balance,
			leak(data),
			&ID,
			false,
			Epoch::default(),
		);
		let authority_info = AccountInfo::new(
			authority_key,
			true,
			true,
			authority_lamports,
			authority_data,
			&system_program::ID,
			false,
			Epoch::default(),
		);
		let new_authority_info = AccountInfo::new(
			new_authority_key,
			true,
			false,
			new_authority_lamports,
			new_authority_data,
			&system_program::ID,
			false,
			Epoch::default(),
		);

		[config_info, authority_info, new_authority_info]
	}
}
