use leptos::prelude::*;

use crate::get_active_player_count;
use crate::use_parsed_param;
use crate::AppParam;

#[component]
pub fn GameInfoBar() -> impl IntoView {
	let section_index = use_parsed_param::<u8>(AppParam::Section);
	let active_player_count_resource = Resource::new(|| {}, |()| get_active_player_count());

	view! {
		<div class="flex gap-4 justify-end items-center">
			<div class="nes-text">"section: "{section_index}</div>
			<div class="nes-text">"players: "{active_player_count_resource}</div>
			<button class="nes-btn is-primary is-small">"ⓘ"</button>
		</div>
	}
}
