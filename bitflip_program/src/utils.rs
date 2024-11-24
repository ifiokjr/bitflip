use steel::ProgramError;

#[inline(always)]
pub fn get_token_amount(tokens: u64, decimals: u8) -> Result<u64, ProgramError> {
	tokens
		.checked_mul(10u64.pow(decimals as u32))
		.ok_or(ProgramError::ArithmeticOverflow)
}

#[cfg(feature = "client")]
pub fn round_up(amount: u64, significant_digits: u8) -> u64 {
	let multiplier = 10u64.pow(significant_digits.into());
	((amount + (multiplier * 120 / 100)) / multiplier) * multiplier
}

#[cfg(feature = "client")]
pub fn round_compute_units_up(compute_units: u64) -> u64 {
	round_up(compute_units, 4)
}
