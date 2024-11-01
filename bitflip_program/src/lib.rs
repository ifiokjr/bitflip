use std::ops::Deref;
use std::ops::DerefMut;

use anchor_lang::prelude::*;
use anchor_lang::solana_program::nonce;
use anchor_lang::solana_program::sysvar::recent_blockhashes::RecentBlockhashes;
use anchor_lang::system_program;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token_2022;
use anchor_spl::token_2022::MintTo;
use anchor_spl::token_interface::Mint;
use anchor_spl::token_interface::Token2022;
use anchor_spl::token_interface::TokenAccount;
use anchor_spl::token_interface::TokenMetadataInitialize;
use anchor_spl::token_interface::token_metadata_initialize;
use program::BitflipProgram;

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
	pub fn initialize_config(mut ctx: Context<InitializeConfig>) -> AnchorResult {
		initialize_config_handler(&mut ctx)
	}

	/// Update the authority for the program.
	/// This will required updating the mint authority for the treasury.
	pub fn update_authority(mut ctx: Context<UpdateAuthority>) -> AnchorResult {
		update_authority_handler(&mut ctx)
	}

	/// Initialize the token account. This must be called before the first game
	/// starts to generate the reward token.
	pub fn initialize_token(
		mut ctx: Context<InitializeToken>,
		props: InitializeTokenProps,
	) -> AnchorResult {
		initialize_token_handler(&mut ctx, props)
	}

	/// INNER: This method can only be called by [`initialize_token`]. It uses
	/// the `treasury` signer as the authority for the mint account.
	pub fn initialize_token_inner(
		mut ctx: Context<InitializeTokenInner>,
		props: InitializeTokenInnerProps,
	) -> Result<u8> {
		initialize_token_inner_handler(&mut ctx, props)
	}

	/// This will initialize the meta state for the bits.
	pub fn initialize_game(mut ctx: Context<InitializeGame>) -> AnchorResult {
		initialize_game_handler(&mut ctx)
	}

	/// Start the game.
	///
	/// This will automatically create the initial section and set the tokens
	/// that can be used to manage accounts.
	pub fn start_game(mut ctx: Context<StartGame>) -> AnchorResult {
		start_game_handler(&mut ctx)
	}

	/// This will unlock a single section of the bits data.
	///
	/// Each section has an owner who pays the costs for creation. The section
	/// only becomes avaiable when the previous section reaches a certain
	/// threshold.
	pub fn unlock_section(mut ctx: Context<UnlockSection>) -> AnchorResult {
		unlock_section_handler(&mut ctx)
	}

	/// Refresh the access signer.
	/// This will be called by the backend to create a new one.
	pub fn refresh_access_signer(mut ctx: Context<RefreshAccessSigner>) -> AnchorResult {
		refresh_access_signer_handler(&mut ctx)
	}

	/// Create a player pda account. This account can be used to make actions on
	/// behalf of the player. They will transfer a minimum amount of lamports to
	/// the created token account.
	pub fn create_derived_player(mut ctx: Context<CreateDerivedPlayer>) -> AnchorResult {
		create_derived_player_handler(&mut ctx)
	}

	/// Flip bits based on the provided props.
	pub fn flip_bits(mut ctx: Context<FlipBits>, props: FlipBitsProps) -> AnchorResult {
		flip_bits_handler(&mut ctx, &props)
	}
}

#[derive(Accounts)]
pub struct InitializeConfig<'info> {
	#[account(
		init,
		payer = authority,
		space = ConfigState::space() * 2,
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump
	)]
	pub config: Account<'info, ConfigState>,
	/// The hard-coded account that is used to initialize the program config
	/// once.
	#[account(
		address = ADMIN_PUBKEY @ BitflipError::UnauthorizedAdmin
	)]
	pub admin: Signer<'info>,
	/// The treasury which stores all tokens created.
	#[account(
		seeds = [SEED_PREFIX, SEED_TREASURY],
		bump
	)]
	pub treasury: SystemAccount<'info>,
	/// The authority which needs to also sign for the transaction to prove
	/// ownership and make payment.
	#[account(
		mut,
		constraint = authority.key() != ADMIN_PUBKEY @ BitflipError::UnauthorizedAdmin
	)]
	pub authority: Signer<'info>,
	/// This is needed for initializing the bit state.
	#[account(address = system_program::ID)]
	pub system_program: Program<'info, System>,
}

