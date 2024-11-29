use fixed::types::U64F64;
use solana_program::msg;
use spl_pod::primitives::PodI64;
use spl_pod::primitives::PodU16;
use spl_pod::primitives::PodU32;
use spl_pod::primitives::PodU64;
use static_assertions::const_assert;
use steel::*;

use crate::FlipBit;
use crate::BASE_LAMPORTS_PER_BIT;
use crate::BITFLIP_SECTION_LENGTH;
use crate::BITFLIP_SECTION_TOTAL_BITS;
use crate::EARNED_TOKENS_PER_SECTION;
use crate::MAX_LAMPORTS_PER_BIT;
use crate::MIN_LAMPORTS_PER_BIT;
use crate::SESSION_DURATION;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum BitflipAccount {
	ConfigState = 0,
	GameState = 1,
	SectionState = 2,
}

const_assert!(ConfigState::space() == 80);
const_assert!(GameState::space() == 149);
const_assert!(SectionState::space() == 600);

account!(BitflipAccount, ConfigState);
account!(BitflipAccount, GameState);
account!(BitflipAccount, SectionState);

pub trait AccountVersion: Pod {
	/// The latest version of the account which should be used as the migration
	/// target and the default value for creating new accounts.
	const VERSION: u8;

	/// Migrate the account to the latest version.
	fn migrate(&mut self) -> Result<(), ProgramError>;
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "client", derive(typed_builder::TypedBuilder))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ConfigState {
	/// The version of the state.
	#[cfg_attr(feature = "client", builder(default = ConfigState::VERSION))]
	pub version: u8,
	/// The authority which can update this config.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	pub authority: Pubkey,
	/// Store the bump to save compute units.
	pub bump: u8,
	/// The treasury account bump where fees are sent and where the minted
	/// tokens are transferred.
	pub treasury_bump: u8,
	/// The mint account bump.
	pub mint_bit_bump: u8,
	/// The mint account bump for KIBIBIT.
	pub mint_kibibit_bump: u8,
	/// The mint account bump for MEBIBIT.
	pub mint_mebibit_bump: u8,
	/// The mint account bump for GIBIBIT.
	pub mint_gibibit_bump: u8,
	/// There will be a maximum of 8 games.
	pub game_index: u8,
	/// Extra space for future use.
	#[cfg_attr(feature = "client", builder(default))]
	pub _padding: [u8; 32],
}

impl AccountVersion for ConfigState {
	const VERSION: u8 = 0;

	fn migrate(&mut self) -> Result<(), ProgramError> {
		Ok(())
	}
}

impl ConfigState {
	pub fn new(
		authority: Pubkey,
		bump: u8,
		treasury_bump: u8,
		mint_bit_bump: u8,
		mint_kibibit_bump: u8,
		mint_mebibit_bump: u8,
		mint_gibibit_bump: u8,
	) -> ConfigState {
		ConfigState {
			version: Self::VERSION,
			authority,
			bump,
			treasury_bump,
			mint_bit_bump,
			mint_kibibit_bump,
			mint_mebibit_bump,
			mint_gibibit_bump,
			game_index: 0,
			_padding: [0; 32],
		}
	}
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum GameStatus {
	#[default]
	Pending = 0,
	Running = 1,
	Ended = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "client", derive(typed_builder::TypedBuilder))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct GameState {
	/// The version of the state.
	#[cfg_attr(feature = "client", builder(default = GameState::VERSION))]
	pub version: u8,
	/// This is a permanent signer created and maintained by the backend. It
	/// needs to be provided to update the access signer.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	#[cfg_attr(feature = "client", builder(default))]
	pub funded_signer: Pubkey,
	/// This is a temporary signer created and maintained by the backend. Which
	/// is allowed to sign certain transactions and expires after a fixed amount
	/// of time.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	#[cfg_attr(feature = "client", builder(default))]
	pub temp_signer: Pubkey,
	/// The start time. If 0 then it hasn't started yet. Using an `Option` here
	/// would waste extra bytes.
	#[cfg_attr(feature = "client", builder(default, setter(into)))]
	pub start_time: PodI64,
	/// The duration of the game.
	#[cfg_attr(feature = "client", builder(default = SESSION_DURATION.into(), setter(into)))]
	pub duration: PodI64,
	/// The minimum amount of lamports that the price of flipping a bit should
	/// be.
	#[cfg_attr(feature = "client", builder(default = MIN_LAMPORTS_PER_BIT.into(), setter(into)))]
	pub min_lamports: PodU64,
	/// The initial price of flipping a bit.
	#[cfg_attr(feature = "client", builder(default = BASE_LAMPORTS_PER_BIT.into(), setter(into)))]
	pub base_lamports: PodU64,
	/// The maximum price of flipping a bit.
	#[cfg_attr(feature = "client", builder(default = MAX_LAMPORTS_PER_BIT.into(), setter(into)))]
	pub max_lamports: PodU64,
	/// The status of the game.
	#[cfg_attr(feature = "client", builder(default = GameStatus::Pending.into()))]
	pub status: u8,
	/// The index of this currently played game.
	pub game_index: u8,
	/// The most recent section which was unlocked. This will be updated every
	/// time a new section is initialized.
	pub section_index: u8,
	/// The bump for this account.
	pub bump: u8,
	/// Extra space for future use.
	#[cfg_attr(feature = "client", builder(default))]
	pub _padding: [u8; 32],
}

impl AccountVersion for GameState {
	const VERSION: u8 = 0;

