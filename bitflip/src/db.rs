use std::str::FromStr;
#[cfg(feature = "ssr")]
use std::sync::Arc;

#[cfg(feature = "ssr")]
use anyhow::Context;
use bitflip_program::GameStatus;
use bitflip_program::SectionData;
use bitflip_program::BITFLIP_SECTION_LENGTH;
use chrono::DateTime;
use chrono::Utc;
use num_enum::IntoPrimitive;
use num_enum::TryFromPrimitive;
use rand::rngs::OsRng;
use serde::Deserialize;
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use uuid::Uuid;
#[cfg(feature = "ssr")]
use welds::connections::sqlite::connect;
#[cfg(feature = "ssr")]
use welds::connections::sqlite::SqliteClient;
#[cfg(feature = "ssr")]
use welds::state::DbState;
#[cfg(feature = "ssr")]
use welds::WeldsModel;

use crate::encryption::decrypt_keypair;
use crate::encryption::derive_key;
use crate::AppResult;
use crate::AppStateConfig;

#[cfg(feature = "ssr")]
#[derive(derive_more::Debug, Clone, derive_more::From, derive_more::Into, derive_more::Deref)]
pub struct Db(#[debug(skip)] Arc<SqliteClient>);

impl Db {
	#[cfg(feature = "ssr")]
	pub async fn try_new(connection_string: &str) -> AppResult<Self> {
		let connection = connect(connection_string)
			.await
			.context("could not connect to the database")?;

		Ok(Arc::new(connection).into())
	}

	#[cfg(feature = "ssr")]
	pub fn as_sqlx_pool(&self) -> &sqlx::SqlitePool {
		self.0.as_sqlx_pool()
	}

	#[cfg(feature = "ssr")]
	pub fn as_client(&self) -> &SqliteClient {
		self.0.as_ref()
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(WeldsModel))]
#[cfg_attr(feature = "ssr", welds(table = "games"))]
#[cfg_attr(feature = "ssr", welds(HasMany(section, Section, "game_index")))]
pub struct Game {
	#[cfg_attr(feature = "ssr", welds(primary_key))]
	id: Uuid,
	game_index: u8,
	start_time: DateTime<Utc>,
	duration: i64,
	min_lamports: i64,
	base_lamports: i64,
	max_lamports: i64,
	status: u8,
	section_index: u8,
	game_signer: String,
	temp_signer: String,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl Game {
	pub fn set_id(&mut self) -> &mut Self {
		self.id = Uuid::now_v7();
		self
	}

	pub fn set_game_index(&mut self, game_index: u8) -> &mut Self {
		self.game_index = game_index;
		self
	}

	pub fn set_start_time(&mut self, start_time: DateTime<Utc>) -> &mut Self {
		self.start_time = start_time;
		self
	}

	pub fn set_duration(&mut self, duration: i64) -> &mut Self {
		self.duration = duration;
		self
	}

	pub fn set_min_lamports(&mut self, lamports: u64) -> &mut Self {
		self.min_lamports = lamports as i64;
		self
	}

	pub fn set_base_lamports(&mut self, lamports: u64) -> &mut Self {
		self.base_lamports = lamports as i64;
		self
	}

	pub fn set_max_lamports(&mut self, lamports: u64) -> &mut Self {
		self.max_lamports = lamports as i64;
		self
	}

	pub fn set_status(&mut self, status: GameStatus) -> &mut Self {
		self.status = status.into();
		self
	}

	pub fn set_section_index(&mut self, section_index: u8) -> &mut Self {
		self.section_index = section_index;
		self
	}

	/// Store the permanent `game_signer`
	pub fn set_game_signer(&mut self, pubkey: &Pubkey) -> &mut Self {
		self.game_signer = pubkey.to_string();
		self
	}

	/// Set the temporary signer pubkey for the game.
	pub fn set_temp_signer(&mut self, pubkey: &Pubkey) -> &mut Self {
		self.temp_signer = pubkey.to_string();
		self
	}

	pub fn id(&self) -> Uuid {
		self.id
	}

	pub fn game_index(&self) -> u8 {
		self.game_index
	}

	pub fn start_time(&self) -> DateTime<Utc> {
		self.start_time
	}

	pub fn duration(&self) -> i64 {
		self.duration
	}

	pub fn min_lamports(&self) -> u64 {
		self.min_lamports as u64
	}

	pub fn base_lamports(&self) -> u64 {
		self.base_lamports as u64
	}

	pub fn max_lamports(&self) -> u64 {
		self.max_lamports as u64
	}

	pub fn status(&self) -> GameStatus {
		GameStatus::try_from(self.status).unwrap_or(GameStatus::Pending)
	}

	pub fn section_index(&self) -> u8 {
		self.section_index
	}

	pub fn game_signer(&self) -> Pubkey {
		Pubkey::from_str_const(&self.game_signer)
	}

	pub fn temp_signer(&self) -> Pubkey {
		Pubkey::from_str_const(&self.temp_signer)
	}

	pub fn created_at(&self) -> DateTime<Utc> {
		self.created_at
	}

	pub fn updated_at(&self) -> DateTime<Utc> {
		self.updated_at
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(WeldsModel))]
#[cfg_attr(feature = "ssr", welds(table = "sections"))]
#[cfg_attr(
	feature = "ssr",
	welds(HasMany(section_event, SectionEvent, "section_id"))
)]
#[cfg_attr(feature = "ssr", welds(BelongsTo(game, Game, "game_index")))]
pub struct Section {
	#[cfg_attr(feature = "ssr", welds(primary_key))]
	id: Uuid,
	game_index: u8,
	section_index: u8,
	data: Vec<u8>,
	reward_tokens: i64,
	lamports: i64,
	owner: String,
	flips: u32,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl Section {
	pub fn set_id(&mut self) -> &mut Self {
		self.id = Uuid::now_v7();
		self
	}

	pub fn set_game_index(&mut self, game_index: u8) -> &mut Self {
		self.game_index = game_index;
		self
	}

	pub fn set_section_index(&mut self, section_index: u8) -> &mut Self {
		self.section_index = section_index;
		self
	}

	pub fn set_data(&mut self, data: SectionData) -> &mut Self {
		self.data = {
			let v: &SectionData = &data;
			v.iter().flat_map(|x| x.to_le_bytes()).collect::<Vec<u8>>()
		};
		self
	}

	pub fn set_reward_tokens(&mut self, reward_tokens: u64) -> &mut Self {
		self.reward_tokens = reward_tokens as i64;
		self
	}

	pub fn set_lamports(&mut self, lamports: u64) -> &mut Self {
		self.lamports = lamports as i64;
		self
	}

	pub fn set_owner(&mut self, owner: Pubkey) -> &mut Self {
		self.owner = owner.to_string();
		self
	}

	pub fn set_flips(&mut self, flips: u32) -> &mut Self {
		self.flips = flips;
		self
	}

	pub fn id(&self) -> Uuid {
		self.id
	}

	pub fn game_index(&self) -> u8 {
		self.game_index
	}

	pub fn section_index(&self) -> u8 {
		self.section_index
	}

	pub fn data(&self) -> SectionData {
		let bytes: &[u8] = &self.data;
		assert_eq!(bytes.len(), 512, "Input bytes must be exactly 512 bytes");
		let mut result = [0u16; BITFLIP_SECTION_LENGTH];

		for (ii, chunk) in bytes.chunks_exact(2).enumerate() {
			result[ii] = u16::from_le_bytes([chunk[0], chunk[1]]);
		}

		result
	}

	pub fn reward_tokens(&self) -> u64 {
		self.reward_tokens as u64
	}

	pub fn lamports(&self) -> u64 {
		self.lamports as u64
	}

	pub fn owner(&self) -> Pubkey {
		Pubkey::from_str(&self.owner).unwrap()
	}

	pub fn flips(&self) -> u32 {
		self.flips
	}

	pub fn created_at(&self) -> DateTime<Utc> {
		self.created_at
	}

	pub fn updated_at(&self) -> DateTime<Utc> {
		self.updated_at
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(WeldsModel))]
#[cfg_attr(feature = "ssr", welds(table = "section_events"))]
#[cfg_attr(feature = "ssr", welds(BelongsTo(player, Player, "player_pubkey")))]
#[cfg_attr(feature = "ssr", welds(BelongsTo(section, Section, "section_id")))]
pub struct SectionEvent {
	#[cfg_attr(feature = "ssr", welds(primary_key))]
	id: Uuid,
	section_id: Uuid,
	player_pubkey: String,
	event_type: u8,
	data: Vec<u8>,
	created_at: DateTime<Utc>,
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[non_exhaustive]
pub enum SectionEventType {
	#[default]
	None,
	Color,
	Minesweeper,
}

impl SectionEvent {
	pub fn set_id(&mut self) -> &mut Self {
		self.id = Uuid::now_v7();
		self
	}

