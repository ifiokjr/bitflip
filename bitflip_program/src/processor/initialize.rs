use solana_program::msg;
use spl_token_2022::extension::ExtensionType;
use steel::*;
use sysvar::rent::Rent;

use super::BitflipInstruction;
use crate::ADMIN_PUBKEY;
use crate::BIT_TOKEN_NAME;
use crate::BIT_TOKEN_SYMBOL;
use crate::BIT_TOKEN_URI;
use crate::BitflipError;
use crate::ConfigState;
use crate::ID;
use crate::SEED_BIT_MINT;
use crate::SEED_CONFIG;
use crate::SEED_PREFIX;
use crate::SEED_TREASURY;
use crate::TOKEN_DECIMALS;
use crate::TOTAL_TOKENS;
use crate::cpi::create_associated_token_account;
use crate::cpi::group_pointer_initialize;
use crate::cpi::initialize_mint2;
use crate::cpi::metadata_pointer_initialize;
use crate::cpi::mint_close_authority_initialize;
use crate::cpi::mint_to;
use crate::cpi::token_metadata_initialize;
use crate::get_pda_config;
use crate::get_pda_mint_bit;
use crate::get_pda_treasury;
use crate::get_token_amount;
use crate::get_treasury_token_account;

pub fn process_initialize(accounts: &[AccountInfo<'_>]) -> ProgramResult {
	// load accounts
	let [
		admin_info,
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

	// validate accounts
	let (config_key, config_bump) = get_pda_config();
	let (treasury_key, treasury_bump) = get_pda_treasury();
	let (mint_bit_key, mint_bit_bump) = get_pda_mint_bit();
	let treasury_token_account_key = get_treasury_token_account(&treasury_key, &mint_bit_key);
	let treasury_seeds = &[SEED_PREFIX, SEED_TREASURY, &[treasury_bump]];

	if config_info.key.ne(&config_key)
		|| treasury_info.key.ne(&treasury_key)
		|| mint_bit_info.key.ne(&mint_bit_key)
		|| treasury_bit_token_account_info
			.key
			.ne(&treasury_token_account_key)
	{
		return Err(ProgramError::InvalidSeeds);
	}

	if admin_info.key.ne(&ADMIN_PUBKEY) {
		return Err(BitflipError::UnauthorizedAdmin.into());
	}

	if authority_info.key.eq(&ADMIN_PUBKEY) {
		return Err(BitflipError::DuplicateAuthority.into());
	}

	config_info.is_empty()?.is_writable()?;
	admin_info.is_signer()?;
	treasury_info
		.is_empty()?
		.is_writable()?
		.has_owner(&system_program::ID)?;
	authority_info.is_signer()?.is_writable()?;
	mint_bit_info.is_empty()?.is_writable()?;
	treasury_bit_token_account_info.is_empty()?.is_writable()?;
	associated_token_program_info.is_program(&spl_associated_token_account::ID)?;
	token_program_info.is_program(&spl_token_2022::ID)?;
	system_program_info.is_program(&system_program::ID)?;

	// initialize config
	create_account_with_bump::<ConfigState>(
		config_info,
		system_program_info,
		authority_info,
		&ID,
		&[SEED_PREFIX, SEED_CONFIG],
		config_bump,
	)?;

	// initialize config
	let config = config_info.as_account_mut::<ConfigState>(&ID)?;
	*config = ConfigState::new(
		*authority_info.key,
		config_bump,
		treasury_bump,
		mint_bit_bump,
	);

	msg!("transfer sol to treasury for rent exemption");
	// transfer sol to treasury for rent exemption
	let rent_sysvar = Rent::default();
	let extra_lamports = rent_sysvar
		.minimum_balance(treasury_info.data_len())
		.checked_sub(treasury_info.lamports())
		.ok_or(ProgramError::ArithmeticOverflow)?;
	treasury_info.collect(extra_lamports, authority_info)?;

	msg!("get mint account size");
	// get mint account size
	let extension_types = vec![
		ExtensionType::MetadataPointer,
		ExtensionType::MintCloseAuthority,
		ExtensionType::GroupPointer,
	];
	let mint_space =
		ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&extension_types)?;
	allocate_account_with_bump(
		mint_bit_info,
		system_program_info,
		authority_info,
		mint_space,
		token_program_info.key,
		&[SEED_PREFIX, SEED_BIT_MINT],
		mint_bit_bump,
	)?;

	msg!("initialize metadata pointer mint extension");
	metadata_pointer_initialize(mint_bit_info, treasury_info, token_program_info, &[])?;

	msg!("initialize mint close authority mint extension");
	mint_close_authority_initialize(mint_bit_info, treasury_info, token_program_info, &[])?;

	msg!("initialize group pointer mint extension");
	group_pointer_initialize(mint_bit_info, treasury_info, token_program_info, &[])?;

	msg!("initialize mint");
	initialize_mint2(
		mint_bit_info,
		token_program_info,
		treasury_info,
		TOKEN_DECIMALS,
		&[],
	)?;

	msg!("initialize token metadata");
	token_metadata_initialize(
		mint_bit_info,
		treasury_info,
		token_program_info,
		BIT_TOKEN_NAME.to_string(),
		BIT_TOKEN_SYMBOL.to_string(),
		BIT_TOKEN_URI.to_string(),
		&[&treasury_seeds[..]],
	)?;

	msg!("get mint account size");
	let extra_lamports = rent_sysvar
		.minimum_balance(mint_bit_info.data_len())
		.checked_sub(mint_bit_info.lamports())
		.ok_or(ProgramError::ArithmeticOverflow)?;

	if extra_lamports > 0 {
		msg!("collect extra lamports");
		mint_bit_info.collect(extra_lamports, authority_info)?;
	}

	msg!("create treasury associated token account");
	create_associated_token_account(
		authority_info,
		treasury_bit_token_account_info,
		treasury_info,
		mint_bit_info,
		token_program_info,
		system_program_info,
		&[&treasury_seeds[..]],
	)?;

	msg!("mint tokens to the treasury_token_account");
	mint_to(
		mint_bit_info,
		treasury_bit_token_account_info,
		treasury_info,
		token_program_info,
		get_token_amount(TOTAL_TOKENS, TOKEN_DECIMALS)?,
		&[&treasury_seeds[..]],
	)?;

	Ok(())
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct Initialize {}

instruction!(BitflipInstruction, Initialize);

mod modname {}

#[cfg(test)]
mod tests {
	use assert2::check;

	use super::*;
	use crate::get_pda_config;
	use crate::get_pda_treasury;
	use crate::get_treasury_bit_token_account;
	use crate::leak;

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_initialize(&accounts[0..8]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn admin_should_be_hardcoded() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let admin_info = &mut accounts[0];
		admin_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::Custom(BitflipError::UnauthorizedAdmin.into()));

		Ok(())
	}

	#[test_log::test]
	fn admin_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let admin_info = &mut accounts[0];
		admin_info.is_signer = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_not_be_admin() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.key = leak(ADMIN_PUBKEY);

		let result = process_initialize(&accounts);
		check!(
			result.unwrap_err() == ProgramError::Custom(BitflipError::DuplicateAuthority.into())
		);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_signer = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[1];
		authority_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[2];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[2];
		config_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[3];
		treasury_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[3];
		treasury_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_should_be_system_account() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[3];
		treasury_info.owner = &ID;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn mint_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_info = &mut accounts[4];
		mint_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_info = &mut accounts[4];
		mint_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_token_account_info = &mut accounts[5];
		treasury_token_account_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_token_account_info = &mut accounts[5];
		treasury_token_account_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 9] {
		let admin_lamports = leak(0);
		let admin_data = leak(vec![]);
		let authority_key = leak(Pubkey::new_unique());
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = leak(vec![]);
		let mint_bit_key = leak(get_pda_mint_bit().0);
		let mint_bit_lamports = leak(0);
		let mint_bit_data = leak(vec![]);
		let treasury_key = leak(get_pda_treasury().0);
		let treasury_lamports = leak(0);
		let treasury_data = leak(vec![]);
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

		let admin_info = AccountInfo::new(
			&ADMIN_PUBKEY,
			true,
			false,
			admin_lamports,
			admin_data,
			&system_program::ID,
			false,
			u64::MAX,
		);
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
		let bit_mint_info = AccountInfo::new(
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
			&spl_associated_token_account::ID,
			true,
			u64::MAX,
		);
		let token_program_info = AccountInfo::new(
			token_program_key,
			false,
			false,
			token_program_lamports,
			token_program_data,
			&spl_token_2022::ID,
			true,
			u64::MAX,
		);
		let system_program_info = AccountInfo::new(
			&system_program::ID,
			false,
			false,
			system_program_lamports,
			system_program_data,
			&system_program::ID,
			true,
			u64::MAX,
		);

		[
			admin_info,
			authority_info,
			config_info,
			treasury_info,
			bit_mint_info,
			treasury_bit_token_account_info,
			associated_token_program_info,
			token_program_info,
			system_program_info,
		]
	}
}
