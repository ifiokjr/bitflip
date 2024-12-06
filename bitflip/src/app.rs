use std::str::FromStr;

use js_sys::Reflect;
use leptos::html::Canvas;
use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_meta::Html;
use leptos_meta::MetaTags;
use leptos_meta::Stylesheet;
use leptos_meta::Title;
use leptos_router::components::FlatRoutes;
use leptos_router::components::Route;
use leptos_router::components::Router;
use leptos_router::hooks::use_url;
use leptos_router::path;
use strum::IntoStaticStr;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use crate::get_default_game_index;
use crate::get_default_section_index;
use crate::get_section_state;

pub fn shell(options: LeptosOptions) -> impl IntoView {
	view! {
		<!DOCTYPE html>
		<html lang="en" data-theme="light">
			<head>
				<meta charset="utf-8" />
				<meta name="viewport" content="width=device-width, initial-scale=1" />
				<AutoReload options=options.clone() />
				<HydrationScripts options islands=true />
				<MetaTags />
			</head>
			<body class="w-full h-full">
				<App />
			</body>
		</html>
	}
}

#[component]
pub fn App() -> impl IntoView {
	// Provides context that manages stylesheets, titles, meta tags, etc.
	provide_meta_context();

	view! {
		<Html attr:data-theme="light" />
		<Stylesheet id="css-font" href="https://fonts.googleapis.com/css?family=Press+Start+2P" />
		<Stylesheet id="css-leptos" href="/pkg/bitflip.css" />

		// sets the document title
		<Title text="bitflip" />

		// content for this welcome page
		<Router>
			<main>
				<FlatRoutes fallback=|| "Page not found.".into_view()>
					<Route path=path!("") view=HomePage />
				</FlatRoutes>
			</main>
		</Router>
	}
}

