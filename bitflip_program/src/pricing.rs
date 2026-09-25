//! Deterministic pricing primitives for a section's finite BIT allocation.
//!
//! This module deliberately contains no account access or token transfers. The
//! same integer arithmetic can therefore drive host-side simulations now and
//! the on-chain price controller after its account ABI is approved.

use core::cmp::{max, min};

/// Number of base BIT that one flip transaction can request.
pub const MAX_REWARD_TOKENS_PER_TRANSACTION: u64 = 16;

/// Staging BIT allocation shared by base issuance and the section reward pool.
pub const DEFAULT_SECTION_ALLOCATION_TOKENS: u64 = crate::BIT_SECTION_ALLOCATION_TOKENS;

/// Staging rewarded-pixel target for one control window.
pub const DEFAULT_TARGET_TOKENS_PER_WINDOW: u64 = 1_024;

/// Sixty-day staging emission period.
pub const DEFAULT_EMISSION_DURATION_SECONDS: u64 = 60 * 24 * 60 * 60;

/// Five-minute staging control window.
pub const DEFAULT_WINDOW_SECONDS: u64 = 5 * 60;

/// Staging controller price before a section has observed traffic.
pub const DEFAULT_START_PRICE_LAMPORTS: u64 = 10_000;

/// Lowest staging price for one issued BIT.
pub const DEFAULT_MIN_PRICE_LAMPORTS: u64 = 5_000;

/// Highest staging price for one issued BIT.
pub const DEFAULT_MAX_PRICE_LAMPORTS: u64 = 1_000_000;

/// Inventory floor reached when the section allocation is exhausted.
pub const DEFAULT_END_FLOOR_PRICE_LAMPORTS: u64 = 100_000;

/// Maximum controller movement is one eighth, or 12.5%, per window.
pub const DEFAULT_CHANGE_DENOMINATOR: u64 = 8;

/// A window can issue at most twice its target before base rewards pause.
pub const DEFAULT_BURST_ELASTICITY: u64 = 2;

/// Basis points in one whole fee share.
pub const BASIS_POINTS_DENOMINATOR: u16 = 10_000;

/// Proposed staging share of section fees paid to a user-owned section.
pub const DEFAULT_OWNER_SHARE_BASIS_POINTS: u16 = 2_000;

/// Invalid input or stale quote encountered by the controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PriceControllerError {
	/// The immutable section parameters are internally inconsistent.
	InvalidConfiguration,
	/// A transaction asked to issue no tokens or more than one flip batch.
	InvalidRequest,
	/// A timestamp predates launch or moves backwards relative to accepted state.
	InvalidTimestamp,
	/// The quote belongs to a control window that has already closed.
	StaleWindow,
	/// The posted unit price or total price exceeds the player's signed limit.
	PriceSlippage,
	/// Less BIT is available than the player's signed minimum reward.
	InsufficientReward,
	/// Integer arithmetic could not represent the requested transition.
	ArithmeticOverflow,
}

/// Immutable price-controller parameters snapshotted when a game is created.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriceControllerConfig {
	/// Maximum base BIT that this section may issue.
	pub allocation_tokens: u64,
	/// Seconds from section launch until base issuance ends.
	pub emission_duration_seconds: u64,
	/// Seconds in one price-control window.
	pub window_seconds: u64,
	/// Sustainable rewarded-token target for every control window.
	pub target_tokens_per_window: u64,
	/// Initial congestion-controller price.
	pub start_price_lamports: u64,
	/// Hard lower bound for the controller and posted price.
	pub minimum_price_lamports: u64,
	/// Hard upper bound for the controller and posted price.
	pub maximum_price_lamports: u64,
	/// Inventory floor at zero issuance.
	pub start_floor_price_lamports: u64,
	/// Inventory floor at full issuance.
	pub end_floor_price_lamports: u64,
	/// Denominator that bounds a control step, eight for 12.5%.
	pub change_denominator: u64,
	/// Multiple of target that may be issued during one window.
	pub burst_elasticity: u64,
}

impl PriceControllerConfig {
	/// Staging parameters used by the deterministic simulation.
	pub const STAGING: Self = Self {
		allocation_tokens: DEFAULT_SECTION_ALLOCATION_TOKENS,
		emission_duration_seconds: DEFAULT_EMISSION_DURATION_SECONDS,
		window_seconds: DEFAULT_WINDOW_SECONDS,
		target_tokens_per_window: DEFAULT_TARGET_TOKENS_PER_WINDOW,
		start_price_lamports: DEFAULT_START_PRICE_LAMPORTS,
		minimum_price_lamports: DEFAULT_MIN_PRICE_LAMPORTS,
		maximum_price_lamports: DEFAULT_MAX_PRICE_LAMPORTS,
		start_floor_price_lamports: DEFAULT_MIN_PRICE_LAMPORTS,
		end_floor_price_lamports: DEFAULT_END_FLOOR_PRICE_LAMPORTS,
		change_denominator: DEFAULT_CHANGE_DENOMINATOR,
		burst_elasticity: DEFAULT_BURST_ELASTICITY,
	};

	/// Validate parameters before they are committed to an account.
	///
	/// # Errors
	///
	/// Returns [`PriceControllerError::InvalidConfiguration`] when a duration,
	/// price bound, floor, or capacity parameter is invalid.
	pub fn validate(&self) -> Result<(), PriceControllerError> {
		let duration_is_valid = self.emission_duration_seconds > 0
			&& self.window_seconds > 0
			&& self.window_seconds <= self.emission_duration_seconds
			&& self
				.emission_duration_seconds
				.is_multiple_of(self.window_seconds);
		let prices_are_valid = self.minimum_price_lamports > 0
			&& self.minimum_price_lamports <= self.start_price_lamports
			&& self.start_price_lamports <= self.maximum_price_lamports
			&& self.minimum_price_lamports <= self.start_floor_price_lamports
			&& self.start_floor_price_lamports <= self.end_floor_price_lamports
			&& self.end_floor_price_lamports <= self.maximum_price_lamports
			&& self
				.maximum_price_lamports
				.checked_mul(MAX_REWARD_TOKENS_PER_TRANSACTION)
				.is_some();
		let control_is_valid = self.allocation_tokens > 0
			&& self.target_tokens_per_window > 0
			&& self.change_denominator >= 2
			&& self.start_price_lamports / self.change_denominator > 0
			&& self.burst_elasticity >= 1;

		if duration_is_valid && prices_are_valid && control_is_valid {
			Ok(())
		} else {
			Err(PriceControllerError::InvalidConfiguration)
		}
	}
}

