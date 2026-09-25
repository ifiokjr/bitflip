//! Bitflip's on-chain canvas, implemented with Pina's zero-copy account model.
//!
//! A game is a 16×16 grid of lazily created sections. The program bootstraps
//! one public 64×64 bitmap, then claimants fund each later account as play
//! unlocks it. Players pay a bounded section-local price to toggle pixels and
//! receive whole BIT from that section's vault. Section
//! owners can trade or freeze their art, and the configured collection
//! authority can attest the compressed NFT created for a frozen section.

#![allow(clippy::inline_always)]
#![no_std]

#[cfg(all(
	not(any(target_os = "solana", target_arch = "bpf")),
	not(feature = "bpf-entrypoint"),
	not(test)
))]
extern crate std;

use pina::sysvars::Sysvar;
use pina::sysvars::clock::Clock;
use pina::*;

pub mod pricing;

declare_id!("5AuNvfV9Xi9gskJpW2qQJndQkFcwbWNV6fjaf2VvuEcM");

/// Marker opening a security.txt block.
pub const SECURITY_TXT_BEGIN: &str = "=======BEGIN SECURITY.TXT V1=======\0";
/// Marker closing a security.txt block.
pub const SECURITY_TXT_END: &str = "=======END SECURITY.TXT V1=======\0";

/// RFC-9116-style contact and policy block, embedded in the program binary.
///
/// `query-security-txt` and the explorer verified badges locate this by
/// scanning program data for [`SECURITY_TXT_BEGIN`]. The delimiters, the `\0`
/// separators, and the alternating NUL-terminated key/value encoding are fixed
/// by that parser, so they are kept byte for byte: a block that does not parse
/// is invisible rather than merely wrong. The body is `concat!`-ed at compile
/// time, so there is no runtime cost and no allocation.
///
/// Kept ungated so the host-side format test can validate it.
#[cfg_attr(not(any(target_os = "solana", target_arch = "bpf")), allow(dead_code))]
const SECURITY_TXT_CONTENT: &str = concat!(
	"=======BEGIN SECURITY.TXT V1=======\0",
	"name\0Bitflip\0",
	"project_url\0https://bitflip.xyz\0",
	"contacts\0email:security@ifiokjr.com,link:https://github.com/ifiokjr/bitflip/blob/main/security.md\0",
	"policy\0https://github.com/ifiokjr/bitflip/blob/main/security.md\0",
	"source_code\0https://github.com/ifiokjr/bitflip\0",
	"auditors\0Internal release audit 2026-09-05; independent audit pending\0",
	"=======END SECURITY.TXT V1=======\0",
);

/// The block as it appears in program data.
///
/// `query-security-txt` finds it by scanning *all* program data for
/// [`SECURITY_TXT_BEGIN`]; it does not read a named ELF section. Two attributes
/// that look correct here are actively harmful, and both were measured against
/// the real-SBF suite rather than assumed:
///
/// - `link_section = ".security.txt"` (what the upstream `solana-security-txt`
///   crate does) emits a writable, allocatable section into the SBF ELF and the
///   loader then misplaces the program image: every instruction returns success
///   without executing anything, so `InitializeConfig` reports ok while creating
///   no account. All 24 isolated SBF tests fail this way.
/// - `#[used]` alone keeps the bytes but breaks the image the same way.
///
/// A plain `no_mangle` static is what survives `--lto` and the size profile
/// without disturbing the program layout: the string stays in `.rodata`, the
/// scan finds it, and the SBF suite passes. `#[used]` must not be re-added.
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub static SECURITY_TXT: &str = SECURITY_TXT_CONTENT;

/// Fresh external bootstrap authority. One-time config creation is safe to
/// sponsor permissionlessly because callers cannot replace this value.
#[cfg(not(feature = "sbf-test-authority"))]
pub const BOOTSTRAP_AUTHORITY: Address = address!("B8yibwGRtrnp55T8uRyt19J6KTTRAZMTD9DgEgjQqVNi");

/// Public, throwaway authority used only by the isolated real-SBF test build.
#[cfg(feature = "sbf-test-authority")]
pub const BOOTSTRAP_AUTHORITY: Address = address!("HMvYWLX41QFw8C3umdL1mbcRDyhGgLWKJK5Zf1dDvFm9");

pub const CANVAS_SIDE: u16 = 1_024;
pub const SECTION_GRID_SIDE: u8 = 16;
pub const SECTION_SIDE: u8 = 64;
pub const SECTION_COUNT: u16 = 256;
pub const SECTION_PIXEL_COUNT: usize = 4_096;
pub const SECTION_BYTES: usize = SECTION_PIXEL_COUNT / u8::BITS as usize;
pub const MAX_FLIPS_PER_TRANSACTION: usize = 16;
pub const FLIP_COORDINATE_BYTES: usize = MAX_FLIPS_PER_TRANSACTION * 2;

/// BIT is always denominated in indivisible whole tokens.
pub const BIT_MINT_DECIMALS: u8 = 0;
/// Fixed supply reserved for four games under the scaled staging economy.
pub const BIT_TOTAL_SUPPLY_TOKENS: u64 = 26_843_545_600;
/// Number of games funded by the fixed BIT supply.
pub const BIT_GAME_COUNT: u8 = 4;
/// BIT reserved for one game.
pub const BIT_GAME_ALLOCATION_TOKENS: u64 = 6_710_886_400;
/// BIT reserved for one of a game's 256 sections.
pub const BIT_SECTION_ALLOCATION_TOKENS: u64 = 26_214_400;
/// Version of the immutable section economy snapshotted by each game.
pub const ECONOMY_VERSION: u8 = 1;

pub const DEFAULT_CLAIM_PRICE_LAMPORTS: u64 = 10_000_000;
pub const DEFAULT_FLIP_FEE_LAMPORTS: u64 = 10_000;
pub const DEFAULT_MIN_FLIP_FEE_LAMPORTS: u64 = 5_000;
pub const DEFAULT_MAX_FLIP_FEE_LAMPORTS: u64 = 1_000_000;
pub const DEFAULT_UNLOCK_INTERVAL_SECONDS: u32 = 3_600;
pub const DEFAULT_EARLY_UNLOCK_FLIPS: u32 = 1_024;

pub const CONFIG_VERSION: u8 = 7;
pub const GAME_STATUS_LIVE: u8 = 1;
pub const GAME_STATUS_CLAIMS_COMPLETE: u8 = 2;
pub const SECTION_STATUS_ACTIVE: u8 = 1;
pub const SECTION_STATUS_SEALED: u8 = 2;
pub const SECTION_STATUS_MINTED: u8 = 3;
pub const SECTION_MODE_OPEN_CANVAS: u8 = 0;
pub const SECTION_MODE_COLOUR_CANVAS: u8 = 1;
pub const SECTION_PALETTE_DEFAULT: u8 = 0;
pub const SECTION_PALETTE_COLOUR_COUNT: u8 = 8;
pub const NO_FLIP_COLOUR: u8 = u8::MAX;
pub const SECTION_REWARD_POLICY_NONE: u8 = 0;
pub const MAX_SECTION_POLICY_DURATION_SECONDS: u64 = 30 * 24 * 60 * 60;
pub const SECTION_POLICY_START_GRACE_SECONDS: i64 = 60;
/// Maximum rent top-up one reserved migration instruction may charge its payer.
pub const MAX_MIGRATION_LAMPORTS: u64 = 1_000_000;

const CONFIG_SEED: &[u8] = b"config";
const GAME_SEED: &[u8] = b"game";
const SECTION_SEED: &[u8] = b"section";
const ZERO_ADDRESS: Address = Address::new_from_array([0; ADDRESS_BYTES]);

#[error]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitflipError {
	/// The signer is not authorized for the requested state transition.
	Unauthorized = 0,
	/// Stored or proposed protocol configuration violates an invariant.
	InvalidConfiguration = 1,
	/// The game index is outside the fixed game range or creation order.
	InvalidGameIndex = 2,
	/// The game cannot accept the requested operation in its current state.
	GameNotLive = 3,
	/// The game launch time has not been reached.
	GameNotStarted = 4,
	/// The section index or section account does not match the instruction.
	InvalidSectionIndex = 5,
	/// The next section has not met its time or activity unlock condition.
	SectionLocked = 6,
	/// The flip batch is empty or exceeds the per-transaction bound.
	InvalidFlipCount = 7,
	/// A pixel coordinate is outside the section canvas.
	InvalidCoordinate = 8,
	/// A paid flip batch contains the same coordinate more than once.
	DuplicateCoordinate = 9,
	/// The current price exceeds a limit signed by the player.
	PriceSlippage = 10,
	/// The section is not active.
	SectionNotActive = 11,
	/// The section has not been sealed.
	SectionNotSealed = 12,
	/// A compressed-NFT receipt was already recorded for the section.
	SectionAlreadyMinted = 13,
	/// The proposed compressed-NFT identity is invalid.
	InvalidAsset = 14,
	/// The paying or custody account cannot cover the requested amount.
	InsufficientFunds = 15,
	/// A section listing must have a nonzero price.
	InvalidSalePrice = 16,
	/// The section has no active sale listing.
	SectionNotForSale = 17,
	/// The section state does not permit ownership transfer.
	SectionNotTransferable = 18,
	/// The current owner cannot buy their own section.
	CannotPurchaseOwnSection = 19,
	/// The section owner changed before a trusted receipt was recorded.
	OwnerChanged = 20,
	/// The runtime timestamp is invalid for the price controller.
	InvalidControllerTimestamp = 21,
	/// The persisted price-controller state is invalid.
	InvalidControllerState = 22,
	/// BIT custody was already configured and is immutable.
	CustodyAlreadyConfigured = 23,
	/// BIT custody or the section vault has not been configured.
	CustodyNotConfigured = 24,
	/// The BIT mint violates the zero-decimal fixed-cap contract.
	InvalidBitMint = 25,
	/// A BIT reserve, vault, or recipient token account is invalid.
	InvalidBitTokenAccount = 26,
	/// The section already received its one-time BIT allocation.
	SectionVaultAlreadyFunded = 27,
	/// The signed quote belongs to a different controller window.
	StalePriceWindow = 28,
	/// The section cannot provide the minimum reward signed by the player.
	InsufficientReward = 29,
	/// The section owner has no accrued fees to withdraw.
	NoOwnerFees = 30,
	/// The proposed section policy is invalid or promises unsupported rewards.
	InvalidSectionPolicy = 31,
	/// A live section policy cannot be replaced or bypassed.
	SectionPolicyLocked = 32,
	/// The signed section-policy version is stale.
	SectionPolicyChanged = 33,
	/// The colour is not valid for the active section mode.
	InvalidFlipColour = 34,
	/// The section has no accrued protocol fees to withdraw.
	NoProtocolFees = 35,
}