	fn migrate(&mut self) -> Result<(), ProgramError> {
		Ok(())
	}
}

impl GameState {
	pub fn new(temp_signer: Pubkey, funded_signer: Pubkey, index: u8, bump: u8) -> Self {
		Self {
			version: GameState::VERSION,
			funded_signer,
			temp_signer,
			start_time: 0.into(),
			duration: SESSION_DURATION.into(),
			min_lamports: MIN_LAMPORTS_PER_BIT.into(),
			base_lamports: BASE_LAMPORTS_PER_BIT.into(),
			max_lamports: MAX_LAMPORTS_PER_BIT.into(),
			status: GameStatus::Pending.into(),
			section_index: 0,
			game_index: index,
			bump,
			_padding: [0; 32],
		}
	}

	#[inline(always)]
	pub fn start_time(&self) -> i64 {
		self.start_time.into()
	}

	#[inline(always)]
	pub fn duration(&self) -> i64 {
		self.duration.into()
	}

	#[inline(always)]
	pub fn min_lamports(&self) -> u64 {
		self.min_lamports.into()
	}

	#[inline(always)]
	pub fn base_lamports(&self) -> u64 {
		self.base_lamports.into()
	}

	#[inline(always)]
	pub fn max_lamports(&self) -> u64 {
		self.max_lamports.into()
	}

	#[inline(always)]
	pub fn status(&self) -> GameStatus {
		GameStatus::try_from(self.status).unwrap()
	}

	/// The end time of the game.
	#[inline(always)]
	pub fn end_time(&self) -> i64 {
		self.start_time().saturating_add(self.duration())
	}

	/// The remaining time of the game.
	pub fn remaining_time(&self, current_time: i64) -> i64 {
		self.end_time().saturating_sub(current_time)
	}

	/// Whether the game is running.
	pub fn running(&self, current_time: i64) -> bool {
		self.start_time() > 0 && self.status() == GameStatus::Running && !self.ended(current_time)
	}

	/// Whether the game has ended.
	pub fn ended(&self, current_time: i64) -> bool {
		current_time > self.end_time() || self.status() == GameStatus::Ended
	}

	pub fn start(&mut self, current_time: i64) {
		self.status = GameStatus::Running.into();
		self.start_time = current_time.into();
	}

