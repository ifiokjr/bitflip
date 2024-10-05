use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022;
use anchor_spl::token_2022::MintTo;
use anchor_spl::token_interface::Mint;
use anchor_spl::token_interface::Token2022;
use anchor_spl::token_interface::TokenAccount;
use anchor_spl::token_interface::TokenMetadataInitialize;
use anchor_spl::token_interface::token_metadata_initialize;

pub use crate::constants::*;
pub use crate::errors::*;
pub use crate::idls::*;
pub use crate::types::*;
pub use crate::utils::*;

pub mod constants;
pub mod errors;
pub mod idls;
pub mod types;
pub mod utils;

declare_id!("5AuNvfV9Xi9gskJpW2qQJndQkFcwbWNV6fjaf2VvuEcM");

#[allow(clippy::needless_pass_by_value)]
#[program]
pub mod bitflip_program {
	use super::*;

	/// Initialize the configuration for the program.
	pub fn initialize_config(
		mut ctx: Context<InitializeConfig>,
		props: InitializeConfigProps,
	) -> AnchorResult {
		InitializeConfig::initialize_config_handler(&mut ctx, props)
	}

	pub fn initialize_token(
		mut ctx: Context<InitializeToken>,
		props: InitializeTokenProps,
	) -> AnchorResult {
		InitializeToken::initialize_token_handler(&mut ctx, props)
	}

	/// This will initialize the meta state for the bits.
	pub fn initialize_bits_meta(mut ctx: Context<InitializeBitsMeta>) -> AnchorResult {
		InitializeBitsMeta::initialize_bits_meta_handler(&mut ctx)
	}

	/// This will initialize a single section of the bits data. It should be
	/// called 16 times with the correct index of the data chunch.
	pub fn initialize_bits_data_section(
		ctx: Context<InitializeBitsDataSection>,
		section: u8,
	) -> AnchorResult {
		InitializeBitsDataSection::initialize_bits_data_section_handler(ctx, section)
	}

	/// Flip bits based on the provided props.
	pub fn set_bits(mut ctx: Context<SetBits>, props: SetBitsProps) -> AnchorResult {
		SetBits::set_bits_handler(&mut ctx, &props)
	}
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
	#[account(
		init,
		payer = admin,
		space = ConfigState::space() * 2,
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump
	)]
	pub config: Account<'info, ConfigState>,
	/// The hard-coded account that is used to initialize the program config
	/// once.
	#[account(
		mut,
		address = ADMIN_PUBKEY @ BitflipError::UnauthorizedAdmin
	)]
	pub admin: Signer<'info>,
	#[account(
		seeds = [SEED_PREFIX, SEED_TREASURY],
		bump
	)]
	pub treasury: SystemAccount<'info>,
	/// This is needed for initializing the bit state.
	#[account(address = system_program::ID)]
	pub system_program: Program<'info, System>,
}

impl InitializeConfig<'_> {
	pub fn initialize_config_handler(
		ctx: &mut Context<Self>,
		props: InitializeConfigProps,
	) -> AnchorResult {
		require_keys_neq!(
			props.authority,
			Pubkey::default(),
			BitflipError::InvalidAccount,
		);

		let config_state = props.into_launchpad_state(ctx.bumps.config, ctx.bumps.treasury);
		ctx.accounts.config.set_inner(config_state);

		Ok(())
	}
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeConfigProps {
	/// The authority is the solana account which will have rights to update
	/// this configuration.
	pub authority: Pubkey,
}

impl From<InitializeConfigProps> for instruction::InitializeConfig {
	fn from(props: InitializeConfigProps) -> Self {
		Self { props }
	}
}

impl InitializeConfigProps {
	pub fn into_launchpad_state(self, bump: u8, treasury_bump: u8) -> ConfigState {
		ConfigState {
			authority: self.authority,
			lamports_per_bit: LAMPORTS_PER_BIT,
			bump,
			treasury_bump,
			mint_bump: 0,
			bits_index: 0,
		}
	}
}

