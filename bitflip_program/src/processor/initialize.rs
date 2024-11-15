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
use crate::GIBIBIT_TOKEN_NAME;
use crate::GIBIBIT_TOKEN_SYMBOL;
use crate::GIBIBIT_TOKEN_URI;
use crate::ID;
use crate::KIBIBIT_TOKEN_NAME;
use crate::KIBIBIT_TOKEN_SYMBOL;
use crate::KIBIBIT_TOKEN_URI;
use crate::MEBIBIT_TOKEN_NAME;
use crate::MEBIBIT_TOKEN_SYMBOL;
use crate::MEBIBIT_TOKEN_URI;
use crate::SEED_BIT_MINT;
use crate::SEED_CONFIG;
use crate::SEED_GIBIBIT_MINT;
use crate::SEED_KIBIBIT_MINT;
use crate::SEED_MEBIBIT_MINT;
use crate::SEED_PREFIX;
use crate::SEED_TREASURY;
use crate::TOKEN_DECIMALS;
use crate::TOTAL_TOKENS;
use crate::cpi::create_associated_token_account;
use crate::cpi::group_member_pointer_initialize;
use crate::cpi::group_pointer_initialize;
use crate::cpi::initialize_mint2;
use crate::cpi::metadata_pointer_initialize;
use crate::cpi::mint_close_authority_initialize;
use crate::cpi::mint_to;
use crate::cpi::token_group_initialize;
use crate::cpi::token_group_member_initialize;
use crate::cpi::token_metadata_initialize;
use crate::get_pda_config;
use crate::get_pda_mint_bit;
use crate::get_pda_mint_gibibit;
use crate::get_pda_mint_kibibit;
use crate::get_pda_mint_mebibit;
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
		mint_kibibit_info,
		treasury_kibibit_token_account_info,
		mint_mebibit_info,
		treasury_mebibit_token_account_info,
		mint_gibibit_info,
		treasury_gibibit_token_account_info,
		associated_token_program_info,
		token_program_info,
		system_program_info,
	] = accounts
	else {
		return Err(ProgramError::NotEnoughAccountKeys);
	};

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
	mint_kibibit_info.is_empty()?.is_writable()?;
	treasury_kibibit_token_account_info
		.is_empty()?
		.is_writable()?;
	mint_mebibit_info.is_empty()?.is_writable()?;
	treasury_mebibit_token_account_info
		.is_empty()?
		.is_writable()?;
	mint_gibibit_info.is_empty()?.is_writable()?;
	treasury_gibibit_token_account_info
		.is_empty()?
		.is_writable()?;
	associated_token_program_info.is_program(&spl_associated_token_account::ID)?;
	token_program_info.is_program(&spl_token_2022::ID)?;
	system_program_info.is_program(&system_program::ID)?;

	let (config_key, config_bump) = get_pda_config();
	let (treasury_key, treasury_bump) = get_pda_treasury();
	let (mint_bit_key, mint_bit_bump) = get_pda_mint_bit();
	let treasury_bit_token_account_key = get_treasury_token_account(&treasury_key, &mint_bit_key);
	let (mint_kibibit_key, mint_kibibit_bump) = get_pda_mint_kibibit();
	let treasury_kibibit_token_account_key =
		get_treasury_token_account(&treasury_key, &mint_kibibit_key);
	let (mint_mebibit_key, mint_mebibit_bump) = get_pda_mint_mebibit();
	let treasury_mebibit_token_account_key =
		get_treasury_token_account(&treasury_key, &mint_mebibit_key);
	let (mint_gibibit_key, mint_gibibit_bump) = get_pda_mint_gibibit();
	let treasury_gibibit_token_account_key =
		get_treasury_token_account(&treasury_key, &mint_gibibit_key);
	let treasury_seeds = &[SEED_PREFIX, SEED_TREASURY, &[treasury_bump]];

	if config_info.key.ne(&config_key)
		|| treasury_info.key.ne(&treasury_key)
		|| mint_bit_info.key.ne(&mint_bit_key)
		|| treasury_bit_token_account_info
			.key
			.ne(&treasury_bit_token_account_key)
		|| mint_kibibit_info.key.ne(&mint_kibibit_key)
		|| treasury_kibibit_token_account_info
			.key
			.ne(&treasury_kibibit_token_account_key)
		|| mint_mebibit_info.key.ne(&mint_mebibit_key)
		|| treasury_mebibit_token_account_info
			.key
			.ne(&treasury_mebibit_token_account_key)
		|| mint_gibibit_info.key.ne(&mint_gibibit_key)
		|| treasury_gibibit_token_account_info
			.key
			.ne(&treasury_gibibit_token_account_key)
	{
		return Err(ProgramError::InvalidSeeds);
	}

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
		mint_kibibit_bump,
		mint_mebibit_bump,
		mint_gibibit_bump,
	);

	msg!("transfer sol to treasury for rent exemption");
	// transfer sol to treasury for rent exemption
	let rent_sysvar = Rent::get()?;
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
	initialize_mint2(
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
		get_token_amount(TOTAL_TOKENS, TOKEN_DECIMALS)?,
		&[&treasury_seeds[..]],
	)?;

	initialize_member_mint(
		mint_bit_info,
		mint_kibibit_info,
		treasury_kibibit_token_account_info,
		authority_info,
		treasury_info,
		token_program_info,
		system_program_info,
		&rent_sysvar,
		KIBIBIT_TOKEN_NAME,
		KIBIBIT_TOKEN_SYMBOL,
		KIBIBIT_TOKEN_URI,
		mint_kibibit_bump,
		&[SEED_PREFIX, SEED_KIBIBIT_MINT],
		treasury_seeds,
	)?;
	initialize_member_mint(
		mint_bit_info,
		mint_mebibit_info,
		treasury_mebibit_token_account_info,
		authority_info,
		treasury_info,
		token_program_info,
		system_program_info,
		&rent_sysvar,
		MEBIBIT_TOKEN_NAME,
		MEBIBIT_TOKEN_SYMBOL,
		MEBIBIT_TOKEN_URI,
		mint_mebibit_bump,
		&[SEED_PREFIX, SEED_MEBIBIT_MINT],
		treasury_seeds,
	)?;

	initialize_member_mint(
		mint_bit_info,
		mint_gibibit_info,
		treasury_gibibit_token_account_info,
		authority_info,
		treasury_info,
		token_program_info,
		system_program_info,
		&rent_sysvar,
		GIBIBIT_TOKEN_NAME,
		GIBIBIT_TOKEN_SYMBOL,
		GIBIBIT_TOKEN_URI,
		mint_gibibit_bump,
		&[SEED_PREFIX, SEED_GIBIBIT_MINT],
		treasury_seeds,
	)?;

	Ok(())
}

