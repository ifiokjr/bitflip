use leptos::prelude::*;
use leptos_router::hooks::use_url;

use crate::use_section_index;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
pub enum Direction {
	Increment,
	Decrement,
}

/// Get the transformed section path and whether the button is disabled
/// Can only be used within a `Suspense` component.
fn use_transformed_section(direction: Direction, jump: u8) -> (Signal<String>, Signal<bool>) {
	let url = use_url();
	let section_index = use_section_index(crate::RouterProp::Default);

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
pub fn NextSectionButton() -> impl IntoView {
	view! {
		<Suspense>
			{move || {
				let (section_path, is_disabled) = use_transformed_section(Direction::Increment, 1);
				view! {
					<a class="nes-btn" href=section_path class:is-disabled=is_disabled>
						"→"
					</a>
				}
			}}
		</Suspense>
	}
}

#[component]
pub fn PreviousSectionButton() -> impl IntoView {
	view! {
		<Suspense>
			{move || {
				let (section_path, is_disabled) = use_transformed_section(Direction::Decrement, 1);

				view! {
					<a class="nes-btn" class:is-disabled=is_disabled href=section_path>
						"←"
					</a>
				}
			}}
		</Suspense>
	}
}
