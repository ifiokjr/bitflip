use solana_program::msg;
use steel::*;

use super::BitflipInstruction;
use crate::BitflipError;
use crate::ConfigState;
use crate::GameState;
use crate::ID;
use crate::SEED_GAME;
use crate::SEED_PREFIX;
use crate::SEED_SECTION;
use crate::SectionState;
use crate::TOKEN_DECIMALS;
use crate::cpi::create_associated_token_account_idempotent;
use crate::cpi::transfer_checked;
use crate::create_pda_config;
use crate::create_pda_game;
use crate::create_pda_mint_bit;
use crate::create_pda_section;
use crate::get_player_token_account;
use crate::get_section_token_account;

pub fn process_flip_bit(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
	// Parse the instruction data.
	let args = FlipBit::try_from_bytes(data)?;
	args.validate()?;

	// load accounts
	let [
		player_info,
		player_bit_token_account_info,
		config_info,
		game_info,
		mint_bit_info,
		section_info,
		section_bit_token_account_info,
		associated_token_program_info,
		token_program_info,
		system_program_info,
	] = accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	player_info.is_signer()?.is_writable()?;
	player_bit_token_account_info.is_writable()?;
	config_info.is_type::<ConfigState>(&ID)?;
	game_info.is_type::<GameState>(&ID)?;
	section_info.is_type::<SectionState>(&ID)?.is_writable()?;
	section_bit_token_account_info.is_writable()?;
	associated_token_program_info.is_program(&spl_associated_token_account::ID)?;
	token_program_info.is_program(&spl_token_2022::ID)?;
	system_program_info.is_program(&system_program::ID)?;

	let config_state = config_info.as_account::<ConfigState>(&ID)?;
	let game_state = game_info.as_account::<GameState>(&ID)?;
	let section_state = section_info.as_account_mut::<SectionState>(&ID)?;

	if config_state.game_index != game_state.game_index {
		return Err(BitflipError::InvalidGameIndex.into());
	}

	if args.section_index != section_state.section_index {
		return Err(BitflipError::InvalidSectionIndex.into());
	}

	let player_bit_token_account_key = get_player_token_account(player_info.key, mint_bit_info.key);
	let config_key = create_pda_config(config_state.bump)?;
	let game_key = create_pda_game(game_state.game_index, game_state.bump)?;
	let mint_bit_key = create_pda_mint_bit(config_state.mint_bit_bump)?;
	let section_key = create_pda_section(
		game_state.game_index,
		section_state.section_index,
		section_state.bump,
	)?;
	let section_bit_token_account_key =
		get_section_token_account(section_info.key, mint_bit_info.key);

	if player_bit_token_account_key.ne(player_bit_token_account_info.key)
		|| config_key.ne(config_info.key)
		|| game_key.ne(game_info.key)
		|| mint_bit_key.ne(mint_bit_info.key)
		|| section_key.ne(section_info.key)
		|| section_bit_token_account_key.ne(section_bit_token_account_info.key)
	{
		return Err(ProgramError::InvalidSeeds);
	}

	let current_time = Clock::get()?.unix_timestamp;

	if !game_state.is_running(current_time) {
		return Err(BitflipError::GameNotRunning.into());
	}

	msg!("game is running");

	let remaining_time = game_state.remaining_time(current_time);
	// TODO: check robustness of bonding curve
	let token_price = section_state.get_token_price_in_lamports(remaining_time);
	let is_changed = section_state.set_bit(args)?;
	let flips = if !is_changed {
		section_state.flip_on(1)?;
		section_state.flip_off(1)?;
		2
	} else if args.on() {
		section_state.flip_on(1)?;
		1
	} else {
		section_state.flip_off(1)?;
		1
	};

	msg!("flips: {}", flips);
	msg!("token price: {}", token_price);

	let lamports_to_transfer = token_price.saturating_mul(flips);

	msg!("creating associated token account");
	create_associated_token_account_idempotent(
		player_info,
		player_bit_token_account_info,
		player_info,
		mint_bit_info,
		token_program_info,
		system_program_info,
		&[],
	)?;

	msg!("transferring lamports to section: {}", lamports_to_transfer);
	transfer_lamports_to_section(section_info, player_info, lamports_to_transfer)?;

	msg!("transferring tokens from section");
	transfer_tokens_from_section(
		mint_bit_info,
		section_info,
		section_bit_token_account_info,
		player_bit_token_account_info,
		token_program_info,
		section_state,
		flips,
	)?;

	Ok(())
}

pub fn transfer_lamports_to_section<'info>(
	section: &AccountInfo<'info>,
	player: &AccountInfo<'info>,
	lamports: u64,
) -> ProgramResult {
	section.collect(lamports, player)?;
	Ok(())
}

