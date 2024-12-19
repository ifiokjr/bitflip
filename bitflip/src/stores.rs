use bitflip_program::BITFLIP_SECTION_LENGTH;
use reactive_stores::Store;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug, Store, Serialize, Deserialize)]
pub struct SectionStateStore {
	pub game_index: u8,
	pub section_index: u8,
	pub on: u32,
	pub off: u32,
	pub flips: u32,
	#[serde(with = "serde_big_array::BigArray")]
	pub data: [u16; BITFLIP_SECTION_LENGTH],
}

impl SectionStateStore {
	pub fn is_checked(&self, index: u8, offset: u8) -> bool {
		self.data[index as usize] & (1 << offset) != 0
	}
}
