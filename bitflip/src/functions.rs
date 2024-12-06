use bitflip_program::get_pda_section;
use bitflip_program::SectionState;
use bitflip_program::BITFLIP_SECTION_LENGTH;
use bitflip_program::BITFLIP_SECTION_TOTAL_BITS;
use leptos::prelude::*;
use rand::Rng;
use rand::RngCore;
use rand::SeedableRng;
use solana_sdk::pubkey::Pubkey;

/// Get the default section index which can be used to choose the section with
/// the lowest price, or the section that has paid for a temporary promotion.
#[allow(clippy::unused_async)]
#[server]
pub async fn get_default_section_index() -> Result<u8, ServerFnError> {
	Ok(0)
}

/// Get the active game index which is currently being played.
#[allow(clippy::unused_async)]
#[server]
pub async fn get_active_game_index() -> Result<u8, ServerFnError> {
	Ok(0)
}

/// Get the number of active players in the game.
#[allow(clippy::unused_async)]
#[server]
pub async fn get_active_players() -> Result<u32, ServerFnError> {
	let mut rng = rand::rngs::StdRng::seed_from_u64(0);
	Ok(rng.next_u32())
}

/// Get the section state for the given game and section index.
#[allow(clippy::unused_async)]
#[server]
pub async fn get_section_state(
	game_index: u8,
	section_index: u8,
) -> Result<SectionState, ServerFnError> {
	let mut rng = rand::rngs::StdRng::seed_from_u64(section_index.into());
	let bump = get_pda_section(game_index, section_index).1;
	let mut section_state =
		SectionState::new(Pubkey::new_unique(), game_index, section_index, bump);
	let section_data: [u16; BITFLIP_SECTION_LENGTH] = std::array::from_fn(|_| rng.r#gen());
	let on = section_data
		.iter()
		.fold(0, |acc, entry| acc + entry.count_ones());
	section_state.data = section_data.map(Into::into);
	section_state.on = on.into();
	section_state.off = (BITFLIP_SECTION_TOTAL_BITS - on).into();
	section_state.flips = section_state.on;

	Ok(section_state)
}
