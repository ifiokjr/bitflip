//! Client code for the Bitflip program.

use solana_sdk::nonce::state::Data as NonceData;
use solana_sdk::nonce::state::State as NonceState;
use solana_sdk::system_instruction::create_nonce_account_with_seed;
use steel::*;
use sysvar::rent::Rent;
use wasm_client_solana::nonce_utils;
use wasm_client_solana::ClientError;
use wasm_client_solana::ClientResult;
use wasm_client_solana::SolanaRpcClient;

use crate::SEED_PREFIX;

/// Get the nonce data for a given nonce account.
pub async fn get_nonce_data(rpc: &SolanaRpcClient, nonce: &Pubkey) -> ClientResult<NonceData> {
	let account = nonce_utils::get_account(rpc, nonce)
		.await
		.map_err(|e| ClientError::Other(e.to_string()))?;
	let data =
		nonce_utils::data_from_account(&account).map_err(|e| ClientError::Other(e.to_string()))?;

	Ok(data)
}

/// Create a nonce account.
pub fn create_nonce_account(
	payer: &Pubkey,
	owner: &Pubkey,
	identifier: &str,
) -> Result<(Pubkey, Vec<Instruction>), ProgramError> {
	let seed = get_nonce_seed(identifier);
	let derived_nonce_account = Pubkey::create_with_seed(owner, &seed, owner)?;
	let lamports = Rent::default().minimum_balance(NonceState::size());
	let instructions = create_nonce_account_with_seed(
		payer,
		&derived_nonce_account,
		owner,
		&seed,
		owner,
		lamports,
	);

	Ok((derived_nonce_account, instructions))
}

/// Get the seed for a nonce account.
pub fn get_nonce_seed(identifier: &str) -> String {
	format!(
		"{}:nonce:{identifier}",
		std::str::from_utf8(SEED_PREFIX).unwrap()
	)
}

#[derive(Clone, Debug, strum::Display, strum::EnumString, PartialEq, Eq)]
#[non_exhaustive]
pub enum NonceIdentifier {
	#[strum(serialize = "section_unlock:{game:#02}:{section:#03}")]
	SectionUnlock { game: u8, section: u8 },
}

#[cfg(test)]
mod tests {
	use rstest::rstest;

	use super::*;

	#[rstest]
	#[case(1, 2, "section_unlock:01:002")]
	#[case(0, 0, "section_unlock:00:000")]
	#[case(99, 1, "section_unlock:99:001")]
	#[case(5, 123, "section_unlock:05:123")]
	#[case(15, 45, "section_unlock:15:045")]
	fn test_nonce_identifier(#[case] game: u8, #[case] section: u8, #[case] expected: &str) {
		assert_eq!(
			NonceIdentifier::SectionUnlock { game, section }.to_string(),
			expected
		);
	}
}