/// Player-signed limits that bind a quote to one controller window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct QuoteLimits {
	/// Window observed when the player approved the transaction.
	pub expected_window_id: u64,
	/// Highest acceptable price for one issued BIT.
	pub maximum_unit_price_lamports: u64,
	/// Highest acceptable issuance charge for this transaction.
	pub maximum_total_price_lamports: u64,
	/// Lowest acceptable BIT payout.
	pub minimum_reward_tokens: u64,
}

/// Deterministic quote for one transaction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriceQuote {
	/// Current section-local control window.
	pub window_id: u64,
	/// Target base issuance for the current window.
	pub target_tokens: u64,
	/// BIT issued to this transaction after capacity constraints.
	pub reward_tokens: u64,
	/// Posted price for one issued BIT, fixed for the window.
	pub unit_price_lamports: u64,
	/// Total issuance charge for this transaction.
	pub total_price_lamports: u64,
	/// Reward capacity left after this transaction.
	pub remaining_window_capacity: u64,
	/// BIT accrued for future protocol-defined section rewards.
	pub reward_pool_tokens: u64,
}

/// Protocol and owner portions of one paid issuance charge.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FeeSplit {
	/// Lamports retained by the protocol.
	pub protocol_lamports: u64,
	/// Lamports accrued for the section owner.
	pub owner_lamports: u64,
}

/// Mutable, section-local state for the time-based price controller.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PriceControllerState {
	/// Unix timestamp at which this section's independent emission clock began.
	pub launched_at: u64,
	/// Beginning of the current five-minute window.
	pub window_started_at: u64,
	/// Latest timestamp accepted by a state transition.
	pub last_updated_at: u64,
	/// Monotonic identifier used to reject stale quotes.
	pub window_id: u64,
	/// Target BIT issuance snapshotted for this window.
	pub window_target_tokens: u64,
	/// BIT issued during this window.
	pub window_rewarded_tokens: u64,
	/// BIT issued over the section's lifetime.
	pub emitted_tokens: u64,
	/// Unclaimed target issuance reserved for future section rewards.
	pub reward_pool_tokens: u64,
	/// Congestion component before applying the inventory floor.
	pub controller_price_lamports: u64,
	/// Price fixed for the current window.
	pub posted_price_lamports: u64,
}

impl PriceControllerState {
	/// Create a controller for a newly launched section.
	///
	/// # Errors
	///
	/// Returns an error for invalid configuration or unrepresentable arithmetic.
	pub fn new(
		config: &PriceControllerConfig,
		launched_at: u64,
	) -> Result<Self, PriceControllerError> {
		config.validate()?;
		launched_at
			.checked_add(config.emission_duration_seconds)
			.ok_or(PriceControllerError::ArithmeticOverflow)?;
		let window_target_tokens = min(config.target_tokens_per_window, config.allocation_tokens);
		let posted_price_lamports = max(config.start_price_lamports, inventory_floor(config, 0)?);

		Ok(Self {
			launched_at,
			window_started_at: launched_at,
			last_updated_at: launched_at,
			window_id: 0,
			window_target_tokens,
			window_rewarded_tokens: 0,
			emitted_tokens: 0,
			reward_pool_tokens: 0,
			controller_price_lamports: config.start_price_lamports,
			posted_price_lamports,
		})
	}

	/// Timestamp at which this section stops issuing base BIT.
	///
	/// # Errors
	///
	/// Returns an error if the configured duration exceeds timestamp capacity.
	pub fn emission_ends_at(
		&self,
		config: &PriceControllerConfig,
	) -> Result<u64, PriceControllerError> {
		self.launched_at
			.checked_add(config.emission_duration_seconds)
			.ok_or(PriceControllerError::ArithmeticOverflow)
	}

	/// Total BIT already issued or reserved for future section rewards.
	///
	/// # Errors
	///
	/// Returns an error if corrupted state cannot be represented.
	pub fn accounted_tokens(&self) -> Result<u64, PriceControllerError> {
		self.emitted_tokens
			.checked_add(self.reward_pool_tokens)
			.ok_or(PriceControllerError::ArithmeticOverflow)
	}

	/// BIT not yet issued or moved into the section reward pool.
	///
	/// # Errors
	///
	/// Returns an error for invalid configuration or corrupted over-allocation.
	pub fn remaining_base_tokens(
		&self,
		config: &PriceControllerConfig,
	) -> Result<u64, PriceControllerError> {
		config.validate()?;
		config
			.allocation_tokens
			.checked_sub(self.accounted_tokens()?)
			.ok_or(PriceControllerError::ArithmeticOverflow)
	}

	/// Settle completed windows without executing a flip.
	///
	/// This is the transition campaign administration uses before reading the
	/// reward pool. It remains atomic when time or arithmetic is invalid.
	///
	/// # Errors
	///
	/// Returns an error for invalid time, configuration, or arithmetic.
	pub fn settle(
		&mut self,
		config: &PriceControllerConfig,
		now: u64,
	) -> Result<(), PriceControllerError> {
		let mut next = *self;
		next.advance_to(config, now)?;
		*self = next;
		Ok(())
	}

