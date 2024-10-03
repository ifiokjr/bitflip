use anchor_lang::prelude::*;
use anchor_spl::token_2022;

use crate::get_token_amount;
use crate::SEED_PREFIX;
use crate::SEED_TREASURY;
use crate::TOKEN_DECIMALS;

pub trait TransferToTreasury<'info> {
	fn treasury(&self) -> AccountInfo<'info>;
	fn player(&self) -> AccountInfo<'info>;
	fn system_program(&self) -> AccountInfo<'info>;

	fn transfer_to_treasury(&self, flipped_bits: u32, lamports_per_bit: u64) -> Result<()> {
		let from = self.player();
		let to = self.treasury();
		let accounts = anchor_lang::system_program::Transfer { from, to };
		let system_program = self.system_program();
		let cpi_context = CpiContext::new(system_program, accounts);
		let lamports = u64::from(flipped_bits).saturating_mul(lamports_per_bit);

		anchor_lang::system_program::transfer(cpi_context, lamports)?;

		Ok(())
	}
}

pub trait TransferFromTreasury<'info> {
	fn mint(&self) -> AccountInfo<'info>;
	fn treasury(&self) -> AccountInfo<'info>;
	fn treasury_bump(&self) -> u8;
	fn treasury_token_account(&self) -> AccountInfo<'info>;
	fn player_token_account(&self) -> AccountInfo<'info>;
	fn token_program(&self) -> AccountInfo<'info>;

	fn transfer_from_treasury(&self, flipped_bits: u32) -> Result<()> {
		let signer = &[SEED_PREFIX, SEED_TREASURY, &[self.treasury_bump()]];
		let signer_seeds = &[&signer[..]];
		let from = self.treasury_token_account();
		let to = self.player_token_account();
		let authority = self.treasury();
		let mint = self.mint();

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