#[derive(Accounts)]
#[instruction(props: InitializeTokenProps)]
pub struct InitializeToken<'info> {
	/// The program configuration.
	#[account(
		mut,
		has_one = authority,
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump = config.bump,
	)]
	pub config: Box<Account<'info, ConfigState>>,
	/// The mint authority.
	#[account(mut, signer)]
	pub authority: SystemAccount<'info>,
	/// The token mint account.
	#[account(
	  init,
	  payer =	authority,
	  seeds = [SEED_PREFIX, SEED_MINT],
	  bump,
	  mint::token_program = token_program,
	  mint::decimals = TOKEN_DECIMALS,
	  mint::authority = authority,
	  mint::freeze_authority = authority,
	  extensions::metadata_pointer::authority = authority,
	  extensions::metadata_pointer::metadata_address = mint,
	  extensions::close_authority::authority = authority
  )]
	pub mint: Box<InterfaceAccount<'info, Mint>>,
	/// The treasury account.
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_TREASURY],
		bump = config.treasury_bump
	)]
	pub treasury: SystemAccount<'info>,
	/// The associated token account for the treasury which will hold the minted
	/// tokens.
	#[account(
	  init,
	  payer =	authority,
	  associated_token::token_program = token_program,
	  associated_token::mint = mint,
	  associated_token::authority = treasury,
  )]
	pub treasury_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
	/// The program for associated tokens
	pub associated_token_program: Program<'info, AssociatedToken>,
	/// The token program.
	pub token_program: Program<'info, Token2022>,
	/// Needed in case a reallocation is required for the project memory.
	pub system_program: Program<'info, System>,
}

impl InitializeToken<'_> {
	pub fn initialize_token_handler(
		ctx: &mut Context<Self>,
		props: InitializeTokenProps,
	) -> AnchorResult {
		let rent_sysvar = Rent::get()?;
		let mint = ctx.accounts.mint.to_account_info();

		ctx.accounts.config.mint_bump = ctx.bumps.mint;
		ctx.accounts.initialize_token_metadata(props)?;
		ctx.accounts.mint.reload()?;

		let extra_lamports = rent_sysvar.minimum_balance(mint.data_len()) - mint.get_lamports();

		// transfer minimum rent to mint account when required
		if extra_lamports > 0 {
			let from = ctx.accounts.authority.to_account_info();
			let to = mint.clone();
			let system_program = ctx.accounts.system_program.to_account_info();
			let cpi_context =
				CpiContext::new(system_program, system_program::Transfer { from, to });

			system_program::transfer(cpi_context, extra_lamports)?;
		}

		let to = ctx.accounts.treasury_token_account.to_account_info();
		let authority = ctx.accounts.authority.to_account_info();
		let token_program = ctx.accounts.token_program.to_account_info();

		let cpi_context = CpiContext::new(token_program, MintTo {
			mint,
			to,
			authority,
		});
		let amount = get_token_amount(MAX_TOKENS, TOKEN_DECIMALS)?;

		token_2022::mint_to(cpi_context, amount)?;

		Ok(())
	}

	fn initialize_token_metadata(
		&self,
		InitializeTokenProps { name, symbol, uri }: InitializeTokenProps,
	) -> AnchorResult {
		let cpi_accounts = TokenMetadataInitialize {
			token_program_id: self.token_program.to_account_info(),
			mint: self.mint.to_account_info(),
			metadata: self.mint.to_account_info(),
			mint_authority: self.authority.to_account_info(),
			update_authority: self.authority.to_account_info(),
		};
		let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
		token_metadata_initialize(cpi_ctx, name, symbol, uri)?;

		Ok(())
	}
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeTokenProps {
	pub name: String,
	pub symbol: String,
	pub uri: String,
}

impl From<InitializeTokenProps> for instruction::InitializeToken {
	fn from(props: InitializeTokenProps) -> Self {
		instruction::InitializeToken { props }
	}
}

