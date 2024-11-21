use steel::*;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
enum BitflipEvent {
	UnlockSection = 0,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct UnlockSection {
	pub owner: Pubkey,
	pub section_index: u8,
}

#[macro_export]
macro_rules! event_cpi {
	($discriminator_name:ident, $struct_name:ident) => {
		::steel::event!($struct_name);

		impl ::steel::Discriminator for $struct_name {
			fn discriminator() -> u8 {
				$discriminator_name::$struct_name as u8
			}
		}
	};
}

event_cpi!(BitflipEvent, UnlockSection);