pub fn transfer_tokens_from_section<'info>(
	mint_bit_info: &AccountInfo<'info>,
	section_info: &AccountInfo<'info>,
	section_bit_token_account_info: &AccountInfo<'info>,
	player_bit_token_account_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	section_state: &SectionState,
	tokens: u64,
) -> ProgramResult {
	msg!("transferring tokens from section: {}", tokens);
	let signer = &[
		SEED_PREFIX,
		SEED_GAME,
		&section_state.game_index.to_le_bytes(),
		SEED_SECTION,
		&section_state.section_index.to_le_bytes(),
		&[section_state.bump],
	];
	transfer_checked(
		section_bit_token_account_info,
		mint_bit_info,
		player_bit_token_account_info,
		section_info,
		token_program_info,
		tokens,
		TOKEN_DECIMALS,
		&[&signer[..]],
	)?;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "client", derive(typed_builder::TypedBuilder))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct FlipBit {
	/// The data section being updated.
	pub section_index: u8,
	/// The index of the `u16` value in the array.
	pub array_index: u8,
	/// The offset of the bit being set.
	pub offset: u8,
	/// The value to set the bit to: `0` or `1`.
	pub value: u8,
	/// Padding to make the struct size a multiple of 8 bytes.
	#[cfg_attr(feature = "client", builder(default))]
	pub _padding: [u8; 4],
}

impl FlipBit {
	pub fn on(&self) -> bool {
		self.value == 1
	}

	pub fn validate(&self) -> ProgramResult {
		if self.offset >= 16 {
			return Err(BitflipError::InvalidBitOffset.into());
		}

		if self.value != 0 && self.value != 1 {
			return Err(BitflipError::InvalidPlayValue.into());
		}

		Ok(())
	}
}

