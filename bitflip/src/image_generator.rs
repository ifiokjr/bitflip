use axum::extract::Path;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use rand::Rng;
use rand::SeedableRng;
use tiny_skia::Color;
use tiny_skia::Paint;
use tiny_skia::Pixmap;
use tiny_skia::Transform;

/// Generate an image for a section of the Bitflip game state.
///
/// The image is a 4096x4096 PNG with each bit of the game state represented by
/// a 16x16 square. This uses `tiny-skia` to generate a png when
/// called. The image is 4096x4096 with each bit represented by a 16x16
/// width square.
///
/// Some more information, there are 16 sections in the `1024x1024` bit
/// structure. Each section is a `[u16; 4096]` array with each bit representing
/// a flipped bit on the canvas. When the bit is `1` then the image should show
/// a black square in that position and when a bit is `0` the image should show
/// a white square.
pub fn generate_section_image(section_data: &[u16; 4096]) -> Vec<u8> {
	let mut pixmap = Pixmap::new(4096, 4096).unwrap();

	let mut paint = Paint::default();
	paint.set_color(Color::BLACK);

	for (index, &value) in section_data.iter().enumerate() {
		let base_x = (index % 64) * 64;
		let base_y = (index / 64) * 64;

		for bit in 0..16 {
			if (value & (1 << bit)) != 0 {
				let bit_x = bit % 4;
				let bit_y = bit / 4;
				let x = base_x + bit_x * 16;
				let y = base_y + bit_y * 16;

				pixmap.fill_rect(
					tiny_skia::Rect::from_xywh(x as f32, y as f32, 16.0, 16.0).unwrap(),
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
pub async fn section_image_handler(Path(section_index): Path<u8>) -> impl IntoResponse {
	// Use a deterministic seed based on the section index
	let mut rng = rand::rngs::StdRng::seed_from_u64(section_index.into());
	let section_data: [u16; 4096] = std::array::from_fn(|_| rng.r#gen());

	let png_data = generate_section_image(&section_data);

	(StatusCode::OK, [("Content-Type", "image/png")], png_data)
}
