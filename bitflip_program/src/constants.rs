use const_str_to_pubkey::str_to_pubkey;
use solana_program::native_token::LAMPORTS_PER_SOL;
use steel::Pubkey;

/// The admin pubkey to initialize the configuration for this program.
///
/// This is required to prevent a frontrunning attack. If the program is
/// deployed it would be possible for someone to maliciously run the
/// `launchpad_config_initialize` method which would give them full control of
/// the authority and treasury of the program. By setting ths admin pubkey, this
/// action can only be performed by the environment variable (provided) at build
/// time. The build will fail if this is not provided at build time.
pub const ADMIN_PUBKEY: Pubkey = str_to_pubkey(env!("ADMIN_PUBKEY"));

/// The total number of bits on the canvas
pub const BITFLIP_TOTAL_BITS: usize = 1024 * 1024;
/// The number of sections the canvas is split into.
pub const BITFLIP_TOTAL_SECTIONS: usize = 256;
/// The number of u16's in each section.
pub const BITFLIP_SECTION_LENGTH: usize = BITFLIP_TOTAL_BITS / BITFLIP_TOTAL_SECTIONS / 16;
/// The total number of bits within a section of the game.
pub const BITFLIP_SECTION_TOTAL_BITS: u32 = BITFLIP_SECTION_LENGTH as u32 * 16;

/// The minimum number of flips the previous section must have before the next
/// section can be flipped.
pub const MINIMUM_FLIPS_PER_SECTION: u32 = BITFLIP_SECTION_TOTAL_BITS / 4;

/// How long a session of the bits canvas game lasts. This can be reduced after
/// the game starts.
pub const SESSION_DURATION: i64 = 60 * 60 * 24 * 7 * 4;
/// The access signer duration.
///
/// 24hrs
pub const ACCESS_SIGNER_DURATION: i64 = 60 * 60 * 24;
/// The number of flips before a game is closed. Depending on how the game is
/// going this can be increased or reduced.
pub const MAXIMUM_FLIPS: u64 = 50_000_000;
/// The number of decimals for this token.
pub const TOKEN_DECIMALS: u8 = 0;
/// The total number of minted tokens.
pub const TOTAL_TOKENS: u64 = 1024u64.pow(3) * 8;
/// The number of tokens assigned to each game.
pub const TOKENS_PER_GAME: u64 = TOTAL_TOKENS / 8;
/// Number of tokens assigned to each section treasury.
pub const TOKENS_PER_SECTION: u64 = TOTAL_TOKENS / 8;
/// The transaction fee for a basic transaction.
pub const TRANSACTION_FEE: u64 = 5_000;

/// All PDA accounts start with this seed for consistency.
pub const SEED_PREFIX: &[u8] = b"bitflip";
/// The PDA seed for the player PDA.
pub const SEED_PLAYER: &[u8] = b"player";
/// The PDA seed for an onchain nonce account.
pub const SEED_NONCE: &[u8] = b"nonce";
/// The PDA seed for the configuration account.
pub const SEED_CONFIG: &[u8] = b"config";
/// The PDA seed for BIT mint token account.
pub const SEED_BIT_MINT: &[u8] = b"mint_bit";
/// The PDA seed for KIBIBIT mint token account.
pub const SEED_KIBIBIT_MINT: &[u8] = b"mint_kibibit";
/// The PDA seed for MEBIBIT mint token account.
pub const SEED_MEBIBIT_MINT: &[u8] = b"mint_mebibit";
/// The PDA seed for GIBIBIT mint token account.
pub const SEED_GIBIBIT_MINT: &[u8] = b"mint_gibibit";
/// The treasury account which is also the authority for the `mint` token
/// account.
pub const SEED_TREASURY: &[u8] = b"treasury";
/// The PDA seed for an instance of the game.
pub const SEED_GAME: &[u8] = b"game";
/// The PDA seed for a section within the game. Each game has 256 sections.
pub const SEED_SECTION: &[u8] = b"section";

/// Assuming a price of 100USD per sol. This is approximately 1 cent.
pub const LAMPORTS_PER_BIT: u64 = LAMPORTS_PER_SOL / 100 / 100;

pub const SEED_BIT_CREATOR: &[u8] = b"bit_creator";
pub const SPACE_DISCRIMINATOR: usize = 8;
pub const SPACE_U32: usize = 4;

// Token Metadata
pub const BIT_TOKEN_NAME: &str = "Bit";
pub const BIT_TOKEN_SYMBOL: &str = "B";
pub const BIT_TOKEN_URI: &str = "https://bitflip.art/bit-meta.json";
pub const KIBIBIT_TOKEN_NAME: &str = "KibiBit";
pub const KIBIBIT_TOKEN_SYMBOL: &str = "KiB";
pub const KIBIBIT_TOKEN_URI: &str = "https://bitflip.art/kbit-meta.json";
pub const MEBIBIT_TOKEN_NAME: &str = "MebiBit";
pub const MEBIBIT_TOKEN_SYMBOL: &str = "MiB";
pub const MEBIBIT_TOKEN_URI: &str = "https://bitflip.art/mbit-meta.json";
pub const GIBIBIT_TOKEN_NAME: &str = "GibiBit";
pub const GIBIBIT_TOKEN_SYMBOL: &str = "GiB";
pub const GIBIBIT_TOKEN_URI: &str = "https://bitflip.art/gbit-meta.json";
