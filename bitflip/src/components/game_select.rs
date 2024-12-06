use leptos::prelude::*;

#[component]
pub fn GameSelect() -> impl IntoView {
	view! {
		<div class="relative">
			<button class="flex gap-2 items-center nes-btn">
				"choose game" <span>"▼"</span>
			</button>
		</div>
	}
}
