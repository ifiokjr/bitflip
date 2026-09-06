/// Number of independently sharded games supported by the current program ABI.
///
/// Keep this in sync with `BIT_GAME_COUNT` in the Rust program.
const bitflipGameCount = 4;

/// Highest valid zero-based game index.
const bitflipMaximumGameIndex = bitflipGameCount - 1;

/// Number of independently owned sections in each game.
const bitflipSectionsPerGame = 256;

/// Highest valid zero-based section index.
const bitflipMaximumSectionIndex = bitflipSectionsPerGame - 1;
