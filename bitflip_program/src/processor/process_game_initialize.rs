use steel::*;
use sysvar::rent::Rent;

use crate::seeds_config;
use crate::seeds_game;
use crate::BitflipError;
use crate::BitflipInstruction;
use crate::ConfigState;
use crate::GameState;
use crate::ID;
use crate::SEED_GAME;
use crate::SEED_PREFIX;
use crate::TRANSACTION_FEE;

pub fn process_game_initialize(accounts: &[AccountInfo]) -> ProgramResult {
	let [authority_info, temp_signer_info, funded_signer_info, config_info, game_info, system_program_info] =
		accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let config_seeds_with_bump = seeds_config!(config.bump);
	let game_seeds = seeds_game!(config.game_index);
	let game_bump = game_info.assert_canonical_bump(game_seeds, &ID)?;

	authority_info
		.assert_signer()?
		.assert_writable()?
		.assert_owner(&system_program::ID)?;
	temp_signer_info.assert_empty()?.assert_signer()?;
	funded_signer_info
		.assert_empty()?
		.assert_signer()?
		.assert_writable()?;
	config_info
		.assert_type::<ConfigState>(&ID)?
		.assert_seeds_with_bump(config_seeds_with_bump, &ID)?;
	game_info.assert_empty()?.assert_writable()?;
	system_program_info.assert_program(&system_program::ID)?;

	if authority_info.key.ne(&config.authority) {
		return Err(BitflipError::Unauthorized.into());
	}

	// create the onchain account
	create_account_with_bump::<GameState>(
		game_info,
		system_program_info,
		authority_info,
		&ID,
		&[SEED_PREFIX, SEED_GAME, &config.game_index.to_le_bytes()],
		game_bump,
	)?;

	let game = game_info.as_account_mut::<GameState>(&ID)?;
	*game = GameState::new(
		*temp_signer_info.key,
		*funded_signer_info.key,
		config.game_index,
		game_bump,
	);

	// store lamports in the refresh signer
	let rent_sysvar = Rent::get()?;
	let funded_signer_lamports = rent_sysvar
		.minimum_balance(0)
		.checked_add(TRANSACTION_FEE.checked_mul(1000).unwrap())
		.unwrap();
	funded_signer_info.collect(funded_signer_lamports, authority_info)?;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GameInitialize {}

instruction!(BitflipInstruction, GameInitialize);

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;

	use assert2::check;
	use solana_sdk::native_loader;

	use super::*;
	use crate::get_pda_config;
	use crate::get_pda_game;
	use crate::leak;

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_game_initialize(&accounts[..5]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.is_signer = false;

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.is_writable = false;

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_from_config() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.key = leak(Pubkey::new_unique());

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::Custom(BitflipError::Unauthorized.into()));

		Ok(())
	}

	#[test_log::test]
	fn temp_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let temp_signer = &mut accounts[1];
		temp_signer.is_signer = false;

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn funded_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let funded_signer_info = &mut accounts[2];
		funded_signer_info.is_signer = false;

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn funded_signer_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let funded_signer_info = &mut accounts[2];
		funded_signer_info.is_writable = false;

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[3];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn config_should_have_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[3];
		config_info.data = Rc::new(RefCell::new(leak(vec![1u8; 8])));

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn config_should_have_valid_owner() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[3];
		config_info.owner = leak(Pubkey::new_unique());

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[4];
		game_info.key = leak(Pubkey::new_unique());

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_empty() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[4];
		game_info.data = Rc::new(RefCell::new(leak(vec![1, 0, 0, 0, 0, 0, 0, 0])));

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::AccountAlreadyInitialized);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[4];
		game_info.is_writable = false;

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos<'info>() -> [AccountInfo<'info>; 6] {
		let authority = Pubkey::new_unique();
		let authority_key = leak(authority);
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let temp_signer_key = leak(Pubkey::new_unique());
		let temp_signer_lamports = leak(0);
		let temp_signer_data = leak(vec![]);
		let funded_signer_key = leak(Pubkey::new_unique());
		let funded_signer_lamports = leak(0);
		let funded_signer_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = {
			let config_bump = get_pda_config().1;
			let mut data = vec![0u8; 8];
			data[0] = ConfigState::discriminator();
			data.append(
				&mut ConfigState::new(
					authority,
					config_bump,
					u8::MAX,
					u8::MAX,
					u8::MAX,
					u8::MAX,
					u8::MAX,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};
		let game_key = leak(get_pda_game(0).0);
		let game_lamports = leak(0);
		let game_data = leak(vec![]);
		let system_program_lamports = leak(1_000_000_000);
		let system_program_data = leak(vec![]);

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
		let temp_signer_info = AccountInfo::new(
			temp_signer_key,
			true,
			false,
			temp_signer_lamports,
			temp_signer_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let funded_signer_info = AccountInfo::new(
			funded_signer_key,
			true,
			true,
			funded_signer_lamports,
			funded_signer_data,
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
		let game_info = AccountInfo::new(
			game_key,
			false,
			true,
			game_lamports,
			game_data,
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

		[
			authority_info,
			temp_signer_info,
			funded_signer_info,
			config_info,
			game_info,
			system_program_info,
		]
	}
}
