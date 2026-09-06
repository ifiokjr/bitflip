use bitflip_program::pricing::DEFAULT_OWNER_SHARE_BASIS_POINTS;
use bitflip_program::pricing::MAX_REWARD_TOKENS_PER_TRANSACTION;
use bitflip_program::pricing::PriceControllerConfig;
use bitflip_program::pricing::PriceControllerError;
use bitflip_program::pricing::PriceControllerState;
use bitflip_program::pricing::PriceQuote;
use bitflip_program::pricing::QuoteLimits;
use bitflip_program::pricing::split_fee;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ScenarioTotals {
	rewarded_tokens: u64,
	player_paid_lamports: u64,
	owner_recovered_lamports: u64,
	protocol_received_lamports: u64,
}

fn exact_limits(quote: PriceQuote) -> QuoteLimits {
	QuoteLimits {
		expected_window_id: quote.window_id,
		maximum_unit_price_lamports: quote.unit_price_lamports,
		maximum_total_price_lamports: quote.total_price_lamports,
		minimum_reward_tokens: quote.reward_tokens,
	}
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
		let accepted = state.execute(config, now, batch, exact_limits(quote))?;
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
	let next = state.preview(config, after_pattern, 1)?;
	println!(
		"{name}\t{}\t{}\t{}\t{}\t{}\t{}",
		totals.rewarded_tokens,
		totals.player_paid_lamports,
		totals.owner_recovered_lamports,
		totals.protocol_received_lamports,
		next.unit_price_lamports,
		state.emitted_tokens,
	);
	Ok(())
}

fn main() -> Result<(), PriceControllerError> {
	let config = PriceControllerConfig::STAGING;
	println!(
		"scenario\trewarded_bit\tpaid_lamports\towner_recovery\tprotocol_revenue\tnext_price\temitted_bit"
	);
	run_pattern("idle", &config, &[0; 12])?;
	run_pattern("at_target", &config, &[1_600; 12])?;
	run_pattern("saturated", &config, &[6_400; 12])?;
	run_pattern(
		"oscillating",
		&config,
		&[3_200, 0, 3_200, 0, 3_200, 0, 3_200, 0, 3_200, 0, 3_200, 0],
	)?;

	let mut late = PriceControllerState::new(&config, 0)?;
	let final_window = config.emission_duration_seconds - config.window_seconds;
	let late_totals = attempt_window(
		&mut late,
		&config,
		final_window,
		102_400,
		DEFAULT_OWNER_SHARE_BASIS_POINTS,
	)?;
	println!(
		"idle_then_final_burst\t{}\t{}\t{}\t{}\t{}\t{}",
		late_totals.rewarded_tokens,
		late_totals.player_paid_lamports,
		late_totals.owner_recovered_lamports,
		late_totals.protocol_received_lamports,
		late.posted_price_lamports,
		late.emitted_tokens,
	);

	Ok(())
}