	/// Increment the section index, without overflowing.
	pub fn increment_section(&mut self) {
		if let Some(next_index) = self.section_index.checked_add(1) {
			self.section_index = next_index;
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SectionState {
	/// The version of the state.
	pub version: u8,
	/// The state of the bits that are represented as flippable bits on the
	/// frontend.
	#[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
	pub data: [PodU16; BITFLIP_SECTION_LENGTH],
	/// The owner of this section.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	pub owner: Pubkey,
	/// The number of bit flips that have occurred.
	pub flips: PodU32,
	/// The number of bits that are on.
	pub on: PodU32,
	/// The number of bits that are off.
	pub off: PodU32,
	/// The index for this game this section is a part of.
	pub game_index: u8,
	/// The index for this section state.
	pub section_index: u8,
	/// The bump for this section state.
	pub bump: u8,
	/// Extra space for future versions.
	pub _padding: [u8; 32],
}

impl AccountVersion for SectionState {
	const VERSION: u8 = 0;

	fn migrate(&mut self) -> Result<(), ProgramError> {
		Ok(())
	}
}

impl SectionState {
	/// Create a new section state in the client. Useful for testing.
	pub fn new(owner: Pubkey, game_index: u8, section_index: u8, bump: u8) -> Self {
		Self {
			version: SectionState::VERSION,
			data: [0.into(); BITFLIP_SECTION_LENGTH],
			owner,
			flips: 0.into(),
			on: 0.into(),
			off: BITFLIP_SECTION_TOTAL_BITS.into(),
			bump,
			game_index,
			section_index,
			_padding: [0; 32],
		}
	}

	/// Initialize the section state without touching the data to prevent using
	/// compute units.
	pub fn init(&mut self, owner: Pubkey, game_index: u8, section_index: u8, bump: u8) {
		self.version = SectionState::VERSION;
		self.owner = owner;
		self.game_index = game_index;
		self.section_index = section_index;
		self.bump = bump;
		self.on = 0.into();
		self.off = BITFLIP_SECTION_TOTAL_BITS.into();
		self.flips = 0.into();
	}

	/// Whether the bit at the given index and offset is `1`.
	pub fn is_checked(&self, index: u8, offset: u8) -> bool {
		let value: u16 = self.data[index as usize].into();
		(value & ((index as u16) << (offset as u16))) != 0
	}

	pub fn on(&self) -> u32 {
		self.on.into()
	}

	pub fn off(&self) -> u32 {
		self.off.into()
	}

	pub fn flips(&self) -> u32 {
		self.flips.into()
	}

	pub fn flip_on(&mut self, changed_bits: u32) -> ProgramResult {
		self.on = self
			.on()
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?
			.into();
		self.off = self
			.off()
			.checked_sub(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?
			.into();
		self.flips = self
			.flips()
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?
			.into();

		Ok(())
	}

	pub fn flip_off(&mut self, changed_bits: u32) -> ProgramResult {
		self.off = self
			.off()
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?
			.into();
		self.on = self
			.on()
			.checked_sub(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?
			.into();
		self.flips = self
			.flips()
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?
			.into();

		Ok(())
	}

	/// Set a bit to the value specified in the `PlaySetBit` instruction.
	///
	/// Returns true if the bit was toggled.
	pub fn set_bit(&mut self, args: &FlipBit) -> Result<bool, ProgramError> {
		msg!("set_bit: {:?}", args);

		let index = args.array_index as usize;
		let current: u16 = self.data[index].into();
		let bit = 1 << args.offset;
		let updated = if args.on() {
			current | bit
		} else {
			current & !bit
		};

		msg!("current: {}, bit: {}, updated: {}", current, bit, updated);

		if updated == current {
			return Ok(false);
		}

		self.data[index..=index].copy_from_slice(&[updated.into()]);
		Ok(true)
	}

	/// Get the price of a bit in lamports.
	pub fn get_token_price_in_lamports(&self, remaining_time: i64) -> u64 {
		let flips: u64 = self.flips().into();
		let remaining_flips = EARNED_TOKENS_PER_SECTION - flips;
		let elapsed_time = SESSION_DURATION - remaining_time;
		let Some(static_price) = U64F64::from_num(flips)
			.checked_sqrt()
			.and_then(|val| val.checked_mul_int(512))
			.and_then(|val| val.checked_add(BASE_LAMPORTS_PER_BIT.into()))
		else {
			return BASE_LAMPORTS_PER_BIT;
		};

		if elapsed_time == 0 || remaining_time == 0 {
			return static_price.to_num();
		}

		let Some(current_rate) =
			U64F64::from_num(flips.max(1)).checked_div((elapsed_time.max(1) as u64).into())
		else {
			return static_price.to_num();
		};

		let Some(required_rate) =
			U64F64::from_num(remaining_flips).checked_div((remaining_time as u64).into())
		else {
			return static_price.to_num();
		};

		let Some(ratio) = current_rate.checked_div(required_rate) else {
			return static_price.to_num();
		};

		let price = static_price
			.checked_mul(ratio.sqrt())
			.unwrap_or(static_price)
			.to_num::<u64>();

		price.max(MIN_LAMPORTS_PER_BIT)
	}
}

#[cfg(test)]
mod tests {
	use std::thread;

	use rstest::fixture;
	use rstest::rstest;

	use super::*;

	macro_rules! set_snapshot_suffix {
    ($($expr:expr),*) => {
        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_suffix(format!($($expr,)*));
        let _guard = settings.bind_to_scope();
		}
	}

	#[fixture]
	pub fn testname() -> String {
		thread::current()
			.name()
			.unwrap()
			.split("::")
			.last()
			.unwrap()
			.split("_")
			.skip(2)
			.collect::<Vec<&str>>()
			.join("_")
			.to_string()
	}

	#[rstest]
	#[case::no_flips(0, SESSION_DURATION)]
	#[case::no_flips_no_time(0, 1)]
	#[case::first_flips(5, SESSION_DURATION - 100)]
	#[case::first_flips_more_time(5, SESSION_DURATION - 200)]
	#[case::first_flips_even_more_time(5, SESSION_DURATION - 400)]
	#[case::first_flips_most_time(5, SESSION_DURATION - 1000)]
	#[case::flips_doubled(10, SESSION_DURATION - 1000)]
	#[case::max_flips(EARNED_TOKENS_PER_SECTION as u32, 1)]
	fn test_token_price_calculation(
		testname: String,
		#[case] flips: u32,
		#[case] remaining_time: i64,
	) {
		set_snapshot_suffix!("{}", testname);
		let section = SectionState {
			version: 0,
			data: [PodU16::from(0); BITFLIP_SECTION_LENGTH],
			owner: Pubkey::default(),
			flips: PodU32::from(flips),
			on: PodU32::from(0),
			off: PodU32::from(BITFLIP_SECTION_TOTAL_BITS),
			game_index: 0,
			section_index: 0,
			bump: 0,
			_padding: [0; 32],
		};

		let lamports = section.get_token_price_in_lamports(remaining_time);
		insta::assert_snapshot!(format!(
			"flips: {flips}\nremaining_time: {remaining_time}\nlamports: {lamports}",
		));
	}
}