	/// Preview a transaction without mutating accepted state.
	///
	/// # Errors
	///
	/// Returns an error for invalid time, request size, or arithmetic.
	pub fn preview(
		&self,
		config: &PriceControllerConfig,
		now: u64,
		requested_reward_tokens: u64,
	) -> Result<PriceQuote, PriceControllerError> {
		let mut next = *self;
		next.advance_to(config, now)?;
		next.quote(config, requested_reward_tokens)
	}

	/// Execute a quoted issuance transition atomically.
	///
	/// # Errors
	///
	/// Returns an error without changing state if the quote is stale, limits are
	/// exceeded, rewards are insufficient, time is invalid, or arithmetic fails.
	pub fn execute(
		&mut self,
		config: &PriceControllerConfig,
		now: u64,
		requested_reward_tokens: u64,
		limits: QuoteLimits,
	) -> Result<PriceQuote, PriceControllerError> {
		let mut next = *self;
		next.advance_to(config, now)?;
		let quote = next.quote(config, requested_reward_tokens)?;

		if quote.window_id != limits.expected_window_id {
			return Err(PriceControllerError::StaleWindow);
		}

		if quote.unit_price_lamports > limits.maximum_unit_price_lamports
			|| quote.total_price_lamports > limits.maximum_total_price_lamports
		{
			return Err(PriceControllerError::PriceSlippage);
		}

		if quote.reward_tokens < limits.minimum_reward_tokens {
			return Err(PriceControllerError::InsufficientReward);
		}

		next.window_rewarded_tokens = next
			.window_rewarded_tokens
			.checked_add(quote.reward_tokens)
			.ok_or(PriceControllerError::ArithmeticOverflow)?;
		next.emitted_tokens = next
			.emitted_tokens
			.checked_add(quote.reward_tokens)
			.ok_or(PriceControllerError::ArithmeticOverflow)?;
		next.last_updated_at = now;
		*self = next;

		Ok(quote)
	}

	fn advance_to(
		&mut self,
		config: &PriceControllerConfig,
		now: u64,
	) -> Result<(), PriceControllerError> {
		config.validate()?;

		if now < self.launched_at || now < self.last_updated_at {
			return Err(PriceControllerError::InvalidTimestamp);
		}

		let emission_ends_at = self.emission_ends_at(config)?;
		let effective_now = min(now, emission_ends_at);
		let elapsed = effective_now
			.checked_sub(self.window_started_at)
			.ok_or(PriceControllerError::InvalidTimestamp)?;
		let completed_windows = elapsed / config.window_seconds;

		if completed_windows > 0 {
			self.controller_price_lamports = adjusted_controller_price(
				config,
				self.controller_price_lamports,
				self.window_rewarded_tokens,
				self.window_target_tokens,
			)?;

			let completed_shortfall = self
				.window_target_tokens
				.saturating_sub(self.window_rewarded_tokens);
			self.accrue_reward_pool(config, completed_shortfall)?;

			let missed_empty_windows = completed_windows - 1;

			if missed_empty_windows > 0 {
				self.controller_price_lamports = decay_empty_windows(
					config,
					self.controller_price_lamports,
					missed_empty_windows,
				)?;
				let remaining = self.remaining_base_tokens(config)?;
				let missed_target = u128::from(config.target_tokens_per_window)
					.checked_mul(u128::from(missed_empty_windows))
					.ok_or(PriceControllerError::ArithmeticOverflow)?
					.min(u128::from(remaining));
				let missed_target = u64::try_from(missed_target)
					.map_err(|_| PriceControllerError::ArithmeticOverflow)?;
				self.accrue_reward_pool(config, missed_target)?;
			}

			let advance_seconds = completed_windows
				.checked_mul(config.window_seconds)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			self.window_started_at = self
				.window_started_at
				.checked_add(advance_seconds)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			self.window_id = self
				.window_id
				.checked_add(completed_windows)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			self.window_rewarded_tokens = 0;

			if self.window_started_at >= emission_ends_at {
				let remaining = self.remaining_base_tokens(config)?;
				self.accrue_reward_pool(config, remaining)?;
			}

			let remaining = self.remaining_base_tokens(config)?;

			if self.window_started_at < emission_ends_at && remaining > 0 {
				self.window_target_tokens = min(config.target_tokens_per_window, remaining);
				self.posted_price_lamports = max(
					self.controller_price_lamports,
					inventory_floor(config, self.emitted_tokens)?,
				);
			} else {
				self.window_target_tokens = 0;
				self.posted_price_lamports = max(
					self.controller_price_lamports,
					inventory_floor(config, self.emitted_tokens)?,
				);
			}
		}

		self.last_updated_at = now;
		Ok(())
	}

	fn accrue_reward_pool(
		&mut self,
		config: &PriceControllerConfig,
		requested_tokens: u64,
	) -> Result<(), PriceControllerError> {
		let accrued = min(requested_tokens, self.remaining_base_tokens(config)?);
		self.reward_pool_tokens = self
			.reward_pool_tokens
			.checked_add(accrued)
			.ok_or(PriceControllerError::ArithmeticOverflow)?;
		Ok(())
	}

	fn quote(
		&self,
		config: &PriceControllerConfig,
		requested_reward_tokens: u64,
	) -> Result<PriceQuote, PriceControllerError> {
		if requested_reward_tokens == 0
			|| requested_reward_tokens > MAX_REWARD_TOKENS_PER_TRANSACTION
		{
			return Err(PriceControllerError::InvalidRequest);
		}

		let remaining_inventory = self.remaining_base_tokens(config)?;
		let window_capacity = window_capacity(config, self.window_target_tokens)?;
		let remaining_window_capacity = window_capacity.saturating_sub(self.window_rewarded_tokens);
		let available = min(remaining_inventory, remaining_window_capacity);
		let reward_tokens = min(requested_reward_tokens, available);
		let total_price_lamports = self
			.posted_price_lamports
			.checked_mul(reward_tokens)
			.ok_or(PriceControllerError::ArithmeticOverflow)?;

		Ok(PriceQuote {
			window_id: self.window_id,
			target_tokens: self.window_target_tokens,
			reward_tokens,
			unit_price_lamports: self.posted_price_lamports,
			total_price_lamports,
			remaining_window_capacity: available - reward_tokens,
			reward_pool_tokens: self.reward_pool_tokens,
		})
	}
}

