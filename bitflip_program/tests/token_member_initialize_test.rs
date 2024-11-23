#![cfg(feature = "client")]

use std::future::Future;

use assert2::check;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::TokenMember;
use bitflip_program::get_pda_treasury;
use bitflip_program::get_treasury_token_account;
use bitflip_program::token_member_initialize;
use rstest::rstest;
use shared::ToRpcClient;
use shared::create_authority_keypair;
use shared::create_config_accounts;
use shared::create_program_context_with_factory;
use shared::create_token_group_accounts;
#[cfg(feature = "test_validator")]
use shared::testname;
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::transaction::VersionedTransaction;
use steel::*;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wasm_client_solana::solana_account_decoder::parse_account_data::SplTokenAdditionalData;
use wasm_client_solana::solana_account_decoder::parse_token::parse_token_v2;

mod shared;

#[rstest]
#[case::kibibit(TokenMember::Kibibit)]
#[case::mebibit(TokenMember::Mebibit)]
#[case::gibibit(TokenMember::Gibibit)]
#[test_log::test(tokio::test)]
async fn token_member_initialize_test(
	testname: String,
	#[case] member: TokenMember,
) -> anyhow::Result<()> {
	set_snapshot_suffix!("{}", testname);
	shared_token_member_initialize_test(create_banks_client_rpc, member).await?;
	Ok(())
}

#[cfg(feature = "test_validator")]
#[rstest]
#[case::kibibit(TokenMember::Kibibit)]
#[case::mebibit(TokenMember::Mebibit)]
#[case::gibibit(TokenMember::Gibibit)]
#[test_log::test(tokio::test)]
async fn token_member_initialize_test_validator(
	testname: String,
	#[case] member: TokenMember,
) -> anyhow::Result<()> {
	set_snapshot_suffix!("{}", testname);
	let compute_units = shared_token_member_initialize_test(create_validator_rpc, member).await?;
	let rounded_compute_units = bitflip_program::round_compute_units_up(compute_units);

	check!(rounded_compute_units <= 90_000);
	insta::assert_snapshot!(format!("{rounded_compute_units} CU"));
	shared::save_compute_units(
		format!("token_member_initialize:{}", member.symbol()).as_str(),
		compute_units,
		"Initialize a token member",
	)?;

	Ok(())
}

#[cfg(feature = "test_validator")]
async fn create_validator_rpc() -> anyhow::Result<impl ToRpcClient> {
	let mut accounts = create_config_accounts();
	accounts.extend(create_token_group_accounts());

	let runner = shared::create_runner_with_accounts(accounts).await;

	Ok(runner)
}

async fn create_banks_client_rpc() -> anyhow::Result<impl ToRpcClient> {
	let provider = create_program_context_with_factory(|p| {
		let mut accounts = create_config_accounts();
		accounts.extend(create_token_group_accounts());

		for (key, account) in accounts {
			p.add_account(key, account.into());
		}
	})
	.await?;
	Ok(provider)
}

async fn shared_token_member_initialize_test<
	T: ToRpcClient,
	Fut: Future<Output = anyhow::Result<T>>,
	Create: FnOnce() -> Fut,
>(
	create_provider: Create,
	member: TokenMember,
) -> anyhow::Result<u64> {
	let provider = create_provider().await?;
	let rpc = provider.to_rpc();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let treasury = get_pda_treasury().0;
	// let mint_group = get_pda_mint_bit().0;
	let mint_member = Pubkey::find_program_address(&member.seeds(), &bitflip_program::ID).0;
	let treasury_member_token_account = get_treasury_token_account(&treasury, &mint_member);
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let instruction = token_member_initialize(&authority, member);
	let compute_limit_instruction = ComputeBudgetInstruction::set_compute_unit_limit(1_400_000);
	let mut transaction = VersionedTransaction::new_unsigned_v0(
		&authority,
		&[compute_limit_instruction, instruction],
		&[],
		recent_blockhash,
	)?;

	let simulation = rpc.simulate_transaction(&transaction).await?;
	log::info!("simulation: {simulation:#?}");

	let compute_units = simulation.value.units_consumed.unwrap();

	transaction.try_sign(&[&authority_keypair], None)?;

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	rpc.confirm_transaction(&signature).await?;

	let authority_redaction = create_insta_redaction(treasury, "treasury:pubkey");
	let mint_member_redaction = create_insta_redaction(mint_member, "mint_member:pubkey");
	let mint_member_data = rpc.get_account_data(&mint_member).await?;
	let mint_member_account = parse_token_v2(
		&mint_member_data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("mint_member account: {mint_member_account:#?}");
	insta::assert_compact_json_snapshot!(mint_member_account, {
		".info.mintAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.freezeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.metadataAddress" =>	insta::dynamic_redaction(mint_member_redaction.clone()),
	  ".info.extensions[1].state.closeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[2].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[2].state.groupAddress" => insta::dynamic_redaction(mint_member_redaction.clone()),
	  ".info.extensions[3].state.updateAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[3].state.mint" =>	insta::dynamic_redaction(mint_member_redaction.clone())
	});

	let data = rpc.get_account_data(&treasury_member_token_account).await?;
	let parsed_treasury_member_token_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("token account: {parsed_treasury_member_token_account:#?}");
	insta::assert_compact_json_snapshot!(parsed_treasury_member_token_account, {
		".info.mint" => insta::dynamic_redaction(mint_member_redaction),
		".info.owner" => insta::dynamic_redaction(authority_redaction),
	});

	Ok(compute_units)
}
