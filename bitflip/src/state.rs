use std::sync::Arc;

use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use typed_builder::TypedBuilder;
use url::Url;

use crate::db::Db;
use crate::AppEnvironment;

#[derive(FromRef, Debug, Clone, TypedBuilder)]
pub struct AppState {
	pub leptos: LeptosOptions,
	pub config: Arc<AppStateConfig>,
	pub db: Db,
}

#[derive(Debug, Clone)]
pub struct AppStateConfig {
	/// The encryption secret used to derive the encryption keys for keypairs.
	pub encryption_secret: String,
	/// The multiplier for the key index which is used to set the band that the
	/// `key_index` can fall within.
	pub key_multiplier: u32,
	/// The number of keys that can be active at a time.
	pub max_active_keys: u32,
	/// The environment that the app is running in.
	pub environment: AppEnvironment,
	/// The database URL.
	pub database_url: Url,
	/// The website URL.
	pub website_url: Url,
}
