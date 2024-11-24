use solana_program::msg;
use steel::*;
use sysvar::rent::Rent;

use super::BitflipInstruction;
use crate::BitflipError;
use crate::ConfigState;
use crate::ID;
use crate::SEED_PREFIX;
use crate::SEED_TREASURY;
use crate::TokenMember;
use crate::cpi::token_group_initialize;
use crate::cpi::token_group_member_initialize;
use crate::create_pda_config;
use crate::create_pda_mint;
use crate::create_pda_treasury;

pub fn process_token_group_initialize(accounts: &[AccountInfo]) -> ProgramResult {
	use TokenMember::*;
	// load accounts
	let [
		authority_info,
		config_info,
		treasury_info,
		mint_bit_info,
		mint_kibibit_info,
		mint_mebibit_info,
		mint_gibibit_info,
		associated_token_program_info,
		token_program_info,
		system_program_info,
	] = accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	authority_info.is_signer()?.is_writable()?;
	config_info.is_type::<ConfigState>(&ID)?;
	treasury_info.has_owner(&system_program::ID)?;
	mint_bit_info.is_writable()?;
	mint_kibibit_info.is_writable()?;
	mint_mebibit_info.is_writable()?;
	mint_gibibit_info.is_writable()?;
	associated_token_program_info.is_program(&spl_associated_token_account::ID)?;
	token_program_info.is_program(&spl_token_2022::ID)?;
	system_program_info.is_program(&system_program::ID)?;

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let config_key = create_pda_config(config.bump)?;
	let treasury_key = create_pda_treasury(config.treasury_bump)?;
	let mint_bit_key = create_pda_mint(Bit, Bit.bump(config))?;
	let mint_kibibit_key = create_pda_mint(Kibibit, Kibibit.bump(config))?;
	let mint_mebibit_key = create_pda_mint(Mebibit, Mebibit.bump(config))?;
	let mint_gibibit_key = create_pda_mint(Gibibit, Gibibit.bump(config))?;
	let treasury_seeds = &[SEED_PREFIX, SEED_TREASURY, &[config.treasury_bump]];

	if config_key.ne(config_info.key)
		|| treasury_key.ne(treasury_info.key)
		|| mint_bit_key.ne(mint_bit_info.key)
		|| mint_kibibit_key.ne(mint_kibibit_info.key)
		|| mint_mebibit_key.ne(mint_mebibit_info.key)
		|| mint_gibibit_key.ne(mint_gibibit_info.key)
	{
		return Err(ProgramError::InvalidSeeds);
	}

	if authority_info.key.ne(&config.authority) {
		return Err(BitflipError::Unauthorized.into());
	}

	let rent_sysvar = Rent::get()?;

	msg!("{}: initialize group", Bit.name());
	token_group_initialize(
		token_program_info,
		mint_bit_info,
		mint_bit_info,
		treasury_info,
		&[&treasury_seeds[..]],
	)?;

	for (info, member) in [
		(mint_kibibit_info, Kibibit),
		(mint_mebibit_info, Mebibit),
		(mint_gibibit_info, Gibibit),
	] {
		msg!("{}: initialize group member", member.name());
		token_group_member_initialize(
			token_program_info,
			info,
			info,
			treasury_info,
			mint_bit_info,
			treasury_info,
			&[treasury_seeds],
		)?;
	}

	for info in [
		mint_bit_info,
		mint_kibibit_info,
		mint_mebibit_info,
		mint_gibibit_info,
	] {
		let extra_lamports = rent_sysvar
			.minimum_balance(info.data_len())
			.checked_sub(info.lamports())
			.ok_or(ProgramError::ArithmeticOverflow)?;

		if extra_lamports > 0 {
			info.collect(extra_lamports, authority_info)?;
		}
	}

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TokenGroupInitialize {}

instruction!(BitflipInstruction, TokenGroupInitialize);

#[cfg(test)]
mod tests {
	use std::cell::RefCell;
	use std::rc::Rc;

	use assert2::check;
	use solana_sdk::bpf_loader_upgradeable;
	use solana_sdk::native_loader;

	use super::*;
	use crate::BitflipError;
	use crate::get_pda_config;
	use crate::get_pda_mint;
	use crate::get_pda_treasury;
	use crate::leak;

	#[test_log::test]
	fn validation_should_pass() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::UnsupportedSysvar);

		Ok(())
	}

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_token_group_initialize(&accounts[0..7]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.is_signer = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.is_writable = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_from_config() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.key = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::Custom(BitflipError::Unauthorized.into()));

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[1];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn config_should_have_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[1];
		config_info.data = Rc::new(RefCell::new(leak(vec![1u8; 8])));

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn config_should_have_valid_owner() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[1];
		config_info.owner = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn treasury_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[2];
		treasury_info.key = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_should_be_system_account() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[2];
		treasury_info.owner = &ID;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn mint_bit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_bit_info = &mut accounts[3];
		mint_bit_info.key = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_bit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_bit_info = &mut accounts[3];
		mint_bit_info.is_writable = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_kibibit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_kibibit_info = &mut accounts[4];
		mint_kibibit_info.is_writable = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_kibibit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_kibibit_info = &mut accounts[4];
		mint_kibibit_info.key = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_mebibit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_mebibit_info = &mut accounts[5];
		mint_mebibit_info.is_writable = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_mebibit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_mebibit_info = &mut accounts[5];
		mint_mebibit_info.key = leak(Pubkey::new_unique());

		Ok(())
	}

	#[test_log::test]
	fn mint_gibibit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_gibibit_info = &mut accounts[6];
		mint_gibibit_info.is_writable = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_gibibit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_gibibit_info = &mut accounts[6];
		mint_gibibit_info.key = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 10] {
		let authority_key = leak(Pubkey::new_unique());
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = {
			let config_bump = get_pda_config().1;
			let mut data = vec![0u8; 8];
			let treasury_bump = get_pda_treasury().1;
			let mint_bump = get_pda_mint(TokenMember::Bit).1;
			let mint_kibibit_bump = get_pda_mint(TokenMember::Kibibit).1;
			let mint_mebibit_bump = get_pda_mint(TokenMember::Mebibit).1;
			let mint_gibibit_bump = get_pda_mint(TokenMember::Gibibit).1;
			data[0] = ConfigState::discriminator();
			data.append(
				&mut ConfigState::new(
					*authority_key,
					config_bump,
					treasury_bump,
					mint_bump,
					mint_kibibit_bump,
					mint_mebibit_bump,
					mint_gibibit_bump,
				)
				.to_bytes()
				.to_vec(),
			);
			leak(data)
		};
		let treasury_key = leak(get_pda_treasury().0);
		let treasury_lamports = leak(0);
		let treasury_data = leak(vec![]);
		let mint_bit_key = leak(get_pda_mint(TokenMember::Bit).0);
		let mint_bit_lamports = leak(0);
		let mint_bit_data = leak(vec![]);
		let mint_kibibit_key = leak(get_pda_mint(TokenMember::Kibibit).0);
		let mint_kibibit_lamports = leak(0);
		let mint_kibibit_data = leak(vec![]);
		let mint_mebibit_key = leak(get_pda_mint(TokenMember::Mebibit).0);
		let mint_mebibit_lamports = leak(0);
		let mint_mebibit_data = leak(vec![]);
		let mint_gibibit_key = leak(get_pda_mint(TokenMember::Gibibit).0);
		let mint_gibibit_lamports = leak(0);
		let mint_gibibit_data = leak(vec![]);
		let associated_token_program_key = leak(spl_associated_token_account::ID);
		let associated_token_program_lamports = leak(1_000_000_000);
		let associated_token_program_data = leak(vec![]);
		let token_program_key = leak(spl_token_2022::ID);
		let token_program_lamports = leak(1_000_000_000);
		let token_program_data = leak(vec![]);
		let system_program_lamports = leak(1_000_000_000);
		let system_program_data = leak(vec![]);

		let authority_info = AccountInfo::new(
			authority_key,
			true,
			true,
			authority_lamports,
			authority_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let config_info = AccountInfo::new(
			config_key,
			false,
			true,
			config_lamports,
			config_data,
			&ID,
			false,
			u64::MAX,
		);
		let treasury_info = AccountInfo::new(
			treasury_key,
			false,
			true,
			treasury_lamports,
			treasury_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let mint_bit_info = AccountInfo::new(
			mint_bit_key,
			false,
			true,
			mint_bit_lamports,
			mint_bit_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let mint_kibibit_info = AccountInfo::new(
			mint_kibibit_key,
			false,
			true,
			mint_kibibit_lamports,
			mint_kibibit_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let mint_mebibit_info = AccountInfo::new(
			mint_mebibit_key,
			false,
			true,
			mint_mebibit_lamports,
			mint_mebibit_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let mint_gibibit_info = AccountInfo::new(
			mint_gibibit_key,
			false,
			true,
			mint_gibibit_lamports,
			mint_gibibit_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
		let associated_token_program_info = AccountInfo::new(
			associated_token_program_key,
			false,
			false,
			associated_token_program_lamports,
			associated_token_program_data,
			&bpf_loader_upgradeable::ID,
			true,
			u64::MAX,
		);
		let token_program_info = AccountInfo::new(
			token_program_key,
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
			authority_info,
			config_info,
			treasury_info,
			mint_bit_info,
			mint_kibibit_info,
			mint_mebibit_info,
			mint_gibibit_info,
			associated_token_program_info,
			token_program_info,
			system_program_info,
		]
	}
}