	pub fn set_section_id(&mut self, section_id: Uuid) -> &mut Self {
		self.section_id = section_id;
		self
	}

	pub fn set_player_pubkey(&mut self, player_pubkey: Pubkey) -> &mut Self {
		self.player_pubkey = player_pubkey.to_string();
		self
	}

	pub fn set_event_type(&mut self, event_type: SectionEventType) -> &mut Self {
		self.event_type = event_type.into();
		self
	}

	pub fn set_data(&mut self, data: Vec<u8>) -> &mut Self {
		self.data = data;
		self
	}

	pub fn id(&self) -> Uuid {
		self.id
	}

	pub fn section_id(&self) -> Uuid {
		self.section_id
	}

	pub fn player_pubkey(&self) -> Pubkey {
		Pubkey::from_str(&self.player_pubkey).unwrap()
	}

	pub fn event_type(&self) -> SectionEventType {
		SectionEventType::try_from(self.event_type).unwrap_or(SectionEventType::None)
	}

	pub fn data(&self) -> &[u8] {
		&self.data
	}

	pub fn created_at(&self) -> DateTime<Utc> {
		self.created_at
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(WeldsModel))]
#[cfg_attr(feature = "ssr", welds(table = "players"))]
#[cfg_attr(
	feature = "ssr",
	welds(HasMany(section_event, SectionEvent, "player_pubkey"))
)]
pub struct Player {
	#[welds(primary_key)]
	pubkey: String,
	signer: String,
	updated_at: DateTime<Utc>,
	created_at: DateTime<Utc>,
}