#[component]
pub fn Header() -> impl IntoView {
	view! {
		<header class="flex justify-between items-center mb-8 nes-container is-rounded">
			<h1 class="nes-text">"bitflip"</h1>
			<button class="nes-btn">"create account"</button>
		</header>
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum AppParam {
	#[strum(serialize = "section")]
	Section,
	#[strum(serialize = "game")]
	Game,
}

pub fn use_parsed_param<T: FromStr + Send + Sync + 'static>(key: AppParam) -> Signal<Option<T>> {
	let parsed_param = move || {
		let url = use_url().get();
		let params = url.search_params();
		log::info!("params: {:?}", params);

		params.get(key.into()).unwrap_or_default().parse::<T>().ok()
	};

	Signal::derive(parsed_param)
}

/// Renders the home page of your application.
#[component]
pub fn HomePage() -> impl IntoView {
	let game_index = use_parsed_param::<u8>(AppParam::Game);
	let section_index = use_parsed_param::<u8>(AppParam::Section);
	log::info!("game: {:?}", game_index());
	log::info!("section: {:?}", section_index());

	view! {
		<div class="flex flex-col items-center">
			<section class="container px-4">
				<Header />
			</section>
			<section class="container p-4">
				// Game info bar
				<div class="flex gap-4 justify-end items-center mb-4">
					<div class="nes-text">"section: "{section_index}</div>
					<div class="nes-text">"players: 97"</div>
					<button class="nes-btn is-primary is-small">"ⓘ"</button>
				</div>

				// Game selection dropdown
				<div class="relative pb-4">
					<button class="flex gap-2 items-center nes-btn">
						"choose game" <span>"▼"</span>
					</button>
				</div>

				// Main game grid container with navigation arrows
				<div class="flex-grow p-0 nes-container is-rounded aspect-square">
					{move || {
						view! {
							<SectionCanvas game_index=game_index() section_index=section_index() />
						}
					}}
				</div>

				// Navigation buttons
				<div class="flex gap-4 justify-between py-4">
					<PreviousSectionButton />
					<NextSectionButton />
				</div>
			</section>
		</div>
	}
}

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
enum Direction {
	Increment,
	Decrement,
}

fn use_transformed_section(direction: Direction, jump: u8) -> (Signal<String>, Signal<bool>) {
	let url = use_url();
	let section_index_resource = Resource::new(move || {}, move |()| get_default_section_index());
	let section_index = Signal::derive(move || {
		if let Some(section_index) = use_parsed_param::<u8>(AppParam::Section).get() {
			section_index
		} else if let Some(Ok(section_index)) = section_index_resource.get() {
			section_index
		} else {
			0
		}
	});

	let is_disabled = Signal::derive(move || {
		let section_index = section_index();
		match direction {
			Direction::Increment => section_index == u8::MAX,
			Direction::Decrement => section_index == 0,
		}
	});
	let section_path = Signal::derive(move || {
		if is_disabled() {
			return String::new();
		}

		let mut url = url.get();
		let section_index = section_index();
		let transformed_section_index = match direction {
			Direction::Increment => section_index.saturating_add(jump),
			Direction::Decrement => section_index.saturating_sub(jump),
		};
		log::info!("section_index: {section_index}");
		log::info!("url: {:?}", url);
		{
			let search_params = url.search_params_mut();
			search_params.replace("section", transformed_section_index.to_string());
		}
		let path = url.path();
		let hash = url.hash();
		let qs = url.search_params().to_query_string();

		format!("{path}{qs}{hash}")
	});

	(section_path, is_disabled)
}

#[component]
fn NextSectionButton() -> impl IntoView {
	let (section_path, is_disabled) = use_transformed_section(Direction::Increment, 1);

	view! {
		<a class="nes-btn" href=section_path class:is-disabled=is_disabled>
			"→"
		</a>
	}
}

#[component]
fn PreviousSectionButton() -> impl IntoView {
	let (section_path, is_disabled) = use_transformed_section(Direction::Decrement, 1);

	view! {
		<a class="nes-btn" class:is-disabled=is_disabled href=section_path>
			"←"
		</a>
	}
}

#[component]
fn SectionImage(game_index: Signal<u8>, section_index: Signal<u8>) -> impl IntoView {
	let url = move || {
		format!(
			"/game/{game_index}/section-image/{section_index}",
			game_index = game_index(),
			section_index = section_index()
		)
	};

	view! { <img src=url class="w-full" /> }
}

pub fn is_mounted() -> ReadSignal<bool> {
	let (is_mounted, set_is_mounted) = signal(false);

	Effect::new(move || {
		set_is_mounted.set(true);
	});

	is_mounted
}

#[island]
pub fn SectionCanvas(game_index: Option<u8>, section_index: Option<u8>) -> impl IntoView {
	let canvas_ref = NodeRef::<Canvas>::new();
	let (show_image, set_show_image) = signal(true);
	let game_index_resource = Resource::new(move || {}, move |()| get_default_game_index());
	let section_index_resource = Resource::new(move || {}, move |()| get_default_section_index());
	let game_index_signal = Signal::derive(move || {
		if let Some(game_index) = game_index {
			game_index
		} else if let Some(Ok(game_index)) = game_index_resource.get() {
			game_index
		} else {
			0
		}
	});
	let section_index_signal = Signal::derive(move || {
		if let Some(section_index) = section_index {
			section_index
		} else if let Some(Ok(section_index)) = section_index_resource.get() {
			section_index
		} else {
			0
		}
	});
	let section_resource = Resource::new(
		move || (game_index_signal(), section_index_signal()),
		move |_| {
			set_show_image(true);
			let (game_index, section_index) = (game_index_signal(), section_index_signal());
			get_section_state(game_index, section_index)
		},
	);

	let effect = move || {
		let Some(canvas) = canvas_ref.get() else {
			log::error!("Canvas not found");
			return;
		};

		let Some(Ok(section)) = section_resource.get() else {
			log::error!("Section not found");
			return;
		};

		Reflect::set(&window(), &JsValue::from_str("_abc"), &canvas).unwrap();

		let Ok(Some(context_object)) = canvas.get_context("2d") else {
			log::error!("Canvas context not found");
			return;
		};

		let Ok(context) = context_object.dyn_into::<CanvasRenderingContext2d>() else {
			log::error!("could not `dyn_into` context");
			return;
		};

		context.set_image_smoothing_enabled(false);

		for x in 0..16u32 {
			for y in 0..16u32 {
				let index = 16 * (x / 4 + (y / 4) * 4) + (x % 4) + (4 * (y % 4));

				for offset in 0..16u32 {
					if !section.is_checked(index as u8, offset as u8) {
						continue;
					}

					let x = (4 * x) + offset % 4;
					let y = (4 * y) + offset / 4;

					context.set_fill_style_str("black");
					context.fill_rect(f64::from(x * 16), f64::from(y * 16), 16f64, 16f64);
				}
			}
		}

		set_show_image(false);
		let _data = context.get_image_data(0., 0., 1024., 1024.).unwrap();
	};

	Effect::new(effect);

	view! {
		<div class="w-full h-full">
			<Show when=show_image>
				<SectionImage game_index=game_index_signal section_index=section_index_signal />
			</Show>
			<canvas
				node_ref=canvas_ref
				width=1024
				height=1024
				class="w-full h-full"
				class:hidden=show_image
			/>
		</div>
	}
}
