use std::collections::HashSet;

use bitflip_program::BITFLIP_SECTION_LENGTH;
use bitflip_program::SectionState;
use bitflip_program::get_pda_section;
use js_sys::Reflect;
use leptos::html::Canvas;
use leptos::prelude::*;
use leptos_meta::Html;
use leptos_meta::MetaTags;
use leptos_meta::Stylesheet;
use leptos_meta::Title;
use leptos_meta::provide_meta_context;
use leptos_router::components::A;
use leptos_router::components::FlatRoutes;
use leptos_router::components::Route;
use leptos_router::components::Router;
use leptos_router::path;
use rand::Rng;
use reactive_stores::Store;
use solana_sdk::pubkey::Pubkey;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::window;

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
				<A href=format!("/section/{section}")>
					<img src=format!("/section-image/{section}") />
				</A>
			</div>
		}
	};

	view! {
		<h1>"bitflip"</h1>
		<div class="grid grid-4x4">{(0u8..16u8).map(render_section).collect_view()}</div>
	}
}

/// A 4x4 bit array
#[component]
fn Bit16(index: u16) -> impl IntoView {
	let context = DataSectionContext::expect();

	let render_item = move |offset| {
		let is_checked = Signal::derive(move || context.is_checked(index, offset));
		let id = Signal::derive(move || context.bit_dom_id(index, offset));
		let on_toggle = Callback::new(move |()| {
			if context.is_checked(index, offset) {
				context.update(index, SetBitsVariant::Off(offset)).ok();
			} else {
				context.update(index, SetBitsVariant::On(offset)).ok();
			};
		});

		view! { <Bit checked=is_checked id=id on_toggle=on_toggle /> }
	};

	view! { <div class="grid grid-4x4">{(0u16..16u16).map(render_item).collect_view()}</div> }
}

/// A 16x16 bit array
#[component]
fn Bit256(index: u16) -> impl IntoView {
	let render_children = move |vector_index| {
		let index = index + (vector_index);

		view! { <Bit16 index=index /> }
	};

	view! { <div class="grid grid-4x4">{(0u16..16u16).map(render_children).collect_view()}</div> }
}

/// A 64x64 bit array
#[component]
fn Bit4096(index: u16) -> impl IntoView {
	let render_children = move |vector_index| {
		let index = index + (vector_index * 16);

		view! { <Bit256 index=index /> }
	};

	view! { <div class="grid grid-4x4">{(0u16..16u16).map(render_children).collect_view()}</div> }
}

