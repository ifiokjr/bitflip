mod constants;
pub mod cpi;
mod errors;
mod events;
#[cfg(feature = "client")]
mod instructions;
mod loaders;
mod pda;
mod processor;
mod state;
mod utils;

use steel::*;

pub use crate::constants::*;
pub use crate::errors::*;
pub use crate::events::*;
#[cfg(feature = "client")]
pub use crate::instructions::*;
pub use crate::loaders::*;
pub use crate::pda::*;
pub use crate::processor::*;
pub use crate::state::*;
pub use crate::utils::*;

declare_id!("5AuNvfV9Xi9gskJpW2qQJndQkFcwbWNV6fjaf2VvuEcM");

#[cfg(not(feature = "no-entrypoint"))]
steel::entrypoint!(process_instruction);

#[cfg(not(feature = "no-entrypoint"))]
solana_security_txt::security_txt! {
	// Required fields
	name: "Bitflip",
	project_url: "https://bitflip.art",
	contacts: "link:https://github.com/ifiokjr/bitflip/security/advisories/new,email:security@kickjump.co,discord:https://bitflip.art/discord",
	policy: "https://github.com/ifiokjr/bitflip/blob/main/security.md",

	// Optional Fields
	preferred_languages: "en",
	source_code: "https://github.com/ifiokjr/bitflip/tree/main/bitflip_program/",
	source_revision: default_env::default_env!("GITHUB_SHA", ""),
	source_release: default_env::default_env!("GITHUB_REF_NAME", ""),
	auditors: concat!("Verifier pubkey: ", default_env::default_env!("GITHUB_SHA", "")),
	encryption: "",
	acknowledgements: "Thank you to all bug bounty degens!"
}

#[cfg(test)]
pub(crate) fn leak<T>(value: T) -> &'static mut T {
	Box::leak(Box::new(value))
}