#[derive(Accounts)]
pub struct InitializeBitsMeta<'info> {
	/// The program configuration.
	#[account(
		mut,
		has_one = authority,
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump = config.bump,
	)]
	pub config: Box<Account<'info, ConfigState>>,
	/// Can't initialize the full state at one time, must incrementally add the
	/// state.
	#[account(
		init,
		payer = authority,
		space = BitsMetaState::space(),
		seeds = [SEED_PREFIX, SEED_BITS, &config.bits_index.to_le_bytes()],
		bump
	)]
	pub bits_meta: Box<Account<'info, BitsMetaState>>,
	/// The authority that is able to sign for updates to the config and
	/// initiate new games.
	#[account(mut)]
	pub authority: Signer<'info>,
	/// This is needed for initializing the bit state.
	#[account(address = system_program::ID)]
	pub system_program: Program<'info, System>,
}

impl InitializeBitsMeta<'_> {
	pub fn initialize_bits_meta_handler(ctx: &mut Context<Self>) -> AnchorResult {
		let bits_meta = &mut ctx.accounts.bits_meta;
		bits_meta.start_time = 0;
		bits_meta.flips = 0;
		bits_meta.on = 0;
		bits_meta.off = BITS_TOTAL as u32;
		bits_meta.index = ctx.accounts.config.bits_index;
		bits_meta.bump = ctx.bumps.bits_meta;
		bits_meta.sections = 0;

		Ok(())
	}
}

#[derive(Accounts)]
#[instruction(section: u8)]
pub struct InitializeBitsDataSection<'info> {
	/// The program configuration.
	#[account(
		mut,
		has_one = authority,
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump = config.bump,
	)]
	pub config: Box<Account<'info, ConfigState>>,
	/// The meta data for the full bits state.
	#[account(
		mut,
		constraint = usize::from(section) < BITS_DATA_SECTIONS @ BitflipError::AllSectionsInitialized,
		seeds = [SEED_PREFIX, SEED_BITS, &bits_meta.index.to_le_bytes()],
		bump = bits_meta.bump
	)]
	pub bits_meta: Box<Account<'info, BitsMetaState>>,
	/// This is a section of the bits data being initialized.
	#[account(
		init,
		payer = authority,
		space = BitsDataSectionState::space(),
		seeds = [SEED_PREFIX, SEED_BITS, &config.bits_index.to_le_bytes(), SEED_BITS_SECTION, &section.to_le_bytes()],
		bump
	)]
	pub bits_data_section: Box<Account<'info, BitsDataSectionState>>,
	/// The authority that is able to sign for updates to the config and
	/// initiate new games.
	#[account(mut)]
	pub authority: Signer<'info>,
	/// This is needed for initializing the bit state.
	#[account(address = system_program::ID)]
	pub system_program: Program<'info, System>,
}

impl InitializeBitsDataSection<'_> {
	#[access_control(validate_data_section(section))]
	pub fn initialize_bits_data_section_handler(ctx: Context<Self>, section: u8) -> AnchorResult {
		let bits_meta = &mut ctx.accounts.bits_meta;
		let bits_data_section = &mut ctx.accounts.bits_data_section;

		bits_data_section.section = section;
		bits_data_section.bump = ctx.bumps.bits_data_section;
		// TODO: Initialize the bits data section if possible on stack instead of heap.
		bits_meta.sections += 1;

		if usize::from(bits_meta.sections) == BITS_DATA_SECTIONS {
			bits_meta.start_time = Clock::get()?.unix_timestamp;
		}

		Ok(())
	}
}

impl From<u8> for instruction::InitializeBitsDataSection {
	fn from(section: u8) -> Self {
		instruction::InitializeBitsDataSection { section }
	}
}

