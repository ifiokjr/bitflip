pub mod app;
#[cfg(feature = "ssr")]
pub mod image_generator;
mod wallet;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
	use app::App;

	_ = console_log::init_with_level(log::Level::Debug);
	console_error_panic_hook::set_once();
	leptos::mount::hydrate_body(App);
}