pub fn initialize_config_handler(ctx: &mut Context<InitializeConfig>) -> AnchorResult {
	let authority = ctx.accounts.authority.key();
	let bump = ctx.bumps.config;
	let treasury_bump = ctx.bumps.treasury;
	let config_state = ConfigState::new(authority, bump, treasury_bump);

	ctx.accounts.config.set_inner(config_state);

	Ok(())
}

#[derive(Accounts)]
pub struct UpdateAuthority<'info> {
	/// The program configuration.
	#[account(
        mut,
        has_one = authority,
        seeds = [SEED_PREFIX, SEED_CONFIG],
        bump = config.bump,
    )]
	pub config: Box<Account<'info, ConfigState>>,
	/// The current authority which must sign to update the authority.
	#[account(mut)]
	pub authority: Signer<'info>,
	/// The new authority that will be set, must be a signer to prove ownership.
	pub new_authority: Signer<'info>,
}

pub fn update_authority_handler(ctx: &mut Context<UpdateAuthority>) -> AnchorResult {
	let config = &mut ctx.accounts.config;
	config.authority = ctx.accounts.new_authority.key();

	Ok(())
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
	/// The program authority which must be a signer to create this token.
	#[account(mut, signer)]
	pub authority: SystemAccount<'info>,
	/// CHECK: Initialized within [`InitializeTokenInner`].
	#[account(mut)]
	pub mint: UncheckedAccount<'info>,
	/// CHECK: Checked within [`InitializeTokenInner`].
	#[account(mut)]
	pub treasury: UncheckedAccount<'info>,
	/// CHECK: Checked within [`InitializeTokenInner`].
	#[account(mut)]
	pub treasury_token_account: UncheckedAccount<'info>,
	/// CHECK: Checked within [`InitializeTokenInner`].
	pub associated_token_program: UncheckedAccount<'info>,
	/// CHECK: Checked within [`InitializeTokenInner`].
	pub token_program: UncheckedAccount<'info>,
	/// CHECK: Checked within [`InitializeTokenInner`].
	pub system_program: UncheckedAccount<'info>,
	/// The program that is for signing.
	pub bitflip_program: Program<'info, BitflipProgram>,
}

pub fn initialize_token_handler(
	ctx: &mut Context<InitializeToken>,
	props: InitializeTokenProps,
) -> AnchorResult {
	let treasury_bump = ctx.accounts.config.treasury_bump;
	let treasury_seeds = &[SEED_PREFIX, SEED_TREASURY, &[treasury_bump]];
	let signer_seeds = &[&treasury_seeds[..]];
	let authority = ctx.accounts.authority.to_account_info();
	let mint = ctx.accounts.mint.to_account_info();
	let treasury = ctx.accounts.treasury.to_account_info();
	let treasury_token_account = ctx.accounts.treasury_token_account.to_account_info();
	let associated_token_program = ctx.accounts.associated_token_program.to_account_info();
	let token_program = ctx.accounts.token_program.to_account_info();
	let system_program = ctx.accounts.system_program.to_account_info();
	let bitflip_program = ctx.accounts.bitflip_program.to_account_info();
	let minimum_balance = Rent::get()?.minimum_balance(0);

	if treasury.lamports() < minimum_balance {
		let from = authority.clone();
		let to = treasury.clone();
		let accounts = system_program::Transfer { from, to };
		let cpi_context = CpiContext::new(system_program.clone(), accounts);

		// transfer the minimum sol to the treasury for rent exemption
		system_program::transfer(cpi_context, minimum_balance)?;
	}

	let accounts = cpi::accounts::InitializeTokenInner {
		authority,
		mint,
		treasury,
		treasury_token_account,
		associated_token_program,
		token_program,
		system_program,
	};
	let cpi_context = CpiContext::new_with_signer(bitflip_program, accounts, signer_seeds);
	let inner_props = props.into_inner(treasury_bump);
	let result = cpi::initialize_token_inner(cpi_context, inner_props)?;

	ctx.accounts.config.mint_bump = result.get();

	Ok(())
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

impl InitializeTokenProps {
	pub fn into_inner(self, treasury_bump: u8) -> InitializeTokenInnerProps {
		let Self { name, symbol, uri } = self;
		InitializeTokenInnerProps {
			name,
			symbol,
			uri,
			treasury_bump,
		}
	}
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitializeTokenInnerProps {
	pub name: String,
	pub symbol: String,
	pub uri: String,
	pub treasury_bump: u8,
}

impl From<InitializeTokenInnerProps> for instruction::InitializeTokenInner {
	fn from(props: InitializeTokenInnerProps) -> Self {
		instruction::InitializeTokenInner { props }
	}
}

impl From<InitializeTokenInnerProps> for InitializeTokenProps {
	fn from(
		InitializeTokenInnerProps {
			name, symbol, uri, ..
		}: InitializeTokenInnerProps,
	) -> Self {
		Self { name, symbol, uri }
	}
}

#[derive(Accounts)]
#[instruction(props: InitializeTokenInnerProps)]
pub struct InitializeTokenInner<'info> {
	/// CHECKED: checked in [`InitializeToken`] outer call.
	#[account(mut)]
	pub authority: Signer<'info>,
	/// The token mint account.
	#[account(
	  init,
	  payer =	authority,
	  seeds = [SEED_PREFIX, SEED_MINT],
	  bump,
	  mint::token_program = token_program,
	  mint::decimals = TOKEN_DECIMALS,
	  mint::authority = treasury,
	  mint::freeze_authority = treasury,
	  extensions::metadata_pointer::authority = treasury,
	  extensions::metadata_pointer::metadata_address = mint,
	  extensions::close_authority::authority = treasury
  )]
	pub mint: Box<InterfaceAccount<'info, Mint>>,
	/// The treasury account.
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_TREASURY],
		bump = props.treasury_bump
	)]
	pub treasury: Signer<'info>,
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

