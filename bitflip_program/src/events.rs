use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
enum BitflipEvent {
	UnlockSection = 0,
	BitFlipped = 1,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct UnlockSection {
	pub owner: Pubkey,
	pub section_index: u8,
}

/// A successful bit change that off-chain game modes can project.
///
/// `color` is deliberately emitted without being stored in [`SectionState`].
/// Indexers should clear the projected pixel when `value` is `0`, regardless
/// of the supplied color.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BitFlipped {
	pub player: Pubkey,
	pub game_index: u8,
	pub section_index: u8,
	pub array_index: u8,
	pub offset: u8,
	pub value: u8,
	pub color: u8,
}

#[macro_export]
macro_rules! event_cpi {
	($discriminator_name:ident, $struct_name:ident) => {
		impl ::steel::Discriminator for $struct_name {
			fn discriminator() -> u8 {
				$discriminator_name::$struct_name as u8
			}
		}

		impl $struct_name {
			pub fn to_event_bytes(&self) -> ::std::vec::Vec<u8> {
				let mut data = ::std::vec::Vec::with_capacity(1 + ::std::mem::size_of::<Self>());
				data.push(<Self as ::steel::Discriminator>::discriminator());
				data.extend_from_slice(::bytemuck::bytes_of(self));
				data
			}

			pub fn try_from_event_bytes(
				data: &[u8],
			) -> ::std::result::Result<Self, ::steel::ProgramError> {
				let expected_len = 1 + ::std::mem::size_of::<Self>();
				if data.len() != expected_len
					|| data.first().copied()
						!= Some(<Self as ::steel::Discriminator>::discriminator())
				{
					return Err(::steel::ProgramError::InvalidInstructionData);
				}

				Ok(::bytemuck::pod_read_unaligned(&data[1..]))
			}
		}

		impl ::steel::Loggable for $struct_name {
			fn log(&self) {
				let data = self.to_event_bytes();
				::steel::solana_program::log::sol_log_data(&[&data]);
			}

			fn log_return(&self) {
				let data = self.to_event_bytes();
				::steel::solana_program::program::set_return_data(&data);
			}
		}
	};
}

event_cpi!(BitflipEvent, UnlockSection);
event_cpi!(BitflipEvent, BitFlipped);

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn bit_flipped_event_round_trips() -> Result<(), ProgramError> {
		let event = BitFlipped {
			player: Pubkey::new_unique(),
			game_index: 2,
			section_index: 3,
			array_index: 4,
			offset: 5,
			value: 1,
			color: 7,
		};

		assert_eq!(BitFlipped::discriminator(), 1);
		assert_eq!(
			BitFlipped::try_from_event_bytes(&event.to_event_bytes())?,
			event
		);

		Ok(())
	}
}
