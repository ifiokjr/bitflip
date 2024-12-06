use leptos::prelude::*;
use leptos_meta::provide_meta_context;
use leptos_meta::Html;
use leptos_meta::MetaTags;
use leptos_meta::Stylesheet;
use leptos_meta::Title;
use leptos_router::components::FlatRoutes;
use leptos_router::components::Route;
use leptos_router::components::Router;
use leptos_router::path;

use crate::components::GameHeader;
use crate::components::GameInfoBar;
use crate::components::GameSelect;
use crate::components::NextSectionButton;
use crate::components::PreviousSectionButton;
use crate::components::SectionCanvas;
use crate::use_parsed_param;
use crate::AppParam;

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
			<body class="w-full h-full">
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
pub fn HomePage() -> impl IntoView {
	let game_index = use_parsed_param::<u8>(AppParam::Game);
	let section_index = use_parsed_param::<u8>(AppParam::Section);

	view! {
		<div class="flex flex-col items-center">
			<section class="container px-4">
				<GameHeader />
			</section>
			<section class="container p-4">
				<GameInfoBar />
				<div class="h-4" />
				<GameSelect />
				<div class="h-4" />
				// Main game grid container with navigation arrows
				<div class="flex-grow p-0 nes-container is-rounded aspect-square">
					{move || {
						view! {
							<SectionCanvas game_index=game_index() section_index=section_index() />
						}
					}}
				</div>
				<div class="h-4" />
				// Navigation buttons
				<div class="flex gap-4 justify-between py-4">
					<PreviousSectionButton />
					<NextSectionButton />
				</div>
			</section>
		</div>
	}
}
