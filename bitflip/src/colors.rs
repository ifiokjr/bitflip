#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct PaletteColor {
	pub name: &'static str,
	pub css: &'static str,
	pub rgba: [u8; 4],
}

/// Stable palette shared by the browser and backend renderer.
pub const COLOR_PALETTE: [PaletteColor; 8] = [
	PaletteColor {
		name: "Ink",
		css: "#0b0f14",
		rgba: [11, 15, 20, 255],
	},
	PaletteColor {
		name: "Coral",
		css: "#e63946",
		rgba: [230, 57, 70, 255],
	},
	PaletteColor {
		name: "Amber",
		css: "#ff9f1c",
		rgba: [255, 159, 28, 255],
	},
	PaletteColor {
		name: "Sun",
		css: "#f4d35e",
		rgba: [244, 211, 94, 255],
	},
	PaletteColor {
		name: "Mint",
		css: "#2ec4b6",
		rgba: [46, 196, 182, 255],
	},
	PaletteColor {
		name: "Sky",
		css: "#3a86ff",
		rgba: [58, 134, 255, 255],
	},
	PaletteColor {
		name: "Violet",
		css: "#8338ec",
		rgba: [131, 56, 236, 255],
	},
	PaletteColor {
		name: "Magenta",
		css: "#ff4d9d",
		rgba: [255, 77, 157, 255],
	},
];

const _: () = assert!(COLOR_PALETTE.len() == bitflip_program::BITFLIP_COLOR_COUNT as usize);
