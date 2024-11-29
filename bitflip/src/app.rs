use bitflip_program::SectionState;
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
use leptos_router::components::A;
use leptos_router::hooks::use_params;
use leptos_router::params::Params;
use leptos_router::path;
use solana_sdk::pubkey::Pubkey;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::window;
use web_sys::CanvasRenderingContext2d;

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
			<body class="doodle">
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
		<Stylesheet id="css-reset" href="/reset.css" />
		<Stylesheet id="css-leptos" href="/pkg/bitflip.css" />

		// sets the document title
		<Title text="bitflip" />

		// content for this welcome page
		<Router>
			<main>
				<FlatRoutes fallback=|| "Page not found.".into_view()>
					<Route path=path!("") view=HomePage />
					<Route
						path=path!("/game/:game_index/section/:section_index")
						view=SectionPage
					/>
				</FlatRoutes>
			</main>
		</Router>
	}
}

/// Renders the home page of your application.
#[component]
fn HomePage() -> impl IntoView {
	let render_section = move |section| {
		view! {
			<div class="w-[25vw] h-[25vw] hover:outline hover:outline-blue-300">
				// <div class="w-[100vw] h-[100vw] hover:outline hover:outline-blue-300">
				<A href=format!("/game/0/section/{section}")>
					<img src=format!("/game/0/section-image/{section}") class="w-full" />
				</A>
			</div>
		}
	};

	view! {
		<h1>"bitflip"</h1>
		<div class="grid grid-4x4">{(0u8..16u8).map(render_section).collect_view()}</div>
	}
}

#[derive(Params, PartialEq, Clone, Debug)]
pub struct SectionPageParams {
	game_index: u8,
	// Params isn't implemented for usize, only Option<usize>
	section_index: u8,
}

#[component]
fn SectionPage() -> impl IntoView {
	log::info!("rendering section page");
	let params = use_params::<SectionPageParams>();
	let url = move || {
		params
			.get()
			.map(|params| {
				log::info!("{:#?}", params);
				format!(
					"/game/{}/section-image/{}",
					params.game_index, params.section_index
				)
			})
			.ok()
	};

	view! { <img src=url class="w-full" /> }
}

#[island]
pub fn BitCanvasSection(game_index: u8, section_index: u8) -> impl IntoView {
	let section = SectionState::new(Pubkey::new_unique(), game_index, section_index, 0);
	let canvas_ref = NodeRef::<Canvas>::new();
	let effect = move || {
		let Some(canvas) = canvas_ref.get() else {
			return;
		};

		let context = canvas
			.get_context("2d")
			.unwrap()
			.unwrap()
			.dyn_into::<CanvasRenderingContext2d>()
			.unwrap();
		context.set_image_smoothing_enabled(false);

		// context.put_image_data(ImageData::new_with_u8_clamped_array(wasm_bindgen::Clamped(), ), , )

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

		let data = context.get_image_data(0., 0., 4096., 4096.).unwrap();
		Reflect::set(&window().unwrap(), &JsValue::from_str("_abc"), &data).unwrap();
		// leptos::logging::log!("{data:#?}");
	};

	Effect::new(effect);

	view! {
		<div class="w-[16rem] h-[16rem]">
			<canvas node_ref=canvas_ref width=4096 height=4096 class="w-[16rem] h-[16rem]" />
		</div>
	}
}