impl InitializeTokenInner<'_> {
	fn initialize_token_metadata(
		&self,
		InitializeTokenProps { name, symbol, uri }: InitializeTokenProps,
	) -> AnchorResult {
		let cpi_accounts = TokenMetadataInitialize {
			token_program_id: self.token_program.to_account_info(),
			mint: self.mint.to_account_info(),
			metadata: self.mint.to_account_info(),
			mint_authority: self.treasury.to_account_info(),
			update_authority: self.treasury.to_account_info(),
		};
		let cpi_ctx = CpiContext::new(self.token_program.to_account_info(), cpi_accounts);
		token_metadata_initialize(cpi_ctx, name, symbol, uri)?;

		Ok(())
	}
}

pub fn initialize_token_inner_handler(
	ctx: &mut Context<InitializeTokenInner>,
	props: InitializeTokenInnerProps,
) -> Result<u8> {
	let rent_sysvar = Rent::get()?;
	let mint = ctx.accounts.mint.to_account_info();

	let result = ctx.bumps.mint;
	ctx.accounts.initialize_token_metadata(props.into())?;
	ctx.accounts.mint.reload()?;

	let extra_lamports = rent_sysvar.minimum_balance(mint.data_len()) - mint.get_lamports();

	// transfer minimum rent to mint account when required
	if extra_lamports > 0 {
		let from = ctx.accounts.authority.to_account_info();
		let to = mint.clone();
		let system_program = ctx.accounts.system_program.to_account_info();
		let cpi_context = CpiContext::new(system_program, system_program::Transfer { from, to });

		system_program::transfer(cpi_context, extra_lamports)?;
	}

	let to = ctx.accounts.treasury_token_account.to_account_info();
	let authority = ctx.accounts.treasury.to_account_info();
	let token_program = ctx.accounts.token_program.to_account_info();

	let cpi_context = CpiContext::new(token_program, MintTo {
		mint,
		to,
		authority,
	});
	let amount = get_token_amount(TOTAL_TOKENS, TOKEN_DECIMALS)?;

	token_2022::mint_to(cpi_context, amount)?;

	Ok(result)
}

#[derive(Accounts)]
pub struct InitializeGame<'info> {
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
		space = GameState::space(),
		seeds = [SEED_PREFIX, SEED_GAME, &config.game_index.to_le_bytes()],
		bump
	)]
	pub game: Box<Account<'info, GameState>>,
	/// The game nonce account which is used to create new sections by storing
	/// signed transactions and allowing the backend to pick the one to use.
	///
	/// The nonce account for this game. Which is a durable nonce account.
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_GAME, &config.game_index.to_le_bytes(), SEED_GAME_NONCE],
		bump
	)]
	pub game_nonce: SystemAccount<'info>,
	/// The authority that is able to sign for updates to the config and
	/// initiate new games.
	#[account(mut)]
	pub authority: Signer<'info>,
	/// The `access_signer` is created by the backend and used for on chain
	/// operations.
	pub access_signer: Signer<'info>,
	/// The `refresh_signer`. The `access_token` expires every 24 hours. In
	/// order to create a new one this is .
	pub refresh_signer: Signer<'info>,
	/// CHECK: This is needed for creating the nonce account.
	#[account(address = anchor_lang::solana_program::sysvar::recent_blockhashes::id())]
	pub recent_blockhashes: Sysvar<'info, RecentBlockhashes>,
	/// CHECK: The rent account.
	pub rent: Sysvar<'info, Rent>,
	/// The system program.
	pub system_program: Program<'info, System>,
}