#[allow(clippy::too_many_arguments)]
fn initialize_member_mint<'info>(
	group_info: &AccountInfo<'info>,
	mint_info: &AccountInfo<'info>,
	treasury_token_account_info: &AccountInfo<'info>,
	payer_info: &AccountInfo<'info>,
	treasury_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	system_program_info: &AccountInfo<'info>,
	rent_sysvar: &Rent,
	name: &str,
	symbol: &str,
	uri: &str,
	mint_bump: u8,
	mint_seeds: &[&[u8]],
	treasury_seeds: &[&[u8]],
) -> ProgramResult {
	msg!("{}: get mint account size", name);
	let extension_types = vec![
		ExtensionType::MetadataPointer,
		ExtensionType::MintCloseAuthority,
		ExtensionType::GroupMemberPointer,
	];
	let mint_space =
		ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(&extension_types)?;
	allocate_account_with_bump(
		mint_info,
		system_program_info,
		payer_info,
		mint_space,
		token_program_info.key,
		mint_seeds,
		mint_bump,
	)?;

	msg!("{}: initialize metadata pointer mint extension", name);
	metadata_pointer_initialize(mint_info, treasury_info, token_program_info, &[])?;

	msg!("{}: initialize mint close authority mint extension", name);
	mint_close_authority_initialize(mint_info, treasury_info, token_program_info, &[])?;

	msg!("{}: initialize group member pointer mint extension", name);
	group_member_pointer_initialize(mint_info, treasury_info, token_program_info, &[])?;

	msg!("{}: initialize mint", name);
	initialize_mint2(
		mint_info,
		token_program_info,
		treasury_info,
		TOKEN_DECIMALS,
		&[],
	)?;

	msg!("{}: initialize token metadata", name);
	token_metadata_initialize(
		mint_info,
		treasury_info,
		token_program_info,
		name.to_string(),
		symbol.to_string(),
		uri.to_string(),
		&[treasury_seeds],
	)?;

	// msg!("{}: initialize group member", name);
	// token_group_member_initialize(
	// 	token_program_info,
	// 	mint_info,
	// 	mint_info,
	// 	treasury_info,
	// 	group_info,
	// 	treasury_info,
	// 	&[treasury_seeds],
	// )?;

	msg!("{}: get mint account size", name);
	let extra_lamports = rent_sysvar
		.minimum_balance(mint_info.data_len())
		.checked_sub(mint_info.lamports())
		.ok_or(ProgramError::ArithmeticOverflow)?;

	if extra_lamports > 0 {
		msg!("{}: collect extra lamports", name);
		mint_info.collect(extra_lamports, payer_info)?;
	}

	msg!("{}: create treasury associated token account", name);
	create_associated_token_account(
		payer_info,
		treasury_token_account_info,
		treasury_info,
		mint_info,
		token_program_info,
		system_program_info,
		&[treasury_seeds],
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
	use crate::get_pda_mint_gibibit;
	use crate::get_pda_mint_mebibit;
	use crate::get_pda_treasury;
	use crate::get_treasury_bit_token_account;
	use crate::get_treasury_gibibit_token_account;
	use crate::get_treasury_kibibit_token_account;
	use crate::get_treasury_mebibit_token_account;
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
	fn mint_bit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_bit_info = &mut accounts[4];
		mint_bit_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_bit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_bit_info = &mut accounts[4];
		mint_bit_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_bit_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_bit_token_account_info = &mut accounts[5];
		treasury_bit_token_account_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_bit_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_bit_token_account_info = &mut accounts[5];
		treasury_bit_token_account_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_kibibit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_kibibit_info = &mut accounts[6];
		mint_kibibit_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_kibibit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_kibibit_info = &mut accounts[6];
		mint_kibibit_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_kibibit_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_kibibit_token_account_info = &mut accounts[7];
		treasury_kibibit_token_account_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_kibibit_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_kibibit_token_account_info = &mut accounts[7];
		treasury_kibibit_token_account_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_mebibit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_mebibit_info = &mut accounts[8];
		mint_mebibit_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_mebibit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_mebibit_info = &mut accounts[8];
		mint_mebibit_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_mebibit_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_mebibit_token_account_info = &mut accounts[9];
		treasury_mebibit_token_account_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_mebibit_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_mebibit_token_account_info = &mut accounts[9];
		treasury_mebibit_token_account_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_gibibit_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_gibibit_info = &mut accounts[10];
		mint_gibibit_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_gibibit_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_gibibit_info = &mut accounts[10];
		mint_gibibit_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_gibibit_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_gibibit_token_account_info = &mut accounts[11];
		treasury_gibibit_token_account_info.key = leak(Pubkey::new_unique());

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_gibibit_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_gibibit_token_account_info = &mut accounts[11];
		treasury_gibibit_token_account_info.is_writable = false;

		let result = process_initialize(&accounts);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 15] {
		let admin_lamports = leak(0);
		let admin_data = leak(vec![]);
		let authority_key = leak(Pubkey::new_unique());
		let authority_lamports = leak(1_000_000_000);
		let authority_data = leak(vec![]);
		let config_key = leak(get_pda_config().0);
		let config_lamports = leak(0);
		let config_data = leak(vec![]);
		let treasury_key = leak(get_pda_treasury().0);
		let treasury_lamports = leak(0);
		let treasury_data = leak(vec![]);
		let mint_bit_key = leak(get_pda_mint_bit().0);
		let mint_bit_lamports = leak(0);
		let mint_bit_data = leak(vec![]);
		let treasury_bit_token_account_key = leak(get_treasury_bit_token_account());
		let treasury_bit_token_account_lamports = leak(0);
		let treasury_bit_token_account_data = leak(vec![]);
		let mint_kibibit_key = leak(get_pda_mint_kibibit().0);
		let mint_kibibit_lamports = leak(0);
		let mint_kibibit_data = leak(vec![]);
		let treasury_kibibit_token_account_key = leak(get_treasury_kibibit_token_account());
		let treasury_kibibit_token_account_lamports = leak(0);
		let treasury_kibibit_token_account_data = leak(vec![]);
		let mint_mebibit_key = leak(get_pda_mint_mebibit().0);
		let mint_mebibit_lamports = leak(0);
		let mint_mebibit_data = leak(vec![]);
		let treasury_mebibit_token_account_key = leak(get_treasury_mebibit_token_account());
		let treasury_mebibit_token_account_lamports = leak(0);
		let treasury_mebibit_token_account_data = leak(vec![]);
		let mint_gibibit_key = leak(get_pda_mint_gibibit().0);
		let mint_gibibit_lamports = leak(0);
		let mint_gibibit_data = leak(vec![]);
		let treasury_gibibit_token_account_key = leak(get_treasury_gibibit_token_account());
		let treasury_gibibit_token_account_lamports = leak(0);
		let treasury_gibibit_token_account_data = leak(vec![]);

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
		let treasury_kibibit_token_account_info = AccountInfo::new(
			treasury_kibibit_token_account_key,
			false,
			true,
			treasury_kibibit_token_account_lamports,
			treasury_kibibit_token_account_data,
			&spl_associated_token_account::ID,
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
		let treasury_mebibit_token_account_info = AccountInfo::new(
			treasury_mebibit_token_account_key,
			false,
			true,
			treasury_mebibit_token_account_lamports,
			treasury_mebibit_token_account_data,
			&spl_associated_token_account::ID,
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
		let treasury_gibibit_token_account_info = AccountInfo::new(
			treasury_gibibit_token_account_key,
			false,
			true,
			treasury_gibibit_token_account_lamports,
			treasury_gibibit_token_account_data,
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
			mint_bit_info,
			treasury_bit_token_account_info,
			mint_kibibit_info,
			treasury_kibibit_token_account_info,
			mint_mebibit_info,
			treasury_mebibit_token_account_info,
			mint_gibibit_info,
			treasury_gibibit_token_account_info,
			associated_token_program_info,
			token_program_info,
			system_program_info,
		]
	}
}
