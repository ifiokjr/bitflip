//! Bitflip's on-chain canvas, implemented with Pina's zero-copy account model.
//!
//! A game is a 16×16 grid of lazily created sections. The program bootstraps
//! one public 64×64 bitmap, then claimants fund each later account as play
//! unlocks it. Players pay a bounded per-flip fee to toggle pixels. Section
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

pub const CONFIG_VERSION: u8 = 3;
pub const GAME_STATUS_LIVE: u8 = 1;
pub const GAME_STATUS_CLAIMS_COMPLETE: u8 = 2;
pub const SECTION_STATUS_ACTIVE: u8 = 1;
pub const SECTION_STATUS_SEALED: u8 = 2;
pub const SECTION_STATUS_MINTED: u8 = 3;

const CONFIG_SEED: &[u8] = b"config";
const GAME_SEED: &[u8] = b"game";
const SECTION_SEED: &[u8] = b"section";
const ZERO_ADDRESS: Address = Address::new_from_array([0; ADDRESS_BYTES]);

#[error]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitflipError {
	Unauthorized = 0,
	InvalidConfiguration = 1,
	InvalidGameIndex = 2,
	GameNotLive = 3,
	GameNotStarted = 4,
	InvalidSectionIndex = 5,
	SectionLocked = 6,
	InvalidFlipCount = 7,
	InvalidCoordinate = 8,
	DuplicateCoordinate = 9,
	PriceSlippage = 10,
	SectionNotActive = 11,
	SectionNotSealed = 12,
	SectionAlreadyMinted = 13,
	InvalidAsset = 14,
	InsufficientFunds = 15,
	InvalidSalePrice = 16,
	SectionNotForSale = 17,
	SectionNotTransferable = 18,
	CannotPurchaseOwnSection = 19,
	OwnerChanged = 20,
	InvalidControllerTimestamp = 21,
	InvalidControllerState = 22,
	CustodyAlreadyConfigured = 23,
	CustodyNotConfigured = 24,
	InvalidBitMint = 25,
	InvalidBitTokenAccount = 26,
	SectionVaultAlreadyFunded = 27,
}

#[discriminator]
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
}

#[discriminator]
pub enum BitflipAccountType {
	ConfigState = 1,
	GameState = 2,
	SectionState = 3,
}

#[account(discriminator = BitflipAccountType)]
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
	pub flip_fee_lamports: u64,
	pub minimum_flip_fee_lamports: u64,
	pub maximum_flip_fee_lamports: u64,
	pub unlock_interval_seconds: u32,
	pub early_unlock_flips: u32,
	pub game_count: u16,
	pub bump: u8,
}

#[account(discriminator = BitflipAccountType)]
#[pda(seeds = [GAME_SEED, game_index: u8], bump = bump)]
pub struct GameState {
	pub game_index: u8,
	pub status: u8,
	pub bump: u8,
	pub economy_version: u8,
	pub starts_at: i64,
	pub next_section: u16,
	pub minted_sections: u16,
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

#[account(discriminator = BitflipAccountType)]
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
	pub pixels: [u8; 512],
}

#[instruction(discriminator = BitflipInstruction::InitializeConfig)]
pub struct InitializeConfigInstruction {
	pub bump: u8,
}

#[instruction(discriminator = BitflipInstruction::UpdateConfig)]
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

#[instruction(discriminator = BitflipInstruction::ProposeAuthority)]
pub struct ProposeAuthorityInstruction {
	pub pending_authority: Address,
}

#[instruction(discriminator = BitflipInstruction::AcceptAuthority)]
pub struct AcceptAuthorityInstruction {}

#[instruction(discriminator = BitflipInstruction::InitializeGame)]
pub struct InitializeGameInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub game_bump: u8,
	pub section_bump: u8,
}

#[instruction(discriminator = BitflipInstruction::ClaimSection)]
pub struct ClaimSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub bump: u8,
	pub maximum_price_lamports: u64,
}

#[instruction(discriminator = BitflipInstruction::FlipPixels)]
pub struct FlipPixelsInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub count: u8,
	pub coordinates: [u8; 32],
	pub maximum_total_fee_lamports: u64,
}

#[instruction(discriminator = BitflipInstruction::SealSection)]
pub struct SealSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::RecordSectionMint)]
pub struct RecordSectionMintInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub expected_owner: Address,
	pub asset_id: Address,
	pub merkle_tree: Address,
	pub leaf_index: u32,
}

#[instruction(discriminator = BitflipInstruction::ListSection)]
pub struct ListSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub price_lamports: u64,
}

