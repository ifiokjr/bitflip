use std::future::IntoFuture;

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
fn use_transformed_section(direction: Direction, jump: u8) -> Resource<(String, bool)> {
	let url = use_url();
	let resource = Resource::new(
		move || url.get(),
		move |mut url| async move {
			let mut result = (String::new(), true);
			let section_index = use_section_index().into_future().await;

			let is_disabled = match direction {
				Direction::Increment => section_index == u8::MAX,
				Direction::Decrement => section_index == 0,
			};

			result.1 = is_disabled;

			if is_disabled {
				return result;
			}

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

			result.0 = format!("{path}{qs}{hash}");
			result
		},
	);

	resource
}

#[component]
pub fn NextSectionButton() -> impl IntoView {
	view! {
		<Suspense>
			{move || {
				let Some((section_path, is_disabled)) = use_transformed_section(
						Direction::Increment,
						1,
					)
					.get() else {
					log::error!("Section path or is_disabled not found");
					return view! {
						<a class="nes-btn" href="#" class:is-disabled=true>
							"→"
						</a>
					}
						.into_any();
				};

				view! {
					<a class="nes-btn" href=section_path class:is-disabled=is_disabled>
						"→"
					</a>
				}
					.into_any()
			}}
		</Suspense>
	}
}

#[component]
pub fn PreviousSectionButton() -> impl IntoView {
	view! {
		<Suspense>
			{move || {
				let Some((section_path, is_disabled)) = use_transformed_section(
						Direction::Decrement,
						1,
					)
					.get() else {
					log::error!("Section path or is_disabled not found");
					return view! {
						<a class="nes-btn" href="#" class:is-disabled=true>
							"←"
						</a>
					}
						.into_any();
				};

				view! {
					<a class="nes-btn" class:is-disabled=is_disabled href=section_path>
						"←"
					</a>
				}
					.into_any()
			}}
		</Suspense>
	}
}
