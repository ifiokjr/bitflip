use steel::*;

use super::BitflipInstruction;
use crate::create_pda_game;
use crate::GameState;
use crate::ID;

pub fn process_game_refresh_signer(accounts: &[AccountInfo]) -> ProgramResult {
	let [access_signer_info, refresh_signer_info, game_info] = accounts else {
		return Err(ProgramError::InvalidArgument);
	};

	access_signer_info.is_signer()?;
	refresh_signer_info.is_signer()?.is_writable()?;
	game_info.is_type::<GameState>(&ID)?.is_writable()?;

	let game_state = game_info.as_account_mut::<GameState>(&ID)?;
	let game_key = create_pda_game(game_state.game_index, game_state.bump)?;

	if game_info.key.ne(&game_key) {
		return Err(ProgramError::InvalidSeeds);
	}

	game_state.access_signer = *access_signer_info.key;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct GameRefreshSigner {}

instruction!(BitflipInstruction, GameRefreshSigner);

#[cfg(test)]
mod tests {
	use assert2::check;

	use super::*;
	use crate::get_pda_game;
	use crate::leak;
	use crate::process_game_initialize;

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_game_initialize(&accounts[..2]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn access_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let access_signer_info = &mut accounts[0];
		access_signer_info.is_signer = false;

		let result = process_game_refresh_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn refresh_signer_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let refresh_signer_info = &mut accounts[1];
		refresh_signer_info.is_signer = false;

		let result = process_game_refresh_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn refresh_signer_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let refresh_signer_info = &mut accounts[1];
		refresh_signer_info.is_writable = false;

		let result = process_game_refresh_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[2];
		game_info.key = leak(Pubkey::new_unique());

		let result = process_game_refresh_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn game_should_have_valid_owner() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[2];
		game_info.owner = leak(Pubkey::new_unique());

		let result = process_game_refresh_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let game_info = &mut accounts[2];
		game_info.is_writable = false;

		let result = process_game_refresh_signer(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos<'info>() -> [AccountInfo<'info>; 3] {
		let access_signer_key = leak(Pubkey::new_unique());
		let access_signer_lamports = leak(0);
		let access_signer_data = leak(vec![]);
		let refresh_signer_key = leak(Pubkey::new_unique());
		let refresh_signer_lamports = leak(500_000);
		let refresh_signer_data = leak(vec![]);
		let game_index = 0;
		let game_key = leak(get_pda_game(game_index).0);
		let game_lamports = leak(0);
		let game_data = {
			let game_bump = get_pda_game(game_index).1;
			let mut data = vec![0u8; 8];
			data[0] = GameState::discriminator();
			data.append(
				&mut GameState::new(
					*access_signer_key,
					*refresh_signer_key,
					game_index,
					game_bump,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};

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
			true,
			refresh_signer_lamports,
			refresh_signer_data,
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

		[access_signer_info, refresh_signer_info, game_info]
	}
}