#[derive(Accounts)]
#[instruction(props: SetBitsProps)]
pub struct SetBits<'info> {
	/// The program configuration.
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump = config.bump,
	)]
	pub config: Box<Account<'info, ConfigState>>,
	/// The token mint account.
	#[account(
	  seeds = [SEED_PREFIX, SEED_MINT],
	  bump = config.mint_bump,
  )]
	pub mint: Box<InterfaceAccount<'info, Mint>>,
	/// The treasury account which will transfer the spl tokens to the player.
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_TREASURY],
		bump = config.treasury_bump
	)]
	pub treasury: SystemAccount<'info>,
	#[account(
		mut,
	  associated_token::token_program = token_program,
		associated_token::mint = mint,
		associated_token::authority = treasury,
	)]
	pub treasury_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
	/// The meta data for the full bits state.
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_BITS, &bits_meta.index.to_le_bytes()],
		bump = bits_meta.bump
	)]
	pub bits_meta: Box<Account<'info, BitsMetaState>>,
	/// The data for this section of the bit canvas.
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_BITS, &bits_meta.index.to_le_bytes(), SEED_BITS_SECTION, &props.section.to_le_bytes()],
		bump = bits_data_section.bump,
	)]
	pub bits_data_section: Box<Account<'info, BitsDataSectionState>>,
	/// The player of the bit games
	#[account(mut)]
	pub player: Signer<'info>,
	/// The associated token account for the main authority.
	#[account(
	  init_if_needed,
	  payer =	player,
	  associated_token::token_program = token_program,
	  associated_token::mint = mint,
	  associated_token::authority = player,
  )]
	pub player_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
	/// The program for associated tokens
	pub associated_token_program: Program<'info, AssociatedToken>,
	/// The token program.
	pub token_program: Program<'info, Token2022>,
	/// Needed for cpi payment instructions instructions.
	#[account(address = system_program::ID)]
	pub system_program: Program<'info, System>,
}

impl SetBits<'_> {
	pub fn set_bits_handler(ctx: &mut Context<Self>, props: &SetBitsProps) -> AnchorResult {
		props.validate()?;
		ctx.accounts.update(props)
	}

	fn update(&mut self, props: &SetBitsProps) -> AnchorResult {
		let changes = self.bits_data_section.set_bits(props)?;
		let flipped_bits = changes.total()?;

		self.bits_meta.flip_on(changes.on)?;
		self.bits_meta.flip_off(changes.off)?;
		self.transfer_from_treasury(flipped_bits)?;
		self.transfer_to_treasury(
			flipped_bits,
			props
				.get_multiplier()
				.checked_mul(self.config.lamports_per_bit)
				.ok_or(ProgramError::ArithmeticOverflow)?,
		)
	}
}

impl<'info> TransferToTreasury<'info> for SetBits<'info> {
	fn treasury(&self) -> AccountInfo<'info> {
		self.treasury.to_account_info()
	}

	fn player(&self) -> AccountInfo<'info> {
		self.player.to_account_info()
	}

	fn system_program(&self) -> AccountInfo<'info> {
		self.system_program.to_account_info()
	}
}

impl<'info> TransferFromTreasury<'info> for SetBits<'info> {
	fn mint(&self) -> AccountInfo<'info> {
		self.mint.to_account_info()
	}

	fn treasury(&self) -> AccountInfo<'info> {
		self.treasury.to_account_info()
	}

	fn treasury_bump(&self) -> u8 {
		self.config.treasury_bump
	}

	fn treasury_token_account(&self) -> AccountInfo<'info> {
		self.treasury_token_account.to_account_info()
	}

	fn player_token_account(&self) -> AccountInfo<'info> {
		self.player_token_account.to_account_info()
	}

	fn token_program(&self) -> AccountInfo<'info> {
		self.token_program.to_account_info()
	}
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
#[cfg_attr(feature = "client", derive(PartialEq, Eq, Hash))]
pub struct SetBitsProps {
	/// The data section being updated.
	pub section: u8,
	/// The index of the bit being set.
	pub index: u16,
	/// The new bit values.
	pub variant: SetBitsVariant,
}

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
#[cfg_attr(feature = "client", derive(PartialEq, Eq, Hash))]
pub enum SetBitsVariant {
	/// The offset of the bit being set.
	On(u16),
	/// The offset of the bit being unset.
	Off(u16),
	/// The value of the 16bit value being set.
	Bit16(u16),
	/// The values of the 256bit value being set.
	Bits256(Vec<u16>),
}

