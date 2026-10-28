//! Bind an atomic mint receipt to Bubblegum's post-mint allocation counter.

use pina::*;

use crate::BitflipError;
use crate::RecordSectionMintInstructionZc;

const BUBBLEGUM_ID: Address = address!("BGUMAp9Gq7iTEuizy4pqaxsTyUCBK68MDfK752saRPUY");
const TREE_CONFIG_DISCRIMINATOR: [u8; 8] = [122, 245, 175, 248, 171, 34, 0, 207];

/// A stale prediction must abort the whole transaction, including its MintV1.
/// Solana holds the tree's write lock for the transaction, so another replica
/// cannot advance this counter between MintV1 and RecordSectionMint.
pub(super) fn assert_mint_receipt(
	tree_authority: &AccountView,
	args: &RecordSectionMintInstructionZc,
) -> ProgramResult {
	let (expected_tree_authority, _) =
		try_find_program_address(&[args.merkle_tree.as_ref()], &BUBBLEGUM_ID)
			.ok_or(ProgramError::InvalidSeeds)?;
	tree_authority
		.assert_owner(&BUBBLEGUM_ID)?
		.assert_address(&expected_tree_authority)?;
	let data = tree_authority.try_borrow()?;

	if data.len() < 96 || data[..8] != TREE_CONFIG_DISCRIMINATOR {
		return Err(BitflipError::InvalidAsset.into());
	}

	let minted = u64::from_le_bytes(
		data[80..88]
			.try_into()
			.map_err(|_| ProgramError::InvalidAccountData)?,
	);

	let leaf_index = u64::from(args.leaf_index.get());

	if minted != leaf_index + 1 {
		return Err(BitflipError::InvalidAsset.into());
	}

	let (asset_id, _) = try_find_program_address(
		&[
			b"asset",
			args.merkle_tree.as_ref(),
			&leaf_index.to_le_bytes(),
		],
		&BUBBLEGUM_ID,
	)
	.ok_or(ProgramError::InvalidSeeds)?;

	if args.asset_id != asset_id {
		return Err(BitflipError::InvalidAsset.into());
	}

	Ok(())
}
