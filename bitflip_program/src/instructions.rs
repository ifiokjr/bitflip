use steel::*;

use crate::ConfigInitialize;
use crate::ConfigUpdateAuthority;
use crate::FlipBit;
use crate::GameInitialize;
use crate::GameRefreshSigner;
use crate::get_pda_config;
use crate::get_pda_game;
use crate::get_pda_mint_bit;
use crate::get_pda_mint_gibibit;
use crate::get_pda_mint_kibibit;
use crate::get_pda_mint_mebibit;
use crate::get_pda_section;
use crate::get_pda_treasury;
use crate::get_player_token_account;
use crate::get_section_token_account;
use crate::get_treasury_token_account;

/// Create an instruction to initialize the mint, treasury and [`ConfigState`].
///
/// ### Arguments
///
/// * `admin` - The admin account which protects this instruction from
///   outsiders: must be a signer.
/// * `authority` - The authority account: must be a signer.
///
/// When using this instruction in a transaction you will need to make sure both
/// the admin and authority are signers for the transaction.
pub fn config_initialize(admin: &Pubkey, authority: &Pubkey) -> Instruction {
	let config = get_pda_config().0;
	let treasury = get_pda_treasury().0;
	let mint_bit = get_pda_mint_bit().0;
	let treasury_bit_token_account = get_treasury_token_account(&treasury, &mint_bit);
	let mint_kibibit = get_pda_mint_kibibit().0;
	let treasury_kibibit_token_account = get_treasury_token_account(&treasury, &mint_kibibit);
	let mint_mebibit = get_pda_mint_mebibit().0;
	let treasury_mebibit_token_account = get_treasury_token_account(&treasury, &mint_mebibit);
	let mint_gibibit = get_pda_mint_gibibit().0;
	let treasury_gibibit_token_account = get_treasury_token_account(&treasury, &mint_gibibit);
	let associated_token_program = spl_associated_token_account::ID;
	let token_program = spl_token_2022::ID;
	let system_program = system_program::ID;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new_readonly(*admin, true),
			AccountMeta::new(*authority, true),
			AccountMeta::new(config, false),
			AccountMeta::new(treasury, false),
			AccountMeta::new(mint_bit, false),
			AccountMeta::new(treasury_bit_token_account, false),
			AccountMeta::new(mint_kibibit, false),
			AccountMeta::new(treasury_kibibit_token_account, false),
			AccountMeta::new(mint_mebibit, false),
			AccountMeta::new(treasury_mebibit_token_account, false),
			AccountMeta::new(mint_gibibit, false),
			AccountMeta::new(treasury_gibibit_token_account, false),
			AccountMeta::new_readonly(associated_token_program, false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(system_program, false),
		],
		data: ConfigInitialize {}.to_bytes(),
	}
}

pub fn game_initialize(
	authority: &Pubkey,
	access_signer: &Pubkey,
	refresh_signer: &Pubkey,
	game_index: u8,
) -> Instruction {
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(*authority, true),
			AccountMeta::new_readonly(*access_signer, true),
			AccountMeta::new(*refresh_signer, true),
			AccountMeta::new(config, false),
			AccountMeta::new(game, false),
			AccountMeta::new_readonly(system_program::ID, false),
		],
		data: GameInitialize {}.to_bytes(),
	}
}

/// Create an instruction to update the authority of the config.
///
/// ### Arguments
///
/// * `authority` - The current authority: must be a signer.
/// * `new_authority` - The new authority: must be a signer.
pub fn config_update_authority(authority: &Pubkey, new_authority: &Pubkey) -> Instruction {
	let config = get_pda_config().0;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(config, false),
			AccountMeta::new(*authority, true),
			AccountMeta::new(*new_authority, true),
		],
		data: ConfigUpdateAuthority {}.to_bytes(),
	}
}

/// Create an instruction to refresh the signer of the game.
///
/// ### Arguments
///
/// * `access_signer` - The access signer: must be a signer.
/// * `refresh_signer` - The refresh signer: must be a signer.
/// * `game_index` - The index of the game.
pub fn game_refresh_signer(
	access_signer: &Pubkey,
	refresh_signer: &Pubkey,
	game_index: u8,
) -> Instruction {
	let accounts = vec![
		AccountMeta::new_readonly(*access_signer, true), // Access signer must sign
		AccountMeta::new(*refresh_signer, true),         // Refresh signer must sign
		AccountMeta::new(get_pda_game(game_index).0, false), // Game account to be modified
	];

	Instruction {
		program_id: crate::ID,
		accounts,
		data: GameRefreshSigner {}.to_bytes(),
	}
}

/// Create an instruction to set a bit on the player's bit token account.
///
/// ### Arguments
///
/// * `player` - The player account: must be a signer.
pub fn flip_bit(
	player: &Pubkey,
	game_index: u8,
	section_index: u8,
	array_index: u8,
	offset: u8,
	value: u8,
) -> Instruction {
	let mint_bit = get_pda_mint_bit().0;
	let player_bit_token_account = get_player_token_account(player, &mint_bit);
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let section = get_pda_section(game_index, section_index).0;
	let section_bit_token_account = get_section_token_account(&section, &mint_bit);
	let associated_token_program = spl_associated_token_account::ID;
	let token_program = spl_token_2022::ID;
	let system_program = system_program::ID;
	let data = FlipBit::builder()
		.section_index(section_index)
		.array_index(array_index)
		.offset(offset)
		.value(value)
		.build()
		.to_bytes();

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(*player, true),
			AccountMeta::new(player_bit_token_account, false),
			AccountMeta::new_readonly(config, false),
			AccountMeta::new_readonly(game, false),
			AccountMeta::new_readonly(mint_bit, false),
			AccountMeta::new(section, false),
			AccountMeta::new(section_bit_token_account, false),
			AccountMeta::new_readonly(associated_token_program, false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(system_program, false),
		],
		data,
	}
}
