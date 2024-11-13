mod constants;
mod errors;
mod events;
#[cfg(feature = "client")]
mod instructions;
mod pda;
mod processor;
mod state;
mod utils;

pub use constants::*;
pub use errors::*;
pub use events::*;
#[cfg(feature = "client")]
pub use instructions::*;
pub use pda::*;
pub use processor::*;
pub use state::*;
use steel::*;
pub use utils::*;

declare_id!("5AuNvfV9Xi9gskJpW2qQJndQkFcwbWNV6fjaf2VvuEcM");

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
	// Required fields
	name: "Bitflip",
	project_url: "https://bitflip.art",
	contacts: "link:https://github.com/ifiokjr/bitflip/security/advisories/new,mailto:security@kickjump.co,discord:https://bitflip.art/discord",
	policy: "https://github.com/ifiokjr/bitflip/blob/main/security.md",

	// Optional Fields
	preferred_languages: "en",
	source_code: "https://github.com/ifiokjr/bitflip/tree/main/bitflip_program/",
	source_revision: "",
	source_release: "bitflip_program-v0.1.0",
	auditors: "https://github.com/solana-labs/security-audits#token-2022"
}

#[cfg(test)]
pub(crate) fn leak<T>(value: T) -> &'static mut T {
	Box::leak(Box::new(value))
}