pub fn initialize_game_handler(ctx: &mut Context<InitializeGame>) -> AnchorResult {
	let access_signer = ctx.accounts.access_signer.key();
	let refresh_signer = ctx.accounts.refresh_signer.key();
	let game_index = ctx.accounts.config.game_index;
	let game_bump = ctx.bumps.game;
	let game_nonce_bump = ctx.bumps.game_nonce;

	ctx.accounts.game.set_inner(GameState::new(
		access_signer,
		refresh_signer,
		game_index,
		game_bump,
		game_nonce_bump,
	));

	let lamports = ctx.accounts.rent.minimum_balance(nonce::State::size());
	let from = ctx.accounts.authority.to_account_info();
	let nonce = ctx.accounts.game_nonce.to_account_info();
	let recent_blockhashes = ctx.accounts.recent_blockhashes.to_account_info();
	let rent = ctx.accounts.rent.to_account_info();
	let system_program = ctx.accounts.system_program.to_account_info();
	let game_nonce_seeds = &[
		SEED_PREFIX,
		SEED_GAME,
		&game_index.to_le_bytes(),
		SEED_GAME_NONCE,
		&[game_nonce_bump],
	];
	let signer_seeds = &[&game_nonce_seeds[..]];
	let accounts = system_program::CreateNonceAccount {
		from,
		nonce,
		recent_blockhashes,
		rent,
	};
	let cpi_context = CpiContext::new_with_signer(system_program, accounts, signer_seeds);
	system_program::create_nonce_account(cpi_context, lamports, &ctx.accounts.access_signer.key())?;

	Ok(())
}

#[derive(Accounts)]
pub struct StartGame<'info> {
	/// The program configuration.
	#[account(
		mut,
		has_one = authority,
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump = config.bump,
	)]
	pub config: Box<Account<'info, ConfigState>>,
	/// The meta data account for the game.
	#[account(
		mut,
		has_one = access_signer,
		seeds = [SEED_PREFIX, SEED_GAME, &config.game_index.to_le_bytes()],
		bump = game.bump
	)]
	pub game: Box<Account<'info, GameState>>,
	/// The authority that is able to sign for updates to the config and
	/// initiate new games. It also is the payer for this transaction.
	#[account(mut)]
	pub authority: Signer<'info>,
	/// The `access_signer` is created by the backend and used for on chain
	/// operations.
	pub access_signer: Signer<'info>,
	/// The system program.
	#[account(address = system_program::ID)]
	pub system_program: Program<'info, System>,
}

impl StartGame<'_> {
	/// Validate the handler.
	pub fn validate(&self) -> AnchorResult {
		require!(
			self.game.section_index == 0,
			BitflipError::InvalidSectionIndex
		);

		Ok(())
	}
}

#[access_control(ctx.accounts.validate())]
pub fn start_game_handler(ctx: &mut Context<StartGame>) -> AnchorResult {
	let game = &mut ctx.accounts.game;
	let config = &mut ctx.accounts.config;

	game.start_time = Clock::get()?.unix_timestamp;
	game.access_expiry = game
		.start_time
		.checked_add(ACCESS_SIGNER_DURATION)
		.ok_or(ProgramError::ArithmeticOverflow)?;
	config.game_index = config
		.game_index
		.checked_add(1)
		.ok_or(ProgramError::ArithmeticOverflow)?;

	Ok(())
}