instruction!(BitflipInstruction, FlipBit);

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;

	use assert2::check;
	use solana_sdk::bpf_loader_upgradeable;
	use solana_sdk::native_loader;

	use super::*;
	use crate::get_pda_config;
	use crate::get_pda_game;
	use crate::get_pda_mint_bit;
	use crate::get_player_bit_token_account;
	use crate::leak;

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let game_index = 0;
		let section_index = 0;
		let account_infos = create_account_infos(game_index, section_index);
		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();
		let result = process_flip_bit(&account_infos[0..9], bytemuck::bytes_of(&args));

		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn player_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let player_info = &mut accounts[0];
		player_info.is_signer = false;
		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn player_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let player_info = &mut accounts[0];
		player_info.is_writable = false;
		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn player_bit_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let player_bit_token_account_info = &mut accounts[1];
		player_bit_token_account_info.is_writable = false;

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn player_bit_token_account_should_be_derived_from_associated_token_program()
	-> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let player_bit_token_account_info = &mut accounts[1];
		player_bit_token_account_info.key = leak(Pubkey::new_unique());

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let config_info = &mut accounts[2];
		config_info.data = Rc::new(RefCell::new(leak(vec![1u8; 8])));

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let config_info = &mut accounts[2];
		config_info.key = leak(Pubkey::new_unique());

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let game_info = &mut accounts[3];
		game_info.data = Rc::new(RefCell::new(leak(vec![0u8; 8])));

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn game_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let game_info = &mut accounts[3];
		game_info.key = leak(Pubkey::new_unique());

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_bit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let mint_bit_info = &mut accounts[4];
		mint_bit_info.key = leak(Pubkey::new_unique());

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn section_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let section_info = &mut accounts[5];
		section_info.is_writable = false;

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn section_should_have_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let section_info = &mut accounts[5];
		section_info.data = Rc::new(RefCell::new(leak(vec![0u8; 8])));

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn section_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let section_info = &mut accounts[5];
		section_info.key = leak(Pubkey::new_unique());

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn section_index_should_match() -> anyhow::Result<()> {
		let accounts = create_account_infos(0, 0);

		let args = FlipBit::builder()
			.section_index(1) // Different from account's section_index (0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == BitflipError::InvalidSectionIndex.into());

		Ok(())
	}

	#[test_log::test]
	fn section_bit_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let section_bit_token_account_info = &mut accounts[6];
		section_bit_token_account_info.is_writable = false;

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn section_bit_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos(0, 0);
		let section_bit_token_account_info = &mut accounts[6];
		section_bit_token_account_info.key = leak(Pubkey::new_unique());

		let args = FlipBit::builder()
			.section_index(0)
			.array_index(0)
			.offset(0)
			.value(1)
			.build();

		let result = process_flip_bit(&accounts, bytemuck::bytes_of(&args));
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	fn create_account_infos<'info>(game_index: u8, section_index: u8) -> [AccountInfo<'info>; 10] {
		let player_key = leak(Pubkey::new_unique());
		let player_lamports = leak(0);
		let player_data = leak(vec![]);
		let player_bit_token_account_key = leak(get_player_bit_token_account(player_key));
		let player_bit_token_account_lamports = leak(0);
		let player_bit_token_account_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = {
			let config_bump = get_pda_config().1;
			let mut data = vec![0u8; 8];
			let mint_bit_bump = get_pda_mint_bit().1;
			data[0] = ConfigState::discriminator();
			data.append(
				&mut ConfigState::new(
					Pubkey::new_unique(),
					config_bump,
					u8::MAX,
					mint_bit_bump,
					u8::MAX,
					u8::MAX,
					u8::MAX,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};
		let game_key = leak(get_pda_game(game_index).0);
		let game_lamports = leak(0);
		let game_data = {
			let game_bump = get_pda_game(game_index).1;
			let mut data = vec![0u8; 8];
			data[0] = GameState::discriminator();
			data.append(
				&mut GameState::new(
					Pubkey::new_unique(),
					Pubkey::new_unique(),
					game_index,
					game_bump,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};
		let mint_bit_key = leak(get_pda_mint_bit().0);
		let mint_bit_lamports = leak(0);
		let mint_bit_data = leak(vec![]);
		let section_key = leak(Pubkey::new_unique());
		let section_lamports = leak(0);
		let section_data = {
			let game_bump = get_pda_game(game_index).1;
			let mut data = vec![0u8; 8];
			data[0] = SectionState::discriminator();
			data.append(
				&mut SectionState::new(Pubkey::new_unique(), game_index, section_index, game_bump)
					.to_bytes()
					.to_vec(),
			);
			leak(data)
		};
		let section_bit_token_account_key = leak(Pubkey::new_unique());
		let section_bit_token_account_lamports = leak(0);
		let section_bit_token_account_data = leak(vec![]);
		let associated_token_program_lamports = leak(0);
		let associated_token_program_data = leak(vec![]);
		let token_program_lamports = leak(0);
		let token_program_data = leak(vec![]);
		let system_program_lamports = leak(0);
		let system_program_data = leak(vec![]);

		let player_info = AccountInfo::new(
			player_key,
			true,
			true,
			player_lamports,
			player_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let player_bit_token_account_info = AccountInfo::new(
			player_bit_token_account_key,
			false,
			true,
			player_bit_token_account_lamports,
			player_bit_token_account_data,
			&spl_associated_token_account::ID,
			false,
			u64::MAX,
		);
		let config_info = AccountInfo::new(
			config_key,
			false,
			false,
			config_lamports,
			config_data,
			&ID,
			false,
			u64::MAX,
		);
		let game_info = AccountInfo::new(
			game_key,
			false,
			false,
			game_lamports,
			game_data,
			&ID,
			false,
			u64::MAX,
		);
		let mint_bit_info = AccountInfo::new(
			mint_bit_key,
			false,
			false,
			mint_bit_lamports,
			mint_bit_data,
			&ID,
			false,
			u64::MAX,
		);
		let section_info = AccountInfo::new(
			section_key,
			false,
			true,
			section_lamports,
			section_data,
			&ID,
			false,
			u64::MAX,
		);
		let section_bit_token_account_info = AccountInfo::new(
			section_bit_token_account_key,
			false,
			true,
			section_bit_token_account_lamports,
			section_bit_token_account_data,
			&spl_associated_token_account::ID,
			false,
			u64::MAX,
		);
		let associated_token_program_info = AccountInfo::new(
			&spl_associated_token_account::ID,
			false,
			false,
			associated_token_program_lamports,
			associated_token_program_data,
			&bpf_loader_upgradeable::ID,
			true,
			u64::MAX,
		);
		let token_program_info = AccountInfo::new(
			&spl_token_2022::ID,
			false,
			false,
			token_program_lamports,
			token_program_data,
			&bpf_loader_upgradeable::ID,
			true,
			u64::MAX,
		);
		let system_program_info = AccountInfo::new(
			&system_program::ID,
			false,
			false,
			system_program_lamports,
			system_program_data,
			&native_loader::ID,
			true,
			u64::MAX,
		);
		[
			player_info,
			player_bit_token_account_info,
			config_info,
			game_info,
			mint_bit_info,
			section_info,
			section_bit_token_account_info,
			associated_token_program_info,
			token_program_info,
			system_program_info,
		]
	}
}
