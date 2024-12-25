use console::Emoji;

// Status indicators
pub const SUCCESS: Emoji = Emoji("✨ ", "* ");
pub const ERROR: Emoji = Emoji("💥 ", "! ");
pub const INFO: Emoji = Emoji("ℹ️  ", "i ");
pub const WARNING: Emoji = Emoji("⚠️  ", "! ");

// Action indicators
pub const LOADING: Emoji = Emoji("⌛ ", "> ");
pub const STOP: Emoji = Emoji("🛑 ", "x ");
pub const START: Emoji = Emoji("🚀 ", "> ");
pub const DONE: Emoji = Emoji("🎉 ", "* ");

// Program specific
pub const VALIDATOR: Emoji = Emoji("🏦 ", "V ");
pub const GAME: Emoji = Emoji("🎮 ", "G ");
pub const CONFIG: Emoji = Emoji("⚙️  ", "C ");
pub const WALLET: Emoji = Emoji("👛 ", "W ");
