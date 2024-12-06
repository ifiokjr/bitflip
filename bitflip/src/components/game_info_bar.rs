use leptos::prelude::*;

use crate::get_active_players;
use crate::use_parsed_param;
use crate::AppParam;

#[component]
pub fn GameInfoBar() -> impl IntoView {
	let section_index = use_parsed_param::<u8>(AppParam::Section);
	let active_players_resource = Resource::new(|| {}, |()| get_active_players());

	view! {
		<div class="flex gap-4 justify-end items-center">
			<div class="nes-text">"section: "{section_index}</div>
			<div class="nes-text">"players: "{active_players_resource}</div>
			<button class="nes-btn is-primary is-small">"ⓘ"</button>
		</div>
	}
}