impl SetBitsProps {
	pub fn validate(&self) -> AnchorResult {
		validate_data_section(self.section)?;
		validate_data_section_index(self.index)?;

		match &self.variant {
			SetBitsVariant::On(offset) | SetBitsVariant::Off(offset) => {
				validate_bit_offset(*offset)?;
			}
			SetBitsVariant::Bit16(_) => {}
			SetBitsVariant::Bits256(bits_array) => {
				let index_with_offset = self
					.index
					.checked_add(15)
					.ok_or(ProgramError::ArithmeticOverflow)?;

				validate_bit_array_length(bits_array, 16)?;
				validate_256bit_data_section_index(self.index)?;
				validate_data_section_index(index_with_offset)?;
			}
		}

		Ok(())
	}

	pub fn get_multiplier(&self) -> u64 {
		match &self.variant {
			SetBitsVariant::On(_) | SetBitsVariant::Off(_) => 1,
			SetBitsVariant::Bit16(_) => 5,
			SetBitsVariant::Bits256(_) => 10,
		}
	}

	/// Check whether this bit setting prop overrides the other.
	pub fn contains(&self, other: &Self) -> bool {
		match &self.variant {
			SetBitsVariant::On(self_offset) | SetBitsVariant::Off(self_offset) => {
				match &other.variant {
					SetBitsVariant::On(other_offset) | SetBitsVariant::Off(other_offset) => {
						self.index == other.index
							&& self_offset == other_offset
							&& self.section == other.section
					}
					_ => false,
				}
			}
			SetBitsVariant::Bit16(_) => {
				match &other.variant {
					SetBitsVariant::On(_) | SetBitsVariant::Off(_) | SetBitsVariant::Bit16(_) => {
						self.index == other.index && self.section == other.section
					}
					SetBitsVariant::Bits256(_) => false,
				}
			}
			SetBitsVariant::Bits256(_) => {
				self.index == other.index && self.section == other.section
			}
		}
	}
}

impl From<SetBitsProps> for instruction::SetBits {
	fn from(props: SetBitsProps) -> Self {
		instruction::SetBits { props }
	}
}

#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "serde", ::serde_with::serde_as)]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct ConfigState {
	/// The authority which can update this config.
	#[cfg_attr(
		feature = "serde",
		serde(with = "::serde_with::As::<serde_with::DisplayFromStr>")
	)]
	pub authority: Pubkey,
	/// The number of lamports per bit change.
	pub lamports_per_bit: u64,
	/// Store the bump to save compute units.
	pub bump: u8,
	/// The treasury account bump where fees are sent and where the minted
	/// tokens are transferred.
	pub treasury_bump: u8,
	/// The mint account bump.
	pub mint_bump: u8,
	/// There will be a maximum of 4 games.
	pub bits_index: u8,
}

impl ConfigState {
	pub const fn space() -> usize {
		SPACE_DISCRIMINATOR + Self::INIT_SPACE
	}
}

/// Adding [`BitState::on`] to [`BitState::off`] should always equal `1_000_000`
#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BitsMetaState {
	/// The start time. If 0 then it hasn't started yet. Using an `Option` here
	/// would waste an extra byte.
	pub start_time: i64,
	/// The number of bit flips that have occurred.
	pub flips: u64,
	/// The number of bits that are on.
	pub on: u32,
	/// The number of bits that are off.
	pub off: u32,
	/// The index of this currently played game.
	pub index: u8,
	/// The bump for this account.
	pub bump: u8,
	/// The number of sections initialized.
	pub sections: u8,
}

impl BitsMetaState {
	pub fn space() -> usize {
		SPACE_DISCRIMINATOR + Self::INIT_SPACE
	}

