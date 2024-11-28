use steel::*;
use thiserror::Error;

use crate::BITFLIP_SECTION_LENGTH;

#[derive(Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum BitflipError {
	#[error("An unknown error occured")]
	Unknown = 0,
	#[error(
		"The incorrect section is being initialized. The sections must be initialized sequentially"
	)]
	IncorrectSectionInitialized = 1,
	#[error("No update recorded")]
	BitsUnchanged = 2,
	#[error("The provided account was invalid")]
	InvalidAccount = 3,
	#[error("The value set must be `0` or `1`")]
	InvalidPlayValue = 4,
	#[error("The bit offset is invalid and must be less than 16")]
	InvalidBitOffset = 5,
	#[error("Invalid section requested")]
	InvalidSectionRequested = 6,
	#[error("Invalid section index requested")]
	InvalidSectionIndex = 7,
	#[error("Invalid bits data section array length")]
	InvalidBitsDataSectionLength = 8,
	#[error("Data sections initialized out of order")]
	InvalidBitsDataSectionOrder = 9,
	#[error("The bits array is an invalid length")]
	InvalidBitsLength = 10,
	#[error("An invalid number of flipped bits was provided")]
	InvalidFlippedBits = 11,
	#[error("The token is not yet initialized")]
	TokenNotInitialized = 12,
	#[error("The admin used was incorrect")]
	UnauthorizedAdmin = 13,
	#[error("The previous section does not meet the minimum flips threshold")]
	MinimumFlipThreshold = 14,
	#[error("The same account cannot own consecutive sections")]
	SectionOwnerDuplicate = 15,
	#[error("The game index request is invalid")]
	GameIndexInvalid = 16,
	#[error("The current `GameState` is not running")]
	GameNotRunning = 17,
	#[error("The provided game signer is unchanged")]
	GameSignerUnchanged = 18,
	#[error("The provided game signer is invalid")]
	GameSignerInvalid = 19,
	#[error("The game has already started")]
	GameAlreadyStarted = 20,
	#[error("The authority used was a duplicate")]
	DuplicateAuthority = 21,
	#[error("The authority is not authorized to update the authority")]
	Unauthorized = 22,
}

error!(BitflipError);

/// Returns the offset.
pub fn validate_section_index(index: u16) -> ProgramResult {
	if usize::from(index) >= BITFLIP_SECTION_LENGTH {
		return Err(BitflipError::InvalidSectionIndex.into());
	}

	Ok(())
}

pub fn validate_bit_offset(offset: u16) -> ProgramResult {
	if u32::from(offset) >= u16::BITS {
		return Err(BitflipError::InvalidBitOffset.into());
	}

	Ok(())
}

pub fn validate_bit_array_length(array: &[u16], expected: usize) -> ProgramResult {
	if array.len() != expected {
		return Err(BitflipError::InvalidBitsLength.into());
	}

	Ok(())
}
