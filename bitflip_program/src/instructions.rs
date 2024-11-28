use steel::*;

use crate::get_pda_config;
use crate::get_pda_game;
use crate::get_pda_mint;
use crate::get_pda_section;
use crate::get_pda_treasury;
use crate::get_token_account;
use crate::ConfigInitialize;
use crate::ConfigUpdateAuthority;
use crate::FlipBit;
use crate::GameInitialize;
use crate::GameUpdateTempSigner;
use crate::SectionUnlock;
use crate::TokenGroupInitialize;
use crate::TokenInitialize;
use crate::TokenMember;

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
	let system_program = system_program::ID;
	let treasury = get_pda_treasury().0;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new_readonly(*admin, true),
			AccountMeta::new(*authority, true),
			AccountMeta::new(config, false),
			AccountMeta::new(treasury, false),
			AccountMeta::new_readonly(system_program, false),
		],
		data: ConfigInitialize {}.to_bytes(),
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

/// Create an instruction to initialize the token member.
///
/// ### Arguments
///
/// * `authority` - The authority account: must be a signer.
/// * `member` - The member to initialize.
pub fn token_initialize(authority: &Pubkey, member: TokenMember) -> Instruction {
	let config = get_pda_config().0;
	let treasury = get_pda_treasury().0;
	let mint = get_pda_mint(member).0;
	let treasury_token_account = get_token_account(&treasury, &mint);
	let associated_token_program = spl_associated_token_account::ID;
	let token_program = spl_token_2022::ID;
	let system_program = system_program::ID;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(*authority, true),
			AccountMeta::new_readonly(config, false),
			AccountMeta::new_readonly(treasury, false),
			AccountMeta::new(mint, false),
			AccountMeta::new(treasury_token_account, false),
			AccountMeta::new_readonly(associated_token_program, false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(system_program, false),
		],
		data: TokenInitialize::new(member).to_bytes(),
	}
}

/// Create an instruction to initialize the token group and its members.
///
/// ### Arguments
///
/// * `authority` - The authority account: must be a signer.
pub fn token_group_initialize(authority: &Pubkey) -> Instruction {
	let config = get_pda_config().0;
	let treasury = get_pda_treasury().0;
	let mint_bit = get_pda_mint(TokenMember::Bit).0;
	let mint_kibibit = get_pda_mint(TokenMember::Kibibit).0;
	let mint_mebibit = get_pda_mint(TokenMember::Mebibit).0;
	let mint_gibibit = get_pda_mint(TokenMember::Gibibit).0;
	let associated_token_program = spl_associated_token_account::ID;
	let token_program = spl_token_2022::ID;
	let system_program = system_program::ID;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(*authority, true),
			AccountMeta::new_readonly(config, false),
			AccountMeta::new(treasury, false),
			AccountMeta::new(mint_bit, false),
			AccountMeta::new(mint_kibibit, false),
			AccountMeta::new(mint_mebibit, false),
			AccountMeta::new(mint_gibibit, false),
			AccountMeta::new_readonly(associated_token_program, false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(system_program, false),
		],
		data: TokenGroupInitialize {}.to_bytes(),
	}
}

/// Create an instruction to initialize the game.
///
/// ### Arguments
///
/// * `authority` - The authority account: must be a signer.
/// * `temp_signer` - The access signer account: must be a signer.
/// * `funded_signer` - The refresh signer account: must be a signer.
pub fn game_initialize(
	authority: &Pubkey,
	temp_signer: &Pubkey,
	funded_signer: &Pubkey,
	game_index: u8,
) -> Instruction {
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;

	Instruction {
		program_id: crate::ID,
		accounts: vec![
			AccountMeta::new(*authority, true),
			AccountMeta::new_readonly(*temp_signer, true),
			AccountMeta::new(*funded_signer, true),
			AccountMeta::new(config, false),
			AccountMeta::new(game, false),
			AccountMeta::new_readonly(system_program::ID, false),
		],
		data: GameInitialize {}.to_bytes(),
	}
}