/// Calculate the inventory floor after `emitted_tokens` have left the reserve.
///
/// # Errors
///
/// Returns an error for invalid configuration, over-issuance, or overflow.
pub fn inventory_floor(
	config: &PriceControllerConfig,
	emitted_tokens: u64,
) -> Result<u64, PriceControllerError> {
	config.validate()?;

	if emitted_tokens > config.allocation_tokens {
		return Err(PriceControllerError::ArithmeticOverflow);
	}

	let floor_range = config
		.end_floor_price_lamports
		.checked_sub(config.start_floor_price_lamports)
		.ok_or(PriceControllerError::InvalidConfiguration)?;
	let increase = u128::from(floor_range)
		.checked_mul(u128::from(emitted_tokens))
		.ok_or(PriceControllerError::ArithmeticOverflow)?
		/ u128::from(config.allocation_tokens);
	let result = u128::from(config.start_floor_price_lamports)
		.checked_add(increase)
		.ok_or(PriceControllerError::ArithmeticOverflow)?;

	u64::try_from(result).map_err(|_| PriceControllerError::ArithmeticOverflow)
}

/// Calculate a completed window's bounded controller update.
///
/// # Errors
///
/// Returns an error when the target is zero or arithmetic is unrepresentable.
pub fn adjusted_controller_price(
	config: &PriceControllerConfig,
	current_price_lamports: u64,
	rewarded_tokens: u64,
	target_tokens: u64,
) -> Result<u64, PriceControllerError> {
	config.validate()?;

	if target_tokens == 0 {
		return Ok(config.minimum_price_lamports);
	}

	let bounded_rewarded = min(rewarded_tokens, target_tokens.saturating_mul(2));
	let distance = bounded_rewarded.abs_diff(target_tokens);
	let maximum_change = config.start_price_lamports / config.change_denominator;
	let change = u128::from(maximum_change)
		.checked_mul(u128::from(distance))
		.ok_or(PriceControllerError::ArithmeticOverflow)?
		/ u128::from(target_tokens);
	let change = u64::try_from(change).map_err(|_| PriceControllerError::ArithmeticOverflow)?;
	let adjusted = if bounded_rewarded >= target_tokens {
		current_price_lamports
			.checked_add(change)
			.ok_or(PriceControllerError::ArithmeticOverflow)?
	} else {
		current_price_lamports.saturating_sub(change)
	};

	Ok(adjusted.clamp(config.minimum_price_lamports, config.maximum_price_lamports))
}

/// Split a fee without creating or losing a lamport to rounding.
///
/// The owner portion rounds down and the protocol receives the remainder. A
/// protocol-owned section should pass an owner share of zero.
///
/// # Errors
///
/// Returns an error when `owner_share_basis_points` exceeds 100%.
pub fn split_fee(
	total_lamports: u64,
	owner_share_basis_points: u16,
) -> Result<FeeSplit, PriceControllerError> {
	if owner_share_basis_points > BASIS_POINTS_DENOMINATOR {
		return Err(PriceControllerError::InvalidConfiguration);
	}

	let owner_lamports = u128::from(total_lamports)
		.checked_mul(u128::from(owner_share_basis_points))
		.ok_or(PriceControllerError::ArithmeticOverflow)?
		/ u128::from(BASIS_POINTS_DENOMINATOR);
	let owner_lamports =
		u64::try_from(owner_lamports).map_err(|_| PriceControllerError::ArithmeticOverflow)?;
	let protocol_lamports = total_lamports
		.checked_sub(owner_lamports)
		.ok_or(PriceControllerError::ArithmeticOverflow)?;

	Ok(FeeSplit {
		protocol_lamports,
		owner_lamports,
	})
}

fn window_capacity(
	config: &PriceControllerConfig,
	target_tokens: u64,
) -> Result<u64, PriceControllerError> {
	let capacity = u128::from(target_tokens)
		.checked_mul(u128::from(config.burst_elasticity))
		.ok_or(PriceControllerError::ArithmeticOverflow)?;
	let capacity = min(capacity, u128::from(config.allocation_tokens));

	u64::try_from(capacity).map_err(|_| PriceControllerError::ArithmeticOverflow)
}

fn decay_empty_windows(
	config: &PriceControllerConfig,
	current_price_lamports: u64,
	empty_windows: u64,
) -> Result<u64, PriceControllerError> {
	let maximum_change = config.start_price_lamports / config.change_denominator;
	let total_change = u128::from(maximum_change)
		.checked_mul(u128::from(empty_windows))
		.ok_or(PriceControllerError::ArithmeticOverflow)?
		.min(u128::from(u64::MAX));
	let total_change =
		u64::try_from(total_change).map_err(|_| PriceControllerError::ArithmeticOverflow)?;
	let decayed = current_price_lamports.saturating_sub(total_change);

	Ok(decayed.clamp(config.minimum_price_lamports, config.maximum_price_lamports))
}

#[cfg(test)]
mod tests {
	use super::*;

	pub(super) fn exact_limits(quote: PriceQuote) -> QuoteLimits {
		QuoteLimits {
			expected_window_id: quote.window_id,
			maximum_unit_price_lamports: quote.unit_price_lamports,
			maximum_total_price_lamports: quote.total_price_lamports,
			minimum_reward_tokens: quote.reward_tokens,
		}
	}

	#[test]
	fn staging_target_and_capacity_are_binary_and_conservative() {
		let state = PriceControllerState::new(&PriceControllerConfig::STAGING, 0)
			.expect("valid staging controller");

		assert_eq!(DEFAULT_SECTION_ALLOCATION_TOKENS, 100 * 262_144);
		assert!(DEFAULT_TARGET_TOKENS_PER_WINDOW.is_power_of_two());
		assert_eq!(state.window_target_tokens, 1_024);
		assert_eq!(
			window_capacity(&PriceControllerConfig::STAGING, state.window_target_tokens,),
			Ok(2_048)
		);
	}

