use anchor_lang::prelude::*;
use anchor_lang::system_program;
use anchor_spl::token_2022;

use crate::SEED_GAME;
use crate::SEED_PREFIX;
use crate::SEED_SECTION;
use crate::TOKEN_DECIMALS;
use crate::get_token_amount;

pub trait TransferSolToSection<'info> {
	fn section(&self) -> AccountInfo<'info>;
	fn player(&self) -> AccountInfo<'info>;
	fn system_program(&self) -> AccountInfo<'info>;

	fn transfer_sol_to_section(&self, flipped_bits: u32, lamports_per_bit: u64) -> Result<()> {
		let from = self.player();
		let to = self.section();
		let accounts = system_program::Transfer { from, to };
		let system_program = self.system_program();
		let cpi_context = CpiContext::new(system_program, accounts);
		let lamports = u64::from(flipped_bits).saturating_mul(lamports_per_bit);

		system_program::transfer(cpi_context, lamports)?;

		Ok(())
	}
}

pub trait TransferTokenFromSection<'info> {
	fn game_index(&self) -> u8;
	fn section_index(&self) -> u8;
	fn section_bump(&self) -> u8;
	fn mint(&self) -> AccountInfo<'info>;
	fn section(&self) -> AccountInfo<'info>;
	fn section_token_account(&self) -> AccountInfo<'info>;
	fn player_token_account(&self) -> AccountInfo<'info>;
	fn token_program(&self) -> AccountInfo<'info>;

	fn transfer_token_from_section(&self, flipped_bits: u32) -> Result<()> {
		let signer = &[
			SEED_PREFIX,
			SEED_GAME,
			&self.game_index().to_le_bytes(),
			SEED_SECTION,
			&self.section_index().to_le_bytes(),
			&[self.section_bump()],
		];
		let from = self.section_token_account();
		let mint = self.mint();
		let to = self.player_token_account();
		let authority = self.section();
		let signer_seeds = &[&signer[..]];
		let cpi_context = CpiContext::new_with_signer(
			self.token_program(),
			token_2022::TransferChecked {
				from,
				mint,
				to,
				authority,
			},
			signer_seeds,
		);
		let amount = get_token_amount(flipped_bits.into(), TOKEN_DECIMALS)?;

		token_2022::transfer_checked(cpi_context, amount, TOKEN_DECIMALS)
	}
}

pub type AnchorResult = Result<()>;
