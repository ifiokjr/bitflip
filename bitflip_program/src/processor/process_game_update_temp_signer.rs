use steel::*;

use crate::seeds_game;
use crate::BitflipError;
use crate::BitflipInstruction;
use crate::GameState;
use crate::ID;

/// Update the temporary signer of the game. This can be done anytime by the
/// backend of the game.
pub fn process_game_update_temp_signer(accounts: &[AccountInfo]) -> ProgramResult {
	let [funded_signer_info, temp_signer_info, game_info] = accounts else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	let game = game_info.as_account_mut::<GameState>(&ID)?;
	let game_seeds_with_bump = seeds_game!(game.game_index, game.bump);

	funded_signer_info.assert_signer()?;
	temp_signer_info.assert_signer()?;
	game_info
		.assert_type::<GameState>(&ID)?
		.assert_writable()?
		.assert_seeds_with_bump(game_seeds_with_bump, &ID)?;

	game.assert_err(
		|state| {
			state.funded_signer.eq(funded_signer_info.key)
				&& state.temp_signer.ne(temp_signer_info.key)
		},
		BitflipError::GameSignerInvalid,
	)?;

	game.temp_signer = *temp_signer_info.key;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GameUpdateTempSigner {}

instruction!(BitflipInstruction, GameUpdateTempSigner);

#[cfg(test)]
mod tests {
	use assert2::check;

	use super::*;
	use crate::get_pda_game;
	use crate::leak;

	#[test_log::test]
	fn should_pass_validation() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_game_update_temp_signer(&accounts);
		check!(result.is_ok());

		Ok(())
	}

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_game_update_temp_signer(&accounts[..2]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn funded_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let funded_signer_info = &mut accounts[0];
		funded_signer_info.is_signer = false;

		let result = process_game_update_temp_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn funded_signer_should_match_game_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let funded_signer_info = &mut accounts[0];
		funded_signer_info.key = leak(Pubkey::new_unique());

		let result = process_game_update_temp_signer(&accounts);
		check!(result.unwrap_err() == BitflipError::GameSignerInvalid.into());

		Ok(())
	}

	#[test_log::test]
	fn temp_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let temp_signer_info = &mut accounts[1];
		temp_signer_info.is_signer = false;

		let result = process_game_update_temp_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn temp_signer_should_new() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let current_temp_signer = {
			let game_info = &accounts[2];
			let game = game_info.as_account::<GameState>(&ID)?;
			game.temp_signer
		};
		let temp_signer_info = &mut accounts[1];
		temp_signer_info.key = leak(current_temp_signer);

		let result = process_game_update_temp_signer(&accounts);
		check!(result.unwrap_err() == BitflipError::GameSignerInvalid.into());

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[2];
		game_info.key = leak(Pubkey::new_unique());

		let result = process_game_update_temp_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn game_should_have_valid_owner() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[2];
		game_info.owner = leak(Pubkey::new_unique());

		let result = process_game_update_temp_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[2];
		game_info.is_writable = false;

		let result = process_game_update_temp_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos<'info>() -> [AccountInfo<'info>; 3] {
		let funded_signer_key = leak(Pubkey::new_unique());
		let funded_signer_lamports = leak(500_000);
		let funded_signer_data = leak(vec![]);
		let temp_signer_key = leak(Pubkey::new_unique());
		let temp_signer_lamports = leak(0);
		let temp_signer_data = leak(vec![]);
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
					*funded_signer_key,
					game_index,
					game_bump,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};

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

		[funded_signer_info, temp_signer_info, game_info]
	}
}
