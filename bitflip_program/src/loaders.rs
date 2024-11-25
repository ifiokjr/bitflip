use spl_token_2022::extension::PodStateWithExtensions;
use spl_token_2022::pod::PodAccount;
use spl_token_2022::pod::PodMint;
use steel::*;

pub fn as_mint_state<'info>(
	info: &AccountInfo<'info>,
) -> Result<PodStateWithExtensions<'info, PodMint>, ProgramError> {
	unsafe {
		let data = info.try_borrow_data()?;
		let state = PodStateWithExtensions::<PodMint>::unpack(std::slice::from_raw_parts(
			data.as_ptr(),
			data.len(),
		))?;

		Ok(state)
	}
}

pub fn as_mint(info: &AccountInfo<'_>) -> Result<PodMint, ProgramError> {
	let state = as_mint_state(info)?;

	Ok(*state.base)
}

pub fn as_token_account_state<'info>(
	info: &AccountInfo<'info>,
) -> Result<PodStateWithExtensions<'info, PodAccount>, ProgramError> {
	unsafe {
		let data = info.try_borrow_data()?;
		let state = PodStateWithExtensions::<PodAccount>::unpack(std::slice::from_raw_parts(
			data.as_ptr(),
			data.len(),
		))?;

		Ok(state)
	}
}

pub fn as_token_account(info: &AccountInfo<'_>) -> Result<PodAccount, ProgramError> {
	let state = as_token_account_state(info)?;

	Ok(*state.base)
}
