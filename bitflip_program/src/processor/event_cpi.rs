use steel::*;

use super::BitflipInstruction;
use crate::ID;
use crate::get_pda_event;

/// Process the event CPI instruction
pub fn process_event_cpi(accounts: &[AccountInfo], _: &[u8]) -> ProgramResult {
	let [event_authority_info, bitflip_program_info] = accounts else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	event_authority_info.is_signer()?;
	bitflip_program_info.is_program(&ID)?;

	let event_authority_key = get_pda_event().0;

	if event_authority_info.key != &event_authority_key {
		return Err(ProgramError::InvalidSeeds);
	}

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct EventCpi {
	pub data: [u8; 256],
}

impl Default for EventCpi {
	fn default() -> Self {
		Self { data: [0u8; 256] }
	}
}

impl EventCpi {
	pub fn new(data: &[u8]) -> Self {
		let mut data_array = [0u8; 256];
		data_array[..data.len()].copy_from_slice(data);
		Self { data: data_array }
	}

	pub fn to_event<T: Pod + Discriminator>(&self) -> Result<&T, ProgramError> {
		let data = self.data.as_ref();
		if T::discriminator().ne(&data[0]) {
			return Err(solana_program::program_error::ProgramError::InvalidAccountData);
		}
		bytemuck::try_from_bytes::<T>(&data[8..]).or(Err(
			solana_program::program_error::ProgramError::InvalidAccountData,
		))
	}

	pub fn to_event_mut<T: Pod + Discriminator>(&mut self) -> Result<&mut T, ProgramError> {
		let data = self.data.as_mut();
		if T::discriminator().ne(&data[0]) {
			return Err(solana_program::program_error::ProgramError::InvalidAccountData);
		}

		bytemuck::try_from_bytes_mut::<T>(&mut data[8..]).or(Err(
			solana_program::program_error::ProgramError::InvalidAccountData,
		))
	}

	pub fn from_event<T: Pod + Discriminator>(event: &T) -> Self {
		let mut data = [0u8; 256];
		data[0] = T::discriminator();
		data[8..].copy_from_slice(bytemuck::bytes_of(event));
		Self { data }
	}
}

instruction!(BitflipInstruction, EventCpi);