	#[test]
	fn bounded_adjustment_matches_the_decision_examples() {
		let config = PriceControllerConfig::STAGING;

		assert_eq!(
			adjusted_controller_price(&config, 10_000, 0, 1_024),
			Ok(8_750)
		);
		assert_eq!(
			adjusted_controller_price(&config, 10_000, 512, 1_024),
			Ok(9_375)
		);
		assert_eq!(
			adjusted_controller_price(&config, 10_000, 1_024, 1_024),
			Ok(10_000)
		);
		assert_eq!(
			adjusted_controller_price(&config, 10_000, 1_536, 1_024),
			Ok(10_625)
		);
		assert_eq!(
			adjusted_controller_price(&config, 10_000, 2_048, 1_024),
			Ok(11_250)
		);
		assert_eq!(
			adjusted_controller_price(&config, 10_000, u64::MAX, 1_024),
			Ok(11_250)
		);
	}

	#[test]
	fn posted_price_is_constant_and_capacity_is_hard_capped() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 1_000).expect("valid controller");
		let first = state.preview(&config, 1_001, 16).expect("first quote");
		state
			.execute(&config, 1_001, 16, exact_limits(first))
			.expect("first batch");

		for _ in 1..128 {
			let quote = state.preview(&config, 1_299, 16).expect("next quote");
			state
				.execute(&config, 1_299, 16, exact_limits(quote))
				.expect("next batch");
		}

		let exhausted = state.preview(&config, 1_299, 1).expect("exhausted quote");

		assert_eq!(first.unit_price_lamports, DEFAULT_START_PRICE_LAMPORTS);
		assert_eq!(state.posted_price_lamports, first.unit_price_lamports);
		assert_eq!(state.window_rewarded_tokens, 2_048);
		assert_eq!(exhausted.reward_tokens, 0);
		assert_eq!(exhausted.total_price_lamports, 0);
	}

	#[test]
	fn stale_window_and_reward_limits_fail_atomically() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 0).expect("valid controller");
		let original = state;
		let stale_quote = state.preview(&config, 299, 1).expect("window zero quote");

		assert_eq!(
			state.execute(&config, 300, 1, exact_limits(stale_quote)),
			Err(PriceControllerError::StaleWindow)
		);
		assert_eq!(state, original);

		let current = state.preview(&config, 300, 16).expect("current quote");
		let impossible_reward = QuoteLimits {
			minimum_reward_tokens: current.reward_tokens + 1,
			..exact_limits(current)
		};
		assert_eq!(
			state.execute(&config, 300, 16, impossible_reward),
			Err(PriceControllerError::InsufficientReward)
		);
		assert_eq!(state, original);
	}

	#[test]
	fn missed_windows_decay_in_constant_time_and_stop_at_the_floor() {
		let config = PriceControllerConfig::STAGING;
		let state = PriceControllerState::new(&config, 0).expect("valid controller");
		let after_one_day = state
			.preview(&config, 24 * 60 * 60, 1)
			.expect("large timestamp jump");

		assert_eq!(
			after_one_day.unit_price_lamports,
			config.minimum_price_lamports
		);
		assert_eq!(after_one_day.window_id, 288);
		assert_eq!(after_one_day.reward_pool_tokens, 288 * 1_024);
	}

	#[test]
	fn unclaimed_partial_window_inventory_moves_to_the_reward_pool() {
		let config = PriceControllerConfig {
			allocation_tokens: 33,
			emission_duration_seconds: 900,
			..PriceControllerConfig::STAGING
		};
		let mut state = PriceControllerState::new(&config, 0).expect("valid controller");

		for _ in 0..2 {
			let quote = state.preview(&config, 0, 16).expect("initial quote");
			state
				.execute(&config, 0, 16, exact_limits(quote))
				.expect("initial execution");
		}

		state.settle(&config, 300).expect("settle partial window");
		let exhausted = state.preview(&config, 300, 16).expect("pooled quote");

		assert_eq!(state.emitted_tokens, 32);
		assert_eq!(state.reward_pool_tokens, 1);
		assert_eq!(state.accounted_tokens(), Ok(config.allocation_tokens));
		assert_eq!(exhausted.target_tokens, 0);
		assert_eq!(exhausted.reward_tokens, 0);
		assert_eq!(exhausted.total_price_lamports, 0);
	}

	#[test]
	fn below_target_issuance_accrues_the_exact_shortfall() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 0).expect("valid controller");

		for _ in 0..32 {
			let quote = state.preview(&config, 1, 16).expect("half-target quote");
			state
				.execute(&config, 1, 16, exact_limits(quote))
				.expect("half-target execution");
		}

		state.settle(&config, 300).expect("settle first window");

		assert_eq!(state.emitted_tokens, 512);
		assert_eq!(state.reward_pool_tokens, 512);
		assert_eq!(state.accounted_tokens(), Ok(1_024));
		assert_eq!(state.remaining_base_tokens(&config), Ok(26_213_376));
	}

	#[test]
	fn settling_a_window_twice_cannot_duplicate_or_reverse_pool_accrual() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 0).expect("valid controller");

		state.settle(&config, 300).expect("settle first window");
		let settled = state;
		state
			.settle(&config, 300)
			.expect("repeat settlement at the same timestamp");

		assert_eq!(state, settled);
		assert_eq!(state.reward_pool_tokens, 1_024);
		assert_eq!(state.emitted_tokens, 0);
	}

	#[test]
	fn expiry_moves_every_unallocated_token_to_the_reward_pool() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 0).expect("valid controller");

		state
			.settle(&config, config.emission_duration_seconds)
			.expect("settle expiry");

		assert_eq!(state.emitted_tokens, 0);
		assert_eq!(state.reward_pool_tokens, config.allocation_tokens);
		assert_eq!(state.remaining_base_tokens(&config), Ok(0));
		assert_eq!(state.window_target_tokens, 0);
	}

	#[test]
	fn late_idle_section_cannot_turn_remaining_inventory_into_a_catch_up_dump() {
		let config = PriceControllerConfig::STAGING;
		let state = PriceControllerState::new(&config, 0).expect("valid controller");
		let final_window = config.emission_duration_seconds - config.window_seconds;
		let quote = state
			.preview(&config, final_window, MAX_REWARD_TOKENS_PER_TRANSACTION)
			.expect("final-window quote");

		assert_eq!(quote.target_tokens, 1_024);
		assert_eq!(quote.reward_tokens, 16);
		assert_eq!(quote.remaining_window_capacity, 2_032);
		assert_eq!(quote.reward_pool_tokens, 17_693_696);
		assert_eq!(quote.unit_price_lamports, config.minimum_price_lamports);
	}

	#[test]
	fn backwards_time_is_rejected_without_mutation() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 100).expect("valid controller");
		let quote = state.preview(&config, 200, 1).expect("forward quote");
		state
			.execute(&config, 200, 1, exact_limits(quote))
			.expect("forward execution");
		let accepted = state;

		assert_eq!(
			state.preview(&config, 199, 1),
			Err(PriceControllerError::InvalidTimestamp)
		);
		assert_eq!(state, accepted);
	}

	#[test]
	fn launch_time_must_leave_room_for_the_emission_period() {
		let config = PriceControllerConfig::STAGING;

		assert_eq!(
			PriceControllerState::new(&config, u64::MAX),
			Err(PriceControllerError::ArithmeticOverflow)
		);
	}

	#[test]
	fn owner_fee_recovery_does_not_erase_manipulation_cost() {
		let split =
			split_fee(160_000, DEFAULT_OWNER_SHARE_BASIS_POINTS).expect("valid staging split");

		assert_eq!(split.owner_lamports, 32_000);
		assert_eq!(split.protocol_lamports, 128_000);
		assert_eq!(split.owner_lamports + split.protocol_lamports, 160_000);
	}

	// Property and fuzz coverage moved in-crate from `tests/pricing_properties.rs`.
	use proptest::prelude::*;

	fn execute_exact(
		state: &mut PriceControllerState,
		config: &PriceControllerConfig,
		now: u64,
		requested: u64,
	) -> PriceQuote {
		let quote = state
			.preview(config, now, requested)
			.expect("generated transaction has a valid quote");
		state
			.execute(config, now, requested, exact_limits(quote))
			.expect("exact quote executes")
	}

	proptest! {
		#![proptest_config(ProptestConfig {
			cases: 1_024,
			max_shrink_iters: 20_000,
			..ProptestConfig::default()
		})]

		#[test]
		fn controller_price_is_bounded_and_one_step_is_limited(
			current in 5_000u64..=1_000_000,
			target in 1u64..=1_000_000,
			rewarded in any::<u64>(),
		) {
			let config = PriceControllerConfig::STAGING;
			let adjusted = adjusted_controller_price(&config, current, rewarded, target)
				.expect("bounded inputs cannot overflow");
			let maximum_change = config.start_price_lamports / config.change_denominator;

			prop_assert!(adjusted >= config.minimum_price_lamports);
			prop_assert!(adjusted <= config.maximum_price_lamports);
			prop_assert!(adjusted.abs_diff(current) <= maximum_change);
		}

		#[test]
		fn inventory_floor_is_monotonic(
			first in 0u64..=DEFAULT_SECTION_ALLOCATION_TOKENS,
			second in 0u64..=DEFAULT_SECTION_ALLOCATION_TOKENS,
		) {
			let config = PriceControllerConfig::STAGING;
			let low = first.min(second);
			let high = first.max(second);
			let low_floor = inventory_floor(&config, low).expect("valid inventory");
			let high_floor = inventory_floor(&config, high).expect("valid inventory");

			prop_assert!(low_floor <= high_floor);
			prop_assert!(low_floor >= config.start_floor_price_lamports);
			prop_assert!(high_floor <= config.end_floor_price_lamports);
		}

		#[test]
		fn adversarial_stream_never_overissues_or_exceeds_window_capacity(
			transactions in prop::collection::vec((0u64..=900, 1u64..=16), 0..512),
		) {
			let config = PriceControllerConfig::STAGING;
			let mut state = PriceControllerState::new(&config, 0).expect("valid controller");
			let mut now = 0u64;

			for (delay, requested) in transactions {
				let previous_pool = state.reward_pool_tokens;
				now = now.saturating_add(delay);
				let _ = execute_exact(&mut state, &config, now, requested);
				let capacity = state
					.window_target_tokens
					.saturating_mul(config.burst_elasticity)
					.min(config.allocation_tokens);

				let accounted = state.accounted_tokens().expect("bounded accounting");
				prop_assert!(accounted <= config.allocation_tokens);
				prop_assert!(state.reward_pool_tokens >= previous_pool);
				prop_assert_eq!(
					state.remaining_base_tokens(&config),
					Ok(config.allocation_tokens - accounted),
				);
				prop_assert!(state.window_rewarded_tokens <= capacity);
				prop_assert!(state.controller_price_lamports >= config.minimum_price_lamports);
				prop_assert!(state.controller_price_lamports <= config.maximum_price_lamports);
				prop_assert!(state.posted_price_lamports >= config.minimum_price_lamports);
				prop_assert!(state.posted_price_lamports <= config.maximum_price_lamports);
			}
		}

		#[test]
		fn transaction_order_cannot_change_same_window_outcome(
			requests in prop::collection::vec(1u64..=16, 1..64),
		) {
			let config = PriceControllerConfig::STAGING;
			let mut forward = PriceControllerState::new(&config, 0).expect("valid controller");
			let mut reversed = forward;
			let mut forward_paid = 0u64;
			let mut reversed_paid = 0u64;

			for requested in &requests {
				forward_paid += execute_exact(&mut forward, &config, 1, *requested).total_price_lamports;
			}

			for requested in requests.iter().rev() {
				reversed_paid += execute_exact(&mut reversed, &config, 1, *requested).total_price_lamports;
			}

			prop_assert_eq!(forward, reversed);
			prop_assert_eq!(forward_paid, reversed_paid);
		}

		#[test]
		fn preview_and_execute_are_deterministic(
			now in 0u64..=10_368_000,
			requested in 1u64..=MAX_REWARD_TOKENS_PER_TRANSACTION,
		) {
			let config = PriceControllerConfig::STAGING;
			let mut first = PriceControllerState::new(&config, 0).expect("valid controller");
			let mut second = first;
			let first_quote = first.preview(&config, now, requested).expect("valid quote");
			let second_quote = second.preview(&config, now, requested).expect("valid quote");

			prop_assert_eq!(first_quote, second_quote);
			let first_result = first.execute(&config, now, requested, exact_limits(first_quote));
			let second_result = second.execute(&config, now, requested, exact_limits(second_quote));
			prop_assert_eq!(first_result, second_result);
			prop_assert_eq!(first, second);
		}
	}

	#[test]
	fn sybil_splitting_does_not_increase_reward_or_reduce_price() {
		let config = PriceControllerConfig::STAGING;
		let mut full_batches = PriceControllerState::new(&config, 0).expect("valid controller");
		let mut split_wallets = full_batches;
		let mut batched_paid = 0;
		let mut sybil_paid = 0;

		for _ in 0..128 {
			batched_paid += execute_exact(&mut full_batches, &config, 1, 16).total_price_lamports;
		}

		for _ in 0..2_048 {
			sybil_paid += execute_exact(&mut split_wallets, &config, 1, 1).total_price_lamports;
		}

		assert_eq!(full_batches, split_wallets);
		assert_eq!(batched_paid, sybil_paid);
		assert_eq!(full_batches.emitted_tokens, 2_048);
		assert_eq!(full_batches.reward_pool_tokens, 0);
		assert_eq!(
			full_batches.preview(&config, 1, 1),
			Ok(PriceQuote {
				window_id: 0,
				target_tokens: 1_024,
				reward_tokens: 0,
				unit_price_lamports: 10_000,
				total_price_lamports: 0,
				remaining_window_capacity: 0,
				reward_pool_tokens: 0,
			})
		);
	}

	#[test]
	fn fixed_window_boundary_allows_two_caps_but_charges_the_higher_second_price() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 0).expect("valid controller");
		let mut rewards = 0;
		let mut paid = 0;

		for _ in 0..128 {
			let quote = execute_exact(&mut state, &config, 299, 16);
			rewards += quote.reward_tokens;
			paid += quote.total_price_lamports;
		}

		for _ in 0..128 {
			let quote = execute_exact(&mut state, &config, 300, 16);
			rewards += quote.reward_tokens;
			paid += quote.total_price_lamports;
		}

		assert_eq!(
			rewards, 4_096,
			"two adjacent windows expose two finite caps"
		);
		assert_eq!(paid, 43_520_000, "the second cap costs 12.5% more");
		assert_eq!(state.posted_price_lamports, 11_250);
	}

	#[test]
	fn alternating_equal_pressure_cannot_ratchet_the_price_down() {
		let config = PriceControllerConfig::STAGING;
		let mut state = PriceControllerState::new(&config, 0).expect("valid controller");

		for cycle in 0..64 {
			let busy_at = cycle * config.window_seconds * 2;

			for _ in 0..128 {
				let _ = execute_exact(&mut state, &config, busy_at, 16);
			}

			let idle_at = busy_at + config.window_seconds;
			let _ = state
				.preview(&config, idle_at, 1)
				.expect("settle busy window");
			let next_busy_at = idle_at + config.window_seconds;
			let _ = state
				.preview(&config, next_busy_at, 1)
				.expect("settle idle window");
		}

		let after_cycles = 64 * config.window_seconds * 2;
		let quote = state
			.preview(&config, after_cycles, 1)
			.expect("quote after balanced traffic");
		assert_eq!(quote.unit_price_lamports, config.start_price_lamports);
	}

	#[test]
	fn stale_or_impossible_limits_leave_state_unchanged() {
		let config = PriceControllerConfig::STAGING;
		let cases = [
			QuoteLimits {
				expected_window_id: 9,
				maximum_unit_price_lamports: u64::MAX,
				maximum_total_price_lamports: u64::MAX,
				minimum_reward_tokens: 0,
			},
			QuoteLimits {
				expected_window_id: 0,
				maximum_unit_price_lamports: 9_999,
				maximum_total_price_lamports: u64::MAX,
				minimum_reward_tokens: 0,
			},
			QuoteLimits {
				expected_window_id: 0,
				maximum_unit_price_lamports: u64::MAX,
				maximum_total_price_lamports: 9_999,
				minimum_reward_tokens: 0,
			},
			QuoteLimits {
				expected_window_id: 0,
				maximum_unit_price_lamports: u64::MAX,
				maximum_total_price_lamports: u64::MAX,
				minimum_reward_tokens: 2,
			},
		];
		let expected_errors = [
			PriceControllerError::StaleWindow,
			PriceControllerError::PriceSlippage,
			PriceControllerError::PriceSlippage,
			PriceControllerError::InsufficientReward,
		];

		for (limits, expected_error) in cases.into_iter().zip(expected_errors) {
			let mut state = PriceControllerState::new(&config, 0).expect("valid controller");
			let before = state;
			assert_eq!(state.execute(&config, 1, 1, limits), Err(expected_error));
			assert_eq!(state, before);
		}
	}

	#[test]
	fn representable_u64_extremes_do_not_overflow_or_loop_per_window() {
		let maximum_price = u64::MAX / MAX_REWARD_TOKENS_PER_TRANSACTION;
		let config = PriceControllerConfig {
			allocation_tokens: u64::MAX,
			emission_duration_seconds: u64::MAX,
			window_seconds: 1,
			target_tokens_per_window: u64::MAX,
			start_price_lamports: maximum_price,
			minimum_price_lamports: 1,
			maximum_price_lamports: maximum_price,
			start_floor_price_lamports: 1,
			end_floor_price_lamports: maximum_price,
			change_denominator: 2,
			burst_elasticity: u64::MAX,
		};
		let mut state =
			PriceControllerState::new(&config, 0).expect("extreme config is representable");
		let first = state
			.preview(&config, 0, 16)
			.expect("extreme initial quote");
		state
			.execute(&config, 0, 16, exact_limits(first))
			.expect("extreme initial execution");
		let end = state
			.preview(&config, u64::MAX, 16)
			.expect("constant-time full-duration jump");

		assert_eq!(end.window_id, u64::MAX);
		assert_eq!(end.reward_tokens, 0);
		assert_eq!(end.total_price_lamports, 0);
		assert_eq!(end.reward_pool_tokens, u64::MAX - 16);
	}
}
#[cfg(test)]
mod economics_simulation {
	//! The runnable economic simulation from the removed `examples/` directory.
	//! The crate is `no_std`, so the scenarios assert their invariants instead
	//! of printing a table: rewarded tokens cannot exceed emission, and every
	//! paid lamport must be split between owner and protocol.
	use super::*;

