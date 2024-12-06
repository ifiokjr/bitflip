use std::str::FromStr;

use leptos::prelude::*;
use leptos_router::hooks::use_url;
use strum::IntoStaticStr;

use crate::get_active_game_index;
use crate::get_default_section_index;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, IntoStaticStr)]
pub enum AppParam {
	#[strum(serialize = "section")]
	Section,
	#[strum(serialize = "game")]
	Game,
}

pub fn use_parsed_param<T: FromStr + Send + Sync + 'static>(key: AppParam) -> Signal<Option<T>> {
	let parsed_param = move || {
		let url = use_url().get();
		let params = url.search_params();
		log::info!("params: {:?}", params);

		params.get(key.into()).unwrap_or_default().parse::<T>().ok()
	};

	Signal::derive(parsed_param)
}

pub fn use_is_mounted() -> Signal<bool> {
	let (is_mounted, set_is_mounted) = signal(false);

	Effect::new(move || {
		set_is_mounted.set(true);
	});

	is_mounted.into()
}

/// Islands don't support the router. This is a workaround to get the section
/// index from the router.
pub enum RouterProp<T: Send + Sync + 'static> {
	Default,
	Signal(Signal<Option<T>>),
	Value(Option<T>),
}

/// Get the section index from the provided props
pub fn use_section_index(props: RouterProp<u8>) -> Signal<u8> {
	let section_index_resource = Resource::new(move || {}, move |()| get_default_section_index());

	Signal::derive(move || {
		let section_index = match props {
			RouterProp::Default => use_parsed_param::<u8>(AppParam::Section)(),
			RouterProp::Signal(signal) => signal(),
			RouterProp::Value(value) => value,
		};

		if let Some(section_index) = section_index {
			section_index
		} else if let Some(Ok(section_index)) = section_index_resource.get() {
			section_index
		} else {
			0
		}
	})
}
/// Get the game index from the provided props
pub fn use_game_index(props: RouterProp<u8>) -> Signal<u8> {
	let game_index_resource = Resource::new(move || {}, move |()| get_active_game_index());

	Signal::derive(move || {
		let game_index = match props {
			RouterProp::Default => use_parsed_param::<u8>(AppParam::Game)(),
			RouterProp::Signal(signal) => signal(),
			RouterProp::Value(value) => value,
		};

		if let Some(game_index) = game_index {
			game_index
		} else if let Some(Ok(game_index)) = game_index_resource.get() {
			game_index
		} else {
			0
		}
	})
}
