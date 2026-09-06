use bitflip_program::pricing::DEFAULT_SECTION_ALLOCATION_TOKENS;
use bitflip_program::pricing::MAX_REWARD_TOKENS_PER_TRANSACTION;
use bitflip_program::pricing::PriceControllerConfig;
use bitflip_program::pricing::PriceControllerError;
use bitflip_program::pricing::PriceControllerState;
use bitflip_program::pricing::PriceQuote;
use bitflip_program::pricing::QuoteLimits;
use bitflip_program::pricing::adjusted_controller_price;
use bitflip_program::pricing::inventory_floor;
use proptest::prelude::*;

fn exact_limits(quote: PriceQuote) -> QuoteLimits {
	QuoteLimits {
		expected_window_id: quote.window_id,
		maximum_unit_price_lamports: quote.unit_price_lamports,
		maximum_total_price_lamports: quote.total_price_lamports,
		minimum_reward_tokens: quote.reward_tokens,
	}
}

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
	let mut state = PriceControllerState::new(&config, 0).expect("extreme config is representable");
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