#[derive(Accounts)]
pub struct UnlockSection<'info> {
	/// The program configuration.
	#[account(
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
	/// The treasury account.
	#[account(
		seeds = [SEED_PREFIX, SEED_TREASURY],
		bump = config.treasury_bump
	)]
	pub treasury: SystemAccount<'info>,
	/// The associated token account for the treasury which will hold the minted
	/// tokens.
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
		has_one = access_signer,
		seeds = [SEED_PREFIX, SEED_GAME, &game.game_index.to_le_bytes()],
		bump = game.bump
	)]
	pub game: Box<Account<'info, GameState>>,
	/// This is a section of the bits data being initialized.
	#[account(
		init,
		payer = player,
		space = SectionState::space(),
		seeds = [SEED_PREFIX, SEED_GAME, &config.game_index.to_le_bytes(), SEED_SECTION, &game.section_index.to_le_bytes()],
		bump
	)]
	pub section: Box<Account<'info, SectionState>>,
	/// The token account which holds the section tokens.
	#[account(
		init,
		payer = player,
	  associated_token::token_program = token_program,
		associated_token::mint = mint,
		associated_token::authority = section,
	)]
	pub section_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
	/// The previous account which is checked for passing a certain threshold
	/// before the next section can be created.
	#[account(
		constraint = game.section_index > 0,
		constraint = previous_section.index.checked_add(1).unwrap() == game.section_index,
		seeds = [SEED_PREFIX, SEED_GAME, &config.game_index.to_le_bytes(), SEED_SECTION, &game.section_index.saturating_sub(1).to_le_bytes()],
		bump = previous_section.bump
	)]
	pub previous_section: Option<Box<Account<'info, SectionState>>>,
	/// The backend signer to authorise this.
	pub access_signer: Signer<'info>,
	/// The player of the bit games is also responsible for paying all the
	/// costs.
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

#[event]
#[cfg_attr(feature = "serde", derive(::serde::Serialize, ::serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "camelCase"))]
pub struct UnlockSectionEvent {
	pub owner: Pubkey,
	pub section_index: u8,
}

pub fn unlock_section_handler(ctx: &mut Context<UnlockSection>) -> AnchorResult {
	let game = &mut ctx.accounts.game;
	let section = &mut ctx.accounts.section;
	let section_index = game.section_index;
	let config = &ctx.accounts.config;
	let owner = ctx.accounts.player.key();
	let bump = ctx.bumps.section;
	let section_state = SectionState::new(owner, bump, section_index);
	let unlock_section_event = UnlockSectionEvent {
		owner,
		section_index,
	};

	section.set_inner(section_state);

	// the game must have started
	require!(game.started(), BitflipError::NotRunning);

	if section_index < u8::MAX {
		game.section_index += 1;
	}

	if let Some(previous_section) = &ctx.accounts.previous_section {
		// check if the previous section has met the minimum flips threshold
		require!(
			previous_section.flips > MINIMUM_FLIPS_PER_SECTION,
			BitflipError::MinimumFlipThreshold
		);

		// ensure that two consecutive section owners are not the same.
		require_keys_neq!(
			owner,
			previous_section.owner,
			BitflipError::SectionOwnerDuplicate
		);
	}

	// todo charge an increasing price for owning a section

	let signer = &[SEED_PREFIX, SEED_TREASURY, &[config.treasury_bump]];
	let from = ctx.accounts.treasury_token_account.to_account_info();
	let to = ctx.accounts.section_token_account.to_account_info();
	let mint = ctx.accounts.mint.to_account_info();
	let authority = ctx.accounts.treasury.to_account_info();
	let token_program = ctx.accounts.token_program.to_account_info();
	let signer_seeds = &[&signer[..]];
	let cpi_context = CpiContext::new_with_signer(
		token_program,
		token_2022::TransferChecked {
			from,
			mint,
			to,
			authority,
		},
		signer_seeds,
	);
	let amount = get_token_amount(TOKENS_PER_SECTION, TOKEN_DECIMALS)?;

	token_2022::transfer_checked(cpi_context, amount, TOKEN_DECIMALS)?;

	emit!(unlock_section_event);

	Ok(())
}

#[derive(Accounts)]
pub struct RefreshAccessSigner<'info> {
	/// The program configuration.
	#[account(
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump = config.bump,
	)]
	pub config: Box<Account<'info, ConfigState>>,
	/// The meta data for the full bits state.
	#[account(
		mut,
		has_one = refresh_signer,
		constraint = game.access_signer != access_signer.key() @ BitflipError::AccessSignerNotUpdated,
		seeds = [SEED_PREFIX, SEED_GAME, &game.game_index.to_le_bytes()],
		bump = game.bump,
	)]
	pub game: Box<Account<'info, GameState>>,
	/// The new `access_signer` is created by the backend and used for on chain
	/// operations.
	pub access_signer: Signer<'info>,
	/// The `refresh_signer`. The `access_token` expires every 24 hours. In
	/// order to create a new one this is used.
	pub refresh_signer: Signer<'info>,
}

