use solana_security_txt::security_txt;
use steel::entrypoint;

use crate::process_instruction;

entrypoint!(process_instruction);

security_txt! {
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
