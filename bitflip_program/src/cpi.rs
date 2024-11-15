use steel::*;

pub(crate) fn metadata_pointer_initialize<'info>(
	mint_info: &AccountInfo<'info>,
	authority_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
	let ix = spl_token_2022::extension::metadata_pointer::instruction::initialize(
		token_program_info.key,
		mint_info.key,
		Some(*authority_info.key),
		Some(*mint_info.key),
	)?;
	solana_program::program::invoke_signed(
		&ix,
		&[token_program_info.clone(), mint_info.clone()],
		signers_seeds,
	)
}

pub(crate) fn initialize_mint2<'info>(
	mint_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	authority_info: &AccountInfo<'info>,
	decimals: u8,
	signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
	let ix = spl_token_2022::instruction::initialize_mint2(
		token_program_info.key,
		mint_info.key,
		authority_info.key,
		Some(authority_info.key),
		decimals,
	)?;
	solana_program::program::invoke_signed(&ix, &[mint_info.clone()], signers_seeds)
}

pub(crate) fn token_metadata_initialize<'info>(
	mint_info: &AccountInfo<'info>,
	authority_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	name: String,
	symbol: String,
	uri: String,
	signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
	let ix = spl_token_metadata_interface::instruction::initialize(
		token_program_info.key,
		mint_info.key,
		authority_info.key,
		mint_info.key,
		authority_info.key,
		name,
		symbol,
		uri,
	);
	solana_program::program::invoke_signed(
		&ix,
		&[
			token_program_info.clone(),
			mint_info.clone(),
			authority_info.clone(),
			mint_info.clone(),
			authority_info.clone(),
		],
		signers_seeds,
	)
}

pub(crate) fn group_pointer_initialize<'info>(
	mint_info: &AccountInfo<'info>,
	authority_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
	let ix = spl_token_2022::extension::group_pointer::instruction::initialize(
		token_program_info.key,
		mint_info.key,
		Some(*authority_info.key),
		Some(*mint_info.key),
	)?;
	solana_program::program::invoke_signed(
		&ix,
		&[token_program_info.clone(), mint_info.clone()],
		signers_seeds,
	)
}

pub(crate) fn mint_close_authority_initialize<'info>(
	mint_info: &AccountInfo<'info>,
	authority_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
	let ix = spl_token_2022::instruction::initialize_mint_close_authority(
		token_program_info.key,
		mint_info.key,
		Some(authority_info.key),
	)?;
	solana_program::program::invoke_signed(
		&ix,
		&[token_program_info.clone(), mint_info.clone()],
		signers_seeds,
	)
}

pub(crate) fn create_associated_token_account<'info>(
	payer_info: &AccountInfo<'info>,
	associated_token_info: &AccountInfo<'info>,
	authority_info: &AccountInfo<'info>,
	mint_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	system_program_info: &AccountInfo<'info>,
	signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
	let ix = spl_associated_token_account::instruction::create_associated_token_account(
		payer_info.key,
		authority_info.key,
		mint_info.key,
		token_program_info.key,
	);
	solana_program::program::invoke_signed(
		&ix,
		&[
			payer_info.clone(),
			associated_token_info.clone(),
			authority_info.clone(),
			mint_info.clone(),
			system_program_info.clone(),
			token_program_info.clone(),
		],
		signers_seeds,
	)
}

pub(crate) fn mint_to<'info>(
	mint_info: &AccountInfo<'info>,
	account_info: &AccountInfo<'info>,
	owner_info: &AccountInfo<'info>,
	token_program_info: &AccountInfo<'info>,
	amount: u64,
	signers_seeds: &[&[&[u8]]],
) -> ProgramResult {
	let ix = spl_token_2022::instruction::mint_to(
		token_program_info.key,
		mint_info.key,
		account_info.key,
		owner_info.key,
		&[],
		amount,
	)?;
	solana_program::program::invoke_signed(
		&ix,
		&[account_info.clone(), mint_info.clone(), owner_info.clone()],
		signers_seeds,
	)
}