pub fn refresh_access_signer_handler(ctx: &mut Context<RefreshAccessSigner>) -> AnchorResult {
	msg!("inside `refresh_access_signer_handler`");
	ctx.accounts.game.access_signer = ctx.accounts.access_signer.key();
	Ok(())
}

#[derive(Accounts)]
pub struct CreateDerivedPlayer<'info> {
	/// The program configuration.
	#[account(
		seeds = [SEED_PREFIX, SEED_CONFIG],
		bump = config.bump,
	)]
	pub config: Box<Account<'info, ConfigState>>,
	/// The token mint account
	#[account(
		seeds = [SEED_PREFIX, SEED_MINT],
		bump = config.mint_bump,
	)]
	pub mint: Box<InterfaceAccount<'info, Mint>>,
	/// The PDA account that will be created to act on behalf of the player
	#[account(
		mut,
		seeds = [SEED_PREFIX, SEED_PLAYER, player.key().as_ref()],
		bump
	)]
	pub derived_player: SystemAccount<'info>,
	/// The associated token account for the onchain player PDA
	#[account(
		init,
		payer = player,
		associated_token::token_program = token_program,
		associated_token::mint = mint,
		associated_token::authority = derived_player,
	)]
	pub derived_player_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
	/// The player who is creating the account and paying for initialization
	#[account(mut)]
	pub player: Signer<'info>,
	/// Required for creating the associated token account
	pub associated_token_program: Program<'info, AssociatedToken>,
	/// Required for token operations
	pub token_program: Program<'info, Token2022>,
	/// Required for account creation
	pub system_program: Program<'info, System>,
}

pub fn create_derived_player_handler(ctx: &mut Context<CreateDerivedPlayer>) -> AnchorResult {
	msg!("creating derived player account");

	// Calculate minimum rent
	let minimum_balance = Rent::get()?.minimum_balance(0);

	// Transfer minimum rent to the derived_player PDA
	let transfer_accounts = system_program::Transfer {
		from: ctx.accounts.player.to_account_info(),
		to: ctx.accounts.derived_player.to_account_info(),
	};
	let transfer_ctx = CpiContext::new(
		ctx.accounts.system_program.to_account_info(),
		transfer_accounts,
	);
	let account_balance = LAMPORTS_PER_BIT
		.checked_mul(100)
		.ok_or(ProgramError::ArithmeticOverflow)?;
	system_program::transfer(
		transfer_ctx,
		minimum_balance
			.checked_add(account_balance)
			.ok_or(ProgramError::ArithmeticOverflow)?,
	)?;

	Ok(())
}

#[derive(Accounts)]
#[instruction(props: FlipBitsProps)]
pub struct FlipBits<'info> {
	/// The program configuration.
	#[account(
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
	/// The meta data for the full bits state.
	#[account(
		seeds = [SEED_PREFIX, SEED_GAME, &game.game_index.to_le_bytes()],
		bump = game.bump,
	)]
	pub game: Box<Account<'info, GameState>>,
	/// The data for this section of the bit canvas.
	#[account(
		mut,
		constraint = props.section_index == section.index @ BitflipError::InvalidSectionIndex,
		seeds = [SEED_PREFIX, SEED_GAME, &game.game_index.to_le_bytes(), SEED_SECTION, &props.section_index.to_le_bytes()],
		bump = section.bump
	)]
	pub section: Box<Account<'info, SectionState>>,
	/// The token account for the section.
	#[account(
		mut,
	  associated_token::token_program = token_program,
		associated_token::mint = mint,
		associated_token::authority = section,
	)]
	pub section_token_account: Box<InterfaceAccount<'info, TokenAccount>>,
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

impl FlipBits<'_> {
	fn validate(&self, props: &FlipBitsProps) -> AnchorResult {
		msg!("validating set bits...");
		let current_time = Clock::get()?.unix_timestamp;
		require!(self.game.running(current_time), BitflipError::NotRunning);
		props.validate()?;

		msg!("VALIDATED set bits!");
		Ok(())
	}

	fn update(&mut self, props: &FlipBitsProps) -> AnchorResult {
		msg!("about to load the account");
		let changes = self.section.set_bits(props)?;

		msg!("here are the changes: {:#?}", changes);
		let flipped_bits = changes.total()?;

		self.section.flip_on(changes.on)?;
		self.section.flip_off(changes.off)?;
		self.transfer_token_from_section(flipped_bits)?;
		self.transfer_sol_to_section(
			flipped_bits,
			props
				.get_multiplier()
				.checked_mul(LAMPORTS_PER_BIT)
				.ok_or(ProgramError::ArithmeticOverflow)?,
		)?;

		Ok(())
	}
}