	pub fn end_time(&self) -> i64 {
		self.start_time.saturating_add(SESSION_DURATION)
	}

	pub fn started(&self) -> bool {
		self.start_time > 0
	}

	pub fn ended(&self, current_time: i64) -> bool {
		self.started() && current_time > self.end_time()
	}

	pub fn flip_on(&mut self, changed_bits: u32) -> AnchorResult {
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
			.checked_add(u64::from(changed_bits))
			.ok_or(ProgramError::ArithmeticOverflow)?;

		Ok(())
	}

	pub fn flip_off(&mut self, changed_bits: u32) -> AnchorResult {
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
			.checked_add(u64::from(changed_bits))
			.ok_or(ProgramError::ArithmeticOverflow)?;

		Ok(())
	}
}

/// The data for each section of the the data. The total data is split into 16
/// sections and this is one of the sections.
#[account]
#[derive(InitSpace)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct BitsDataSectionState {
	/// The state of the bits that are represented as checkboxes on the
	/// frontend.
	#[cfg_attr(feature = "serde", serde(with = "serde_big_array::BigArray"))]
	pub data: [u16; BITS_DATA_SECTION_LENGTH],
	/// The section index for this account.
	pub section: u8,
	/// The bump for this account.
	pub bump: u8,
}

impl BitsDataSectionState {
	pub fn space() -> usize {
		SPACE_DISCRIMINATOR + Self::INIT_SPACE
	}

	// fn get_data_slice(&mut self, props: &SetBitsProps) -> &mut [u16] {
	// 	let index = props.index as usize;

	// 	match &props.variant {
	// 		SetBitsVariant::On(_) | SetBitsVariant::Off(_) | SetBitsVariant::Bit16(_)
	// => { 			&mut self.data[index..=index]
	// 		}
	// 		SetBitsVariant::Bits256(_) => {
	// 			let index_with_offset = props.index.saturating_add(15) as usize;
	// 			&mut self.data[index..=index_with_offset]
	// 		}
	// 	}
	// }

	pub fn set_bits(&mut self, props: &SetBitsProps) -> Result<BitChanges> {
		let index = props.index as usize;
		let mut changes = BitChanges::default();

		match &props.variant {
			SetBitsVariant::On(offset) => {
				let current = self.data[index..=index][0];
				let bit = 1 << *offset;
				let updated = current | bit;

				require!(updated != current, BitflipError::BitsUnchanged);

				self.data[index..=index].copy_from_slice(&[updated]);
				changes.on = 1;
			}

			SetBitsVariant::Off(offset) => {
				let current = self.data[index..=index][0];
				let bit = 1 << *offset;
				let updated = current & !bit;

				require!(updated != current, BitflipError::BitsUnchanged);

				self.data[index..=index].copy_from_slice(&[updated]);
				changes.off = 1;
			}

			SetBitsVariant::Bit16(bits) => {
				let slice = &mut self.data[index..=index];
				let previous = slice[0];

				changes += get_bit_changes(previous, *bits)?;
				slice.copy_from_slice(&[*bits]);
			}

			SetBitsVariant::Bits256(bits_array) => {
				let index_with_offset = props.index.saturating_add(15) as usize;
				let previous_bits = &mut self.data[index..=index_with_offset];

				for (ii, next) in bits_array.iter().copied().enumerate() {
					let previous = previous_bits[ii];
					changes += get_bit_changes(previous, next)?;
				}

				previous_bits.copy_from_slice(bits_array);
			}
		}

		Ok(changes)
	}
}

#[account]
#[derive(InitSpace)]
pub struct BitCreatorState {
	/// The initial creator of this state.
	pub creator: Pubkey,
	/// The last user to update this state.
	pub updater: Pubkey,
	/// The timestamp of the update.
	pub timestamp: i64,
	/// The bump for the creator state.
	pub bump: u8,
}

impl BitCreatorState {
	pub fn space() -> usize {
		SPACE_DISCRIMINATOR + Self::INIT_SPACE
	}
}
