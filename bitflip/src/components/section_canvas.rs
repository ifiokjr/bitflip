use js_sys::Reflect;
use leptos::html::Canvas;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;
use web_sys::MouseEvent;

use crate::get_active_game_index;
use crate::get_default_section_index;
use crate::get_section_state;

#[island]
pub fn SectionCanvas(game_index: Option<u8>, section_index: Option<u8>) -> impl IntoView {
	let canvas_ref = NodeRef::<Canvas>::new();
	let (show_image, set_show_image) = signal(true);
	let game_index_resource = Resource::new(move || {}, move |()| get_active_game_index());
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

	let canvas_click_handler = move |e: MouseEvent| {
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
		let (index, offset) = get_index_offset(x, y);
		log::info!(
			"Clicked grid cell: ({}, {}) - index: {} - offset: {}",
			x,
			y,
			index,
			offset
		);
	};

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
				on:click=canvas_click_handler
			/>
		</div>
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

pub fn get_index_offset(x: u16, y: u16) -> (u16, u16) {
	let index = ((((x / 4) / 4) + ((y / 4) / 4) * 4) * 16) + ((x / 4) % 4) + 4 * ((y / 4) % 4);
	let offset = x % 4 + (y % 4) * 4;

	(index, offset)
}