#[access_control(ctx.accounts.validate(props))]
pub fn flip_bits_handler(ctx: &mut Context<FlipBits>, props: &FlipBitsProps) -> AnchorResult {
	msg!("props: {:#?}", props);
	msg!("section: {:#?}", &ctx.accounts.section);
	msg!("inside `flip_bits_handler`");
	let rent = Rent::get()?;
	ctx.accounts.update(props)?;
	let config = ctx.accounts.config.to_account_info();
	let config_balance = config.lamports();
	let config_space = config.data_len();
	let config_min = rent.minimum_balance(config_space);
	let mint = ctx.accounts.mint.to_account_info();
	let mint_balance = mint.lamports();
	let mint_space = mint.data_len();
	let mint_min = rent.minimum_balance(mint_space);
	let pta = ctx.accounts.player_token_account.to_account_info();
	let pta_balance = pta.lamports();
	let pta_space = pta.data_len();
	let pta_min = rent.minimum_balance(pta_space);
	let game_state = ctx.accounts.game.to_account_info();
	let game_state_balance = game_state.lamports();
	let game_state_space = game_state.data_len();
	let game_state_min = rent.minimum_balance(game_state_space);
	let section = ctx.accounts.section.to_account_info();
	let section_balance = section.lamports();
	let section_space = section.data_len();
	let section_min = rent.minimum_balance(section_space);
	let tta = ctx.accounts.section_token_account.to_account_info();
	let tta_balance = tta.lamports();
	let tta_space = tta.data_len();
	let tta_min = rent.minimum_balance(tta_space);

	msg!(
		"`config` balance: {}, space: {}, required rent: {}, valid: {}",
		config_balance,
		config_space,
		config_min,
		config_min == config_balance,
	);
	msg!(
		"`mint` balance: {}, space: {}, required rent: {}, valid: {}",
		mint_balance,
		mint_space,
		mint_min,
		mint_min == mint_balance,
	);
	msg!(
		"`player_token_account` balance: {}, space: {}, required rent: {}, valid:
	{}",
		pta_balance,
		pta_space,
		pta_min,
		pta_min == pta_balance,
	);
	msg!(
		"`game_state` balance: {}, space: {}, required rent: {}, valid: {}",
		game_state_balance,
		game_state_space,
		game_state_min,
		game_state_min == game_state_balance,
	);
	msg!(
		"`bits_data_section` balance: {}, space: {}, required rent: {}, valid: {}",
		section_balance,
		section_space,
		section_min,
		section_min == section_balance,
	);
	msg!(
		"`treasury_token_account` balance: {}, space: {}, required rent: {}, valid:
	{}",
		tta_balance,
		tta_space,
		tta_min,
		tta_min == tta_balance,
	);

	msg!("completed update");

	Ok(())
}

impl<'info> TransferSolToSection<'info> for FlipBits<'info> {
	fn section(&self) -> AccountInfo<'info> {
		self.section.to_account_info()
	}

	fn player(&self) -> AccountInfo<'info> {
		self.player.to_account_info()
	}

	fn system_program(&self) -> AccountInfo<'info> {
		self.system_program.to_account_info()
	}
}

impl<'info> TransferTokenFromSection<'info> for FlipBits<'info> {
	fn game_index(&self) -> u8 {
		self.game.game_index
	}

	fn section_index(&self) -> u8 {
		self.section.index
	}

	fn section_bump(&self) -> u8 {
		self.section.bump
	}

	fn mint(&self) -> AccountInfo<'info> {
		self.mint.to_account_info()
	}

	fn section(&self) -> AccountInfo<'info> {
		self.section.to_account_info()
	}

	fn section_token_account(&self) -> AccountInfo<'info> {
		self.section_token_account.to_account_info()
	}

	fn player_token_account(&self) -> AccountInfo<'info> {
		self.player_token_account.to_account_info()
	}

	fn token_program(&self) -> AccountInfo<'info> {
		self.token_program.to_account_info()
	}
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
#[cfg_attr(feature = "client", derive(PartialEq, Eq, Hash))]
pub struct FlipBitsProps {
	/// The data section being updated.
	pub section_index: u8,
	/// The index of the `u16` value in the array.
	pub array_index: u16,
	/// The new bit values.
	pub variant: SetBitsVariant,
}

