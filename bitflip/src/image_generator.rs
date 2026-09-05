use axum::extract::Path;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bitflip_program::SectionState;
use tiny_skia::Color;
use tiny_skia::Paint;
use tiny_skia::Pixmap;
use tiny_skia::Transform;

use crate::AppError;
use crate::color_projection::ColorPixel;
use crate::color_projection::load_section_colors;
use crate::colors::COLOR_PALETTE;
use crate::get_section_state;
use crate::state::AppState;

/// Generate an image for a section of the Bitflip game state.
///
/// The image is a 1024x1024 PNG with each bit of the game state represented by
/// a 16x16 square. This uses `tiny-skia` to generate a PNG when called.
///
/// Each section is a 64x64 grid backed by a `[u16; 256]` array. When the bit is
/// `1` the image shows a black square; when it is `0`, the pixel is transparent.
pub fn generate_section_image(section: &SectionState) -> Vec<u8> {
	render_section(section, &[]).encode_png().unwrap()
}

/// Generate the cosmetic color projection for a section.
///
/// Bits without an indexed color retain the original black rendering. Color
/// pixels never make an off-chain bit appear on when the on-chain bit is off.
pub fn generate_colored_section_image(section: &SectionState, colors: &[ColorPixel]) -> Vec<u8> {
	render_section(section, colors).encode_png().unwrap()
}

fn render_section(section: &SectionState, colors: &[ColorPixel]) -> Pixmap {
	let mut pixmap = Pixmap::new(1024, 1024).unwrap();
	let mut projected_colors = [None; bitflip_program::BITFLIP_SECTION_TOTAL_BITS as usize];
	for pixel in colors {
		if pixel.offset < 16 && usize::from(pixel.color) < COLOR_PALETTE.len() {
			let index = usize::from(pixel.array_index) * 16 + usize::from(pixel.offset);
			projected_colors[index] = Some(pixel.color);
		}
	}

	for array_index in 0..bitflip_program::BITFLIP_SECTION_LENGTH {
		for offset in 0..16u8 {
			if !section.is_checked(array_index as u8, offset) {
				continue;
			}

			let color = projected_colors[array_index * 16 + usize::from(offset)]
				.and_then(|index| COLOR_PALETTE.get(usize::from(index)))
				.map_or(Color::BLACK, |entry| {
					let [red, green, blue, alpha] = entry.rgba;
					Color::from_rgba8(red, green, blue, alpha)
				});
			let mut paint = Paint::default();
			paint.set_color(color);
			let (x, y) = pixel_coordinates(array_index as u8, offset);

			pixmap.fill_rect(
				tiny_skia::Rect::from_xywh(f32::from(x) * 16.0, f32::from(y) * 16.0, 16.0, 16.0)
					.unwrap(),
				&paint,
				Transform::identity(),
				None,
			);
		}
	}

	pixmap
}

fn pixel_coordinates(array_index: u8, offset: u8) -> (u8, u8) {
	let region = array_index / 16;
	let local = array_index % 16;
	let block_x = (region % 4) * 4 + local % 4;
	let block_y = (region / 4) * 4 + local / 4;
	let x = block_x * 4 + offset % 4;
	let y = block_y * 4 + offset / 4;

	(x, y)
}

#[allow(clippy::unused_async)]
pub async fn section_image_handler(
	State(state): State<AppState>,
	Path((game_index, section_index)): Path<(u8, u8)>,
) -> Result<impl IntoResponse, AppError> {
	let section_state = get_section_state(game_index, section_index)
		.await
		.map_err(|e| anyhow::anyhow!("Failed to get section state: {}", e))?;
	let colors = load_section_colors(state.db.as_sqlx_pool(), game_index, section_index).await?;

	let png_data = generate_colored_section_image(&section_state, &colors);

	Ok((StatusCode::OK, [("Content-Type", "image/png")], png_data))
}

#[cfg(test)]
mod tests {
	use bitflip_program::FlipBit;
	use solana_sdk::pubkey::Pubkey;

	use super::*;

	#[test]
	fn renders_projected_colors_without_overriding_on_chain_bits() -> anyhow::Result<()> {
		let mut section = SectionState::new(Pubkey::new_unique(), 0, 0, 0);
		section.set_bit(&FlipBit {
			section_index: 0,
			array_index: 0,
			offset: 0,
			value: 1,
			color: 0,
		})?;
		let player = Pubkey::new_unique();
		let pixmap = render_section(
			&section,
			&[
				ColorPixel {
					array_index: 0,
					offset: 0,
					color: 6,
					player,
				},
				ColorPixel {
					array_index: 0,
					offset: 1,
					color: 1,
					player,
				},
			],
		);

		let painted = pixmap.pixel(8, 8).unwrap();
		assert_eq!(
			[
				painted.red(),
				painted.green(),
				painted.blue(),
				painted.alpha()
			],
			COLOR_PALETTE[6].rgba
		);
		assert_eq!(pixmap.pixel(24, 8).unwrap().alpha(), 0);

		Ok(())
	}

	#[test]
	fn pixel_coordinates_cover_the_section_once() {
		let mut seen = [false; bitflip_program::BITFLIP_SECTION_TOTAL_BITS as usize];

		for array_index in 0..=u8::MAX {
			for offset in 0..16 {
				let (x, y) = pixel_coordinates(array_index, offset);
				let index = usize::from(y) * 64 + usize::from(x);
				assert!(!seen[index]);
				seen[index] = true;
			}
		}

		assert!(seen.into_iter().all(std::convert::identity));
	}
}
