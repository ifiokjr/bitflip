use steel::*;

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct UnlockSectionEvent {
	pub owner: Pubkey,
	pub section_index: u8,
}

event!(UnlockSectionEvent);