#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
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

impl FlipBitsProps {
	pub fn validate(&self) -> AnchorResult {
		validate_section_index(self.array_index)?;

		match &self.variant {
			SetBitsVariant::On(offset) | SetBitsVariant::Off(offset) => {
				validate_bit_offset(*offset)?;
			}
			SetBitsVariant::Bit16(_) => {}
			SetBitsVariant::Bits256(bits_array) => {
				let index_with_offset = self
					.array_index
					.checked_add(15)
					.ok_or(ProgramError::ArithmeticOverflow)?;

				validate_bit_array_length(bits_array, 16)?;
				validate_256bit_data_section_index(self.array_index)?;
				validate_section_index(index_with_offset)?;
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
						self.array_index == other.array_index
							&& self_offset == other_offset
							&& self.section_index == other.section_index
					}
					_ => false,
				}
			}
			SetBitsVariant::Bit16(_) => {
				match &other.variant {
					SetBitsVariant::On(_) | SetBitsVariant::Off(_) | SetBitsVariant::Bit16(_) => {
						self.array_index == other.array_index
							&& self.section_index == other.section_index
					}
					SetBitsVariant::Bits256(_) => false,
				}
			}
			SetBitsVariant::Bits256(_) => {
				self.array_index == other.array_index && self.section_index == other.section_index
			}
		}
	}
}

impl From<FlipBitsProps> for instruction::FlipBits {
	fn from(props: FlipBitsProps) -> Self {
		instruction::FlipBits { props }
	}
}

#[account]
#[derive(InitSpace)]
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
	pub mint_bump: u8,
	/// There will be a maximum of 256 games.
	pub game_index: u8,
}

impl ConfigState {
	pub const fn space() -> usize {
		SPACE_DISCRIMINATOR + Self::INIT_SPACE
	}

	pub fn new(authority: Pubkey, bump: u8, treasury_bump: u8) -> ConfigState {
		ConfigState {
			authority,
			bump,
			treasury_bump,
			mint_bump: 0,
			game_index: 0,
		}
	}
}

/// Adding [`BitState::on`] to [`BitState::off`] should always equal `1_000_000`
#[account]
#[derive(InitSpace, Debug)]
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
		}
	}

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

	pub fn running(&self, current_time: i64) -> bool {
		self.started() && !self.ended(current_time)
	}
}

/// The data for each section of the the data. The total data is split into 16
/// sections and this is one of the sections.
///
/// This will also store the lamports that are accumulated by this game section.
#[account]
#[derive(Debug, InitSpace)]
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
	/// The bump for this section state.
	pub bump: u8,
	/// The index for this section state.
	pub index: u8,
}

impl SectionState {
	pub fn space() -> usize {
		SPACE_DISCRIMINATOR + Self::INIT_SPACE
	}

	pub fn new(owner: Pubkey, bump: u8, index: u8) -> Self {
		Self {
			data: [0; BITFLIP_SECTION_LENGTH],
			owner,
			flips: 0,
			on: 0,
			off: BITFLIP_SECTION_TOTAL_BITS,
			bump,
			index,
		}
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
			.checked_add(changed_bits)
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
			.checked_add(changed_bits)
			.ok_or(ProgramError::ArithmeticOverflow)?;

		Ok(())
	}
}

pub trait SetBitsDataSection: Deref<Target = SectionState> + DerefMut {
	fn set_bits(
		// mut state: RefMut<'_, BitsDataSectionState>,
		&mut self,
		props: &FlipBitsProps,
	) -> Result<BitChanges> {
		let index = props.array_index as usize;
		msg!("index: {}", index);
		let mut changes = BitChanges::default();

		match &props.variant {
			SetBitsVariant::On(offset) => {
				msg!("set bits variant ON");
				let current = self.data[index..=index][0];
				let bit = 1 << *offset;
				let updated = current | bit;
				msg!("current: {}, bit: {}, updated: {}", current, bit, updated);

				require!(updated != current, BitflipError::BitsUnchanged);

				self.data[index..=index].copy_from_slice(&[updated]);
				msg!("data after update: {:#?}", &self.data[index..=index]);
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
				let index_with_offset = props.array_index.saturating_add(15) as usize;
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

impl<T: Deref<Target = SectionState> + DerefMut> SetBitsDataSection for T {}

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
