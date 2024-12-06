pub use functions::*;
pub use shared::*;
pub use stores::*;

pub mod app;
#[cfg(feature = "ssr")]
pub mod image_generator;
pub mod shared;

mod functions;
mod stores;
mod wallet;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
	// use app::App;

	_ = console_log::init_with_level(log::Level::Debug);
	console_error_panic_hook::set_once();
	leptos::mount::hydrate_islands();
}
