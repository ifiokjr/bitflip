use solana_program::msg;
use spl_token_2022::extension::ExtensionType;
// use spl_token_group_interface::state::TokenGroup;
// use spl_type_length_value::state::TlvStateMut;
use steel::*;
use sysvar::rent::Rent;

use super::BitflipInstruction;
use crate::BIT_TOKEN_NAME;
use crate::BIT_TOKEN_SYMBOL;
use crate::BIT_TOKEN_URI;
use crate::BitflipError;
use crate::ConfigState;
use crate::ID;
use crate::SEED_BIT_MINT;
use crate::SEED_PREFIX;
use crate::SEED_TREASURY;
use crate::TOKEN_DECIMALS;
use crate::TOTAL_BIT_TOKENS;
use crate::cpi::create_associated_token_account;
use crate::cpi::group_pointer_initialize;
use crate::cpi::initialize_mint;
use crate::cpi::metadata_pointer_initialize;
use crate::cpi::mint_close_authority_initialize;
use crate::cpi::mint_to;
// use crate::cpi::token_group_initialize;
use crate::cpi::token_metadata_initialize;
use crate::create_pda_config;
use crate::create_pda_mint_bit;
use crate::create_pda_treasury;
use crate::get_token_amount;
use crate::get_treasury_token_account;

pub fn process_token_group_initialize(accounts: &[AccountInfo]) -> ProgramResult {
	// load accounts
	let [
		authority_info,
		config_info,
		treasury_info,
		mint_bit_info,
		treasury_bit_token_account_info,
		associated_token_program_info,
		token_program_info,
		system_program_info,
	] = accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

	authority_info.is_signer()?.is_writable()?;
	config_info.is_type::<ConfigState>(&ID)?;
	treasury_info
		.is_empty()?
		.is_writable()?
		.has_owner(&system_program::ID)?;
	mint_bit_info.is_empty()?.is_writable()?;
	treasury_bit_token_account_info.is_empty()?.is_writable()?;
	associated_token_program_info.is_program(&spl_associated_token_account::ID)?;
	token_program_info.is_program(&spl_token_2022::ID)?;
	system_program_info.is_program(&system_program::ID)?;

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let config_key = create_pda_config(config.bump)?;
	let treasury_key = create_pda_treasury(config.treasury_bump)?;
	let mint_bit_key = create_pda_mint_bit(config.mint_bit_bump)?;
	let treasury_bit_token_account_key = get_treasury_token_account(&treasury_key, &mint_bit_key);
	let treasury_seeds = &[SEED_PREFIX, SEED_TREASURY, &[config.treasury_bump]];

	if config_info.key.ne(&config_key)
		|| treasury_info.key.ne(&treasury_key)
		|| mint_bit_info.key.ne(&mint_bit_key)
		|| treasury_bit_token_account_info
			.key
			.ne(&treasury_bit_token_account_key)
	{
		return Err(ProgramError::InvalidSeeds);
	}

	if authority_info.key.ne(&config.authority) {
		return Err(BitflipError::Unauthorized.into());
	}

	msg!("transfer sol to treasury for rent exemption");
	// transfer sol to treasury for rent exemption
	let rent_sysvar = Rent::get()?;
	let extra_lamports = rent_sysvar
		.minimum_balance(treasury_info.data_len())
		.checked_sub(treasury_info.lamports())
		.ok_or(ProgramError::ArithmeticOverflow)?;
	treasury_info.collect(extra_lamports, authority_info)?;

	allocate_account_with_bump(
		mint_bit_info,
		system_program_info,
		authority_info,
		get_mint_space()?,
		token_program_info.key,
		&[SEED_PREFIX, SEED_BIT_MINT],
		config.mint_bit_bump,
	)?;

	msg!(
		"{}: initialize metadata pointer mint extension",
		BIT_TOKEN_NAME
	);
	metadata_pointer_initialize(mint_bit_info, treasury_info, token_program_info, &[])?;

	msg!(
		"{}: initialize mint close authority mint extension",
		BIT_TOKEN_NAME
	);
	mint_close_authority_initialize(mint_bit_info, treasury_info, token_program_info, &[])?;

	msg!(
		"{}: initialize group pointer mint extension",
		BIT_TOKEN_NAME
	);
	group_pointer_initialize(mint_bit_info, treasury_info, token_program_info, &[])?;

	msg!("{}: initialize mint", BIT_TOKEN_NAME);
	initialize_mint(
		mint_bit_info,
		token_program_info,
		treasury_info,
		TOKEN_DECIMALS,
		&[],
	)?;

	msg!("{}: initialize token metadata", BIT_TOKEN_NAME);
	token_metadata_initialize(
		mint_bit_info,
		treasury_info,
		token_program_info,
		BIT_TOKEN_NAME.to_string(),
		BIT_TOKEN_SYMBOL.to_string(),
		BIT_TOKEN_URI.to_string(),
		&[&treasury_seeds[..]],
	)?;

	// msg!("{}: initialize group", BIT_TOKEN_NAME);
	// token_group_initialize(
	// 	token_program_info,
	// 	mint_bit_info,
	// 	mint_bit_info,
	// 	treasury_info,
	// 	&[&treasury_seeds[..]],
	// )?;

	// Allocate a TLV entry for the space and write it in
	// let mut buffer = mint_bit_info.try_borrow_mut_data()?;
	// let mut state = TlvStateMut::unpack(&mut buffer)?;
	// let (group, _) = state.init_value::<TokenGroup>(false)?;
	// *group = TokenGroup::new(mint_bit_info.key,
	// Some(*treasury_info.key).try_into()?, 8);

	msg!("{}: get mint account size", BIT_TOKEN_NAME);
	let extra_lamports = rent_sysvar
		.minimum_balance(mint_bit_info.data_len())
		.checked_sub(mint_bit_info.lamports())
		.ok_or(ProgramError::ArithmeticOverflow)?;

	if extra_lamports > 0 {
		msg!("{}: collect extra lamports", BIT_TOKEN_NAME);
		mint_bit_info.collect(extra_lamports, authority_info)?;
	}

	msg!(
		"{}: create treasury associated token account",
		BIT_TOKEN_NAME
	);
	create_associated_token_account(
		authority_info,
		treasury_bit_token_account_info,
		treasury_info,
		mint_bit_info,
		token_program_info,
		system_program_info,
		&[&treasury_seeds[..]],
	)?;

	msg!(
		"{}: mint tokens to the treasury_bit_token_account",
		BIT_TOKEN_NAME
	);
	mint_to(
		mint_bit_info,
		treasury_bit_token_account_info,
		treasury_info,
		token_program_info,
		get_token_amount(TOTAL_BIT_TOKENS, TOKEN_DECIMALS)?,
		&[&treasury_seeds[..]],
	)?;

	Ok(())
}

