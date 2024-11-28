use steel::*;

use crate::seeds_config;
use crate::BitflipError;
use crate::BitflipInstruction;
use crate::ConfigState;
use crate::ID;

pub fn process_config_update_authority(accounts: &[AccountInfo]) -> ProgramResult {
	let [config_info, authority_info, new_authority_info] = accounts else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	let config = config_info.as_account_mut::<ConfigState>(&ID)?;
	let config_seeds_with_bump = seeds_config!(config.bump);

	// validate accounts
	config_info
		.is_writable()?
		.is_type::<ConfigState>(&ID)?
		.has_seeds_with_bump(config_seeds_with_bump, &ID)?;
	authority_info
		.is_signer()?
		.is_writable()?
		.has_owner(&system_program::ID)?;
	new_authority_info
		.is_signer()?
		.has_owner(&system_program::ID)?;

	config.assert_err(
		|config| config.authority.eq(authority_info.key),
		BitflipError::Unauthorized,
	)?;
	config.assert_err(
		|config| new_authority_info.key.ne(&config.authority),
		BitflipError::DuplicateAuthority,
	)?;

	config.authority = *new_authority_info.key;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct ConfigUpdateAuthority {}

instruction!(BitflipInstruction, ConfigUpdateAuthority);

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;

	use assert2::check;
	use solana_sdk::clock::Epoch;
	use solana_sdk::sysvar::rent::Rent;

	use super::*;
	use crate::get_pda_config;
	use crate::get_pda_mint;
	use crate::get_pda_treasury;
	use crate::leak;
	use crate::TokenMember;

	#[test_log::test]
	fn should_pass_validation() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		process_config_update_authority(&accounts)?;

		let config_info = &accounts[0];
		let config_state = config_info.as_account::<ConfigState>(&ID)?;
		check!(config_state.authority == *accounts[2].key);

		Ok(())
	}

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_config_update_authority(&accounts[0..2]);

		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[0];
		config_info.data = Rc::new(RefCell::new(leak(vec![1u8; 8])));

		let result = process_config_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_writeable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[0];
		config_info.is_writable = false;

		let result = process_config_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[0];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_config_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_signer = false;

		let result = process_config_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_writable = false;

		let result = process_config_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_from_config() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.key = leak(Pubkey::new_unique());

		let result = process_config_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::Custom(BitflipError::Unauthorized.into()));

		Ok(())
	}
	#[test_log::test]
	fn new_authority_should_change() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_key = *accounts[1].key;
		let new_authority_info = &mut accounts[2];
		new_authority_info.key = leak(authority_key);

		let result = process_config_update_authority(&accounts);
		check!(
			result.unwrap_err() == ProgramError::Custom(BitflipError::DuplicateAuthority.into())
		);

		Ok(())
	}

	#[test_log::test]
	fn new_authority_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let new_authority_info = &mut accounts[2];
		new_authority_info.is_signer = false;

		let result = process_config_update_authority(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 3] {
		let (config_key, config_bump) = leak(get_pda_config());
		let treasury_bump = get_pda_treasury().1;
		let mint_bit_bump = get_pda_mint(TokenMember::Bit).1;
		let mint_kibibit_bump = get_pda_mint(TokenMember::Kibibit).1;
		let mint_mebibit_bump = get_pda_mint(TokenMember::Mebibit).1;
		let mint_gibibit_bump = get_pda_mint(TokenMember::Gibibit).1;
		let authority_lamports = leak(1_000_000_000);
		let new_authority_lamports = leak(1_000_000_000);
		let authority_key = leak(Pubkey::new_unique());
		let new_authority_key = leak(Pubkey::new_unique());
		let rent_sysvar = Rent::default();
		let minimum_balance = leak(rent_sysvar.minimum_balance(ConfigState::space()));
		let mut data = vec![0u8; 8];
		data[0] = ConfigState::discriminator();
		data.append(
			&mut ConfigState::new(
				*authority_key,
				*config_bump,
				treasury_bump,
				mint_bit_bump,
				mint_kibibit_bump,
				mint_mebibit_bump,
				mint_gibibit_bump,
			)
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
