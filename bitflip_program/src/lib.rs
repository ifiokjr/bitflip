//! Bitflip's on-chain canvas, implemented with Pina's zero-copy account model.
//!
//! A game is a 16×16 grid of independently claimed sections. Each section is a
//! 64×64 bitmap, so the complete canvas remains 1024×1024 pixels. Players pay
//! a bounded per-flip fee to toggle pixels. Section owners can freeze their
//! finished art and the configured collection authority can attest the
//! compressed NFT created for that frozen section.

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

pub const DEFAULT_CLAIM_PRICE_LAMPORTS: u64 = 10_000_000;
pub const DEFAULT_FLIP_FEE_LAMPORTS: u64 = 10_000;
pub const DEFAULT_MIN_FLIP_FEE_LAMPORTS: u64 = 5_000;
pub const DEFAULT_MAX_FLIP_FEE_LAMPORTS: u64 = 1_000_000;
pub const DEFAULT_UNLOCK_INTERVAL_SECONDS: u32 = 3_600;
pub const DEFAULT_EARLY_UNLOCK_FLIPS: u32 = 1_024;

pub const CONFIG_VERSION: u8 = 1;
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
	pub starts_at: i64,
	pub next_section: u16,
	pub minted_sections: u16,
	pub flip_fee_lamports: u64,
	pub total_flips: u64,
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
	pub game_index: u8,
	pub section_index: u8,
	pub status: u8,
	pub bump: u8,
	pub on_pixels: u16,
	pub leaf_index: u32,
	pub flip_count: u64,
	pub revision: u64,
	pub last_flip_at: i64,
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
	pub bump: u8,
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
	pub asset_id: Address,
	pub merkle_tree: Address,
	pub leaf_index: u32,
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
		let seeds_with_bump = seeds.with_bump(args.bump);
		let canonical_bump = self.game.assert_canonical_bump(&seeds.as_slices(), &ID)?;
		if canonical_bump != args.bump {
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
			bump: args.bump,
		}
		.invoke::<GameState>()?;

		let clock = Clock::get()?;
		let mut game = self.game.as_account_mut::<GameState>(&ID)?;
		game.game_index = args.game_index;
		game.status = GAME_STATUS_LIVE;
		game.bump = args.bump;
		game.starts_at.set(clock.unix_timestamp);
		game.next_section.set(0);
		game.minted_sections.set(0);
		game.flip_fee_lamports.set(flip_fee_lamports);
		game.total_flips.set(0);

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

		let (starts_at, next_section, status) = {
			let game = self.game.as_account::<GameState>(&ID)?;
			(game.starts_at.get(), game.next_section.get(), game.status)
		};
		if status != GAME_STATUS_LIVE || next_section != u16::from(args.section_index) {
			return Err(BitflipError::GameNotLive.into());
		}

		let clock = Clock::get()?;
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

		if claim_price > 0 {
			if self.owner.lamports() < claim_price {
				return Err(BitflipError::InsufficientFunds.into());
			}
			system::instructions::Transfer {
				from: self.owner,
				to: self.treasury,
				lamports: claim_price,
			}
			.invoke()?;
		}

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		section.owner = *self.owner.address();
		section.asset_id = ZERO_ADDRESS;
		section.merkle_tree = ZERO_ADDRESS;
		section.game_index = args.game_index;
		section.section_index = args.section_index;
		section.status = SECTION_STATUS_ACTIVE;
		section.bump = args.bump;
		section.on_pixels.set(0);
		section.leaf_index.set(0);
		section.flip_count.set(0);
		section.revision.set(0);
		section.last_flip_at.set(0);
		section.pixels.fill(0);

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
		if self.player.lamports() < total_fee {
			return Err(BitflipError::InsufficientFunds.into());
		}

		system::instructions::Transfer {
			from: self.player,
			to: self.treasury,
			lamports: total_fee,
		}
		.invoke()?;

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
		if args.asset_id == ZERO_ADDRESS || args.merkle_tree == ZERO_ADDRESS {
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
		}

		let mut section = self.section.as_account_mut::<SectionState>(&ID)?;
		section.asset_id = args.asset_id;
		section.merkle_tree = args.merkle_tree;
		section.leaf_index = args.leaf_index;
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
	fn account_layouts_are_stable() {
		assert_eq!(ConfigState::SIZE, 173);
		assert_eq!(GameState::SIZE, 32);
		assert_eq!(SectionState::SIZE, 643);
	}

	#[test]
	fn instruction_layouts_are_stable() {
		assert_eq!(InitializeConfigInstruction::SIZE, 2);
		assert_eq!(FlipPixelsInstruction::SIZE, 44);
		assert_eq!(RecordSectionMintInstruction::SIZE, 71);
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