	#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
	struct ScenarioTotals {
		rewarded_tokens: u64,
		player_paid_lamports: u64,
		owner_recovered_lamports: u64,
		protocol_received_lamports: u64,
	}

	fn attempt_window(
		state: &mut PriceControllerState,
		config: &PriceControllerConfig,
		now: u64,
		mut requested_tokens: u64,
		owner_share_basis_points: u16,
	) -> Result<ScenarioTotals, PriceControllerError> {
		let mut totals = ScenarioTotals::default();
		while requested_tokens > 0 {
			let batch = requested_tokens.min(MAX_REWARD_TOKENS_PER_TRANSACTION);
			let quote = state.preview(config, now, batch)?;
			let accepted = state.execute(config, now, batch, tests::exact_limits(quote))?;
			let split = split_fee(accepted.total_price_lamports, owner_share_basis_points)?;

			totals.rewarded_tokens = totals
				.rewarded_tokens
				.checked_add(accepted.reward_tokens)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			totals.player_paid_lamports = totals
				.player_paid_lamports
				.checked_add(accepted.total_price_lamports)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			totals.owner_recovered_lamports = totals
				.owner_recovered_lamports
				.checked_add(split.owner_lamports)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			totals.protocol_received_lamports = totals
				.protocol_received_lamports
				.checked_add(split.protocol_lamports)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			requested_tokens -= batch;
		}

		Ok(totals)
	}

