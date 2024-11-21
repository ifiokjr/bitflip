use fixed::types::U64F64;
use solana_program::msg;
use spl_pod::primitives::PodI64;
use spl_pod::primitives::PodU16;
use spl_pod::primitives::PodU32;
use steel::*;

use crate::BASE_LAMPORTS_PER_BIT;
use crate::BITFLIP_SECTION_LENGTH;
use crate::BITFLIP_SECTION_TOTAL_BITS;
use crate::EARNED_TOKENS_PER_SECTION;
use crate::FlipBit;
use crate::MIN_LAMPORTS_PER_BIT;
use crate::SESSION_DURATION;

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, IntoPrimitive, TryFromPrimitive)]
pub enum BitflipAccount {
	ConfigState = 0,
	GameState = 1,
	SectionState = 2,
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "client", derive(typed_builder::TypedBuilder))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ConfigState {
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
	/// There will be a maximum of 256 games.
	pub game_index: u8,
}

impl ConfigState {
	pub const fn space() -> usize {
		8 + std::mem::size_of::<Self>()
	}

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
			authority,
			bump,
			treasury_bump,
			mint_bit_bump,
			mint_kibibit_bump,
			mint_mebibit_bump,
			mint_gibibit_bump,
			game_index: 0,
		}
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "client", derive(typed_builder::TypedBuilder))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct GameState {
	/// This is a refresh signer created and maintained by the backend. It needs
	/// to be provided to update the access signer. It will need to be updated
	/// after every game.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	#[cfg_attr(feature = "client", builder(default))]
	pub refresh_signer: Pubkey,
	/// This is an access signer created and maintained by the backend. Which is
	/// allowed to sign certain transactions and expires daily.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	#[cfg_attr(feature = "client", builder(default))]
	pub access_signer: Pubkey,
	/// The timestamp that the access expiry will end.
	#[cfg_attr(feature = "client", builder(default, setter(into)))]
	pub access_expiry: PodI64,
	/// The start time. If 0 then it hasn't started yet. Using an `Option` here
	/// would waste an extra byte.
	#[cfg_attr(feature = "client", builder(default, setter(into)))]
	pub start_time: PodI64,
	/// The index of this currently played game.
	pub game_index: u8,
	/// The most recent section which was unlocked. This will be updated every
	/// time a new section is initialized.
	pub section_index: u8,
	/// The bump for this account.
	pub bump: u8,
}

impl GameState {
	pub fn new(access_signer: Pubkey, refresh_signer: Pubkey, index: u8, bump: u8) -> Self {
		Self {
			refresh_signer,
			access_signer,
			access_expiry: 0.into(),
			start_time: 0.into(),
			section_index: 0,
			game_index: index,
			bump,
		}
	}

	pub fn start_time(&self) -> i64 {
		self.start_time.into()
	}

	pub fn access_expiry(&self) -> i64 {
		self.access_expiry.into()
	}

	/// The end time of the game.
	pub fn end_time(&self) -> i64 {
		self.start_time().saturating_add(SESSION_DURATION)
	}

	/// The remaining time of the game.
	pub fn remaining_time(&self, current_time: i64) -> i64 {
		self.end_time().saturating_sub(current_time)
	}

	/// Whether the game has started.
	pub fn has_started(&self) -> bool {
		msg!("start_time: {}", self.start_time());
		self.start_time() > 0
	}

	/// Whether the game has ended.
	pub fn has_ended(&self, current_time: i64) -> bool {
		msg!(
			"current_time: {}, end_time: {}",
			current_time,
			self.end_time()
		);
		self.has_started() && current_time > self.end_time()
	}

	/// Whether the game is running.
	pub fn is_running(&self, current_time: i64) -> bool {
		self.has_started() && !self.has_ended(current_time)
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct SectionState {
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
}

impl SectionState {
	/// Create a new section state in the client. Useful for testing.
	pub fn new(owner: Pubkey, game_index: u8, section_index: u8, bump: u8) -> Self {
		Self {
			data: [0.into(); BITFLIP_SECTION_LENGTH],
			owner,
			flips: 0.into(),
			on: 0.into(),
			off: BITFLIP_SECTION_TOTAL_BITS.into(),
			bump,
			game_index,
			section_index,
		}
	}

	/// Initialize the section state without touching the data to prevent using
	/// compute units.
	pub fn init(&mut self, owner: Pubkey, index: u8, bump: u8) {
		self.owner = owner;
		self.section_index = index;
		self.bump = bump;
		self.on = 0.into();
		self.off = BITFLIP_SECTION_TOTAL_BITS.into();
		self.flips = 0.into();
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
		let Some(current_rate) =
			U64F64::from_num(flips.max(1)).checked_div((elapsed_time as u64).into())
		else {
			return static_price.to_num();
		};
		let Some(required_rate) =
			U64F64::from_num(remaining_flips).checked_div((remaining_time as u64).into())
		else {
			return static_price.to_num();
		};
		// let diff = current_rate.abs_diff(required_rate);
		let Some(ratio) = current_rate.checked_div(required_rate) else {
			return static_price.to_num();
		};

		static_price
			.checked_mul(ratio)
			.and_then(|val| val.checked_mul(ratio))
			.unwrap_or(static_price)
			.to_num::<u64>()
			.max(MIN_LAMPORTS_PER_BIT)
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct TempState {
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
	// /// Padding to make the size of the struct a multiple of 8.
	// pub _padding: [u8; 1],
}

account!(BitflipAccount, ConfigState);
account!(BitflipAccount, GameState);
account!(BitflipAccount, SectionState);

#[cfg(test)]
mod tests {
	use std::thread;

	use assert2::check;
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
			.to_string()
	}

	#[rstest]
	#[case::no_flips(0, SESSION_DURATION, 100_000)] // no flips
	// #[case::first_flip(1, SESSION_DURATION / 2, 100_000)] // 1 flip
	#[case::max_flips(EARNED_TOKENS_PER_SECTION as u32, 1, 362_144)]
	fn test_token_price_calculation(
		testname: String,
		#[case] flips: u32,
		#[case] remaining_time: i64,
		#[case] expected_price: u64,
	) {
		set_snapshot_suffix!("{}", testname);
		let section = SectionState {
			data: [PodU16::from(0); BITFLIP_SECTION_LENGTH],
			owner: Pubkey::default(),
			flips: PodU32::from(flips),
			on: PodU32::from(0),
			off: PodU32::from(BITFLIP_SECTION_TOTAL_BITS),
			game_index: 0,
			section_index: 0,
			bump: 0,
		};

		let price = section.get_token_price_in_lamports(remaining_time);
		check!(price == expected_price);

		// insta::assert_snapshot!(format!("price: {}", price));
		// assert_eq!(price, expected_price);
	}
}
