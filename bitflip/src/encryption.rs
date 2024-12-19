use aes_gcm_siv::aead::Aead;
use aes_gcm_siv::aead::KeyInit;
use aes_gcm_siv::aead::OsRng;
use aes_gcm_siv::Aes256GcmSiv;
use aes_gcm_siv::Nonce;
use anyhow::Context;
use argon2::password_hash::PasswordHash;
use argon2::password_hash::PasswordHasher;
use argon2::password_hash::SaltString;
use argon2::Argon2;
use rand::RngCore;
use solana_sdk::signature::Keypair;

use crate::AppResult;

pub fn derive_key(
	encryption_secret: &[u8],
	salt: &SaltString,
	key_index: u32,
) -> AppResult<Vec<u8>> {
	let argon2 = Argon2::default();
	let encryption_secret_with_index =
		[encryption_secret, key_index.to_string().as_bytes()].concat();
	let password_hash = argon2.hash_password(&encryption_secret_with_index, salt)?;
	let key = PasswordHash::new(&password_hash.to_string())?
		.hash
		.ok_or(anyhow::anyhow!("could not unwrap hash"))?;
	let result = key.as_bytes().to_vec();

	Ok(result)
}

pub(crate) fn encrypt_keypair(
	keypair: &Keypair,
	encryption_key: &[u8],
	nonce: &Nonce,
) -> AppResult<Vec<u8>> {
	let cipher = Aes256GcmSiv::new(encryption_key.into());
	let serialized_keypair = keypair.to_bytes();

	let ciphertext = cipher
		.encrypt(nonce, serialized_keypair.as_ref())
		.context("encryption failure!")?;

	Ok(ciphertext)
}

pub(crate) fn decrypt_keypair(
	ciphertext: &[u8],
	decryption_key: &[u8],
	nonce: &Nonce,
) -> AppResult<Keypair> {
	let cipher = Aes256GcmSiv::new(decryption_key.into());
	let decrypted_bytes = cipher
		.decrypt(nonce, ciphertext.as_ref())
		.context("decryption failure!")?;
	let keypair = Keypair::from_bytes(&decrypted_bytes).context("keypair generation failed")?;

	Ok(keypair)
}

pub(crate) fn generate_nonce() -> Nonce {
	let mut nonce_bytes = [0u8; 12];
	OsRng.fill_bytes(&mut nonce_bytes);

	*Nonce::from_slice(&nonce_bytes)
}

pub(crate) fn generate_salt() -> SaltString {
	SaltString::generate(&mut OsRng)
}
