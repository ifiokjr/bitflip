use num_enum::IntoPrimitive;
use solana_program::msg;
use spl_token_2022::extension::ExtensionType;
use steel::*;
use sysvar::rent::Rent;

use super::BitflipInstruction;
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
use crate::SEED_GIBIBIT_MINT;
use crate::SEED_KIBIBIT_MINT;
use crate::SEED_MEBIBIT_MINT;
use crate::SEED_PREFIX;
use crate::SEED_TREASURY;
use crate::TOKEN_DECIMALS;
use crate::cpi::create_associated_token_account;
use crate::cpi::group_member_pointer_initialize;
use crate::cpi::initialize_mint;
use crate::cpi::metadata_pointer_initialize;
use crate::cpi::mint_close_authority_initialize;
// use crate::cpi::token_group_member_initialize;
use crate::cpi::token_metadata_initialize;
use crate::create_pda_config;
use crate::create_pda_mint_bit;
use crate::create_pda_treasury;
use crate::get_treasury_token_account;

pub fn process_token_member_initialize(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
	// parse the instruction data.
	let args = TokenMemberInitialize::try_from_bytes(data)?;
	let member = args.member()?;

	// load accounts
	let [
		authority_info,
		config_info,
		treasury_info,
		mint_group_info,
		mint_member_info,
		treasury_member_token_account_info,
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
	mint_group_info
		.is_writable()?
		.has_owner(token_program_info.key)?;
	mint_member_info.is_empty()?.is_writable()?;
	treasury_member_token_account_info
		.is_empty()?
		.is_writable()?;
	associated_token_program_info.is_program(&spl_associated_token_account::ID)?;
	token_program_info.is_program(&spl_token_2022::ID)?;
	system_program_info.is_program(&system_program::ID)?;

	let config = config_info.as_account::<ConfigState>(&ID)?;
	let config_key = create_pda_config(config.bump)?;
	let treasury_key = create_pda_treasury(config.treasury_bump)?;
	let mint_group_key = create_pda_mint_bit(config.mint_bit_bump)?;
	let seeds = member.seeds();
	let mint_seeds = &[seeds[0], seeds[1], &[member.bump(config)]];
	let mint_member_key = Pubkey::create_program_address(mint_seeds, &ID)?;
	let treasury_token_account_key = get_treasury_token_account(&treasury_key, &mint_member_key);
	let treasury_seeds = &[SEED_PREFIX, SEED_TREASURY, &[config.treasury_bump]];

	if authority_info.key.ne(&config.authority) {
		return Err(BitflipError::Unauthorized.into());
	}

	if config_key.ne(config_info.key)
		|| treasury_key.ne(treasury_info.key)
		|| mint_group_key.ne(mint_group_info.key)
		|| mint_member_key.ne(mint_member_info.key)
		|| treasury_token_account_key.ne(treasury_member_token_account_info.key)
	{
		return Err(ProgramError::InvalidSeeds);
	}

	member.initialize_mint(
		mint_group_info,
		mint_member_info,
		treasury_member_token_account_info,
		authority_info,
		treasury_info,
		token_program_info,
		system_program_info,
		config,
		treasury_seeds,
	)?;

	Ok(())
}

#[repr(u8)]
#[derive(Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
pub enum TokenMember {
	Kibibit = 0,
	Mebibit = 1,
	Gibibit = 2,
}

impl TokenMember {
	pub fn name(&self) -> String {
		match self {
			TokenMember::Kibibit => KIBIBIT_TOKEN_NAME.to_string(),
			TokenMember::Mebibit => MEBIBIT_TOKEN_NAME.to_string(),
			TokenMember::Gibibit => GIBIBIT_TOKEN_NAME.to_string(),
		}
	}

	pub fn symbol(&self) -> String {
		match self {
			TokenMember::Kibibit => KIBIBIT_TOKEN_SYMBOL.to_string(),
			TokenMember::Mebibit => MEBIBIT_TOKEN_SYMBOL.to_string(),
			TokenMember::Gibibit => GIBIBIT_TOKEN_SYMBOL.to_string(),
		}
	}

	pub fn uri(&self) -> String {
		match self {
			TokenMember::Kibibit => KIBIBIT_TOKEN_URI.to_string(),
			TokenMember::Mebibit => MEBIBIT_TOKEN_URI.to_string(),
			TokenMember::Gibibit => GIBIBIT_TOKEN_URI.to_string(),
		}
	}

	pub fn seeds(&self) -> [&[u8]; 2] {
		match self {
			TokenMember::Kibibit => [SEED_PREFIX, SEED_KIBIBIT_MINT],
			TokenMember::Mebibit => [SEED_PREFIX, SEED_MEBIBIT_MINT],
			TokenMember::Gibibit => [SEED_PREFIX, SEED_GIBIBIT_MINT],
		}
	}

	pub fn bump(&self, config: &ConfigState) -> u8 {
		match self {
			TokenMember::Kibibit => config.mint_kibibit_bump,
			TokenMember::Mebibit => config.mint_mebibit_bump,
			TokenMember::Gibibit => config.mint_gibibit_bump,
		}
	}

	#[allow(clippy::too_many_arguments)]
	pub fn initialize_mint<'info>(
		&self,
		_mint_group_info: &AccountInfo<'info>,
		mint_member_info: &AccountInfo<'info>,
		treasury_member_token_account_info: &AccountInfo<'info>,
		payer_info: &AccountInfo<'info>,
		treasury_info: &AccountInfo<'info>,
		token_program_info: &AccountInfo<'info>,
		system_program_info: &AccountInfo<'info>,
		config: &ConfigState,
		treasury_seeds: &[&[u8]],
	) -> ProgramResult {
		let rent_sysvar = Rent::get()?;
		let name = self.name();

		allocate_account_with_bump(
			mint_member_info,
			system_program_info,
			payer_info,
			get_member_mint_space()?,
			token_program_info.key,
			&self.seeds(),
			self.bump(config),
		)?;

		msg!("{}: initialize metadata pointer mint extension", name);
		metadata_pointer_initialize(mint_member_info, treasury_info, token_program_info, &[])?;

		msg!("{}: initialize mint close authority mint extension", name);
		mint_close_authority_initialize(mint_member_info, treasury_info, token_program_info, &[])?;

		msg!("{}: initialize group member pointer mint extension", name);
		group_member_pointer_initialize(mint_member_info, treasury_info, token_program_info, &[])?;

		msg!("{}: initialize mint", name);
		initialize_mint(
			mint_member_info,
			token_program_info,
			treasury_info,
			TOKEN_DECIMALS,
			&[],
		)?;

		msg!("{}: initialize token metadata", name);
		token_metadata_initialize(
			mint_member_info,
			treasury_info,
			token_program_info,
			self.name(),
			self.symbol(),
			self.uri(),
			&[treasury_seeds],
		)?;

		msg!("{}: initialize group member", name);
		// token_group_member_initialize(
		// 	token_program_info,
		// 	mint_member_info,
		// 	mint_member_info,
		// 	treasury_info,
		// 	_mint_group_info,
		// 	treasury_info,
		// 	&[treasury_seeds],
		// )?;

		msg!("{}: get mint account size", name);
		let extra_lamports = rent_sysvar
			.minimum_balance(mint_member_info.data_len())
			.checked_sub(mint_member_info.lamports())
			.ok_or(ProgramError::ArithmeticOverflow)?;

		if extra_lamports > 0 {
			msg!("{}: collect extra lamports", name);
			mint_member_info.collect(extra_lamports, payer_info)?;
		}

		msg!("{}: create treasury associated token account", name);
		create_associated_token_account(
			payer_info,
			treasury_member_token_account_info,
			treasury_info,
			mint_member_info,
			token_program_info,
			system_program_info,
			&[treasury_seeds],
		)?;

		Ok(())
	}
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub struct TokenMemberInitialize {
	pub member: u8,
}

impl TokenMemberInitialize {
	pub fn new(member: TokenMember) -> Self {
		Self {
			member: member.into(),
		}
	}

	pub fn member(&self) -> Result<TokenMember, ProgramError> {
		TokenMember::try_from(self.member).or(Err(ProgramError::InvalidInstructionData))
	}
}

instruction!(BitflipInstruction, TokenMemberInitialize);

const MEMBER_EXTENSION_TYPES: &[ExtensionType] = &[
	ExtensionType::MetadataPointer,
	ExtensionType::MintCloseAuthority,
	ExtensionType::GroupMemberPointer,
];

pub fn get_member_mint_space() -> Result<usize, ProgramError> {
	let mint_space = ExtensionType::try_calculate_account_len::<spl_token_2022::state::Mint>(
		MEMBER_EXTENSION_TYPES,
	)?;
	Ok(mint_space)
}

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
	use crate::get_treasury_kibibit_token_account;
	use crate::leak;

	#[test_log::test]
	fn validation_should_pass() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::UnsupportedSysvar);

		Ok(())
	}

	#[test_log::test]
	fn should_have_enough_accounts() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_token_member_initialize(&accounts[0..8], &[0]);
		check!(result.unwrap_err() == ProgramError::NotEnoughAccountKeys);

		Ok(())
	}

	#[test_log::test]
	fn should_have_valid_args() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_token_member_initialize(&accounts, &[]);
		check!(result.unwrap_err() == ProgramError::InvalidInstructionData);

		Ok(())
	}

	#[test_log::test]
	fn should_have_valid_token_member() -> anyhow::Result<()> {
		let accounts = create_account_infos();
		let result = process_token_member_initialize(&accounts, &[3]);
		check!(result.unwrap_err() == ProgramError::InvalidInstructionData);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_signer() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.is_signer = false;

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.is_writable = false;

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn authority_should_be_from_config() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let authority_info = &mut accounts[0];
		authority_info.key = leak(Pubkey::new_unique());

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::Custom(BitflipError::Unauthorized.into()));

		Ok(())
	}

	#[test_log::test]
	fn config_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[1];
		config_info.key = leak(Pubkey::new_unique());

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn config_should_have_valid_data() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[1];
		config_info.data = Rc::new(RefCell::new(leak(vec![1u8; 8])));

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidAccountData);

		Ok(())
	}

	#[test_log::test]
	fn config_should_have_valid_owner() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let config_info = &mut accounts[1];
		config_info.owner = leak(Pubkey::new_unique());

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn treasury_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[2];
		treasury_info.key = leak(Pubkey::new_unique());

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_should_be_system_account() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_info = &mut accounts[2];
		treasury_info.owner = &ID;

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidAccountOwner);

		Ok(())
	}

	#[test_log::test]
	fn mint_group_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_group_info = &mut accounts[3];
		mint_group_info.key = leak(Pubkey::new_unique());

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_group_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_group_info = &mut accounts[3];
		mint_group_info.is_writable = false;

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn mint_member_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_member_info = &mut accounts[4];
		mint_member_info.key = leak(Pubkey::new_unique());

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn mint_member_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let mint_member_info = &mut accounts[4];
		mint_member_info.is_writable = false;

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	#[test_log::test]
	fn treasury_member_token_account_should_be_pda() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_member_token_account_info = &mut accounts[5];
		treasury_member_token_account_info.key = leak(Pubkey::new_unique());

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::InvalidSeeds);

		Ok(())
	}

	#[test_log::test]
	fn treasury_member_token_account_should_be_writable() -> anyhow::Result<()> {
		let mut accounts = create_account_infos();
		let treasury_member_token_account_info = &mut accounts[5];
		treasury_member_token_account_info.is_writable = false;

		let result = process_token_member_initialize(&accounts, &[0]);
		check!(result.unwrap_err() == ProgramError::MissingRequiredSignature);

		Ok(())
	}

	fn create_account_infos() -> [AccountInfo<'static>; 9] {
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
		let mint_group_key = leak(get_pda_mint_bit().0);
		let mint_group_lamports = leak(100_000_000);
		let mint_group_data = leak(vec![]);
		let mint_member_key = leak(get_pda_mint_kibibit().0);
		let mint_member_lamports = leak(0);
		let mint_member_data = leak(vec![]);
		let treasury_member_token_account_key = leak(get_treasury_kibibit_token_account());
		let treasury_member_token_account_lamports = leak(0);
		let treasury_member_token_account_data = leak(vec![]);
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
		let mint_group_info = AccountInfo::new(
			mint_group_key,
			false,
			true,
			mint_group_lamports,
			mint_group_data,
			&spl_token_2022::ID,
			false,
			u64::MAX,
		);
		let mint_member_info = AccountInfo::new(
			mint_member_key,
			false,
			true,
			mint_member_lamports,
			mint_member_data,
			&system_program::ID,
			false,
			0,
		);
		let treasury_bit_token_account_info = AccountInfo::new(
			treasury_member_token_account_key,
			false,
			true,
			treasury_member_token_account_lamports,
			treasury_member_token_account_data,
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
			mint_group_info,
			mint_member_info,
			treasury_bit_token_account_info,
			associated_token_program_info,
			token_program_info,
			system_program_info,
		]
	}
}