const EXTENSION_TYPES: &[ExtensionType] = &[
	ExtensionType::MetadataPointer,
	ExtensionType::MintCloseAuthority,
	ExtensionType::GroupPointer,
];

pub fn get_mint_space() -> Result<usize, ProgramError> {
	let mint_space =
		ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(EXTENSION_TYPES)?;
	Ok(mint_space)
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
	use crate::get_pda_mint_bit;
	use crate::get_pda_mint_gibibit;
	use crate::get_pda_mint_kibibit;
	use crate::get_pda_mint_mebibit;
	use crate::get_pda_treasury;
	use crate::get_treasury_bit_token_account;
	use crate::leak;

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
	fn treasury_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[2];
		treasury_info.is_writable = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

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
	fn treasury_bit_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_bit_token_account_info = &mut accounts[4];
		treasury_bit_token_account_info.key = leak(Pubkey::new_unique());

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_bit_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_bit_token_account_info = &mut accounts[4];
		treasury_bit_token_account_info.is_writable = false;

		let result = process_token_group_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 8] {
		let authority_key = leak(Pubkey::new_unique());
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = {
			let config_bump = get_pda_config().1;
			let mut data = vec![0u8; 8];
			let treasury_bump = get_pda_treasury().1;
			let mint_bit_bump = get_pda_mint_bit().1;
			let mint_kibibit_bump = get_pda_mint_kibibit().1;
			let mint_mebibit_bump = get_pda_mint_mebibit().1;
			let mint_gibibit_bump = get_pda_mint_gibibit().1;
			data[0] = ConfigState::discriminator();
			data.append(
				&mut ConfigState::new(
					*authority_key,
					config_bump,
					treasury_bump,
					mint_bit_bump,
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
		let mint_bit_key = leak(get_pda_mint_bit().0);
		let mint_bit_lamports = leak(0);
		let mint_bit_data = leak(vec![]);
		let treasury_bit_token_account_key = leak(get_treasury_bit_token_account());
		let treasury_bit_token_account_lamports = leak(0);
		let treasury_bit_token_account_data = leak(vec![]);
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
		let treasury_bit_token_account_info = AccountInfo::new(
			treasury_bit_token_account_key,
			false,
			true,
			treasury_bit_token_account_lamports,
			treasury_bit_token_account_data,
			&spl_associated_token_account::ID,
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
			treasury_bit_token_account_info,
			associated_token_program_info,
			token_program_info,
			system_program_info,
		]
	}
}