#[discriminator(
	entrypoint,
	migrations(ConfigState, GameState, SectionState),
	migrations_max_lamports = MAX_MIGRATION_LAMPORTS
)]
pub enum BitflipInstruction {
	InitializeConfig = 0,
	UpdateConfig = 1,
	ProposeAuthority = 2,
	AcceptAuthority = 3,
	InitializeGame = 4,
	ClaimSection = 5,
	FlipPixels = 6,
	SealSection = 7,
	RecordSectionMint = 8,
	ListSection = 9,
	CancelSectionListing = 10,
	PurchaseSection = 11,
	SettleSectionEconomy = 12,
	ConfigureBitCustody = 13,
	FundSectionVault = 14,
	WithdrawSectionOwnerFees = 15,
	ConfigureSectionPolicy = 16,
	WithdrawProtocolFees = 17,
}

#[discriminator]
pub enum BitflipAccountType {
	ConfigState = 1,
	GameState = 2,
	SectionState = 3,
}

#[discriminator]
pub enum BitflipEvent {
	ColourPixelsFlipped = 1,
}

#[account(discriminator = BitflipAccountType, migrations)]
#[pda(seeds = [CONFIG_SEED], bump = bump)]
pub struct ConfigState {
	pub version: u8,
	pub authority: Address,
	pub pending_authority: Address,
	pub treasury: Address,
	pub collection_authority: Address,
	pub bit_mint: Address,
	pub bit_reserve: Address,
	pub claim_price_lamports: u64,
	/// Per-flip price at game creation. Renamed in spirit: it is no longer a flat
	/// fee but the controller's *start price* (`start_price_lamports`), bounded by
	/// `minimum_flip_fee_lamports` and `maximum_flip_fee_lamports`. The controller
	/// then moves the posted price within those bounds as windows settle.
	pub flip_fee_lamports: u64,
	/// Lower bound for the congestion controller and the posted price.
	pub minimum_flip_fee_lamports: u64,
	/// Upper bound for the congestion controller and the posted price.
	pub maximum_flip_fee_lamports: u64,
	pub unlock_interval_seconds: u32,
	pub early_unlock_flips: u32,
	pub game_count: u16,
	pub bump: u8,
}

#[account(discriminator = BitflipAccountType, migrations)]
#[pda(seeds = [GAME_SEED, game_index: u8], bump = bump)]
pub struct GameState {
	pub game_index: u8,
	pub status: u8,
	pub bump: u8,
	pub economy_version: u8,
	pub starts_at: i64,
	pub next_section: u16,
	pub minted_sections: u16,
	/// Snapshot of the config start price at game creation.
	///
	/// Vestigial for pricing: the live controller reads [`Self::start_price_lamports`],
	/// which is written from the same source in the same instruction. This field is
	/// retained only because removing it changes the ABI layout. Do not read it as
	/// the current price — use `start_price_lamports` and the section controller.
	pub flip_fee_lamports: u64,
	pub total_flips: u64,
	pub section_allocation_tokens: u64,
	pub emission_duration_seconds: u64,
	pub window_seconds: u64,
	pub target_tokens_per_window: u64,
	pub start_price_lamports: u64,
	pub minimum_price_lamports: u64,
	pub maximum_price_lamports: u64,
	pub start_floor_price_lamports: u64,
	pub end_floor_price_lamports: u64,
	pub change_denominator: u64,
	pub burst_elasticity: u64,
	pub owner_share_basis_points: u16,
}

#[account(discriminator = BitflipAccountType, migrations)]
#[pda(
	seeds = [SECTION_SEED, game_index: u8, section_index: u8],
	bump = bump
)]
pub struct SectionState {
	pub owner: Address,
	pub asset_id: Address,
	pub merkle_tree: Address,
	pub bit_vault: Address,
	pub game_index: u8,
	pub section_index: u8,
	pub status: u8,
	pub bump: u8,
	pub on_pixels: u16,
	pub leaf_index: u32,
	pub flip_count: u64,
	pub revision: u64,
	pub last_flip_at: i64,
	pub sale_price_lamports: u64,
	pub economy_launched_at: u64,
	pub economy_window_started_at: u64,
	pub economy_last_updated_at: u64,
	pub economy_window_id: u64,
	pub economy_window_target_tokens: u64,
	pub economy_window_rewarded_tokens: u64,
	pub emitted_tokens: u64,
	pub reward_pool_tokens: u64,
	pub controller_price_lamports: u64,
	pub posted_price_lamports: u64,
	pub protocol_fee_lamports: u64,
	pub owner_fee_lamports: u64,
	pub policy_version: u64,
	pub policy_starts_at: i64,
	pub policy_ends_at: i64,
	pub policy_entry_price_tokens: u64,
	pub policy_reward_per_action_tokens: u64,
	pub policy_rules_digest: [u8; 32],
	pub policy_mode: u8,
	pub policy_palette_id: u8,
	pub policy_reward_policy: u8,
	pub pixels: [u8; 512],
}

#[instruction(discriminator = BitflipInstruction::InitializeConfig, migrations)]
pub struct InitializeConfigInstruction {
	pub bump: u8,
}

#[instruction(discriminator = BitflipInstruction::UpdateConfig, migrations)]
pub struct UpdateConfigInstruction {
	pub treasury: Address,
	pub collection_authority: Address,
	pub claim_price_lamports: u64,
	pub flip_fee_lamports: u64,
	pub minimum_flip_fee_lamports: u64,
	pub maximum_flip_fee_lamports: u64,
	pub unlock_interval_seconds: u32,
	pub early_unlock_flips: u32,
}

#[instruction(discriminator = BitflipInstruction::ProposeAuthority, migrations)]
pub struct ProposeAuthorityInstruction {
	pub pending_authority: Address,
}

#[instruction(discriminator = BitflipInstruction::AcceptAuthority, migrations)]
pub struct AcceptAuthorityInstruction {}

#[instruction(discriminator = BitflipInstruction::InitializeGame, migrations)]
pub struct InitializeGameInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub game_bump: u8,
	pub section_bump: u8,
}

#[instruction(discriminator = BitflipInstruction::ClaimSection, migrations)]
pub struct ClaimSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub bump: u8,
	pub maximum_price_lamports: u64,
}

#[instruction(
	discriminator = BitflipInstruction::FlipPixels,
	migrations,
	validate(with = validate_flip_pixel_coordinates)
)]
pub struct FlipPixelsInstruction {
	pub game_index: u8,
	pub section_index: u8,
	#[pina(validate(
		min = 1,
		max = MAX_FLIPS_PER_TRANSACTION as u8,
		error = BitflipError::InvalidFlipCount
	))]
	pub count: u8,
	pub coordinates: [u8; 32],
	pub colour: u8,
	pub expected_policy_version: u64,
	pub expected_window_id: u64,
	pub maximum_unit_price_lamports: u64,
	pub maximum_total_price_lamports: u64,
	pub minimum_reward_tokens: u64,
}

#[event(discriminator = BitflipEvent, variant = ColourPixelsFlipped, migrations)]
pub struct ColourPixelsFlippedEvent {
	pub player: Address,
	pub policy_version: u64,
	pub revision: u64,
	pub coordinates: [u8; 32],
	#[pina(validate(max = BIT_GAME_COUNT - 1))]
	pub game_index: u8,
	pub section_index: u8,
	#[pina(validate(
		min = 1,
		max = MAX_FLIPS_PER_TRANSACTION as u8,
		error = BitflipError::InvalidFlipCount
	))]
	pub count: u8,
	#[pina(validate(max = SECTION_PALETTE_COLOUR_COUNT - 1))]
	pub colour: u8,
}

#[instruction(discriminator = BitflipInstruction::SealSection, migrations)]
pub struct SealSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::RecordSectionMint, migrations)]
pub struct RecordSectionMintInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub expected_owner: Address,
	pub asset_id: Address,
	pub merkle_tree: Address,
	pub leaf_index: u32,
}

#[instruction(discriminator = BitflipInstruction::ListSection, migrations)]
pub struct ListSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
	#[pina(validate(min = 1, error = BitflipError::InvalidSalePrice))]
	pub price_lamports: u64,
}

#[instruction(discriminator = BitflipInstruction::CancelSectionListing, migrations)]
pub struct CancelSectionListingInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::PurchaseSection, migrations)]
pub struct PurchaseSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub maximum_price_lamports: u64,
}

#[instruction(discriminator = BitflipInstruction::SettleSectionEconomy, migrations)]
pub struct SettleSectionEconomyInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::ConfigureBitCustody, migrations)]
pub struct ConfigureBitCustodyInstruction {}

#[instruction(discriminator = BitflipInstruction::FundSectionVault, migrations)]
pub struct FundSectionVaultInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::WithdrawSectionOwnerFees, migrations)]
pub struct WithdrawSectionOwnerFeesInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::ConfigureSectionPolicy, migrations)]
pub struct ConfigureSectionPolicyInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub expected_policy_version: u64,
	pub mode: u8,
	pub palette_id: u8,
	pub reward_policy: u8,
	pub starts_at: i64,
	pub ends_at: i64,
	pub entry_price_tokens: u64,
	pub reward_per_action_tokens: u64,
	pub rules_digest: [u8; 32],
}

