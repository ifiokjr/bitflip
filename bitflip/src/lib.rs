pub use environment::*;
pub use functions::*;
pub use hooks::*;
pub use shared::*;
pub use stores::*;

pub mod app;

pub mod components;
#[cfg(feature = "ssr")]
pub mod db;
#[cfg(feature = "ssr")]
pub mod encryption;
mod environment;
#[cfg(feature = "ssr")]
pub mod image_generator;
mod shared;
#[cfg(feature = "ssr")]
pub mod state;

mod functions;
pub mod hooks;
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
