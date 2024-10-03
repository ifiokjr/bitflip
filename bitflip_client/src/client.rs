use derive_more::Deref;
use derive_more::DerefMut;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use wasm_client_anchor::AnchorClientResult;
pub use wasm_client_anchor::AnchorProgram;
use wasm_client_anchor::AnchorProgramBuilder;
use wasm_client_anchor::WalletAnchor;

use crate::bitflip_request_builder;

/// This wraps the [`AnchorProgram`] with utilty functions.
#[derive(Debug, Clone, Deref, DerefMut)]
pub struct BitflipProgram<W: WalletAnchor>(AnchorProgram<W>);

pub trait IntoBitflipProgram<W: WalletAnchor> {
	fn into_launchpad(self) -> BitflipProgram<W>;
}

impl<W: WalletAnchor> IntoBitflipProgram<W> for AnchorProgram<W> {
	fn into_launchpad(self) -> BitflipProgram<W> {
		self.into()
	}
}

impl<W: WalletAnchor> From<AnchorProgram<W>> for BitflipProgram<W> {
	fn from(program: AnchorProgram<W>) -> Self {
		BitflipProgram(program)
	}
}

type AnchorProgramPartialBuilder<W> = AnchorProgramBuilder<W, ((Pubkey,), (), ())>;

impl<W: WalletAnchor> BitflipProgram<W> {
	/// Start the `AnchorProgram` builder with the `program_id` already set to
	/// the default.
	pub fn builder() -> AnchorProgramPartialBuilder<W> {
		AnchorProgram::builder().program_id(bitflip_program::ID_CONST)
	}

	/// Start the `AnchorProgram` builder with a custom `program_id`.
	pub fn builder_with_program(program_id: &Pubkey) -> AnchorProgramPartialBuilder<W> {
		AnchorProgram::builder().program_id(*program_id)
	}

	/// Get the program
	pub fn program(&self) -> &AnchorProgram<W> {
		self
	}

	/// Request an airdrop to the payer account. This will only work on
	/// `localnet`, `testnet` and `devnet`.
	pub async fn request_airdrop(
		&self,
		pubkey: &Pubkey,
		lamports: u64,
	) -> AnchorClientResult<Signature> {
		let signature = self.rpc().request_airdrop(pubkey, lamports).await?;
		Ok(signature)
	}
}

bitflip_request_builder!(InitializeConfig);
bitflip_request_builder!(InitializeToken);
bitflip_request_builder!(InitializeBitsMeta, "optional:args");
bitflip_request_builder!(InitializeBitsDataSection);
bitflip_request_builder!(SetBits);
