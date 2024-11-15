use solana_program::msg;
use steel::*;

use crate::BITFLIP_SECTION_LENGTH;
use crate::BITFLIP_SECTION_TOTAL_BITS;
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
	/// There will be a maximum of 256 games.
	pub game_index: u8,
}

impl ConfigState {
	pub const fn space() -> usize {
		8 + std::mem::size_of::<Self>()
	}

	pub fn new(authority: Pubkey, bump: u8, treasury_bump: u8, mint_bit_bump: u8) -> ConfigState {
		ConfigState {
			authority,
			bump,
			treasury_bump,
			mint_bit_bump,
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
	#[cfg_attr(feature = "client", builder(default))]
	pub access_expiry: i64,
	/// The start time. If 0 then it hasn't started yet. Using an `Option` here
	/// would waste an extra byte.
	#[cfg_attr(feature = "client", builder(default))]
	pub start_time: i64,
	/// The index of this currently played game.
	pub game_index: u8,
	/// The most recent section which was unlocked. This will be updated every
	/// time a new section is initialized.
	pub section_index: u8,
	/// The bump for this account.
	pub bump: u8,
	/// The bump for the nonce account.
	pub nonce_bump: u8,
	/// Padding to make the size of the struct a multiple of 8.
	#[cfg_attr(feature = "client", builder(default))]
	pub _padding: [u8; 4],
}

impl GameState {
	pub fn new(
		access_signer: Pubkey,
		refresh_signer: Pubkey,
		index: u8,
		bump: u8,
		nonce_bump: u8,
	) -> Self {
		Self {
			refresh_signer,
			access_signer,
			access_expiry: 0,
			start_time: 0,
			section_index: 0,
			game_index: index,
			bump,
			nonce_bump,
			_padding: [0; 4],
		}
	}

	pub fn end_time(&self) -> i64 {
		self.start_time.saturating_add(SESSION_DURATION)
	}

	pub fn started(&self) -> bool {
		msg!("start_time: {}", self.start_time);
		self.start_time > 0
	}

	pub fn ended(&self, current_time: i64) -> bool {
		msg!(
			"current_time: {}, end_time: {}",
			current_time,
			self.end_time()
		);
		self.started() && current_time > self.end_time()
	}

	pub fn running(&self, current_time: i64) -> bool {
		self.started() && !self.ended(current_time)
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
	pub data: [u16; BITFLIP_SECTION_LENGTH],
	/// The owner of this section.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	pub owner: Pubkey,
	/// The number of bit flips that have occurred.
	pub flips: u32,
	/// The number of bits that are on.
	pub on: u32,
	/// The number of bits that are off.
	pub off: u32,
	/// The index for this section state.
	pub index: u8,
	/// The bump for this section state.
	pub bump: u8,
	/// Padding to make the size of the struct a multiple of 8.
	pub _padding: [u8; 2],
}

impl SectionState {
	/// Create a new section state in the client. Useful for testing.
	pub fn new(owner: Pubkey, index: u8, bump: u8) -> Self {
		Self {
			data: [0; BITFLIP_SECTION_LENGTH],
			owner,
			flips: 0,
			on: 0,
			off: BITFLIP_SECTION_TOTAL_BITS,
			bump,
			index,
			_padding: [0; 2],
		}
	}

	/// Initialize the section state without touching the data to prevent using
	/// compute units.
	pub fn init(&mut self, owner: Pubkey, index: u8, bump: u8) {
		self.owner = owner;
		self.index = index;
		self.bump = bump;
		self.on = BITFLIP_SECTION_TOTAL_BITS;
		self.off = 0;
		self.flips = 0;
	}

	pub fn flip_on(&mut self, changed_bits: u32) -> ProgramResult {
		self.on = self
			.on
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		self.off = self
			.off
			.checked_sub(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		self.flips = self
			.flips
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?;

		Ok(())
	}

	pub fn flip_off(&mut self, changed_bits: u32) -> ProgramResult {
		self.off = self
			.off
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		self.on = self
			.on
			.checked_sub(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		self.flips = self
			.flips
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?;

		Ok(())
	}
}

account!(BitflipAccount, ConfigState);
account!(BitflipAccount, GameState);
account!(BitflipAccount, SectionState);
