use std::str::FromStr;

use bitflip_program::BITFLIP_COLOR_COUNT;
use bitflip_program::BitFlipped;
use solana_sdk::pubkey::Pubkey;
use sqlx::Row;
use sqlx::SqlitePool;

use crate::AppResult;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ChainEventPosition {
	pub slot: u64,
	pub transaction_index: u32,
	pub instruction_index: u16,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectionOutcome {
	Applied,
	Duplicate,
	Stale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ColorPixel {
	pub array_index: u8,
	pub offset: u8,
	pub color: u8,
	pub player: Pubkey,
}

/// Persist a finalized color event and update its materialized pixel.
///
/// Callers must only submit events from successful, finalized transactions.
/// The event ledger makes retries idempotent, while the position tuple makes
/// out-of-order delivery safe.
pub async fn project_color_flip(
	pool: &SqlitePool,
	signature: &str,
	position: ChainEventPosition,
	event: &BitFlipped,
) -> AppResult<ProjectionOutcome> {
	validate_event(signature, event)?;

	let slot = i64::try_from(position.slot)?;
	let transaction_index = i64::from(position.transaction_index);
	let instruction_index = i64::from(position.instruction_index);
	let mut transaction = pool.begin().await?;
	let inserted = sqlx::query(
		r"
		INSERT OR IGNORE INTO color_flip_events (
			signature,
			instruction_index,
			slot,
			transaction_index,
			player_pubkey,
			game_index,
			section_index,
			array_index,
			bit_offset,
			value,
			color
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
		",
	)
	.bind(signature)
	.bind(instruction_index)
	.bind(slot)
	.bind(transaction_index)
	.bind(event.player.to_string())
	.bind(i64::from(event.game_index))
	.bind(i64::from(event.section_index))
	.bind(i64::from(event.array_index))
	.bind(i64::from(event.offset))
	.bind(i64::from(event.value))
	.bind(i64::from(event.color))
	.execute(&mut *transaction)
	.await?;

	if inserted.rows_affected() == 0 {
		transaction.commit().await?;
		return Ok(ProjectionOutcome::Duplicate);
	}

	let projected = sqlx::query(
		r"
		INSERT INTO section_color_pixels (
			game_index,
			section_index,
			array_index,
			bit_offset,
			value,
			color,
			player_pubkey,
			slot,
			transaction_index,
			instruction_index,
			signature
		) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
		ON CONFLICT (game_index, section_index, array_index, bit_offset)
		DO UPDATE SET
			value = excluded.value,
			color = excluded.color,
			player_pubkey = excluded.player_pubkey,
			slot = excluded.slot,
			transaction_index = excluded.transaction_index,
			instruction_index = excluded.instruction_index,
			signature = excluded.signature,
			updated_at = CURRENT_TIMESTAMP
		WHERE (
			excluded.slot,
			excluded.transaction_index,
			excluded.instruction_index
		) > (
			section_color_pixels.slot,
			section_color_pixels.transaction_index,
			section_color_pixels.instruction_index
		)
		",
	)
	.bind(i64::from(event.game_index))
	.bind(i64::from(event.section_index))
	.bind(i64::from(event.array_index))
	.bind(i64::from(event.offset))
	.bind(i64::from(event.value))
	.bind(i64::from(event.color))
	.bind(event.player.to_string())
	.bind(slot)
	.bind(transaction_index)
	.bind(instruction_index)
	.bind(signature)
	.execute(&mut *transaction)
	.await?;
	transaction.commit().await?;

	Ok(if projected.rows_affected() == 0 {
		ProjectionOutcome::Stale
	} else {
		ProjectionOutcome::Applied
	})
}

pub async fn load_section_colors(
	pool: &SqlitePool,
	game_index: u8,
	section_index: u8,
) -> AppResult<Vec<ColorPixel>> {
	let rows = sqlx::query(
		r"
		SELECT array_index, bit_offset, color, player_pubkey
		FROM section_color_pixels
		WHERE game_index = ? AND section_index = ? AND value = 1
		ORDER BY array_index, bit_offset
		",
	)
	.bind(i64::from(game_index))
	.bind(i64::from(section_index))
	.fetch_all(pool)
	.await?;

	rows.into_iter()
		.map(|row| {
			Ok(ColorPixel {
				array_index: u8::try_from(row.try_get::<i64, _>("array_index")?)?,
				offset: u8::try_from(row.try_get::<i64, _>("bit_offset")?)?,
				color: u8::try_from(row.try_get::<i64, _>("color")?)?,
				player: Pubkey::from_str(row.try_get::<String, _>("player_pubkey")?.as_str())?,
			})
		})
		.collect()
}

fn validate_event(signature: &str, event: &BitFlipped) -> anyhow::Result<()> {
	anyhow::ensure!(!signature.trim().is_empty(), "signature must not be empty");
	anyhow::ensure!(event.offset < 16, "bit offset must be less than 16");
	anyhow::ensure!(event.value <= 1, "bit value must be 0 or 1");
	anyhow::ensure!(
		event.color < BITFLIP_COLOR_COUNT,
		"color index must be less than {BITFLIP_COLOR_COUNT}"
	);

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	async fn setup_db() -> AppResult<SqlitePool> {
		let pool = SqlitePool::connect("sqlite::memory:").await?;
		sqlx::migrate!("../migrations").run(&pool).await?;
		Ok(pool)
	}

	fn event(value: u8, color: u8) -> BitFlipped {
		BitFlipped {
			player: Pubkey::new_unique(),
			game_index: 1,
			section_index: 2,
			array_index: 3,
			offset: 4,
			value,
			color,
		}
	}

	#[tokio::test]
	async fn projection_is_idempotent_and_ordered() -> AppResult<()> {
		let pool = setup_db().await?;
		let initial = event(1, 2);
		let initial_position = ChainEventPosition {
			slot: 10,
			transaction_index: 1,
			instruction_index: 0,
		};

		assert_eq!(
			project_color_flip(&pool, "initial", initial_position, &initial).await?,
			ProjectionOutcome::Applied
		);
		assert_eq!(
			project_color_flip(&pool, "initial", initial_position, &initial).await?,
			ProjectionOutcome::Duplicate
		);

		let cleared = event(0, 7);
		assert_eq!(
			project_color_flip(
				&pool,
				"clear",
				ChainEventPosition {
					slot: 11,
					transaction_index: 0,
					instruction_index: 0,
				},
				&cleared,
			)
			.await?,
			ProjectionOutcome::Applied
		);
		assert!(load_section_colors(&pool, 1, 2).await?.is_empty());

		let stale = event(1, 5);
		assert_eq!(
			project_color_flip(
				&pool,
				"stale",
				ChainEventPosition {
					slot: 9,
					transaction_index: 99,
					instruction_index: 0,
				},
				&stale,
			)
			.await?,
			ProjectionOutcome::Stale
		);
		assert!(load_section_colors(&pool, 1, 2).await?.is_empty());

		let latest = event(1, 6);
		assert_eq!(
			project_color_flip(
				&pool,
				"latest",
				ChainEventPosition {
					slot: 11,
					transaction_index: 0,
					instruction_index: 1,
				},
				&latest,
			)
			.await?,
			ProjectionOutcome::Applied
		);
		assert_eq!(
			load_section_colors(&pool, 1, 2).await?,
			vec![ColorPixel {
				array_index: 3,
				offset: 4,
				color: 6,
				player: latest.player,
			}]
		);

		let event_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM color_flip_events")
			.fetch_one(&pool)
			.await?;
		assert_eq!(event_count, 4);

		Ok(())
	}

	#[tokio::test]
	async fn invalid_events_are_not_persisted() -> AppResult<()> {
		let pool = setup_db().await?;
		let invalid = event(1, BITFLIP_COLOR_COUNT);
		let result = project_color_flip(
			&pool,
			"invalid",
			ChainEventPosition {
				slot: 1,
				transaction_index: 0,
				instruction_index: 0,
			},
			&invalid,
		)
		.await;

		assert!(result.is_err());
		let event_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM color_flip_events")
			.fetch_one(&pool)
			.await?;
		assert_eq!(event_count, 0);

		Ok(())
	}
}
