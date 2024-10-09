use anchor_lang::prelude::Pubkey;
use anchor_lang::solana_program::native_token::LAMPORTS_PER_SOL;
use const_str_to_pubkey::str_to_pubkey;

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
pub const BITS_TOTAL: usize = 1024 * 1024;
/// The number of sections the canvas is split into.
pub const BITS_DATA_SECTIONS: usize = 16;
/// The number of u16's in each section.
pub const BITS_DATA_SECTION_LENGTH: usize = BITS_TOTAL / BITS_DATA_SECTIONS / (u16::BITS as usize);

/// How long a session of the bits canvas game lasts. This can be reduced after
/// the game starts.
pub const SESSION_DURATION: i64 = 60 * 60 * 24 * 7 * 4;
/// The number of flips before a game is closed. Depending on how the game is
/// going this can be increased or reduced.
pub const MAXIMUM_FLIPS: u64 = 50_000_000;
/// The number of decimals for this token.
pub const TOKEN_DECIMALS: u8 = 6;
/// The total number of minted tokens.
pub const MAX_TOKENS: u64 = 1_000_000_000;

/// All PDA accounts start with this seed for consistency.
pub const SEED_PREFIX: &[u8] = b"bitflip";
/// The PDA seed for the configuration account.
pub const SEED_CONFIG: &[u8] = b"config";
/// The PDA seed for the bit meta account.
pub const SEED_BITS: &[u8] = b"bits";
/// The PDA seed for a section of the bits.
pub const SEED_BITS_SECTION: &[u8] = b"bits_section";
pub const SEED_MINT: &[u8] = b"mint";
pub const SEED_MINT_AUTHORITY: &[u8] = b"mint_authority";
pub const SEED_TREASURY: &[u8] = b"treasury";

/// Assuming a price of 100USD per sol. This is approximately 1 cent.
pub const LAMPORTS_PER_BIT: u64 = LAMPORTS_PER_SOL / 100 / 100;

pub const SEED_BIT_CREATOR: &[u8] = b"bit_creator";
pub const SPACE_DISCRIMINATOR: usize = 8;
pub const SPACE_U32: usize = 4;
