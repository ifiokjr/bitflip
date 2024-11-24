use spl_token_2022::extension::PodStateWithExtensions;
use spl_token_2022::pod::PodAccount;
use spl_token_2022::pod::PodMint;
use steel::*;

pub fn as_mint(info: &AccountInfo<'_>) -> Result<PodMint, ProgramError> {
	let data = info.try_borrow_data()?;
	let state = PodStateWithExtensions::<PodMint>::unpack(&data)?;

	Ok(*state.base)
}

pub fn as_token_account(info: &AccountInfo<'_>) -> Result<PodAccount, ProgramError> {
	let data = info.try_borrow_data()?;
	let state = PodStateWithExtensions::<PodAccount>::unpack(&data)?;

	Ok(*state.base)
}