	fn run_pattern(
		name: &str,
		config: &PriceControllerConfig,
		requests_by_window: &[u64],
	) -> Result<(), PriceControllerError> {
		let mut state = PriceControllerState::new(config, 0)?;

		let mut totals = ScenarioTotals::default();
		for (window, requested_tokens) in requests_by_window.iter().copied().enumerate() {
			let now = u64::try_from(window)
				.map_err(|_| PriceControllerError::ArithmeticOverflow)?
				.checked_mul(config.window_seconds)
				.ok_or(PriceControllerError::ArithmeticOverflow)?;
			let window_totals = attempt_window(
				&mut state,
				config,
				now,
				requested_tokens,
				DEFAULT_OWNER_SHARE_BASIS_POINTS,
			)?;
			totals.rewarded_tokens += window_totals.rewarded_tokens;
			totals.player_paid_lamports += window_totals.player_paid_lamports;
			totals.owner_recovered_lamports += window_totals.owner_recovered_lamports;
			totals.protocol_received_lamports += window_totals.protocol_received_lamports;
		}

		let after_pattern = u64::try_from(requests_by_window.len())
			.map_err(|_| PriceControllerError::ArithmeticOverflow)?
			.checked_mul(config.window_seconds)
			.ok_or(PriceControllerError::ArithmeticOverflow)?;
		state.settle(config, after_pattern)?;

		// Invariants the printed table was eyeballed for.
		assert!(
			totals.rewarded_tokens <= state.emitted_tokens,
			"{name}: rewarded tokens cannot exceed emission"
		);
		assert!(
			totals.owner_recovered_lamports + totals.protocol_received_lamports
				== totals.player_paid_lamports,
			"{name}: owner and protocol shares must split every paid lamport"
		);
		// Every rewarded token was minted by this program.
		assert!(state.emitted_tokens >= totals.rewarded_tokens);
		Ok(())
	}

	#[test]
	fn scenarios_run_to_completion_within_bounds() {
		let config = PriceControllerConfig::STAGING;

		run_pattern("idle", &config, &[0; 12]).unwrap();
		run_pattern("at_target", &config, &[1_024; 12]).unwrap();
		run_pattern("saturated", &config, &[4_096; 12]).unwrap();
		run_pattern(
			"oscillating",
			&config,
			&[2_048, 0, 2_048, 0, 2_048, 0, 2_048, 0, 2_048, 0, 2_048, 0],
		)
		.unwrap();

		// A late burst after a long idle stretch must still settle and quote.
		let mut late = PriceControllerState::new(&config, 0).unwrap();
		let final_window = config.emission_duration_seconds - config.window_seconds;
		let totals = attempt_window(
			&mut late,
			&config,
			final_window,
			102_400,
			DEFAULT_OWNER_SHARE_BASIS_POINTS,
		)
		.unwrap();
		late.settle(&config, config.emission_duration_seconds)
			.unwrap();

		assert!(totals.rewarded_tokens > 0);
		assert!(late.emitted_tokens > 0);
		assert!(late.reward_pool_tokens >= totals.rewarded_tokens);
	}
}