impl Player {
	pub fn set_pubkey(&mut self, pubkey: Pubkey) -> &mut Self {
		self.pubkey = pubkey.to_string();
		self
	}

	pub fn set_signer(&mut self, signer: &Pubkey) -> &mut Self {
		self.signer = signer.to_string();
		self
	}

	pub fn pubkey(&self) -> Pubkey {
		Pubkey::from_str(&self.pubkey).unwrap()
	}

	pub fn signer(&self) -> Pubkey {
		Pubkey::from_str_const(&self.signer)
	}

	pub fn created_at(&self) -> DateTime<Utc> {
		self.created_at
	}

	pub fn updated_at(&self) -> DateTime<Utc> {
		self.updated_at
	}
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "ssr", derive(WeldsModel))]
#[cfg_attr(feature = "ssr", welds(table = "encrypted_keypairs"))]
pub struct EncryptedKeypair {
	#[welds(primary_key)]
	pubkey: String,
	encrypted_keypair: Vec<u8>,
	key_index: u32,
	nonce: Vec<u8>,
	salt: String,
	status: u8,
	created_at: DateTime<Utc>,
	updated_at: DateTime<Utc>,
}

impl EncryptedKeypair {
	#[cfg(feature = "ssr")]
	pub async fn find_by_pubkey(
		client: &SqliteClient,
		pubkey: &Pubkey,
	) -> AppResult<Option<DbState<Self>>> {
		let result = Self::find_by_id(client, pubkey.to_string()).await?;

		Ok(result)
	}

	#[cfg(feature = "ssr")]
	pub fn generate(&mut self, config: &AppStateConfig) -> AppResult<Keypair> {
		use rand::Rng;
		use solana_sdk::signer::Signer;

		use crate::encryption::encrypt_keypair;
		use crate::encryption::generate_nonce;
		use crate::encryption::generate_salt;

		let key_index = OsRng
			.gen_range(0..config.max_active_keys)
			.saturating_add(config.key_multiplier.saturating_mul(config.max_active_keys));
		let keypair = Keypair::new();
		let nonce = generate_nonce();
		let salt = generate_salt();
		let encryption_key = derive_key(config.encryption_secret.as_bytes(), &salt, key_index)?;
		let encrypted_keypair = encrypt_keypair(&keypair, &encryption_key, &nonce)?;

		self.pubkey = keypair.pubkey().to_string();
		self.encrypted_keypair = encrypted_keypair;
		self.key_index = key_index;
		self.nonce = nonce.to_vec();
		self.salt = salt.to_string();
		self.status = EncryptedKeypairStatus::Active.into();

		Ok(keypair)
	}

	pub fn pubkey(&self) -> Pubkey {
		Pubkey::from_str(&self.pubkey).unwrap()
	}

