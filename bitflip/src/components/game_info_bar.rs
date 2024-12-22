use leptos::prelude::*;

use crate::get_active_player_count;
use crate::use_section_index;

#[component]
pub fn GameInfoBar() -> impl IntoView {
	let section_index = use_section_index();
	let active_player_count_resource = Resource::new(|| {}, |()| get_active_player_count());

	view! {
		<Suspense fallback=|| view! { <div class="nes-text">"loading..."</div> }>
			<div class="flex gap-4 justify-end items-center">
				<div class="nes-text">"section: "{section_index}</div>
				<div class="nes-text">
					"players: "
					{move || Suspend::new(async move { active_player_count_resource.await })}
				</div>
				<button class="nes-btn is-primary is-small">"ⓘ"</button>
			</div>
		</Suspense>
	}
}
