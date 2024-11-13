use derive_more::Add;
use derive_more::AddAssign;
use steel::ProgramError;

use crate::BitflipError;

#[derive(Default, Debug, PartialEq, Eq, Add, AddAssign)]
pub struct BitChanges {
	pub on: u32,
	pub off: u32,
}

impl BitChanges {
	pub fn total(&self) -> Result<u32, ProgramError> {
		self.on
			.checked_add(self.off)
			.ok_or(ProgramError::ArithmeticOverflow)
	}
}
pub fn get_bit_changes(previous: u16, next: u16) -> Result<BitChanges, ProgramError> {
	if next == previous {
		return Err(BitflipError::BitsUnchanged.into());
	}

	let mut changes = BitChanges::default();
	let previous_string = format!("{previous:016b}");
	let next_string = format!("{next:016b}");

	// println!("{previous}: {previous_string}");
	// println!("{next}: {next_string}");

	for n in 0..16 {
		let Some((previous, next)) = previous_string.get(n..=n).zip(next_string.get(n..=n)) else {
			continue;
		};

		if previous == next {
			continue;
		}

		if next == "1" {
			changes.on += 1;
		}

		if next == "0" {
			changes.off += 1;
		}
	}

	Ok(changes)
}

pub fn get_token_amount(tokens: u64, decimals: u8) -> Result<u64, ProgramError> {
	tokens
		.checked_mul(10u64.pow(decimals.into()))
		.ok_or(ProgramError::ArithmeticOverflow)
}

#[cfg(test)]
mod tests {
	use assert2::check;
	use rstest::rstest;
	use steel::ProgramResult;

	use super::*;

	#[rstest]
	#[case(0, 1, (1, 0))]
	#[case(u16::MAX, 1, (0, 15))]
	#[case(1, u16::MAX, (15, 0))]
	#[case(32767, 32768, (1, 15))]
	fn can_check_bit_changes(
		#[case] previous: u16,
		#[case] next: u16,
		#[case] (on, off): (u32, u32),
	) -> ProgramResult {
		let result = get_bit_changes(previous, next)?;
		check!(result == BitChanges { on, off });
		check!(result.total()? == on + off);

		Ok(())
	}

	#[rstest]
	#[case(1, 1)]
	#[case(3124, 3124)]
	#[case(u16::MAX, u16::MAX)]
	fn errors_with_identical_bits(#[case] previous: u16, #[case] next: u16) {
		check!(get_bit_changes(previous, next).is_err());
	}
}