	pub fn encrypted_keypair(&self) -> &[u8] {
		&self.encrypted_keypair
	}

	#[cfg(feature = "ssr")]
	pub fn keypair(&self, config: &AppStateConfig) -> AppResult<Keypair> {
		use aes_gcm_siv::Nonce;
		use argon2::password_hash::SaltString;

		let nonce = Nonce::from_slice(self.nonce());
		let salt = SaltString::from_b64(self.salt())?;
		let key_index = self.key_index;
		let encrypted_keypair = &self.encrypted_keypair;
		let decryption_key = derive_key(config.encryption_secret.as_bytes(), &salt, key_index)?;
		let result = decrypt_keypair(encrypted_keypair, &decryption_key, nonce)?;

		Ok(result)
	}

	pub fn nonce(&self) -> &[u8] {
		&self.nonce
	}

	pub fn salt(&self) -> &str {
		&self.salt
	}

	pub fn status(&self) -> EncryptedKeypairStatus {
		EncryptedKeypairStatus::try_from(self.status).unwrap_or(EncryptedKeypairStatus::Active)
	}

	pub fn created_at(&self) -> DateTime<Utc> {
		self.created_at
	}

	pub fn updated_at(&self) -> DateTime<Utc> {
		self.updated_at
	}
}

#[repr(u8)]
#[derive(Default, Clone, Copy, Debug, Eq, PartialEq, TryFromPrimitive, IntoPrimitive)]
#[non_exhaustive]
pub enum EncryptedKeypairStatus {
	/// The keypair is active and can be used
	#[default]
	Active,
	/// The keypair has been revoked and emptied.
	Revoked,
	/// The keypair has been compromised and should not be used
	Compromised,
}

#[cfg(all(feature = "ssr", test))]
mod tests {
	use solana_sdk::signer::Signer;
	use test_log::test;

	use super::*;
	use crate::AppEnvironment;

	async fn setup_db() -> AppResult<Db> {
		let db = Db::try_new("sqlite::memory:").await?;
		sqlx::migrate!("../migrations")
			.run(db.as_sqlx_pool())
			.await?;

		Ok(db)
	}

	#[test(tokio::test)]
	async fn games_table_snapshot() -> AppResult<()> {
		let db = setup_db().await?;
		let diff = welds::check::schema::<Game>(db.as_ref()).await?;
		insta::assert_debug_snapshot!(diff);

		Ok(())
	}

	#[test(tokio::test)]
	async fn sections_table_snapshot() -> AppResult<()> {
		let db = setup_db().await?;
		let diff = welds::check::schema::<Section>(db.as_ref()).await?;
		insta::assert_debug_snapshot!(diff);

		Ok(())
	}

	#[test(tokio::test)]
	async fn section_events_table_snapshot() -> AppResult<()> {
		let db = setup_db().await?;
		let diff = welds::check::schema::<SectionEvent>(db.as_ref()).await?;
		insta::assert_debug_snapshot!(diff);

		Ok(())
	}

	#[test(tokio::test)]
	async fn players_table_snapshot() -> AppResult<()> {
		let db = setup_db().await?;
		let diff = welds::check::schema::<Player>(db.as_ref()).await?;
		insta::assert_debug_snapshot!(diff);

		Ok(())
	}

	#[test(tokio::test)]
	async fn encrypted_keypairs_table_snapshot() -> AppResult<()> {
		let db = setup_db().await?;
		let diff = welds::check::schema::<EncryptedKeypair>(db.as_ref()).await?;
		insta::assert_debug_snapshot!(diff);

		Ok(())
	}

	#[test(tokio::test)]
	async fn encrypted_keypair_generate() -> AppResult<()> {
		let db = setup_db().await?;
		let client = db.as_client();
		let config = AppStateConfig {
			encryption_secret: "test encryption secret".to_string(),
			key_multiplier: 0,
			max_active_keys: 512,
			environment: AppEnvironment::Local,
			database_url: "sqlite::memory:".parse()?,
			website_url: "http://localhost:3000".parse()?,
		};
		let mut encrypted_keypair = EncryptedKeypair::new();
		let keypair = encrypted_keypair.generate(&config)?;
		encrypted_keypair.save(client).await?;

		let result = EncryptedKeypair::find_by_id(client, keypair.pubkey().to_string())
			.await?
			.unwrap();

		assert_eq!(result.keypair(&config)?, keypair);

		Ok(())
	}
}
