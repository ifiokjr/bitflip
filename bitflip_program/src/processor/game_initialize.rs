use steel::*;

use super::BitflipInstruction;
use crate::BitflipError;
use crate::ConfigState;
use crate::GameState;
use crate::ID;
use crate::SEED_GAME;
use crate::SEED_PREFIX;
use crate::create_pda_config;
use crate::get_pda_game;

pub fn process_game_initialize(accounts: &[AccountInfo]) -> ProgramResult {
	let [
		authority_info,
		access_signer_info,
		refresh_signer_info,
		config_info,
		game_info,
		system_program_info,
	] = accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	authority_info
		.is_signer()?
		.is_writable()?
		.has_owner(&system_program::ID)?;
	access_signer_info.is_empty()?.is_signer()?;
	refresh_signer_info.is_empty()?.is_signer()?;
	config_info.is_type::<ConfigState>(&ID)?;
	game_info.is_empty()?.is_writable()?;
	system_program_info.is_program(&system_program::ID)?;

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let config_key = create_pda_config(config.bump)?;
	let (game_key, game_bump) = get_pda_game(config.game_index);

	if authority_info.key.ne(&config.authority) {
		return Err(BitflipError::Unauthorized.into());
	}

	if config_info.key.ne(&config_key) || game_info.key.ne(&game_key) {
		return Err(ProgramError::InvalidSeeds);
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
		*access_signer_info.key,
		*refresh_signer_info.key,
		config.game_index,
		game_bump,
	);

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
	fn access_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let access_signer_info = &mut accounts[1];
		access_signer_info.is_signer = false;

		let result = process_game_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn refresh_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let refresh_signer_info = &mut accounts[2];
		refresh_signer_info.is_signer = false;

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

		Ok(())
	}

	fn create_account_infos<'info>() -> [AccountInfo<'info>; 6] {
		let authority = Pubkey::new_unique();
		let authority_key = leak(authority);
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let access_signer_key = leak(Pubkey::new_unique());
		let access_signer_lamports = leak(0);
		let access_signer_data = leak(vec![]);
		let refresh_signer_key = leak(Pubkey::new_unique());
		let refresh_signer_lamports = leak(0);
		let refresh_signer_data = leak(vec![]);
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
		let access_signer_info = AccountInfo::new(
			access_signer_key,
			true,
			false,
			access_signer_lamports,
			access_signer_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let refresh_signer_info = AccountInfo::new(
			refresh_signer_key,
			true,
			false,
			refresh_signer_lamports,
			refresh_signer_data,
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
			access_signer_info,
			refresh_signer_info,
			config_info,
			game_info,
			system_program_info,
		]
	}
}