#[instruction(discriminator = BitflipInstruction::CancelSectionListing)]
pub struct CancelSectionListingInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::PurchaseSection)]
pub struct PurchaseSectionInstruction {
	pub game_index: u8,
	pub section_index: u8,
	pub maximum_price_lamports: u64,
}

#[instruction(discriminator = BitflipInstruction::SettleSectionEconomy)]
pub struct SettleSectionEconomyInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[instruction(discriminator = BitflipInstruction::ConfigureBitCustody)]
pub struct ConfigureBitCustodyInstruction {}

#[instruction(discriminator = BitflipInstruction::FundSectionVault)]
pub struct FundSectionVaultInstruction {
	pub game_index: u8,
	pub section_index: u8,
}

#[derive(Accounts, Debug)]
pub struct InitializeConfigAccounts<'a> {
	pub payer: &'a mut AccountView,
	pub config: &'a mut AccountView,
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct UpdateConfigAccounts<'a> {
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct ProposeAuthorityAccounts<'a> {
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct AcceptAuthorityAccounts<'a> {
	pub pending_authority: &'a AccountView,
	pub config: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct InitializeGameAccounts<'a> {
	pub payer: &'a mut AccountView,
	pub config: &'a mut AccountView,
	pub game: &'a mut AccountView,
	pub section: &'a mut AccountView,
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct ClaimSectionAccounts<'a> {
	pub owner: &'a mut AccountView,
	pub config: &'a AccountView,
	pub game: &'a mut AccountView,
	pub previous_section: &'a AccountView,
	pub section: &'a mut AccountView,
	pub treasury: &'a mut AccountView,
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct FlipPixelsAccounts<'a> {
	pub player: &'a mut AccountView,
	pub config: &'a AccountView,
	pub game: &'a mut AccountView,
	pub section: &'a mut AccountView,
	pub treasury: &'a mut AccountView,
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct SealSectionAccounts<'a> {
	pub owner: &'a AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct RecordSectionMintAccounts<'a> {
	pub collection_authority: &'a AccountView,
	pub config: &'a AccountView,
	pub game: &'a mut AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct ListSectionAccounts<'a> {
	pub owner: &'a AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct CancelSectionListingAccounts<'a> {
	pub owner: &'a AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct PurchaseSectionAccounts<'a> {
	pub buyer: &'a mut AccountView,
	pub seller: &'a mut AccountView,
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
	pub system_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct SettleSectionEconomyAccounts<'a> {
	pub game: &'a AccountView,
	pub section: &'a mut AccountView,
}

#[derive(Accounts, Debug)]
pub struct ConfigureBitCustodyAccounts<'a> {
	pub authority: &'a AccountView,
	pub config: &'a mut AccountView,
	pub bit_mint: &'a AccountView,
	pub bit_reserve: &'a AccountView,
	pub token_program: &'a AccountView,
}

#[derive(Accounts, Debug)]
pub struct FundSectionVaultAccounts<'a> {
	pub funder: &'a mut AccountView,
	pub config: &'a AccountView,
	pub section: &'a mut AccountView,
	pub bit_mint: &'a AccountView,
	pub bit_reserve: &'a mut AccountView,
	pub section_vault: &'a mut AccountView,
	pub associated_token_program: &'a AccountView,
	pub token_program: &'a AccountView,
	pub system_program: &'a AccountView,
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

	if !addresses_are_valid || !fees_are_valid || !progression_is_valid {
		return Err(BitflipError::InvalidConfiguration.into());
	}

	Ok(())
}

fn assert_config_account(config: &AccountView) -> ProgramResult {
	config.assert_not_empty()?.assert_type::<ConfigState>(&ID)?;
	if config.as_account::<ConfigState>(&ID)?.version != CONFIG_VERSION {
		return Err(BitflipError::InvalidConfiguration.into());
	}
	let seeds = ConfigState::seeds();
	config.assert_seeds_with_bump(
		&seeds
			.with_bump(config.as_account::<ConfigState>(&ID)?.bump)
			.as_slices(),
		&ID,
	)?;

	Ok(())
}

fn assert_game_account(game: &AccountView, game_index: u8) -> ProgramResult {
	game.assert_not_empty()?.assert_type::<GameState>(&ID)?;
	let state = game.as_account::<GameState>(&ID)?;
	if state.game_index != game_index {
		return Err(BitflipError::InvalidGameIndex.into());
	}
	if state.economy_version != ECONOMY_VERSION {
		return Err(BitflipError::InvalidConfiguration.into());
	}
	GameState::assert_seeds(game, game_index, &ID)
}

fn assert_section_account(
	section: &AccountView,
	game_index: u8,
	section_index: u8,
) -> ProgramResult {
	section
		.assert_not_empty()?
		.assert_type::<SectionState>(&ID)?;
	let state = section.as_account::<SectionState>(&ID)?;
	if state.game_index != game_index || state.section_index != section_index {
		return Err(BitflipError::InvalidSectionIndex.into());
	}
	SectionState::assert_seeds(section, game_index, section_index, &ID)
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
		pricing::PriceControllerError::InvalidRequest
		| pricing::PriceControllerError::StaleWindow
		| pricing::PriceControllerError::PriceSlippage
		| pricing::PriceControllerError::InsufficientReward => {
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

fn assert_bit_mint(bit_mint: &AccountView, token_program: &Address) -> ProgramResult {
	let mint = bit_mint
		.as_token_mint_for_program(token_program)
		.and_then(token::TokenMintRef::assert_no_extensions)
		.map_err(|_| ProgramError::from(BitflipError::InvalidBitMint))?;

	if !mint.is_initialized()
		|| mint.decimals() != BIT_MINT_DECIMALS
		|| mint.supply() != BIT_TOTAL_SUPPLY_TOKENS
		|| mint.mint_authority().is_some()
		|| mint.freeze_authority().is_some()
	{
		return Err(BitflipError::InvalidBitMint.into());
	}

	Ok(())
}

fn bit_token_account_balance(
	account: &AccountView,
	owner: &Address,
	bit_mint: &Address,
	token_program: &Address,
) -> Result<u64, ProgramError> {
	let token_account = account
		.as_associated_token_account_checked(owner, bit_mint, token_program)
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
		let seeds_with_bump = seeds.with_bump(args.bump);

		self.payer
			.assert_signer()?
			.assert_writable()?
			.assert_owner(&system::ID)?;
		self.system_program.assert_address(&system::ID)?;
		let canonical_bump = self.config.assert_canonical_bump(&seeds.as_slices(), &ID)?;
		if canonical_bump != args.bump {
			return Err(ProgramError::InvalidSeeds);
		}
		self.config
			.assert_empty()?
			.assert_writable()?
			.assert_seeds_with_bump(&seeds_with_bump.as_slices(), &ID)?;

		CreateProgramAccountWithBump {
			account: self.config,
			payer: self.payer,
			owner: &ID,
			seeds: &seeds.as_slices(),
			bump: args.bump,
		}
		.invoke::<ConfigState>()?;

		let mut config = self.config.as_account_mut::<ConfigState>(&ID)?;
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

		log!("Bitflip config initialized");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for UpdateConfigAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = UpdateConfigInstruction::try_from_bytes(data)?;
		assert_config_account(self.config)?;
		self.authority.assert_signer()?;

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
		self.authority.assert_signer()?;

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
		self.pending_authority.assert_signer()?;

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
		self.payer
			.assert_signer()?
			.assert_writable()?
			.assert_owner(&system::ID)?;
		self.system_program.assert_address(&system::ID)?;

		let flip_fee_lamports = {
			let config = self.config.as_account::<ConfigState>(&ID)?;
			self.payer.assert_address(&config.authority)?;
			if config.game_count.get() != u16::from(args.game_index) {
				return Err(BitflipError::InvalidGameIndex.into());
			}
			config.flip_fee_lamports.get()
		};

		let seeds = GameState::seeds(args.game_index);
		let seeds_with_bump = seeds.with_bump(args.game_bump);
		let canonical_bump = self.game.assert_canonical_bump(&seeds.as_slices(), &ID)?;
		if canonical_bump != args.game_bump {
			return Err(ProgramError::InvalidSeeds);
		}
		self.game
			.assert_empty()?
			.assert_writable()?
			.assert_seeds_with_bump(&seeds_with_bump.as_slices(), &ID)?;

		CreateProgramAccountWithBump {
			account: self.game,
			payer: self.payer,
			owner: &ID,
			seeds: &seeds.as_slices(),
			bump: args.game_bump,
		}
		.invoke::<GameState>()?;

		let section_seeds = SectionState::seeds(args.game_index, args.section_index);
		let section_seeds_with_bump = section_seeds.with_bump(args.section_bump);
		let canonical_section_bump = self
			.section
			.assert_canonical_bump(&section_seeds.as_slices(), &ID)?;
		if canonical_section_bump != args.section_bump {
			return Err(ProgramError::InvalidSeeds);
		}
		self.section
			.assert_empty()?
			.assert_writable()?
			.assert_seeds_with_bump(&section_seeds_with_bump.as_slices(), &ID)?;

		CreateProgramAccountWithBump {
			account: self.section,
			payer: self.payer,
			owner: &ID,
			seeds: &section_seeds.as_slices(),
			bump: args.section_bump,
		}
		.invoke::<SectionState>()?;

		let clock = Clock::get()?;
		let launched_at = controller_timestamp(clock.unix_timestamp)?;
		let price_config = pricing::PriceControllerConfig::STAGING;
		let controller = pricing::PriceControllerState::new(&price_config, launched_at)
			.map_err(controller_error)?;
		let game_address = *self.game.address();
		let mut game = self.game.as_account_mut::<GameState>(&ID)?;
		initialize_game_state(
			&mut game,
			args.game_index,
			args.game_bump,
			clock.unix_timestamp,
			flip_fee_lamports,
			price_config,
		);

		let mut initial_section = self.section.as_account_mut::<SectionState>(&ID)?;
		initialize_section_state(
			&mut initial_section,
			game_address,
			args.game_index,
			args.section_index,
			args.section_bump,
			controller,
		);

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
		self.owner
			.assert_signer()?
			.assert_writable()?
			.assert_owner(&system::ID)?;
		self.system_program.assert_address(&system::ID)?;

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
		let seeds_with_bump = seeds.with_bump(args.bump);
		let canonical_bump = self
			.section
			.assert_canonical_bump(&seeds.as_slices(), &ID)?;
		if canonical_bump != args.bump {
			return Err(ProgramError::InvalidSeeds);
		}
		self.section
			.assert_empty()?
			.assert_writable()?
			.assert_seeds_with_bump(&seeds_with_bump.as_slices(), &ID)?;

		CreateProgramAccountWithBump {
			account: self.section,
			payer: self.owner,
			owner: &ID,
			seeds: &seeds.as_slices(),
			bump: args.bump,
		}
		.invoke::<SectionState>()?;

		transfer_lamports(self.owner, self.treasury, claim_price, self.system_program)?;

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		initialize_section_state(
			&mut section,
			*self.owner.address(),
			args.game_index,
			args.section_index,
			args.bump,
			controller,
		);

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
		validate_flip_coordinates(args.count, &args.coordinates)?;
		assert_config_account(self.config)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;
		self.player
			.assert_signer()?
			.assert_writable()?
			.assert_owner(&system::ID)?;
		self.system_program.assert_address(&system::ID)?;

		let treasury = self.config.as_account::<ConfigState>(&ID)?.treasury;
		self.treasury.assert_address(&treasury)?;

		let (game_status, starts_at, fee_per_flip) = {
			let game = self.game.as_account::<GameState>(&ID)?;
			(
				game.status,
				game.starts_at.get(),
				game.flip_fee_lamports.get(),
			)
		};
		if game_status != GAME_STATUS_LIVE && game_status != GAME_STATUS_CLAIMS_COMPLETE {
			return Err(BitflipError::GameNotLive.into());
		}
		let clock = Clock::get()?;
		if clock.unix_timestamp < starts_at {
			return Err(BitflipError::GameNotStarted.into());
		}
		if self.section.as_account::<SectionState>(&ID)?.status != SECTION_STATUS_ACTIVE {
			return Err(BitflipError::SectionNotActive.into());
		}

		let flip_count = u64::from(args.count);
		let total_fee = fee_per_flip
			.checked_mul(flip_count)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		if total_fee > args.maximum_total_fee_lamports.get() {
			return Err(BitflipError::PriceSlippage.into());
		}
		transfer_lamports(self.player, self.treasury, total_fee, self.system_program)?;

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		let mut on_pixels = section.on_pixels.get();
		for index in 0..usize::from(args.count) {
			let offset = index * 2;
			let turned_on = toggle_pixel(
				&mut section.pixels,
				args.coordinates[offset],
				args.coordinates[offset + 1],
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
		let next_flip_count = section
			.flip_count
			.get()
			.checked_add(flip_count)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		let next_revision = section
			.revision
			.get()
			.checked_add(1)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		section.flip_count.set(next_flip_count);
		section.revision.set(next_revision);
		section.last_flip_at.set(clock.unix_timestamp);

		let mut game = self.game.as_account_mut::<GameState>(&ID)?;
		let next_total_flips = game
			.total_flips
			.get()
			.checked_add(flip_count)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		game.total_flips.set(next_total_flips);

		log!("Bitflip pixels toggled");
		Ok(())
	}
}

impl<'a> ProcessAccountInfos<'a> for SealSectionAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = SealSectionInstruction::try_from_bytes(data)?;
		assert_game_account(self.game, args.game_index)?;
		assert_section_account(self.section, args.game_index, args.section_index)?;
		self.owner.assert_signer()?;

		{
			let section = self.section.as_account::<SectionState>(&ID)?;
			self.owner.assert_address(&section.owner)?;
			if section.status != SECTION_STATUS_ACTIVE {
				return Err(BitflipError::SectionNotActive.into());
			}
		}

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
		self.collection_authority.assert_signer()?;

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
		self.owner.assert_signer()?;
		if args.price_lamports.get() == 0 {
			return Err(BitflipError::InvalidSalePrice.into());
		}

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
		self.owner.assert_signer()?;

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
		self.buyer
			.assert_signer()?
			.assert_writable()?
			.assert_owner(&system::ID)?;
		self.seller.assert_writable()?.assert_owner(&system::ID)?;
		self.system_program.assert_address(&system::ID)?;

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
		self.authority.assert_signer()?;
		let token_program = *self
			.token_program
			.assert_address(&token_2022::ID)?
			.address();

		{
			let config = self.config.as_account::<ConfigState>(&ID)?;
			self.authority.assert_address(&config.authority)?;
			if config.bit_mint != ZERO_ADDRESS || config.bit_reserve != ZERO_ADDRESS {
				return Err(BitflipError::CustodyAlreadyConfigured.into());
			}
		}

		assert_bit_mint(self.bit_mint, &token_program)?;
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
		self.funder
			.assert_signer()?
			.assert_writable()?
			.assert_owner(&system::ID)?;
		self.system_program.assert_address(&system::ID)?;
		self.associated_token_program
			.assert_address(&associated_token_account::ID)?;
		let token_program = *self
			.token_program
			.assert_address(&token_2022::ID)?
			.address();

		let (bit_mint, bit_reserve, config_bump) = {
			let config = self.config.as_account::<ConfigState>(&ID)?;
			if config.bit_mint == ZERO_ADDRESS || config.bit_reserve == ZERO_ADDRESS {
				return Err(BitflipError::CustodyNotConfigured.into());
			}
			(config.bit_mint, config.bit_reserve, config.bump)
		};
		self.bit_mint.assert_address(&bit_mint)?;
		self.bit_reserve.assert_address(&bit_reserve)?;
		if self.section.as_account::<SectionState>(&ID)?.bit_vault != ZERO_ADDRESS {
			return Err(BitflipError::SectionVaultAlreadyFunded.into());
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

#[cfg(feature = "bpf-entrypoint")]
pub mod entrypoint {
	use super::*;

	nostd_entrypoint!(process_instruction);

	#[inline(always)]
	pub fn process_instruction(
		program_id: &Address,
		accounts: &mut [AccountView],
		data: &[u8],
	) -> ProgramResult {
		let instruction: BitflipInstruction = parse_instruction(program_id, &ID, data)?;

		match instruction {
			BitflipInstruction::InitializeConfig => {
				InitializeConfigAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::UpdateConfig => {
				UpdateConfigAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::ProposeAuthority => {
				ProposeAuthorityAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::AcceptAuthority => {
				AcceptAuthorityAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::InitializeGame => {
				InitializeGameAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::ClaimSection => {
				ClaimSectionAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::FlipPixels => {
				FlipPixelsAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::SealSection => {
				SealSectionAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::RecordSectionMint => {
				RecordSectionMintAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::ListSection => {
				ListSectionAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::CancelSectionListing => {
				CancelSectionListingAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::PurchaseSection => {
				PurchaseSectionAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::SettleSectionEconomy => {
				SettleSectionEconomyAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::ConfigureBitCustody => {
				ConfigureBitCustodyAccounts::try_from((program_id, accounts))?.process(data)
			}
			BitflipInstruction::FundSectionVault => {
				FundSectionVaultAccounts::try_from((program_id, accounts))?.process(data)
			}
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

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
		assert_eq!(ConfigState::SIZE, 237);
		assert_eq!(GameState::SIZE, 123);
		assert_eq!(SectionState::SIZE, 763);
	}

	#[test]
	fn instruction_layouts_are_stable() {
		assert_eq!(InitializeConfigInstruction::SIZE, 2);
		assert_eq!(InitializeGameInstruction::SIZE, 5);
		assert_eq!(FlipPixelsInstruction::SIZE, 44);
		assert_eq!(RecordSectionMintInstruction::SIZE, 103);
		assert_eq!(ListSectionInstruction::SIZE, 11);
		assert_eq!(CancelSectionListingInstruction::SIZE, 3);
		assert_eq!(PurchaseSectionInstruction::SIZE, 11);
		assert_eq!(SettleSectionEconomyInstruction::SIZE, 3);
		assert_eq!(ConfigureBitCustodyInstruction::SIZE, 1);
		assert_eq!(FundSectionVaultInstruction::SIZE, 3);
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
