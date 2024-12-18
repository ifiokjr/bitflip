use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use bitflip_program::SectionState;
use tiny_skia::Color;
use tiny_skia::Paint;
use tiny_skia::Pixmap;
use tiny_skia::Transform;

use crate::get_section_state;
use crate::AppError;

/// Generate an image for a section of the Bitflip game state.
///
/// The image is a 4096x4096 PNG with each bit of the game state represented by
/// a 16x16 square. This uses `tiny-skia` to generate a png when
/// called. The image is 4096x4096 with each bit represented by a 16x16
/// width square.
///
/// Some more information, there are 16 sections in the `1024x1024` bit
/// structure. Each section is a `[u16; 256]` array with each bit representing
/// a flipped bit on the canvas. When the bit is `1` then the image should show
/// a black square in that position and when a bit is `0` the image should show
/// a white square.
pub fn generate_section_image(section: &SectionState) -> Vec<u8> {
	let mut pixmap = Pixmap::new(1024, 1024).unwrap();
	let mut paint = Paint::default();
	paint.set_color(Color::BLACK);

	for x in 0..16u32 {
		for y in 0..16u32 {
			let index = 16 * (x / 4 + (y / 4) * 4) + (x % 4) + (4 * (y % 4));

			for offset in 0..16u32 {
				if !section.is_checked(index as u8, offset as u8) {
					continue;
				}

				let x = (4 * x) + offset % 4;
				let y = (4 * y) + offset / 4;

				pixmap.fill_rect(
					tiny_skia::Rect::from_xywh(x as f32 * 16.0, y as f32 * 16.0, 16.0, 16.0)
						.unwrap(),
					&paint,
					Transform::identity(),
					None,
				);
			}
		}
	}

	pixmap.encode_png().unwrap()
}

#[allow(clippy::unused_async)]
pub async fn section_image_handler(
	Path((game_index, section_index)): Path<(u8, u8)>,
) -> Result<impl IntoResponse, AppError> {
	// Use a deterministic seed based on the section index
	let section_state = get_section_state(game_index, section_index)
		.await
		.map_err(|e| anyhow::anyhow!("Failed to get section state: {}", e))?;

	let png_data = generate_section_image(&section_state);

	Ok((StatusCode::OK, [("Content-Type", "image/png")], png_data))
}
