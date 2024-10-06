#[cfg(feature = "test_banks_client")]
use anchor_spl::token_2022;
use bitflip_client::get_pda_config;
use bitflip_client::get_pda_mint;
use bitflip_client::get_pda_treasury;
use bitflip_program::ConfigState;
use bitflip_program::ID_CONST;
use bitflip_program::InitializeConfigProps;
use bitflip_program::InitializeTokenProps;
use bitflip_program::TOKEN_DECIMALS;
use bitflip_program::accounts::InitializeConfig;
use bitflip_program::accounts::InitializeToken;
use solana_sdk::account::Account;
use solana_sdk::signer::Signer;
use solana_sdk::system_program;
use spl_associated_token_account::get_associated_token_address_with_program_id;
use test_log::test;
use test_utils::create_insta_redaction;
use test_utils_solana::prelude::*;
use wasm_client_solana::solana_account_decoder::parse_account_data::SplTokenAdditionalData;
use wasm_client_solana::solana_account_decoder::parse_token::parse_token_v2;

use crate::shared::create_admin_keypair;
use crate::shared::create_authority_keypair;
use crate::shared::create_config_state;
use crate::shared::create_program_context_with_factory;
use crate::shared::create_rpc;
use crate::shared::get_admin_program;

mod shared;

#[test(tokio::test)]
async fn initialize_config_test() -> anyhow::Result<()> {
	let admin_keypair = create_admin_keypair();
	let admin = admin_keypair.pubkey();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let system_program = system_program::id();
	let (config, _) = get_pda_config();
	let (treasury, _) = get_pda_treasury();
	let mut ctx = create_program_context_with_factory(|p| {
		p.add_account(config, create_config_state(None).into());
	})
	.await?;

	let rpc = create_rpc();
	let bitflip_program_client = get_admin_program(&rpc);

	let initialize_config_request = bitflip_program_client
		.initialize_config()
		.args(InitializeConfigProps { authority })
		.accounts(InitializeConfig {
			config,
			admin,
			treasury,
			system_program,
		})
		.signers(vec![&admin_keypair])
		.build();

	let simulation = initialize_config_request
		.sign_and_simulate_banks_client_transaction(&mut ctx.banks_client)
		.await?;

	log::info!("simulation: {simulation:#?}");

	initialize_config_request
		.sign_and_process_banks_client_transaction(&mut ctx.banks_client)
		.await?;

	let config_state_account = ctx.get_anchor_account::<ConfigState>(&config).await?;

	insta::assert_yaml_snapshot!(config_state_account);

	Ok(())
}

#[test(tokio::test)]
async fn initialize_token() -> anyhow::Result<()> {
	let (config, _) = get_pda_config();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let (mint, _) = get_pda_mint();
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);
	let associated_token_program = spl_associated_token_account::ID;
	let system_program = system_program::ID;
	let bitflip_program = ID_CONST;

	let mut ctx = create_program_context_with_factory(|p| {
		p.add_account(config, create_config_state(None).into());
	})
	.await?;

	let rpc = create_rpc();
	let bitflip_program_client = get_admin_program(&rpc);

	let initialize_token_request = bitflip_program_client
		.initialize_token()
		.args(InitializeTokenProps {
			name: "Test".into(),
			symbol: "TEST".into(),
			uri: "https://test.com/config.json".into(),
		})
		.accounts(InitializeToken {
			config,
			authority,
			mint,
			treasury,
			treasury_token_account,
			associated_token_program,
			token_program,
			system_program,
			bitflip_program,
		})
		.signers(vec![&authority_keypair])
		.build();

	let simulation = initialize_token_request
		.sign_and_simulate_banks_client_transaction(&mut ctx.banks_client)
		.await?;

	log::info!("simulation: {simulation:#?}");

	initialize_token_request
		.sign_and_process_banks_client_transaction(&mut ctx.banks_client)
		.await?;
	let authority_redaction = create_insta_redaction(treasury, "authority:pubkey");
	let mint_redaction = create_insta_redaction(mint, "mint:pubkey");
	let program_redaction = create_insta_redaction(ID_CONST, "program:pubkey");
	let Account { data, .. } = ctx.banks_client.get_account(mint).await?.unwrap();
	let mint_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;

	log::info!("mint account: {mint_account:#?}");

	insta::assert_yaml_snapshot!(mint_account, {
		".info.mintAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.freezeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[0].state.metadataAddress" =>	insta::dynamic_redaction(mint_redaction.clone()),
	  ".info.extensions[1].state.closeAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
		".info.extensions[2].state.authority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[2].state.programId" => insta::dynamic_redaction(program_redaction),
		".info.extensions[3].state.delegate" =>	insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[4].state.updateAuthority" => insta::dynamic_redaction(authority_redaction.clone()),
	  ".info.extensions[4].state.mint" =>	insta::dynamic_redaction(mint_redaction.clone())
	});

	let Account { data, .. } = ctx
		.banks_client
		.get_account(treasury_token_account)
		.await?
		.unwrap();
	let token_account = parse_token_v2(
		&data,
		Some(&SplTokenAdditionalData::with_decimals(TOKEN_DECIMALS)),
	)?;
	log::info!("token account: {token_account:#?}");
	insta::assert_yaml_snapshot!(token_account, {
		".info.mint" => insta::dynamic_redaction(mint_redaction),
		".info.owner" => insta::dynamic_redaction(authority_redaction),
	});

	Ok(())
}

#[test(tokio::test)]
async fn initialize_bits_meta() -> anyhow::Result<()> {
	let (config, _) = get_pda_config();
	let authority_keypair = create_authority_keypair();
	let authority = authority_keypair.pubkey();
	let (mint, _) = get_pda_mint();
	let (treasury, _) = get_pda_treasury();
	let token_program = token_2022::ID;
	let treasury_token_account =
		get_associated_token_address_with_program_id(&treasury, &mint, &token_program);
	let associated_token_program = spl_associated_token_account::ID;
	let system_program = system_program::ID;

	let mut ctx = create_program_context_with_factory(|p| {
		p.add_account(config, create_config_state(None).into());
	})
	.await?;

	let rpc = create_rpc();
	let bitflip_program_client = get_admin_program(&rpc);

	let initialize_token_request = bitflip_program_client.initialize_bits_meta();

	Ok(())
}
