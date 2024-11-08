use anchor_lang::prelude::*;

use crate::BITFLIP_SECTION_LENGTH;

#[error_code]
pub enum BitflipError {
	#[msg(
		"The incorrect section is being initialized. The sections must be initialized sequentially"
	)]
	IncorrectSectionInitialized,
	#[msg("The space is already fully initialized")]
	BitsIncreaseSpaceInvalid,
	#[msg("No update recorded")]
	BitsUnchanged,
	#[msg("The data section index must be a multiple of 16")]
	Invalid256BitsDataSectionIndex,
	#[msg("The provided account was invalid")]
	InvalidAccount,
	#[msg("There are invalid bit changes. This should not be possible")]
	InvalidBitChanges,
	#[msg("The bit offset is invalid and must be less than 16")]
	InvalidBitOffset,
	#[msg("Invalid section requested")]
	InvalidSectionRequested,
	#[msg("Invalid section index requested")]
	InvalidSectionIndex,
	#[msg("Invalid bits data section array length")]
	InvalidBitsDataSectionLength,
	#[msg("Data sections initialized out of order")]
	InvalidBitsDataSectionOrder,
	#[msg("The bits array is an invalid length")]
	InvalidBitsLength,
	#[msg("An invalid number of flipped bits was provided")]
	InvalidFlippedBits,
	#[msg("The current `GameState` is not running")]
	GameNotRunning,
	#[msg("The token is not yet initialized")]
	TokenNotInitialized,
	#[msg("The admin used was incorrect")]
	UnauthorizedAdmin,
	#[msg("The previous section does not meet the minimum flips threshold")]
	MinimumFlipThreshold,
	#[msg("The same account cannot own consecutive sections")]
	SectionOwnerDuplicate,
	#[msg("The access signer has not been updated")]
	AccessSignerNotUpdated,
}

/// Returns the offset.
pub fn validate_section_index(index: u16) -> Result<()> {
	require!(
		usize::from(index) < BITFLIP_SECTION_LENGTH,
		BitflipError::InvalidSectionIndex
	);

	Ok(())
}

pub fn validate_256bit_data_section_index(index: u16) -> Result<()> {
	require!(
		index % 16 == 0,
		BitflipError::Invalid256BitsDataSectionIndex
	);

	Ok(())
}

pub fn validate_bit_offset(offset: u16) -> Result<()> {
	require!(
		u32::from(offset) < u16::BITS,
		BitflipError::InvalidBitOffset
	);

	Ok(())
}

pub fn validate_bit_array_length(array: &[u16], expected: usize) -> Result<()> {
	require!(array.len() == expected, BitflipError::InvalidBitsLength);

	Ok(())
}