/// Create an instruction to refresh the signer of the game.
///
/// ### Arguments
///
/// * `funded_signer` - The permanent game signer: must be a signer.
/// * `temp_signer` - The new temporary game signer: must be a signer.
/// * `game_index` - The index of the game to update.
pub fn game_update_temp_signer(
	funded_signer: &Pubkey,
	temp_signer: &Pubkey,
	game_index: u8,
) -> Instruction {
	let game = get_pda_game(game_index).0;
	let accounts = vec![
		AccountMeta::new(*funded_signer, true), // funded signer must sign
		AccountMeta::new_readonly(*temp_signer, true), // temp signer must sign
		AccountMeta::new(game, false),
	];

	Instruction {
		program_id: crate::ID,
		accounts,
		data: GameUpdateTempSigner {}.to_bytes(),
	}
}

/// Create an instruction to reset the signers of the game.
///
/// ### Arguments
///
/// * `authority` - The authority account: must be a signer.
/// * `funded_signer` - The new permanent game signer: must be a signer.
/// * `temp_signer` - The new temporary game signer: must be a signer.
/// * `previous_funded_signer` - The previous permanent game signer: if provided
///   must be a signer.
/// * `game_index` - The index of the game.
pub fn game_reset_signers(
	authority: &Pubkey,
	funded_signer: &Pubkey,
	temp_signer: &Pubkey,
	previous_funded_signer: Option<Pubkey>,
	game_index: u8,
) -> Instruction {
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let accounts = vec![
		AccountMeta::new(*authority, true),
		AccountMeta::new(*funded_signer, true),
		AccountMeta::new_readonly(*temp_signer, true),
		if let Some(previous_funded_signer) = previous_funded_signer {
			AccountMeta::new(previous_funded_signer, true)
		} else {
			AccountMeta::new_readonly(Pubkey::default(), false)
		},
		AccountMeta::new(config, false),
		AccountMeta::new(game, false),
	];

	Instruction {
		program_id: crate::ID,
		accounts,
		data: GameUpdateTempSigner {}.to_bytes(),
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
	let mint = get_pda_mint(TokenMember::Bit).0;
	let player_token_account = get_token_account(player, &mint);
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let section = get_pda_section(game_index, section_index).0;
	let section_token_account = get_token_account(&section, &mint);
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
			AccountMeta::new(player_token_account, false),
			AccountMeta::new_readonly(config, false),
			AccountMeta::new_readonly(game, false),
			AccountMeta::new_readonly(mint, false),
			AccountMeta::new(section, false),
			AccountMeta::new(section_token_account, false),
			AccountMeta::new_readonly(associated_token_program, false),
			AccountMeta::new_readonly(token_program, false),
			AccountMeta::new_readonly(system_program, false),
		],
		data,
	}
}

/// Create an instruction to unlock a section.
///
/// This instruction will be paired with an advance nonce instruction where the
/// nonce
///
/// ### Arguments
///
/// * `owner` - The owner account: must be a signer.
/// * `temp_signer` - The access signer: must be a signer.
/// * `game_index` - The index of the game.
/// * `section_index` - The index of the section.
/// * `lamports` - The amount of lamports that is being bid on the section. The
///   highest bid will win.
pub fn section_unlock(
	owner: &Pubkey,
	temp_signer: &Pubkey,
	game_index: u8,
	section_index: u8,
	lamports: u64,
) -> Instruction {
	let config = get_pda_config().0;
	let game = get_pda_game(game_index).0;
	let section = get_pda_section(game_index, section_index).0;
	let treasury = get_pda_treasury().0;
	let system_program = system_program::ID;
	let accounts = vec![
		AccountMeta::new(*owner, true),
		AccountMeta::new_readonly(*temp_signer, true),
		AccountMeta::new_readonly(config, false),
		AccountMeta::new(game, false),
		AccountMeta::new(section, false),
		AccountMeta::new(treasury, false),
		AccountMeta::new_readonly(system_program, false),
	];
	let data = SectionUnlock {
		lamports: lamports.into(),
	}
	.to_bytes();

	Instruction {
		program_id: crate::ID,
		accounts,
		data,
	}
}
