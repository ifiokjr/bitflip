use steel::*;
use sysvar::rent::Rent;

use crate::seeds_config;
use crate::seeds_game;
use crate::BitflipError;
use crate::BitflipInstruction;
use crate::ConfigState;
use crate::GameState;
use crate::ID;
use crate::TRANSACTION_FEE;

pub fn process_game_reset_signers(accounts: &[AccountInfo]) -> ProgramResult {
	let [authority_info, funded_signer_info, temp_signer_info, previous_funded_signer_info, config_info, game_info] =
		accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let game = game_info.as_account_mut::<GameState>(&ID)?;
	let config_seeds_with_bump = seeds_config!(config.bump);
	let game_seeds_with_bump = seeds_game!(game.game_index, game.bump);

	authority_info.is_signer()?.is_writable()?;
	temp_signer_info.is_signer()?;
	funded_signer_info.is_signer()?;
	config_info
		.is_type::<ConfigState>(&ID)?
		.has_seeds_with_bump(config_seeds_with_bump, &ID)?;
	game_info
		.is_type::<GameState>(&ID)?
		.is_writable()?
		.has_seeds_with_bump(game_seeds_with_bump, &ID)?;
	config.assert_err(
		// Check that the authority is the same as the one in the config
		|state| state.authority.eq(authority_info.key),
		BitflipError::Unauthorized,
	)?;
	game.assert_err(
		// Check that both the signers are new
		|state| {
			state.funded_signer.ne(funded_signer_info.key)
				&& state.temp_signer.ne(temp_signer_info.key)
		},
		BitflipError::GameSignerInvalid,
	)?;

	// transfer lamports from the previous funded signer to the authority
	if game.funded_signer.eq(previous_funded_signer_info.key) {
		// check that the previous funded signer is a signer and writable
		previous_funded_signer_info.is_signer()?.is_writable()?;
		// transfer lamports from the previous funded signer to the authority
		authority_info.collect(
			previous_funded_signer_info.lamports(),
			previous_funded_signer_info,
		)?;
	}

	// update the game state
	game.temp_signer = *temp_signer_info.key;
	game.funded_signer = *funded_signer_info.key;

	// transfer lamports from the authority to the funded signer
	let rent_sysvar = Rent::get()?;
	let funded_signer_lamports = rent_sysvar
		.minimum_balance(0)
		.checked_add(TRANSACTION_FEE.checked_mul(1000).unwrap())
		.unwrap();
	funded_signer_info.collect(funded_signer_lamports, authority_info)?;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
pub struct GameResetSigners {}

instruction!(BitflipInstruction, GameResetSigners);

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;

	use assert2::check;

	use super::*;
	use crate::get_pda_config;
	use crate::get_pda_game;
	use crate::get_pda_mint;
	use crate::get_pda_treasury;
	use crate::leak;
	use crate::TokenMember;

	#[test_log::test]
	fn should_pass_validation() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::UnsupportedSysvar);

		Ok(())
	}

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_game_reset_signers(&accounts[..5]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn funded_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let funded_signer_info = &mut accounts[1];
		funded_signer_info.is_signer = false;

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn funded_signer_should_not_match_game_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let previous_funded_signer = {
			let game_info = &accounts[5];
			let game = game_info.as_account::<GameState>(&ID)?;
			game.funded_signer
		};
		let funded_signer_info = &mut accounts[1];
		funded_signer_info.key = leak(previous_funded_signer);

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == BitflipError::GameSignerInvalid.into());

		Ok(())
	}

	#[test_log::test]
	fn temp_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let temp_signer_info = &mut accounts[2];
		temp_signer_info.is_signer = false;

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn temp_signer_should_be_new() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let current_temp_signer = {
			let game_info = &accounts[5];
			let game = game_info.as_account::<GameState>(&ID)?;
			game.temp_signer
		};
		let temp_signer_info = &mut accounts[2];
		temp_signer_info.key = leak(current_temp_signer);

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == BitflipError::GameSignerInvalid.into());

		Ok(())
	}

	#[test_log::test]
	fn previous_funded_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let previous_funded_signer_info = &mut accounts[3];
		previous_funded_signer_info.is_signer = false;

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn previous_funded_signer_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let previous_funded_signer_info = &mut accounts[3];
		previous_funded_signer_info.is_writable = false;

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[4];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn config_should_have_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[4];
		config_info.data = Rc::new(RefCell::new(leak(vec![1u8; 8])));

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[5];
		game_info.key = leak(Pubkey::new_unique());

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn game_should_have_valid_owner() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[5];
		game_info.owner = leak(Pubkey::new_unique());

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[5];
		game_info.is_writable = false;

		let result = process_game_reset_signers(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos<'info>() -> [AccountInfo<'info>; 6] {
		let authority_key = leak(Pubkey::new_unique());
		let authority_lamports = leak(0);
		let authority_data = leak(vec![]);
		let funded_signer_key = leak(Pubkey::new_unique());
		let funded_signer_lamports = leak(500_000);
		let funded_signer_data = leak(vec![]);
		let temp_signer_key = leak(Pubkey::new_unique());
		let temp_signer_lamports = leak(0);
		let temp_signer_data = leak(vec![]);
		let previous_funded_signer_key = leak(Pubkey::new_unique());
		let previous_funded_signer_lamports = leak(0);
		let previous_funded_signer_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = {
			let config_bump = get_pda_config().1;
			let mut data = vec![0u8; 8];
			let treasury_bump = get_pda_treasury().1;
			let mint_bump = get_pda_mint(TokenMember::Bit).1;
			let mint_kibibit_bump = get_pda_mint(TokenMember::Kibibit).1;
			let mint_mebibit_bump = get_pda_mint(TokenMember::Mebibit).1;
			let mint_gibibit_bump = get_pda_mint(TokenMember::Gibibit).1;
			data[0] = ConfigState::discriminator();
			data.append(
				&mut ConfigState::new(
					*authority_key,
					config_bump,
					treasury_bump,
					mint_bump,
					mint_kibibit_bump,
					mint_mebibit_bump,
					mint_gibibit_bump,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};
		let game_index = 0;
		let game_key = leak(get_pda_game(game_index).0);
		let game_lamports = leak(0);
		let game_data = {
			let game_bump = get_pda_game(game_index).1;
			let mut data = vec![0u8; 8];
			data[0] = GameState::discriminator();
			data.append(
				&mut GameState::new(
					Pubkey::new_unique(),
					*previous_funded_signer_key,
					game_index,
					game_bump,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};

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
		let previous_funded_signer_info = AccountInfo::new(
			previous_funded_signer_key,
			true,
			true,
			previous_funded_signer_lamports,
			previous_funded_signer_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let config_info = AccountInfo::new(
			config_key,
			false,
			false,
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

		[
			authority_info,
			funded_signer_info,
			temp_signer_info,
			previous_funded_signer_info,
			config_info,
			game_info,
		]
	}
}
