use steel::*;

use crate::BITFLIP_SECTION_LENGTH;

#[derive(Default, Debug, Error, Clone, Copy, PartialEq, Eq, IntoPrimitive)]
#[repr(u32)]
pub enum BitflipError {
	#[default]
	#[error("An unknown error occured")]
	Unknown,
	#[error(
		"The incorrect section is being initialized. The sections must be initialized sequentially"
	)]
	IncorrectSectionInitialized,
	#[error("The space is already fully initialized")]
	BitsIncreaseSpaceInvalid,
	#[error("No update recorded")]
	BitsUnchanged,
	#[error("The data section index must be a multiple of 16")]
	Invalid256BitsDataSectionIndex,
	#[error("The provided account was invalid")]
	InvalidAccount,
	#[error("There are invalid bit changes. This should not be possible")]
	InvalidBitChanges,
	#[error("The bit offset is invalid and must be less than 16")]
	InvalidBitOffset,
	#[error("Invalid section requested")]
	InvalidSectionRequested,
	#[error("Invalid section index requested")]
	InvalidSectionIndex,
	#[error("Invalid bits data section array length")]
	InvalidBitsDataSectionLength,
	#[error("Data sections initialized out of order")]
	InvalidBitsDataSectionOrder,
	#[error("The bits array is an invalid length")]
	InvalidBitsLength,
	#[error("An invalid number of flipped bits was provided")]
	InvalidFlippedBits,
	#[error("The current `GameState` is not running")]
	GameNotRunning,
	#[error("The token is not yet initialized")]
	TokenNotInitialized,
	#[error("The admin used was incorrect")]
	UnauthorizedAdmin,
	#[error("The previous section does not meet the minimum flips threshold")]
	MinimumFlipThreshold,
	#[error("The same account cannot own consecutive sections")]
	SectionOwnerDuplicate,
	#[error("The access signer has not been updated")]
	AccessSignerNotUpdated,
	#[error("The authority used was a duplicate")]
	DuplicateAuthority,
	#[error("The authority is not authorized to update the authority")]
	Unauthorized,
}

error!(BitflipError);

/// Returns the offset.
pub fn validate_section_index(index: u16) -> ProgramResult {
	if usize::from(index) >= BITFLIP_SECTION_LENGTH {
		return Err(BitflipError::InvalidSectionIndex.into());
	}

	Ok(())
}

pub fn validate_256bit_data_section_index(index: u16) -> ProgramResult {
	if index % 16 != 0 {
		return Err(BitflipError::Invalid256BitsDataSectionIndex.into());
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