#[component]
pub fn BitCanvasSection(game_index: u8, section_index: u8) -> impl IntoView {
	let data_section = DataSectionContext::new(game_index, section_index);
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

		for ii in 0u16..16u16 {
			let index = ii * 256;
			let bit_4096_dx = 64 * (ii % 4);
			let bit_4096_dy = 64 * (ii / 4);

			for ii in 0u16..16u16 {
				let index = index + ii * 16;
				let bit_256_dx = 16 * (ii % 4);
				let bit_256_dy = 16 * (ii / 4);

				for ii in 0u16..16u16 {
					let index = index + ii;
					let bit_16_dx = 4 * (ii % 4);
					let bit_16_dy = 4 * (ii / 4);

					for offset in 0u16..16u16 {
						if !data_section.is_checked(index, offset) {
							continue;
						}

						let x = offset % 4 + bit_16_dx + bit_256_dx + bit_4096_dx;
						let y = offset / 4 + bit_16_dy + bit_256_dy + bit_4096_dy;

						let color = JsValue::from_str("black");
						context.set_fill_style(&color);
						context.fill_rect(f64::from(x * 16), f64::from(y * 16), 16f64, 16f64);
						// context.set_fill_style(&color);
						// context.set_stroke_style(&color);
						// context.begin_path();
						// context
						// 	.round_rect_with_f64(
						// 		f64::from(x * 16),
						// 		f64::from(y * 16),
						// 		16f64,
						// 		16f64,
						// 		4f64,
						// 	)
						// 	.unwrap();
						// context.fill();
						// context.stroke();
					}
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

/// A 256x256 bit array
#[component]
fn BitSection(game_index: u8, section_index: u8) -> impl IntoView {
	DataSectionContext::new(game_index, section_index);

	let render_children = move |vector_index| {
		view! { <Bit4096 index=vector_index * 256 /> }
	};

	view! { <div class="grid grid-4x4">{(0u16..1u16).map(render_children).collect_view()}</div> }
}

#[component]
fn Bit(checked: Signal<bool>, id: Signal<String>, on_toggle: Callback<()>) -> impl IntoView {
	let on_input = move |_| {
		on_toggle.run(());
	};

	view! {
		<label class="w-2 h-2 text-[8px] swap hover:outline hover:-outline-offset-1 hover:outline-blue-300 focus:offset focus:-outline-offset-1 focus:outline-blue-300">
			<input
				on:input=on_input
				type="checkbox"
				checked=checked
				prop:checked=checked
				class="hidden"
				id=id
			/>
			<CheckboxIcon class="fill-current swap-on" />
			<span class="bg-transparent swap-off" />
		</label>
	}
}

#[component]
fn CheckboxIcon(
	#[prop(optional, into)] size: Option<String>,
	#[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
	let width = size.unwrap_or("1em".into());
	let height = width.clone();
	let class = move || class.clone().unwrap_or_default();

	view! {
		<svg
			class=class
			width=width
			height=height
			viewBox="0 0 144.7578125 144.7578125"
			fill="none"
			stroke-linecap="round"
			stroke="currentColor"
			stroke-width="2"
			role="img"
			aria-hidden="true"
		>
			<rect x="0" y="0" width="144.7578125" height="144.7578125" fill="none" />
			<g
				stroke-linecap="round"
				transform="translate(10 10) rotate(0 62.37890625 62.37890625)"
			>
				<path
					d="M31.19 0 C49.18 3.77, 74.82 -3.1, 93.57 0 C116.86 -2.01, 129.69 10.39, 124.76 31.19 C122.74 44.17, 129.57 55.95, 124.76 93.57 C127.3 115.89, 109.75 124.98, 93.57 124.76 C75.54 126.01, 63.16 123.68, 31.19 124.76 C8.96 120.3, -2.58 115.51, 0 93.57 C-3.1 75.84, 6.16 66.27, 0 31.19 C5.13 7.07, 13.69 4.77, 31.19 0"
					stroke="none"
					stroke-width="0"
					fill="currentColor"
				/>
				<path
					d="M31.19 0 C53.05 -0.89, 76.95 3.93, 93.57 0 M31.19 0 C47.73 1.22, 63.26 1.15, 93.57 0 M93.57 0 C110.38 1.66, 124.42 9.91, 124.76 31.19 M93.57 0 C112.24 4.09, 124.1 8.56, 124.76 31.19 M124.76 31.19 C125.65 51.23, 125.9 77.04, 124.76 93.57 M124.76 31.19 C127.74 52.89, 126.01 74.98, 124.76 93.57 M124.76 93.57 C125.69 114.31, 114.48 128.1, 93.57 124.76 M124.76 93.57 C128.39 116.41, 115.39 126.71, 93.57 124.76 M93.57 124.76 C73.67 123.1, 51.97 122.16, 31.19 124.76 M93.57 124.76 C70.64 127.14, 51.22 127.24, 31.19 124.76 M31.19 124.76 C13.43 123.3, 2 115.24, 0 93.57 M31.19 124.76 C14.35 126.71, 0.76 116.42, 0 93.57 M0 93.57 C0.87 78.21, -1.32 58.52, 0 31.19 M0 93.57 C2.66 68.21, 3.18 44, 0 31.19 M0 31.19 C0.59 9.13, 12.49 1.58, 31.19 0 M0 31.19 C1 8.67, 5.94 -3.37, 31.19 0"
					stroke="currentColor"
					stroke-width="4"
					fill="none"
				/>
			</g>
		</svg>
	}
}

#[derive(Clone, Copy, Store)]
pub struct DataSectionContext {
	section: ReadSignal<u8>,
	state: RwSignal<SectionState>,
	updates: RwSignal<HashSet<FlipBitsProps>>,
}

impl DataSectionContext {
	pub fn new(game_index: u8, section_index: u8) -> Self {
		let mut rng = rand::thread_rng();
		let section_bump = get_pda_section(game_index, section_index).1;
		let mut inner_state = SectionState::new(Pubkey::new_unique(), section_bump, section_index);

		for ii in 0..BITFLIP_SECTION_LENGTH {
			inner_state.data[ii] = rng.r#gen();
		}

		let (section, _) = signal(section_index);
		let state = RwSignal::new(inner_state);
		let updates = RwSignal::new(HashSet::new());

		let context = Self {
			section,
			state,
			updates,
		};

		provide_context(context);

		context
	}

	pub fn expect() -> Self {
		expect_context()
	}

	pub fn update(&self, index: u16, variant: SetBitsVariant) -> AnchorResult {
		let section = self.section.get_untracked();
		let mut state = self.state.write();
		let updates = self.updates.read();
		let new_update = FlipBitsProps {
			section_index: section,
			array_index: index,
			variant,
		};
		let mut updates_to_remove = vec![];
		let mut should_add_update = true;

		for update in updates.iter() {
			if new_update.contains(update) {
				updates_to_remove.push(update);
			} else if update.contains(&new_update) {
				should_add_update = false;
			}
		}

		if !should_add_update {
			return Ok(());
		}

		let mut updates = self.updates.write();

		for update in updates_to_remove {
			updates.remove(update);
		}

		state.set_bits(&new_update)?;
		updates.insert(new_update);

		Ok(())
	}

	pub fn is_checked(&self, index: u16, offset: u16) -> bool {
		let value = self.state.read().data[index as usize];
		let bit = index << offset;

		(value & bit) > 0
	}

	pub fn bit_is_updated(&self, index: u16, offset: u16) -> bool {
		let updates = self.updates.read();
		let section = self.section.get_untracked();
		let on_update = FlipBitsProps {
			section_index: section,
			array_index: index,
			variant: SetBitsVariant::On(offset),
		};
		let off_update = FlipBitsProps {
			section_index: section,
			array_index: index,
			variant: SetBitsVariant::Off(offset),
		};

		updates.contains(&on_update) || updates.contains(&off_update)
	}

	pub fn bit_16_is_updated(&self, index: u16) -> bool {
		self.updates.read().iter().any(|update| {
			update.array_index == index && matches!(update.variant, SetBitsVariant::Bit16(_))
		})
	}

	pub fn bit_256_is_updated(&self, index: u16) -> bool {
		self.updates.read().iter().any(|update| {
			update.array_index == index && matches!(update.variant, SetBitsVariant::Bits256(_))
		})
	}

	pub fn bit_dom_id(&self, index: u16, offset: u16) -> String {
		let section = self.section.get_untracked();
		format!("bit:{section}:{index}:{offset}")
	}
}
