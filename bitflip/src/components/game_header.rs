use leptos::prelude::*;

#[component]
pub fn GameHeader() -> impl IntoView {
	view! {
		<header class="flex justify-between items-center mb-8 nes-container is-rounded">
			<h1 class="nes-text">"bitflip"</h1>
			<button class="nes-btn">"create account"</button>
		</header>
	}
}
