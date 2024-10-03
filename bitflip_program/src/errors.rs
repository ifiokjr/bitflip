use anchor_lang::prelude::*;

use crate::BITS_DATA_SECTION_LENGTH;
use crate::BITS_DATA_SECTIONS;

#[error_code]
pub enum BitflipError {
	#[msg("The space is already fully initialized")]
	BitsIncreaseSpaceInvalid,
	#[msg("No update recorded")]
	BitsUnchanged,
	#[msg("The provided account was invalid")]
	InvalidAccount,
	#[msg("The bits array is an invalid length")]
	InvalidBitsArrayLength,
	#[msg("Invalid bit data section requested")]
	InvalidBitsDataSection,
	#[msg("Invalid bits data section index requested")]
	InvalidBitsDataSectionIndex,
	#[msg("The data section index must be a multiple of 16")]
	Invalid256BitsDataSectionIndex,
	#[msg("The bit offset is invalid and must be less than 16")]
	InvalidBitOffset,
	#[msg("The admin used was incorrect")]
	UnauthorizedAdmin,
	#[msg("All bit data sections have already been initialized")]
	AllSectionsInitialized,
	#[msg("The token is not yet initialized")]
	TokenNotInitialized,
}

pub fn validate_data_section(section: u8) -> Result<()> {
	require!(
		usize::from(section) < BITS_DATA_SECTIONS,
		BitflipError::InvalidBitsDataSection
	);

	Ok(())
}

/// Returns the offset.
pub fn validate_data_section_index(index: u16) -> Result<()> {
	require!(
		usize::from(index) < BITS_DATA_SECTION_LENGTH,
		BitflipError::InvalidBitsDataSectionIndex
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
	require!(
		array.len() == expected,
		BitflipError::InvalidBitsArrayLength
	);

	Ok(())
}
