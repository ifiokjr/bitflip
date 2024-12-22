use console::Emoji;

// Status indicators
pub static SUCCESS: Emoji = Emoji("✨ ", "* ");
pub static ERROR: Emoji = Emoji("💥 ", "! ");
pub static INFO: Emoji = Emoji("ℹ️  ", "i ");
pub static WARNING: Emoji = Emoji("⚠️  ", "! ");

// Action indicators
pub static LOADING: Emoji = Emoji("⌛ ", "> ");
pub static STOP: Emoji = Emoji("🛑 ", "x ");
pub static START: Emoji = Emoji("🚀 ", "> ");
pub static DONE: Emoji = Emoji("🎉 ", "* ");

// Program specific
pub static VALIDATOR: Emoji = Emoji("🏦 ", "V ");
pub static GAME: Emoji = Emoji("🎮 ", "G ");
pub static CONFIG: Emoji = Emoji("⚙️  ", "C ");
pub static WALLET: Emoji = Emoji("👛 ", "W ");
