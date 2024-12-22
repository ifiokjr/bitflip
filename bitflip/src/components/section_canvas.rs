use bitflip_program::FlipBit;
use js_sys::Reflect;
use leptos::html::Canvas;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::MouseEvent;

use crate::get_section_state;
use crate::use_game_index;
use crate::use_section_index;

#[component]
pub fn SectionCanvas() -> impl IntoView {
	let game_index = use_game_index();
	let section_index = use_section_index();
	let canvas_ref = NodeRef::<Canvas>::new();
	let (show_image, set_show_image) = signal(true);
	let section_resource = Resource::new(
		move || (game_index(), section_index()),
		move |_| {
			set_show_image(true);
			let (game_index, section_index) = (game_index(), section_index());
			get_section_state(game_index, section_index)
		},
	);
	let section_state = RwSignal::new(None);

	let effect = move || {
		let context = get_2d_context(canvas_ref);
		let Some(Ok(section)) = section_resource.get() else {
			log::error!("Section not found");
			return;
		};

		section_state.set(Some(section));
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

	let canvas_click_handler = move |e: MouseEvent| {
		let Some(_) = section_state.get() else {
			return;
		};
		let section_index = section_index.get();
		let rect = e
			.target()
			.and_then(|t| t.dyn_into::<web_sys::Element>().ok())
			.map(|el| el.get_bounding_client_rect())
			.expect("Failed to get canvas bounds");
		let dx = f64::from(e.client_x()) - rect.left();
		let dy = f64::from(e.client_y()) - rect.top();
		let canvas_x = (dx * 1024.0 / rect.width()) as u16;
		let canvas_y = (dy * 1024.0 / rect.height()) as u16;
		let x = canvas_x / 16;
		let y = canvas_y / 16;
		let (array_index, offset) = get_index_offset(x, y);
		log::info!(
			"Clicked grid cell: ({}, {}) - index: {} - offset: {}",
			x,
			y,
			array_index,
			offset
		);

		section_state.update(move |state| {
			let Some(state) = state else {
				log::error!("Section state not found");
				return;
			};

			let is_checked = state.is_checked(array_index, offset);
			log::info!("is_checked: {}", is_checked);

			let context = get_2d_context(canvas_ref);
			let result = state.set_bit(&FlipBit {
				section_index,
				array_index,
				offset,
				value: u8::from(!is_checked),
			});
			log::info!("result: {:?}", result);

			if is_checked {
				let _ = state.flip_off(1);
				context.clear_rect(f64::from(x * 16), f64::from(y * 16), 16f64, 16f64);
			} else {
				let _ = state.flip_on(1);
				context.set_fill_style_str("black");
				context.fill_rect(f64::from(x * 16), f64::from(y * 16), 16f64, 16f64);
			}
		});
	};

	view! {
		<Suspense fallback=|| view! { <div class="nes-text">"loading..."</div> }>
			<div class="w-full h-full">
				<Show when=show_image>
					<SectionImage game_index=game_index section_index=section_index />
				</Show>
				<canvas
					node_ref=canvas_ref
					width=1024
					height=1024
					class="w-full h-full"
					class:hidden=show_image
					on:click=canvas_click_handler
				/>
			</div>
		</Suspense>
	}
}

#[component]
pub fn SectionImage(game_index: Signal<u8>, section_index: Signal<u8>) -> impl IntoView {
	let url = move || {
		format!(
			"/game/{game_index}/section-image/{section_index}",
			game_index = game_index(),
			section_index = section_index()
		)
	};

	view! { <img src=url class="w-full" /> }
}

fn get_2d_context(canvas_ref: NodeRef<Canvas>) -> CanvasRenderingContext2d {
	let Some(canvas) = canvas_ref.get() else {
		log::error!("Canvas not found");
		panic!();
	};

	Reflect::set(&window(), &JsValue::from_str("_abc"), &canvas).unwrap();

	let Ok(Some(context_object)) = canvas.get_context("2d") else {
		log::error!("Canvas context not found");
		panic!();
	};

	let Ok(context) = context_object.dyn_into::<CanvasRenderingContext2d>() else {
		log::error!("could not `dyn_into` context");
		panic!();
	};

	context
}

/// Will panic if the index or offset is greater than `u8::MAX`.
pub fn get_index_offset(x: u16, y: u16) -> (u8, u8) {
	let index = ((((x / 4) / 4) + ((y / 4) / 4) * 4) * 16) + ((x / 4) % 4) + 4 * ((y / 4) % 4);
	let offset = x % 4 + (y % 4) * 4;
	assert!(offset <= 16);

	(u8::try_from(index).unwrap(), u8::try_from(offset).unwrap())
}