#[instruction(discriminator = BitflipInstruction::WithdrawProtocolFees, migrations)]
pub struct WithdrawProtocolFeesInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[derive(Accounts, Debug)]
pub struct InitializeConfigAccounts<'a> {
	#[pina(validate(signer))]
	#[pina(validate(owner = system::ID))]
	pub payer: &'a mut AccountView,
	#[pina(validate(empty))]
	pub config: &'a mut AccountView,
	#[pina(validate(address = system::ID))]
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct UpdateConfigAccounts<'a> {
	#[pina(validate(signer))]
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct ProposeAuthorityAccounts<'a> {
	#[pina(validate(signer))]
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct AcceptAuthorityAccounts<'a> {
	#[pina(validate(signer))]
	pub pending_authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct InitializeGameAccounts<'a> {
	#[pina(validate(signer))]
	#[pina(validate(owner = system::ID))]
	pub payer: &'a mut AccountView,
	pub config: &'a mut AccountView,
	#[pina(validate(empty))]
	pub game: &'a mut AccountView,
	#[pina(validate(empty))]
	pub section: &'a mut AccountView,
	#[pina(validate(address = system::ID))]
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct ClaimSectionAccounts<'a> {
	#[pina(validate(signer))]
	#[pina(validate(owner = system::ID))]
	pub owner: &'a mut AccountView,
	pub config: &'a AccountView,
	pub game: &'a mut AccountView,
	pub previous_section: &'a AccountView,
	pub section: &'a mut AccountView,
	pub treasury: &'a mut AccountView,
	#[pina(validate(address = system::ID))]
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct FlipPixelsAccounts<'a> {
	#[pina(validate(signer))]
	#[pina(validate(owner = system::ID))]
	pub player: &'a mut AccountView,
	pub config: &'a AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
	pub bit_mint: &'a AccountView,
	pub section_vault: &'a mut AccountView,
	pub player_bit_account: &'a mut AccountView,
	pub token_program: &'a AccountView,
	#[pina(validate(address = system::ID))]
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct SealSectionAccounts<'a> {
	#[pina(validate(signer))]
	pub owner: &'a mut AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct RecordSectionMintAccounts<'a> {
	#[pina(validate(signer))]
	pub collection_authority: &'a AccountView,
	pub config: &'a AccountView,
	pub game: &'a mut AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct ListSectionAccounts<'a> {
	#[pina(validate(signer))]
	pub owner: &'a AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct CancelSectionListingAccounts<'a> {
	#[pina(validate(signer))]
	pub owner: &'a AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct PurchaseSectionAccounts<'a> {
	#[pina(validate(signer))]
	#[pina(validate(owner = system::ID))]
	pub buyer: &'a mut AccountView,
	#[pina(validate(owner = system::ID))]
	pub seller: &'a mut AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
	#[pina(validate(address = system::ID))]
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct SettleSectionEconomyAccounts<'a> {
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct ConfigureBitCustodyAccounts<'a> {
	#[pina(validate(signer))]
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
	pub bit_mint: &'a AccountView,
	pub bit_reserve: &'a AccountView,
	#[pina(validate(address = token_2022::ID))]
	pub token_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct FundSectionVaultAccounts<'a> {
	#[pina(validate(signer))]
	#[pina(validate(owner = system::ID))]
	pub funder: &'a mut AccountView,
	pub config: &'a AccountView,
	pub section: &'a mut AccountView,
	pub bit_mint: &'a AccountView,
	pub bit_reserve: &'a mut AccountView,
	pub section_vault: &'a mut AccountView,
	#[pina(validate(address = associated_token_account::ID))]
	pub associated_token_program: &'a AccountView,
	#[pina(validate(address = token_2022::ID))]
	pub token_program: &'a AccountView,
	#[pina(validate(address = system::ID))]
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct WithdrawSectionOwnerFeesAccounts<'a> {
	#[pina(validate(signer))]
	pub owner: &'a mut AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct ConfigureSectionPolicyAccounts<'a> {
	#[pina(validate(signer))]
	pub owner: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct WithdrawProtocolFeesAccounts<'a> {
	#[pina(validate(signer))]
	pub authority: &'a AccountView,
	pub config: &'a AccountView,
	pub section: &'a mut AccountView,
	pub treasury: &'a mut AccountView,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct PixelLocation {
	byte_index: usize,
	mask: u8,
}

fn pixel_location(x: u8, y: u8) -> Result<PixelLocation, ProgramError> {
	if x >= SECTION_SIDE || y >= SECTION_SIDE {
		return Err(BitflipError::InvalidCoordinate.into());
	}

	let linear_index = usize::from(y) * usize::from(SECTION_SIDE) + usize::from(x);

	Ok(PixelLocation {
		byte_index: linear_index / u8::BITS as usize,
		mask: 1 << (linear_index % u8::BITS as usize),
	})
}

/// Every declared pixel must sit inside the section canvas, and no pixel
/// may repeat within one flip batch.
fn validate_flip_pixel_coordinates(value: &FlipPixelsInstructionZc) -> ProgramResult {
	validate_flip_coordinates(value.count(), value.coordinates())
}

fn validate_flip_coordinates(
	count: u8,
	coordinates: &[u8; FLIP_COORDINATE_BYTES],
) -> ProgramResult {
	let count = usize::from(count);

	if count == 0 || count > MAX_FLIPS_PER_TRANSACTION {
		return Err(BitflipError::InvalidFlipCount.into());
	}

	for index in 0..count {
		let offset = index * 2;
		let x = coordinates[offset];
		let y = coordinates[offset + 1];
		let _ = pixel_location(x, y)?;

		for prior in 0..index {
			let prior_offset = prior * 2;

			if coordinates[prior_offset] == x && coordinates[prior_offset + 1] == y {
				return Err(BitflipError::DuplicateCoordinate.into());
			}
		}
	}

	Ok(())
}

#[allow(clippy::too_many_arguments)]
fn validate_section_policy(
	mode: u8,
	palette_id: u8,
	reward_policy: u8,
	starts_at: i64,
	ends_at: i64,
	entry_price_tokens: u64,
	reward_per_action_tokens: u64,
	rules_digest: &[u8; 32],
	now: i64,
) -> ProgramResult {
	let duration = ends_at
		.checked_sub(starts_at)
		.and_then(|value| u64::try_from(value).ok())
		.ok_or(BitflipError::InvalidSectionPolicy)?;
	let mode_is_allowed = mode == SECTION_MODE_OPEN_CANVAS || mode == SECTION_MODE_COLOUR_CANVAS;
	let inactive_reward_terms = reward_policy == SECTION_REWARD_POLICY_NONE
		&& entry_price_tokens == 0
		&& reward_per_action_tokens == 0;

	if !mode_is_allowed
		|| palette_id != SECTION_PALETTE_DEFAULT
		|| starts_at < now.saturating_sub(SECTION_POLICY_START_GRACE_SECONDS)
		|| duration == 0
		|| duration > MAX_SECTION_POLICY_DURATION_SECONDS
		|| !inactive_reward_terms
		|| rules_digest.iter().all(|byte| *byte == 0)
	{
		return Err(BitflipError::InvalidSectionPolicy.into());
	}

	Ok(())
}

fn section_policy_is_live(section: &SectionStateZc, now: i64) -> bool {
	let starts_at = section.policy_starts_at.get();
	starts_at != 0 && now >= starts_at && now < section.policy_ends_at.get()
}

fn assert_section_policy_version(section: &SectionStateZc, expected_version: u64) -> ProgramResult {
	if section.policy_version.get() != expected_version {
		return Err(BitflipError::SectionPolicyChanged.into());
	}

	Ok(())
}

fn validate_flip_colour(section: &SectionStateZc, colour: u8, now: i64) -> ProgramResult {
	let colour_mode_is_live =
		section_policy_is_live(section, now) && section.policy_mode == SECTION_MODE_COLOUR_CANVAS;
	let colour_is_valid = if colour_mode_is_live {
		colour < SECTION_PALETTE_COLOUR_COUNT
	} else {
		colour == NO_FLIP_COLOUR
	};

	if !colour_is_valid {
		return Err(BitflipError::InvalidFlipColour.into());
	}

	Ok(())
}

struct SectionFlipState {
	bump: u8,
	owner: Address,
	vault: Address,
	policy_version: u64,
	controller: pricing::PriceControllerState,
}

fn section_flip_state(
	section_account: &AccountView,
	args: &FlipPixelsInstructionZc,
	now: i64,
) -> Result<SectionFlipState, ProgramError> {
	let section = section_account.as_account::<SectionState>(&ID)?;

	if section.status != SECTION_STATUS_ACTIVE {
		return Err(BitflipError::SectionNotActive.into());
	}

	if section.bit_vault == ZERO_ADDRESS {
		return Err(BitflipError::CustodyNotConfigured.into());
	}

	assert_section_policy_version(&section, args.expected_policy_version.get())?;
	validate_flip_colour(&section, args.colour, now)?;

	Ok(SectionFlipState {
		bump: section.bump,
		owner: section.owner,
		vault: section.bit_vault,
		policy_version: section.policy_version.get(),
		controller: section_controller_state(&section),
	})
}

fn emit_colour_pixels_flipped(
	player: Address,
	policy_version: u64,
	revision: u64,
	args: &FlipPixelsInstructionZc,
) -> ProgramResult {
	let mut data = [0; ColourPixelsFlippedEvent::SIZE];
	ColourPixelsFlippedEvent::initialize(&mut data, |event| {
		event.player = player;
		event.policy_version.set(policy_version);
		event.revision.set(revision);
		event.coordinates.copy_from_slice(&args.coordinates);
		event.game_index = args.game_index;
		event.section_index = args.section_index;
		event.count = args.count;
		event.colour = args.colour;

		Ok(())
	})?;
	solana_program_log::log_data(&[&data]);

	Ok(())
}

fn toggle_pixel(pixels: &mut [u8; SECTION_BYTES], x: u8, y: u8) -> Result<bool, ProgramError> {
	let location = pixel_location(x, y)?;
	pixels[location.byte_index] ^= location.mask;

	Ok(pixels[location.byte_index] & location.mask != 0)
}

fn section_unlock_at(
	starts_at: i64,
	section_index: u8,
	interval_seconds: u32,
) -> Result<i64, ProgramError> {
	let offset = i64::from(section_index)
		.checked_mul(i64::from(interval_seconds))
		.ok_or(ProgramError::ArithmeticOverflow)?;

	starts_at
		.checked_add(offset)
		.ok_or(ProgramError::ArithmeticOverflow)
}

#[allow(clippy::too_many_arguments)]
fn validate_configuration(
	treasury: &Address,
	collection_authority: &Address,
	flip_fee_lamports: u64,
	minimum_flip_fee_lamports: u64,
	maximum_flip_fee_lamports: u64,
	unlock_interval_seconds: u32,
	early_unlock_flips: u32,
) -> ProgramResult {
	let addresses_are_valid = treasury != &ZERO_ADDRESS && collection_authority != &ZERO_ADDRESS;
	let fees_are_valid = minimum_flip_fee_lamports > 0
		&& minimum_flip_fee_lamports <= flip_fee_lamports
		&& flip_fee_lamports <= maximum_flip_fee_lamports;
	let progression_is_valid = unlock_interval_seconds > 0
		&& early_unlock_flips > 0
		&& usize::try_from(early_unlock_flips).is_ok_and(|value| value <= SECTION_PIXEL_COUNT);

	let price_config_is_valid = fees_are_valid
		&& configured_price_config(
			flip_fee_lamports,
			minimum_flip_fee_lamports,
			maximum_flip_fee_lamports,
		)
		.validate()
		.is_ok();

	if !addresses_are_valid || !fees_are_valid || !progression_is_valid || !price_config_is_valid {
		return Err(BitflipError::InvalidConfiguration.into());
	}

	Ok(())
}

fn assert_config_account(config: &AccountView) -> ProgramResult {
	let state = ConfigState::load_pda(config, &ID)?;

	if state.version != CONFIG_VERSION {
		return Err(BitflipError::InvalidConfiguration.into());
	}

	Ok(())
}

fn assert_game_account(game: &AccountView, game_index: u8) -> ProgramResult {
	let state = GameState::load_pda(game, game_index, &ID)?;

	if state.game_index != game_index {
		return Err(BitflipError::InvalidGameIndex.into());
	}

	if state.economy_version != ECONOMY_VERSION {
		return Err(BitflipError::InvalidConfiguration.into());
	}

	Ok(())
}

fn assert_section_account(
	section: &AccountView,
	game_index: u8,
	section_index: u8,
) -> ProgramResult {
	let state = SectionState::load_pda(section, game_index, section_index, &ID)?;

	if state.game_index != game_index || state.section_index != section_index {
		return Err(BitflipError::InvalidSectionIndex.into());
	}

	Ok(())
}

fn controller_error(error: pricing::PriceControllerError) -> ProgramError {
	match error {
		pricing::PriceControllerError::InvalidConfiguration => {
			BitflipError::InvalidConfiguration.into()
		}
		pricing::PriceControllerError::InvalidTimestamp => {
			BitflipError::InvalidControllerTimestamp.into()
		}
		pricing::PriceControllerError::ArithmeticOverflow => ProgramError::ArithmeticOverflow,
		pricing::PriceControllerError::StaleWindow => BitflipError::StalePriceWindow.into(),
		pricing::PriceControllerError::PriceSlippage => BitflipError::PriceSlippage.into(),
		pricing::PriceControllerError::InsufficientReward => {
			BitflipError::InsufficientReward.into()
		}
		pricing::PriceControllerError::InvalidRequest => {
			BitflipError::InvalidControllerState.into()
		}
	}
}

fn controller_timestamp(timestamp: i64) -> Result<u64, ProgramError> {
	u64::try_from(timestamp).map_err(|_| BitflipError::InvalidControllerTimestamp.into())
}

fn game_price_config(game: &GameStateZc) -> Result<pricing::PriceControllerConfig, ProgramError> {
	if game.economy_version != ECONOMY_VERSION {
		return Err(BitflipError::InvalidConfiguration.into());
	}

	let config = pricing::PriceControllerConfig {
		allocation_tokens: game.section_allocation_tokens.get(),
		emission_duration_seconds: game.emission_duration_seconds.get(),
		window_seconds: game.window_seconds.get(),
		target_tokens_per_window: game.target_tokens_per_window.get(),
		start_price_lamports: game.start_price_lamports.get(),
		minimum_price_lamports: game.minimum_price_lamports.get(),
		maximum_price_lamports: game.maximum_price_lamports.get(),
		start_floor_price_lamports: game.start_floor_price_lamports.get(),
		end_floor_price_lamports: game.end_floor_price_lamports.get(),
		change_denominator: game.change_denominator.get(),
		burst_elasticity: game.burst_elasticity.get(),
	};
	config.validate().map_err(controller_error)?;

	Ok(config)
}

fn configured_price_config(
	start_price_lamports: u64,
	minimum_price_lamports: u64,
	maximum_price_lamports: u64,
) -> pricing::PriceControllerConfig {
	pricing::PriceControllerConfig {
		start_price_lamports,
		minimum_price_lamports,
		maximum_price_lamports,
		start_floor_price_lamports: minimum_price_lamports,
		end_floor_price_lamports: pricing::DEFAULT_END_FLOOR_PRICE_LAMPORTS
			.clamp(minimum_price_lamports, maximum_price_lamports),
		..pricing::PriceControllerConfig::STAGING
	}
}

fn initial_game_price_config(config: &ConfigStateZc) -> pricing::PriceControllerConfig {
	configured_price_config(
		config.flip_fee_lamports.get(),
		config.minimum_flip_fee_lamports.get(),
		config.maximum_flip_fee_lamports.get(),
	)
}

fn live_game_price_config(
	game_account: &AccountView,
) -> Result<(i64, u16, pricing::PriceControllerConfig), ProgramError> {
	let game = game_account.as_account::<GameState>(&ID)?;

	if game.status != GAME_STATUS_LIVE && game.status != GAME_STATUS_CLAIMS_COMPLETE {
		return Err(BitflipError::GameNotLive.into());
	}

	Ok((
		game.starts_at.get(),
		game.owner_share_basis_points.get(),
		game_price_config(&game)?,
	))
}

fn configured_bit_mint(config_account: &AccountView) -> Result<Address, ProgramError> {
	let bit_mint = config_account.as_account::<ConfigState>(&ID)?.bit_mint;

	if bit_mint == ZERO_ADDRESS {
		return Err(BitflipError::CustodyNotConfigured.into());
	}

	Ok(bit_mint)
}

fn section_controller_state(section: &SectionStateZc) -> pricing::PriceControllerState {
	pricing::PriceControllerState {
		launched_at: section.economy_launched_at.get(),
		window_started_at: section.economy_window_started_at.get(),
		last_updated_at: section.economy_last_updated_at.get(),
		window_id: section.economy_window_id.get(),
		window_target_tokens: section.economy_window_target_tokens.get(),
		window_rewarded_tokens: section.economy_window_rewarded_tokens.get(),
		emitted_tokens: section.emitted_tokens.get(),
		reward_pool_tokens: section.reward_pool_tokens.get(),
		controller_price_lamports: section.controller_price_lamports.get(),
		posted_price_lamports: section.posted_price_lamports.get(),
	}
}

fn store_section_controller_state(
	section: &mut SectionStateZc,
	state: pricing::PriceControllerState,
) {
	section.economy_launched_at.set(state.launched_at);
	section
		.economy_window_started_at
		.set(state.window_started_at);
	section.economy_last_updated_at.set(state.last_updated_at);
	section.economy_window_id.set(state.window_id);
	section
		.economy_window_target_tokens
		.set(state.window_target_tokens);
	section
		.economy_window_rewarded_tokens
		.set(state.window_rewarded_tokens);
	section.emitted_tokens.set(state.emitted_tokens);
	section.reward_pool_tokens.set(state.reward_pool_tokens);
	section
		.controller_price_lamports
		.set(state.controller_price_lamports);
	section
		.posted_price_lamports
		.set(state.posted_price_lamports);
}

fn store_paid_flip(
	section: &mut SectionStateZc,
	count: u8,
	coordinates: &[u8; FLIP_COORDINATE_BYTES],
	clock_timestamp: i64,
	fee_split: pricing::FeeSplit,
	controller: pricing::PriceControllerState,
) -> Result<u64, ProgramError> {
	let mut on_pixels = section.on_pixels.get();

	for index in 0..usize::from(count) {
		let offset = index * 2;
		let turned_on = toggle_pixel(
			&mut section.pixels,
			coordinates[offset],
			coordinates[offset + 1],
		)?;
		on_pixels = if turned_on {
			on_pixels
				.checked_add(1)
				.ok_or(ProgramError::ArithmeticOverflow)?
		} else {
			on_pixels
				.checked_sub(1)
				.ok_or(ProgramError::ArithmeticOverflow)?
		};
	}

	section.on_pixels.set(on_pixels);
	section.flip_count.set(
		section
			.flip_count
			.get()
			.checked_add(u64::from(count))
			.ok_or(ProgramError::ArithmeticOverflow)?,
	);
	let revision = section
		.revision
		.get()
		.checked_add(1)
		.ok_or(ProgramError::ArithmeticOverflow)?;
	section.revision.set(revision);
	section.last_flip_at.set(clock_timestamp);
	section.protocol_fee_lamports.set(
		section
			.protocol_fee_lamports
			.get()
			.checked_add(fee_split.protocol_lamports)
			.ok_or(ProgramError::ArithmeticOverflow)?,
	);
	section.owner_fee_lamports.set(
		section
			.owner_fee_lamports
			.get()
			.checked_add(fee_split.owner_lamports)
			.ok_or(ProgramError::ArithmeticOverflow)?,
	);
	store_section_controller_state(section, controller);

	Ok(revision)
}

fn initialize_game_state(
	game: &mut GameStateZc,
	game_index: u8,
	bump: u8,
	starts_at: i64,
	flip_fee_lamports: u64,
	price_config: pricing::PriceControllerConfig,
) {
	game.game_index = game_index;
	game.status = GAME_STATUS_LIVE;
	game.bump = bump;
	game.economy_version = ECONOMY_VERSION;
	game.starts_at.set(starts_at);
	game.next_section.set(1);
	game.minted_sections.set(0);
	game.flip_fee_lamports.set(flip_fee_lamports);
	game.total_flips.set(0);
	game.section_allocation_tokens
		.set(price_config.allocation_tokens);
	game.emission_duration_seconds
		.set(price_config.emission_duration_seconds);
	game.window_seconds.set(price_config.window_seconds);
	game.target_tokens_per_window
		.set(price_config.target_tokens_per_window);
	game.start_price_lamports
		.set(price_config.start_price_lamports);
	game.minimum_price_lamports
		.set(price_config.minimum_price_lamports);
	game.maximum_price_lamports
		.set(price_config.maximum_price_lamports);
	game.start_floor_price_lamports
		.set(price_config.start_floor_price_lamports);
	game.end_floor_price_lamports
		.set(price_config.end_floor_price_lamports);
	game.change_denominator.set(price_config.change_denominator);
	game.burst_elasticity.set(price_config.burst_elasticity);
	game.owner_share_basis_points
		.set(pricing::DEFAULT_OWNER_SHARE_BASIS_POINTS);
}

fn initialize_section_state(
	section: &mut SectionStateZc,
	owner: Address,
	game_index: u8,
	section_index: u8,
	bump: u8,
	controller: pricing::PriceControllerState,
) {
	section.owner = owner;
	section.asset_id = ZERO_ADDRESS;
	section.merkle_tree = ZERO_ADDRESS;
	section.bit_vault = ZERO_ADDRESS;
	section.game_index = game_index;
	section.section_index = section_index;
	section.status = SECTION_STATUS_ACTIVE;
	section.bump = bump;
	section.on_pixels.set(0);
	section.leaf_index.set(0);
	section.flip_count.set(0);
	section.revision.set(0);
	section.last_flip_at.set(0);
	section.sale_price_lamports.set(0);
	section.protocol_fee_lamports.set(0);
	section.owner_fee_lamports.set(0);
	section.policy_version.set(0);
	section.policy_starts_at.set(0);
	section.policy_ends_at.set(0);
	section.policy_entry_price_tokens.set(0);
	section.policy_reward_per_action_tokens.set(0);
	section.policy_rules_digest.fill(0);
	section.policy_mode = SECTION_MODE_OPEN_CANVAS;
	section.policy_palette_id = SECTION_PALETTE_DEFAULT;
	section.policy_reward_policy = SECTION_REWARD_POLICY_NONE;
	store_section_controller_state(section, controller);
	section.pixels.fill(0);
}

fn transfer_lamports(
	from: &AccountView,
	to: &AccountView,
	lamports: u64,
	system_program: &AccountView,
) -> ProgramResult {
	if lamports == 0 {
		return Ok(());
	}

	if from.lamports() < lamports {
		return Err(BitflipError::InsufficientFunds.into());
	}

	system_program.assert_address(&system::ID)?;
	system::instructions::Transfer { from, to, lamports }.invoke()
}

fn pay_accrued_owner_fees(
	section: &mut AccountView,
	recipient: &mut AccountView,
	require_nonzero: bool,
) -> Result<u64, ProgramError> {
	let (owner, amount) = {
		let state = section.as_account::<SectionState>(&ID)?;
		(state.owner, state.owner_fee_lamports.get())
	};
	recipient
		.assert_address(&owner)?
		.assert_writable()?
		.assert_owner(&system::ID)?;

	if amount == 0 {
		return if require_nonzero {
			Err(BitflipError::NoOwnerFees.into())
		} else {
			Ok(0)
		};
	}

	section.assert_writable()?.assert_owner(&ID)?;
	section
		.as_account_mut::<SectionState>(&ID)?
		.owner_fee_lamports
		.set(0);
	section.send_owned(&ID, amount, recipient)?;

	Ok(amount)
}

fn pay_accrued_protocol_fees(
	section: &mut AccountView,
	treasury: &mut AccountView,
) -> Result<u64, ProgramError> {
	let amount = section
		.as_account::<SectionState>(&ID)?
		.protocol_fee_lamports
		.get();

	if amount == 0 {
		return Err(BitflipError::NoProtocolFees.into());
	}

	treasury.assert_writable()?.assert_owner(&system::ID)?;
	section.assert_writable()?.assert_owner(&ID)?;
	section
		.as_account_mut::<SectionState>(&ID)?
		.protocol_fee_lamports
		.set(0);
	section.send_owned(&ID, amount, treasury)?;

	Ok(amount)
}

fn split_flip_fee(
	section_owner: &Address,
	game: &Address,
	total_price_lamports: u64,
	owner_share_basis_points: u16,
) -> Result<pricing::FeeSplit, ProgramError> {
	let protocol_owned = section_owner == game;
	pricing::split_fee(
		total_price_lamports,
		if protocol_owned {
			0
		} else {
			owner_share_basis_points
		},
	)
	.map_err(controller_error)
}

fn assert_bit_mint(bit_mint: &AccountView, token_program: &Address) -> Result<u64, ProgramError> {
	let mint = bit_mint
		.as_token_mint_for_program(token_program)
		.and_then(token::TokenMintRef::assert_no_extensions)
		.map_err(|_| ProgramError::from(BitflipError::InvalidBitMint))?;

	if !mint.is_initialized()
		|| mint.decimals() != BIT_MINT_DECIMALS
		|| mint.supply() > BIT_TOTAL_SUPPLY_TOKENS
		|| mint.mint_authority().is_some()
		|| mint.freeze_authority().is_some()
	{
		return Err(BitflipError::InvalidBitMint.into());
	}

	Ok(mint.supply())
}

fn bit_token_account_balance(
	account: &AccountView,
	owner: &Address,
	bit_mint: &Address,
	token_program: &Address,
) -> Result<u64, ProgramError> {
	let token_account = account
		.as_associated_token_account(owner, bit_mint, token_program)
		.and_then(|account| {
			account.assert_extensions_allowed(&[token_2022::state::ExtensionType::ImmutableOwner])
		})
		.map_err(|_| ProgramError::from(BitflipError::InvalidBitTokenAccount))?;

	if !token_account.is_initialized()
		|| token_account.is_frozen()
		|| token_account.is_native()
		|| token_account.mint() != bit_mint
		|| token_account.owner() != owner
		|| token_account.delegate().is_some()
		|| token_account.delegated_amount() != 0
		|| token_account.close_authority().is_some()
	{
		return Err(BitflipError::InvalidBitTokenAccount.into());
	}

	Ok(token_account.amount())
}

fn bit_recipient_account_balance(
	account: &AccountView,
	owner: &Address,
	bit_mint: &Address,
	token_program: &Address,
) -> Result<u64, ProgramError> {
	let token_account = account
		.as_associated_token_account(owner, bit_mint, token_program)
		.map_err(|_| ProgramError::from(BitflipError::InvalidBitTokenAccount))?;

	if !token_account.is_initialized()
		|| token_account.is_frozen()
		|| token_account.is_native()
		|| token_account.mint() != bit_mint
		|| token_account.owner() != owner
	{
		return Err(BitflipError::InvalidBitTokenAccount.into());
	}

	Ok(token_account.amount())
}

fn assert_flip_custody(
	[
		section,
		bit_mint,
		section_vault,
		player,
		player_bit_account,
		token_program_account,
	]: [&AccountView; 6],
	expected_bit_mint: &Address,
	expected_section_vault: &Address,
	emitted_tokens: u64,
	allocation_tokens: u64,
) -> ProgramResult {
	let token_program = *token_program_account
		.assert_address(&token_2022::ID)?
		.address();
	bit_mint.assert_address(expected_bit_mint)?;
	assert_bit_mint(bit_mint, &token_program)?;
	section_vault.assert_address(expected_section_vault)?;
	let vault_balance = bit_token_account_balance(
		section_vault,
		section.address(),
		bit_mint.address(),
		&token_program,
	)?;
	let _ = bit_recipient_account_balance(
		player_bit_account,
		player.address(),
		bit_mint.address(),
		&token_program,
	)?;

	if vault_balance
		.checked_add(emitted_tokens)
		.ok_or(ProgramError::ArithmeticOverflow)?
		< allocation_tokens
	{
		return Err(BitflipError::InvalidBitTokenAccount.into());
	}

	Ok(())
}

fn transfer_bit_reward(
	[
		section,
		bit_mint,
		section_vault,
		player_bit_account,
		token_program_account,
	]: [&AccountView; 5],
	reward_tokens: u64,
	game_index: u8,
	section_index: u8,
	section_bump: u8,
) -> ProgramResult {
	if reward_tokens == 0 {
		return Ok(());
	}

	token_program_account.assert_address(&token_2022::ID)?;
	let token_program = *token_program_account.address();
	let vault_amount_before = section_vault
		.as_token_account_for_program(&token_program)?
		.amount();
	let recipient_amount_before = player_bit_account
		.as_token_account_for_program(&token_program)?
		.amount();
	let section_seeds = SectionState::seeds(game_index, section_index).with_bump(section_bump);
	let section_signer = section_seeds.to_signer();
	let signers = [section_signer.as_signer()];
	token_2022::instructions::TransferChecked::new(
		section_vault,
		bit_mint,
		player_bit_account,
		section,
		reward_tokens,
		BIT_MINT_DECIMALS,
	)
	.invoke_signed_with_program(&signers, &token_program)?;

	let vault_amount_after = section_vault
		.as_token_account_for_program(&token_program)?
		.amount();
	let recipient_amount_after = player_bit_account
		.as_token_account_for_program(&token_program)?
		.amount();
	let vault_debit = vault_amount_before
		.checked_sub(vault_amount_after)
		.ok_or(BitflipError::InvalidBitTokenAccount)?;
	let recipient_credit = recipient_amount_after
		.checked_sub(recipient_amount_before)
		.ok_or(BitflipError::InvalidBitTokenAccount)?;

	if vault_debit != reward_tokens || recipient_credit != reward_tokens {
		return Err(BitflipError::InvalidBitTokenAccount.into());
	}

	Ok(())
}

fn transfer_section_allocation(
	config: &AccountView,
	bit_mint: &AccountView,
	bit_reserve: &AccountView,
	section_vault: &AccountView,
	token_program_account: &AccountView,
	config_bump: u8,
) -> ProgramResult {
	token_program_account.assert_address(&token_2022::ID)?;
	let token_program = *token_program_account.address();
	let reserve_amount_before = {
		let reserve = bit_reserve.as_token_account_for_program(&token_program)?;
		reserve.amount()
	};
	let destination_amount_before = {
		let destination = section_vault.as_token_account_for_program(&token_program)?;
		destination.amount()
	};
	let config_seeds = ConfigState::seeds().with_bump(config_bump);
	let config_signer = config_seeds.to_signer();
	let signers = [config_signer.as_signer()];
	token_2022::instructions::TransferChecked::new(
		bit_reserve,
		bit_mint,
		section_vault,
		config,
		BIT_SECTION_ALLOCATION_TOKENS,
		BIT_MINT_DECIMALS,
	)
	.invoke_signed_with_program(&signers, &token_program)?;

	let reserve_amount_after = {
		let reserve = bit_reserve.as_token_account_for_program(&token_program)?;
		reserve.amount()
	};
	let destination_amount_after = {
		let destination = section_vault.as_token_account_for_program(&token_program)?;
		destination.amount()
	};
	let reserve_debit = reserve_amount_before
		.checked_sub(reserve_amount_after)
		.ok_or(BitflipError::InvalidBitTokenAccount)?;
	let destination_credit = destination_amount_after
		.checked_sub(destination_amount_before)
		.ok_or(BitflipError::InvalidBitTokenAccount)?;

	if reserve_debit != BIT_SECTION_ALLOCATION_TOKENS
		|| destination_credit != BIT_SECTION_ALLOCATION_TOKENS
	{
		return Err(BitflipError::InvalidBitTokenAccount.into());
	}

	Ok(())
}

impl<'a> ProcessAccountInfos<'a> for InitializeConfigAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = InitializeConfigInstruction::try_from_bytes(data)?;
		let seeds = ConfigState::seeds();

		CreateProgramAccountWithBump {
			account: self.config,
			payer: self.payer,
			owner: &ID,
			seeds: &seeds.as_slices(),
			bump: args.bump,
		}
		.invoke_with::<ConfigState>(|config| {
			config.version = CONFIG_VERSION;
			config.authority = BOOTSTRAP_AUTHORITY;
			config.pending_authority = ZERO_ADDRESS;
			config.treasury = BOOTSTRAP_AUTHORITY;
			config.collection_authority = BOOTSTRAP_AUTHORITY;
			config.bit_mint = ZERO_ADDRESS;
			config.bit_reserve = ZERO_ADDRESS;
			config
				.claim_price_lamports
				.set(DEFAULT_CLAIM_PRICE_LAMPORTS);
			config.flip_fee_lamports.set(DEFAULT_FLIP_FEE_LAMPORTS);
			config
				.minimum_flip_fee_lamports
				.set(DEFAULT_MIN_FLIP_FEE_LAMPORTS);
			config
				.maximum_flip_fee_lamports
				.set(DEFAULT_MAX_FLIP_FEE_LAMPORTS);
			config
				.unlock_interval_seconds
				.set(DEFAULT_UNLOCK_INTERVAL_SECONDS);
			config.early_unlock_flips.set(DEFAULT_EARLY_UNLOCK_FLIPS);
			config.game_count.set(0);
			config.bump = args.bump;

			Ok(())
		})?;

		log!("Bitflip config initialized");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for UpdateConfigAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = UpdateConfigInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;

		{
			let config = self.config.as_account::<ConfigState>(&ID)?;
			self.authority.assert_address(&config.authority)?;
		}

		validate_configuration(
			&args.treasury,
			&args.collection_authority,
			args.flip_fee_lamports.get(),
			args.minimum_flip_fee_lamports.get(),
			args.maximum_flip_fee_lamports.get(),
			args.unlock_interval_seconds.get(),
			args.early_unlock_flips.get(),
		)?;

		let mut config = self.config.as_account_mut::<ConfigState>(&ID)?;
		config.treasury = args.treasury;
		config.collection_authority = args.collection_authority;
		config.claim_price_lamports = args.claim_price_lamports;
		config.flip_fee_lamports = args.flip_fee_lamports;
		config.minimum_flip_fee_lamports = args.minimum_flip_fee_lamports;
		config.maximum_flip_fee_lamports = args.maximum_flip_fee_lamports;
		config.unlock_interval_seconds = args.unlock_interval_seconds;
		config.early_unlock_flips = args.early_unlock_flips;

		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for ProposeAuthorityAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = ProposeAuthorityInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;

		{
			let config = self.config.as_account::<ConfigState>(&ID)?;
			self.authority.assert_address(&config.authority)?;
		}

		if args.pending_authority == ZERO_ADDRESS
			|| args.pending_authority == *self.authority.address()
		{
			return Err(BitflipError::InvalidConfiguration.into());
		}

		self.config
			.as_account_mut::<ConfigState>(&ID)?
			.pending_authority = args.pending_authority;

		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for AcceptAuthorityAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let _ = AcceptAuthorityInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;

		{
			let config = self.config.as_account::<ConfigState>(&ID)?;

			if config.pending_authority == ZERO_ADDRESS {
				return Err(BitflipError::Unauthorized.into());
			}

			self.pending_authority
				.assert_address(&config.pending_authority)?;
		}

		let mut config = self.config.as_account_mut::<ConfigState>(&ID)?;
		config.authority = config.pending_authority;
		config.pending_authority = ZERO_ADDRESS;

		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for InitializeGameAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = InitializeGameInstruction::try_from_bytes(data)?;

		if args.section_index != 0 {
			return Err(BitflipError::InvalidSectionIndex.into());
		}

		if args.game_index >= BIT_GAME_COUNT {
			return Err(BitflipError::InvalidGameIndex.into());
		}

		assert_config_account(self.config)?;

		let price_config = {
			let config = self.config.as_account::<ConfigState>(&ID)?;
			self.payer.assert_address(&config.authority)?;

			if config.game_count.get() != u16::from(args.game_index) {
				return Err(BitflipError::InvalidGameIndex.into());
			}

			initial_game_price_config(&config)
		};

		let clock = Clock::get()?;
		let launched_at = controller_timestamp(clock.unix_timestamp)?;
		let controller = pricing::PriceControllerState::new(&price_config, launched_at)
			.map_err(controller_error)?;
		let game_address = *self.game.address();
		let seeds = GameState::seeds(args.game_index);

		CreateProgramAccountWithBump {
			account: self.game,
			payer: self.payer,
			owner: &ID,
			seeds: &seeds.as_slices(),
			bump: args.game_bump,
		}
		.invoke_with::<GameState>(|game| {
			initialize_game_state(
				game,
				args.game_index,
				args.game_bump,
				clock.unix_timestamp,
				price_config.start_price_lamports,
				price_config,
			);

			Ok(())
		})?;

		let section_seeds = SectionState::seeds(args.game_index, args.section_index);

		CreateProgramAccountWithBump {
			account: self.section,
			payer: self.payer,
			owner: &ID,
			seeds: &section_seeds.as_slices(),
			bump: args.section_bump,
		}
		.invoke_with::<SectionState>(|section| {
			initialize_section_state(
				section,
				game_address,
				args.game_index,
				args.section_index,
				args.section_bump,
				controller,
			);

			Ok(())
		})?;

		let mut config = self.config.as_account_mut::<ConfigState>(&ID)?;
		let game_count = config
			.game_count
			.get()
			.checked_add(1)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		config.game_count.set(game_count);

		log!("Bitflip game initialized");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for ClaimSectionAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = ClaimSectionInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;
		assert_game_account(self.game, args.game_index)?;
		let (claim_price, treasury, interval_seconds, early_unlock_flips) = {
			let config = self.config.as_account::<ConfigState>(&ID)?;
			(
				config.claim_price_lamports.get(),
				config.treasury,
				config.unlock_interval_seconds.get(),
				config.early_unlock_flips.get(),
			)
		};
		self.treasury.assert_address(&treasury)?;

		if claim_price > args.maximum_price_lamports.get() {
			return Err(BitflipError::PriceSlippage.into());
		}

		let (starts_at, next_section, status, price_config) = {
			let game = self.game.as_account::<GameState>(&ID)?;
			(
				game.starts_at.get(),
				game.next_section.get(),
				game.status,
				game_price_config(&game)?,
			)
		};

		if status != GAME_STATUS_LIVE || next_section != u16::from(args.section_index) {
			return Err(BitflipError::GameNotLive.into());
		}

		let clock = Clock::get()?;
		let launched_at = controller_timestamp(clock.unix_timestamp)?;
		let controller = pricing::PriceControllerState::new(&price_config, launched_at)
			.map_err(controller_error)?;
		let unlocked_by_time = clock.unix_timestamp
			>= section_unlock_at(starts_at, args.section_index, interval_seconds)?;
		let unlocked_by_activity = if args.section_index == 0 {
			false
		} else {
			let previous_index = args.section_index - 1;
			assert_section_account(self.previous_section, args.game_index, previous_index)?;
			self.previous_section
				.as_account::<SectionState>(&ID)?
				.flip_count
				.get() >= u64::from(early_unlock_flips)
		};

		if !unlocked_by_time && !unlocked_by_activity {
			return Err(BitflipError::SectionLocked.into());
		}

		let seeds = SectionState::seeds(args.game_index, args.section_index);

		CreateProgramAccountWithBump {
			account: self.section,
			payer: self.owner,
			owner: &ID,
			seeds: &seeds.as_slices(),
			bump: args.bump,
		}
		.invoke_with::<SectionState>(|section| {
			initialize_section_state(
				section,
				*self.owner.address(),
				args.game_index,
				args.section_index,
				args.bump,
				controller,
			);

			Ok(())
		})?;

		transfer_lamports(self.owner, self.treasury, claim_price, self.system_program)?;

		let next_section = next_section
			.checked_add(1)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		let mut game = self.game.as_account_mut::<GameState>(&ID)?;
		game.next_section.set(next_section);

		if next_section == SECTION_COUNT {
			game.status = GAME_STATUS_CLAIMS_COMPLETE;
		}

		log!("Bitflip section claimed");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for FlipPixelsAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = FlipPixelsInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;
		let bit_mint = configured_bit_mint(self.config)?;

		let (starts_at, owner_share_basis_points, price_config) =
			live_game_price_config(self.game)?;
		let clock = Clock::get()?;

		if clock.unix_timestamp < starts_at {
			return Err(BitflipError::GameNotStarted.into());
		}

		let mut section_state = section_flip_state(self.section, args, clock.unix_timestamp)?;
		assert_flip_custody(
			[
				self.section,
				self.bit_mint,
				self.section_vault,
				self.player,
				self.player_bit_account,
				self.token_program,
			],
			&bit_mint,
			&section_state.vault,
			section_state.controller.emitted_tokens,
			price_config.allocation_tokens,
		)?;

		let flip_count = u64::from(args.count);
		let quote = section_state
			.controller
			.execute(
				&price_config,
				controller_timestamp(clock.unix_timestamp)?,
				flip_count,
				pricing::QuoteLimits {
					expected_window_id: args.expected_window_id.get(),
					maximum_unit_price_lamports: args.maximum_unit_price_lamports.get(),
					maximum_total_price_lamports: args.maximum_total_price_lamports.get(),
					minimum_reward_tokens: args.minimum_reward_tokens.get(),
				},
			)
			.map_err(controller_error)?;

		if quote.reward_tokens != flip_count {
			return Err(BitflipError::InsufficientReward.into());
		}

		let fee_split = split_flip_fee(
			&section_state.owner,
			self.game.address(),
			quote.total_price_lamports,
			owner_share_basis_points,
		)?;
		transfer_lamports(
			self.player,
			self.section,
			quote.total_price_lamports,
			self.system_program,
		)?;
		transfer_bit_reward(
			[
				self.section,
				self.bit_mint,
				self.section_vault,
				self.player_bit_account,
				self.token_program,
			],
			quote.reward_tokens,
			args.game_index,
			args.section_index,
			section_state.bump,
		)?;

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		let revision = store_paid_flip(
			&mut section,
			args.count,
			&args.coordinates,
			clock.unix_timestamp,
			fee_split,
			section_state.controller,
		)?;
		drop(section);

		if args.colour != NO_FLIP_COLOUR {
			emit_colour_pixels_flipped(
				*self.player.address(),
				section_state.policy_version,
				revision,
				args,
			)?;
		}

		log!("Bitflip pixels toggled and BIT distributed");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for SealSectionAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = SealSectionInstruction::try_from_bytes(data)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;
		let clock = Clock::get()?;

		{
			let section = self.section.as_account::<SectionState>(&ID)?;
			self.owner.assert_address(&section.owner)?;

			if section.status != SECTION_STATUS_ACTIVE {
				return Err(BitflipError::SectionNotActive.into());
			}

			if section_policy_is_live(&section, clock.unix_timestamp) {
				return Err(BitflipError::SectionPolicyLocked.into());
			}
		}

		pay_accrued_owner_fees(self.section, self.owner, false)?;
		self.section.as_account_mut::<SectionState>(&ID)?.status = SECTION_STATUS_SEALED;

		log!("Bitflip section sealed");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for RecordSectionMintAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = RecordSectionMintInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;

		let config = self.config.as_account::<ConfigState>(&ID)?;
		self.collection_authority
			.assert_address(&config.collection_authority)?;

		if args.expected_owner == ZERO_ADDRESS
			|| args.asset_id == ZERO_ADDRESS
			|| args.merkle_tree == ZERO_ADDRESS
		{
			return Err(BitflipError::InvalidAsset.into());
		}

		{
			let section = self.section.as_account::<SectionState>(&ID)?;

			if section.status == SECTION_STATUS_MINTED {
				return Err(BitflipError::SectionAlreadyMinted.into());
			}

			if section.status != SECTION_STATUS_SEALED {
				return Err(BitflipError::SectionNotSealed.into());
			}

			if section.owner != args.expected_owner {
				return Err(BitflipError::OwnerChanged.into());
			}
		}

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		section.asset_id = args.asset_id;
		section.merkle_tree = args.merkle_tree;
		section.leaf_index = args.leaf_index;
		section.sale_price_lamports.set(0);
		section.status = SECTION_STATUS_MINTED;

		let mut game = self.game.as_account_mut::<GameState>(&ID)?;
		let next_minted_sections = game
			.minted_sections
			.get()
			.checked_add(1)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		game.minted_sections.set(next_minted_sections);

		log!("Bitflip section mint recorded");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for ListSectionAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = ListSectionInstruction::try_from_bytes(data)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;

		{
			let section = self.section.as_account::<SectionState>(&ID)?;
			self.owner.assert_address(&section.owner)?;

			if section.status != SECTION_STATUS_ACTIVE && section.status != SECTION_STATUS_SEALED {
				return Err(BitflipError::SectionNotTransferable.into());
			}
		}

		self.section
			.as_account_mut::<SectionState>(&ID)?
			.sale_price_lamports = args.price_lamports;

		log!("Bitflip section listed");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for CancelSectionListingAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = CancelSectionListingInstruction::try_from_bytes(data)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;

		{
			let section = self.section.as_account::<SectionState>(&ID)?;
			self.owner.assert_address(&section.owner)?;

			if section.sale_price_lamports.get() == 0 {
				return Err(BitflipError::SectionNotForSale.into());
			}
		}

		self.section
			.as_account_mut::<SectionState>(&ID)?
			.sale_price_lamports
			.set(0);

		log!("Bitflip section listing cancelled");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for PurchaseSectionAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = PurchaseSectionInstruction::try_from_bytes(data)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;

		let price = {
			let section = self.section.as_account::<SectionState>(&ID)?;
			self.seller.assert_address(&section.owner)?;

			if self.buyer.address() == self.seller.address() {
				return Err(BitflipError::CannotPurchaseOwnSection.into());
			}

			if section.status != SECTION_STATUS_ACTIVE && section.status != SECTION_STATUS_SEALED {
				return Err(BitflipError::SectionNotTransferable.into());
			}

			let price = section.sale_price_lamports.get();

			if price == 0 {
				return Err(BitflipError::SectionNotForSale.into());
			}

			if price > args.maximum_price_lamports.get() {
				return Err(BitflipError::PriceSlippage.into());
			}

			price
		};

		transfer_lamports(self.buyer, self.seller, price, self.system_program)?;
		pay_accrued_owner_fees(self.section, self.seller, false)?;

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		section.owner = *self.buyer.address();
		section.sale_price_lamports.set(0);

		log!("Bitflip section purchased");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for SettleSectionEconomyAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = SettleSectionEconomyInstruction::try_from_bytes(data)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;

		let price_config = {
			let game = self.game.as_account::<GameState>(&ID)?;
			game_price_config(&game)?
		};
		let mut controller = {
			let section = self.section.as_account::<SectionState>(&ID)?;
			section_controller_state(&section)
		};
		let clock = Clock::get()?;
		controller
			.settle(&price_config, controller_timestamp(clock.unix_timestamp)?)
			.map_err(controller_error)?;
		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		store_section_controller_state(&mut section, controller);

		log!("Bitflip section economy settled");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for ConfigureBitCustodyAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let _ = ConfigureBitCustodyInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;
		let token_program = *self.token_program.address();

		{
			let config = self.config.as_account::<ConfigState>(&ID)?;
			self.authority.assert_address(&config.authority)?;

			if config.bit_mint != ZERO_ADDRESS || config.bit_reserve != ZERO_ADDRESS {
				return Err(BitflipError::CustodyAlreadyConfigured.into());
			}
		}

		let mint_supply = assert_bit_mint(self.bit_mint, &token_program)?;

		if mint_supply != BIT_TOTAL_SUPPLY_TOKENS {
			return Err(BitflipError::InvalidBitMint.into());
		}

		let reserve_balance = bit_token_account_balance(
			self.bit_reserve,
			self.config.address(),
			self.bit_mint.address(),
			&token_program,
		)?;

		if reserve_balance != BIT_TOTAL_SUPPLY_TOKENS {
			return Err(BitflipError::InvalidBitTokenAccount.into());
		}

		let mut config = self.config.as_account_mut::<ConfigState>(&ID)?;
		config.bit_mint = *self.bit_mint.address();
		config.bit_reserve = *self.bit_reserve.address();

		log!("BIT custody configured");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for FundSectionVaultAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = FundSectionVaultInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;
		let token_program = *self.token_program.address();

		let (bit_mint, bit_reserve, config_bump) = {
			let config = self.config.as_account::<ConfigState>(&ID)?;

			if config.bit_mint == ZERO_ADDRESS || config.bit_reserve == ZERO_ADDRESS {
				return Err(BitflipError::CustodyNotConfigured.into());
			}

			(config.bit_mint, config.bit_reserve, config.bump)
		};
		self.bit_mint.assert_address(&bit_mint)?;
		self.bit_reserve.assert_address(&bit_reserve)?;
		{
			let section = self.section.as_account::<SectionState>(&ID)?;

			if section.status != SECTION_STATUS_ACTIVE {
				return Err(BitflipError::SectionNotActive.into());
			}

			if section.bit_vault != ZERO_ADDRESS {
				return Err(BitflipError::SectionVaultAlreadyFunded.into());
			}
		}

		assert_bit_mint(self.bit_mint, &token_program)?;
		let reserve_balance = bit_token_account_balance(
			self.bit_reserve,
			self.config.address(),
			self.bit_mint.address(),
			&token_program,
		)?;

		if reserve_balance < BIT_SECTION_ALLOCATION_TOKENS {
			return Err(BitflipError::InsufficientFunds.into());
		}

		self.section_vault.assert_associated_token_address(
			self.section.address(),
			self.bit_mint.address(),
			&token_program,
		)?;
		associated_token_account::instructions::CreateIdempotent {
			funding_account: self.funder,
			account: self.section_vault,
			wallet: self.section,
			mint: self.bit_mint,
			system_program: self.system_program,
			token_program: self.token_program,
		}
		.invoke()?;
		let _ = bit_token_account_balance(
			self.section_vault,
			self.section.address(),
			self.bit_mint.address(),
			&token_program,
		)?;
		transfer_section_allocation(
			self.config,
			self.bit_mint,
			self.bit_reserve,
			self.section_vault,
			self.token_program,
			config_bump,
		)?;

		self.section.as_account_mut::<SectionState>(&ID)?.bit_vault = *self.section_vault.address();

		log!("BIT section vault funded");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for WithdrawSectionOwnerFeesAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = WithdrawSectionOwnerFeesInstruction::try_from_bytes(data)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;
		pay_accrued_owner_fees(self.section, self.owner, true)?;

		log!("Bitflip section owner fees withdrawn");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for ConfigureSectionPolicyAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = ConfigureSectionPolicyInstruction::try_from_bytes(data)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;
		let clock = Clock::get()?;

		{
			let section = self.section.as_account::<SectionState>(&ID)?;
			self.owner.assert_address(&section.owner)?;

			if section.status != SECTION_STATUS_ACTIVE {
				return Err(BitflipError::SectionNotActive.into());
			}

			assert_section_policy_version(&section, args.expected_policy_version.get())?;

			if section_policy_is_live(&section, clock.unix_timestamp) {
				return Err(BitflipError::SectionPolicyLocked.into());
			}
		}
		validate_section_policy(
			args.mode,
			args.palette_id,
			args.reward_policy,
			args.starts_at.get(),
			args.ends_at.get(),
			args.entry_price_tokens.get(),
			args.reward_per_action_tokens.get(),
			&args.rules_digest,
			clock.unix_timestamp,
		)?;

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		let policy_version = section
			.policy_version
			.get()
			.checked_add(1)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		section.policy_version.set(policy_version);
		section.policy_starts_at = args.starts_at;
		section.policy_ends_at = args.ends_at;
		section.policy_entry_price_tokens = args.entry_price_tokens;
		section.policy_reward_per_action_tokens = args.reward_per_action_tokens;
		section.policy_rules_digest = args.rules_digest;
		section.policy_mode = args.mode;
		section.policy_palette_id = args.palette_id;
		section.policy_reward_policy = args.reward_policy;

		log!("Bitflip section policy configured");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for WithdrawProtocolFeesAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = WithdrawProtocolFeesInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;

		{
			let config = self.config.as_account::<ConfigState>(&ID)?;
			self.authority.assert_address(&config.authority)?;
			self.treasury.assert_address(&config.treasury)?;
		}
		pay_accrued_protocol_fees(self.section, self.treasury)?;

		log!("Bitflip protocol fees withdrawn");
		Ok(())
	}
}

#[cfg(feature = "bpf-entrypoint")]
pub mod entrypoint {
	use super::*;

	nostd_entrypoint!(BitflipInstruction::process_instruction);
}

#[cfg(test)]
mod tests {
	use super::*;

	/// Parse the embedded block the way `query-security-txt` does. A block that
	/// scans but does not parse is worse than none, because explorers and
	/// researchers would show nothing while the source claims disclosure.
	#[test]
	fn security_txt_block_is_well_formed_for_the_standard_parser() {
		let data = SECURITY_TXT_CONTENT.as_bytes();
		assert!(
			data.starts_with(SECURITY_TXT_BEGIN.as_bytes()),
			"block must open with the parser's begin marker"
		);
		let end = data
			.windows(SECURITY_TXT_END.len())
			.position(|window| window == SECURITY_TXT_END.as_bytes())
			.expect("block must contain the end marker");
		let body = &data[SECURITY_TXT_BEGIN.len()..end];

		// The parser alternates NUL-terminated field names and values, and the
		// body ends with a trailing NUL. Dropping it must leave an even number of
		// parts, or some field would be left without a value.
		assert_eq!(
			body.last(),
			Some(&0),
			"the block body must end with a NUL separator"
		);
		let parts = &body[..body.len() - 1];
		assert_eq!(
			parts.split(|byte| *byte == 0).count() % 2,
			0,
			"fields and values must alternate"
		);

		let mut seen_name = false;
		let mut seen_project_url = false;
		let mut seen_policy = false;
		let mut contacts = None;
		let mut expect_value_for: Option<&str> = None;

		for part in parts.split(|byte| *byte == 0) {
			let text = core::str::from_utf8(part).expect("fields and values are UTF-8");

			match expect_value_for.take() {
				Some(field) => match field {
					"name" => seen_name = !text.is_empty(),
					"project_url" => seen_project_url = text.starts_with("https://"),
					"policy" => seen_policy = text.starts_with("https://"),
					"contacts" => contacts = Some(text),
					_ => {}
				},

				None => expect_value_for = Some(text),
			}
		}

		assert!(expect_value_for.is_none(), "every field must have a value");
		assert!(seen_name, "name must be present and non-empty");
		assert!(seen_project_url, "project_url must be an HTTPS URL");
		assert!(seen_policy, "policy must be an HTTPS URL");

		// The contact parser rejects the whole block on an unknown prefix.
		let contacts = contacts.expect("contacts must be present");

		for contact in contacts.split(',') {
			let (kind, value) = contact.split_once(':').expect("contact is typed");
			assert!(
				matches!(
					kind.trim(),
					"email" | "discord" | "telegram" | "twitter" | "link" | "other"
				),
				"unsupported contact type {kind}"
			);
			assert!(!value.trim().is_empty(), "contact value must not be empty");
		}
	}

	#[test]
	fn canvas_geometry_is_exact() {
		assert_eq!(
			usize::from(CANVAS_SIDE) * usize::from(CANVAS_SIDE),
			1_048_576
		);
		assert_eq!(u16::from(SECTION_GRID_SIDE).pow(2), SECTION_COUNT);
		assert_eq!(SECTION_BYTES, 512);
	}

	#[test]
	fn scaled_bit_supply_keeps_zero_decimal_and_allocation_invariants() {
		assert_eq!(BIT_MINT_DECIMALS, 0);
		assert_eq!(BIT_TOTAL_SUPPLY_TOKENS, 25 * 1_073_741_824);
		assert_eq!(BIT_SECTION_ALLOCATION_TOKENS, 100 * 262_144);
		assert_eq!(
			BIT_TOTAL_SUPPLY_TOKENS,
			BIT_GAME_ALLOCATION_TOKENS * u64::from(BIT_GAME_COUNT)
		);
		assert_eq!(
			BIT_GAME_ALLOCATION_TOKENS,
			BIT_SECTION_ALLOCATION_TOKENS * u64::from(SECTION_COUNT)
		);
	}

	#[test]
	fn account_layouts_are_stable() {
		assert_eq!(ConfigState::SIZE, 238);
		assert_eq!(GameState::SIZE, 124);
		assert_eq!(SectionState::SIZE, 855);
	}

	#[test]
	fn instruction_layouts_are_stable() {
		assert_eq!(InitializeConfigInstruction::SIZE, 3);
		assert_eq!(InitializeGameInstruction::SIZE, 6);
		assert_eq!(FlipPixelsInstruction::SIZE, 78);
		assert_eq!(RecordSectionMintInstruction::SIZE, 104);
		assert_eq!(ListSectionInstruction::SIZE, 12);
		assert_eq!(CancelSectionListingInstruction::SIZE, 4);
		assert_eq!(PurchaseSectionInstruction::SIZE, 12);
		assert_eq!(SettleSectionEconomyInstruction::SIZE, 4);
		assert_eq!(ConfigureBitCustodyInstruction::SIZE, 2);
		assert_eq!(FundSectionVaultInstruction::SIZE, 4);
		assert_eq!(WithdrawSectionOwnerFeesInstruction::SIZE, 4);
		assert_eq!(ConfigureSectionPolicyInstruction::SIZE, 79);
		assert_eq!(WithdrawProtocolFeesInstruction::SIZE, 4);
		assert_eq!(ColourPixelsFlippedEvent::SIZE, 86);
	}

	#[test]
	fn colour_event_layout_is_cross_language_stable() {
		let player = Address::new_from_array([9; ADDRESS_BYTES]);
		let mut data = [0; ColourPixelsFlippedEvent::SIZE];
		ColourPixelsFlippedEvent::initialize(&mut data, |event| {
			event.player = player;
			event.policy_version.set(7);
			event.revision.set(42);
			event.coordinates[0..4].copy_from_slice(&[1, 2, 63, 0]);
			event.game_index = 3;
			event.section_index = 255;
			event.count = 2;
			event.colour = 6;

			Ok(())
		})
		.expect("initialize event");

		assert_eq!(data[0], BitflipEvent::ColourPixelsFlipped as u8);
		assert_eq!(data[1], 0, "migration version envelope");
		assert_eq!(&data[2..34], player.as_ref());
		assert_eq!(u64::from_le_bytes(data[34..42].try_into().unwrap()), 7);
		assert_eq!(u64::from_le_bytes(data[42..50].try_into().unwrap()), 42);
		assert_eq!(&data[50..54], &[1, 2, 63, 0]);
		assert_eq!(&data[82..86], &[3, 255, 2, 6]);
	}

	#[test]
	fn section_policies_are_bounded_and_cannot_advertise_unfunded_rewards() {
		let now = 1_000;
		let digest = [1; 32];
		assert_eq!(
			validate_section_policy(
				SECTION_MODE_COLOUR_CANVAS,
				SECTION_PALETTE_DEFAULT,
				SECTION_REWARD_POLICY_NONE,
				now,
				now + 600,
				0,
				0,
				&digest,
				now,
			),
			Ok(())
		);
		assert_eq!(
			validate_section_policy(
				SECTION_MODE_COLOUR_CANVAS,
				SECTION_PALETTE_DEFAULT,
				SECTION_REWARD_POLICY_NONE,
				now,
				now + 600,
				1,
				1,
				&digest,
				now,
			),
			Err(BitflipError::InvalidSectionPolicy.into())
		);
		assert_eq!(
			validate_section_policy(
				SECTION_MODE_COLOUR_CANVAS,
				SECTION_PALETTE_DEFAULT,
				SECTION_REWARD_POLICY_NONE,
				now,
				now + i64::try_from(MAX_SECTION_POLICY_DURATION_SECONDS).unwrap() + 1,
				0,
				0,
				&digest,
				now,
			),
			Err(BitflipError::InvalidSectionPolicy.into())
		);
	}

	#[test]
	fn all_canvas_corners_map_inside_section_storage() {
		assert_eq!(
			pixel_location(0, 0),
			Ok(PixelLocation {
				byte_index: 0,
				mask: 1,
			})
		);
		assert_eq!(
			pixel_location(63, 63),
			Ok(PixelLocation {
				byte_index: 511,
				mask: 128,
			})
		);
		assert_eq!(
			pixel_location(64, 0),
			Err(BitflipError::InvalidCoordinate.into())
		);
		assert_eq!(
			pixel_location(0, 64),
			Err(BitflipError::InvalidCoordinate.into())
		);
	}

	#[test]
	fn toggles_have_no_noop_path() {
		let mut pixels = [0; SECTION_BYTES];
		assert_eq!(toggle_pixel(&mut pixels, 11, 7), Ok(true));
		assert_ne!(pixels, [0; SECTION_BYTES]);
		assert_eq!(toggle_pixel(&mut pixels, 11, 7), Ok(false));
		assert_eq!(pixels, [0; SECTION_BYTES]);
	}

	#[test]
	fn duplicate_coordinates_are_rejected_within_a_paid_batch() {
		let mut coordinates = [0; FLIP_COORDINATE_BYTES];
		coordinates[..4].copy_from_slice(&[4, 9, 4, 9]);
		assert_eq!(
			validate_flip_coordinates(2, &coordinates),
			Err(BitflipError::DuplicateCoordinate.into())
		);
	}

	#[test]
	fn batch_limits_and_coordinates_are_checked() {
		let coordinates = [0; FLIP_COORDINATE_BYTES];
		assert_eq!(
			validate_flip_coordinates(0, &coordinates),
			Err(BitflipError::InvalidFlipCount.into())
		);
		assert_eq!(
			validate_flip_coordinates(17, &coordinates),
			Err(BitflipError::InvalidFlipCount.into())
		);

		let mut invalid = coordinates;
		invalid[0] = SECTION_SIDE;
		assert_eq!(
			validate_flip_coordinates(1, &invalid),
			Err(BitflipError::InvalidCoordinate.into())
		);
	}

	#[test]
	fn section_schedule_is_monotonic() {
		assert_eq!(section_unlock_at(1_000, 0, 60), Ok(1_000));
		assert_eq!(section_unlock_at(1_000, 1, 60), Ok(1_060));
		assert_eq!(section_unlock_at(1_000, 255, 60), Ok(16_300));
	}

	#[test]
	fn invalid_economic_configuration_is_rejected() {
		assert_eq!(
			validate_configuration(
				&BOOTSTRAP_AUTHORITY,
				&ID,
				4_999,
				5_000,
				1_000_000,
				3_600,
				1_024,
			),
			Err(BitflipError::InvalidConfiguration.into())
		);
		assert_eq!(
			validate_configuration(
				&BOOTSTRAP_AUTHORITY,
				&ID,
				10_000,
				5_000,
				1_000_000,
				3_600,
				1_024,
			),
			Ok(())
		);
	}
}
