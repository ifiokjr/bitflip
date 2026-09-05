use leptos::prelude::*;
use leptos_meta::Html;
use leptos_use::use_color_mode_with_options;
use leptos_use::ColorMode;
use leptos_use::UseColorModeOptions;
use leptos_use::UseColorModeReturn;

#[component]
pub fn GameHeader() -> impl IntoView {
	let UseColorModeReturn { mode, set_mode, .. } = use_color_mode_with_options(
		UseColorModeOptions::default()
			.cookie_enabled(true)
			.attribute("data-theme"),
	);

	view! {
		<Html attr:data-theme=move || mode.get().to_string() />
		<header class="flex justify-between items-center mb-8 nes-container is-rounded">
			<h1 class="nes-text">"bitflip"</h1>
			<button class="nes-btn">"create account"</button>
			<button on:click=move |_| {
				set_mode
					.set(
						if mode.get() == ColorMode::Light {
							ColorMode::Dark
						} else {
							ColorMode::Light
						},
					);
			}>{move || mode.get().to_string()}</button>
		</header>
	}
}
