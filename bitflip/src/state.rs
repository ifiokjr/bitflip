use std::sync::Arc;

use anyhow::Context;
use axum::extract::FromRef;
use leptos::config::LeptosOptions;
use typed_builder::TypedBuilder;
use url::Url;

use crate::AppEnvironment;
use crate::AppResult;
use crate::db::Db;

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

impl AppStateConfig {
	pub fn from_env() -> AppResult<Self> {
		let environment = std::env::var("BITFLIP_ENVIRONMENT")
			.unwrap_or_else(|_| AppEnvironment::Local.to_string())
			.parse()
			.context("BITFLIP_ENVIRONMENT must be local, test, development, or production")?;
		let encryption_secret = std::env::var("BITFLIP_ENCRYPTION_SECRET")
			.unwrap_or_else(|_| "local-development-only-change-me".to_string());
		let key_multiplier = parse_env("BITFLIP_KEY_MULTIPLIER", 0)?;
		let max_active_keys = parse_env("BITFLIP_MAX_ACTIVE_KEYS", 512)?;
		let database_url = std::env::var("BITFLIP_DATABASE_URL")
			.unwrap_or_else(|_| "sqlite://bitflip.db?mode=rwc".to_string())
			.parse()
			.context("BITFLIP_DATABASE_URL must be a valid URL")?;
		let website_url = std::env::var("BITFLIP_WEBSITE_URL")
			.unwrap_or_else(|_| "http://127.0.0.1:3000".to_string())
			.parse()
			.context("BITFLIP_WEBSITE_URL must be a valid URL")?;
		let config = Self {
			encryption_secret,
			key_multiplier,
			max_active_keys,
			environment,
			database_url,
			website_url,
		};
		config.validate()?;

		Ok(config)
	}

	pub fn validate(&self) -> anyhow::Result<()> {
		anyhow::ensure!(
			self.database_url.scheme() == "sqlite",
			"BITFLIP_DATABASE_URL must use the sqlite scheme"
		);
		anyhow::ensure!(
			self.max_active_keys > 0,
			"BITFLIP_MAX_ACTIVE_KEYS must be greater than zero"
		);

		if self.environment == AppEnvironment::Production {
			anyhow::ensure!(
				self.encryption_secret.len() >= 32
					&& self.encryption_secret != "local-development-only-change-me",
				"BITFLIP_ENCRYPTION_SECRET must be a unique value of at least 32 bytes in production"
			);
			anyhow::ensure!(
				self.website_url.scheme() == "https",
				"BITFLIP_WEBSITE_URL must use HTTPS in production"
			);
		}

		Ok(())
	}
}

fn parse_env<T>(name: &str, default: T) -> AppResult<T>
where
	T: std::str::FromStr,
	T::Err: std::error::Error + Send + Sync + 'static,
{
	match std::env::var(name) {
		Ok(value) => value
			.parse()
			.with_context(|| format!("{name} is invalid"))
			.map_err(Into::into),
		Err(std::env::VarError::NotPresent) => Ok(default),
		Err(error) => Err(error.into()),
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn config(environment: AppEnvironment) -> AppStateConfig {
		AppStateConfig {
			encryption_secret: "local-development-only-change-me".to_string(),
			key_multiplier: 0,
			max_active_keys: 512,
			environment,
			database_url: "sqlite::memory:".parse().unwrap(),
			website_url: "http://127.0.0.1:3000".parse().unwrap(),
		}
	}

	#[test]
	fn local_defaults_are_valid() -> anyhow::Result<()> {
		config(AppEnvironment::Local).validate()
	}

	#[test]
	fn production_requires_a_secret_and_https() {
		let error = config(AppEnvironment::Production).validate().unwrap_err();
		assert!(error.to_string().contains("BITFLIP_ENCRYPTION_SECRET"));

		let mut config = config(AppEnvironment::Production);
		config.encryption_secret = "this-is-a-unique-production-secret".to_string();
		let error = config.validate().unwrap_err();
		assert!(error.to_string().contains("BITFLIP_WEBSITE_URL"));
	}
}
