use bitflip_program::ID_CONST;
use bitflip_program::SEED_BITS;
use bitflip_program::SEED_BITS_SECTION;
use bitflip_program::SEED_CONFIG;
use bitflip_program::SEED_MINT;
use bitflip_program::SEED_PREFIX;
use bitflip_program::SEED_TREASURY;
use solana_sdk::pubkey::Pubkey;

pub fn get_pda_config() -> (Pubkey, u8) {
	get_pda_config_with_program(&ID_CONST)
}

pub fn get_pda_config_with_program(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_CONFIG], program_id)
}

pub fn get_pda_treasury() -> (Pubkey, u8) {
	get_pda_treasury_with_program(&ID_CONST)
}

pub fn get_pda_treasury_with_program(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_TREASURY], program_id)
}

pub fn get_pda_mint() -> (Pubkey, u8) {
	get_pda_mint_with_program(&ID_CONST)
}

pub fn get_pda_mint_with_program(program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(&[SEED_PREFIX, SEED_MINT], program_id)
}

pub fn get_pda_bits_meta(game_index: u8) -> (Pubkey, u8) {
	get_pda_bits_meta_with_program(game_index, &ID_CONST)
}

pub fn get_pda_bits_meta_with_program(game_index: u8, program_id: &Pubkey) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[SEED_PREFIX, SEED_BITS, &game_index.to_le_bytes()],
		program_id,
	)
}

pub fn get_pda_bits_data_section(game_index: u8, section: u8) -> (Pubkey, u8) {
	get_pda_bits_data_section_with_program(game_index, section, &ID_CONST)
}

pub fn get_pda_bits_data_section_with_program(
	game_index: u8,
	section: u8,
	program_id: &Pubkey,
) -> (Pubkey, u8) {
	Pubkey::find_program_address(
		&[
			SEED_PREFIX,
			SEED_BITS,
			&game_index.to_le_bytes(),
			SEED_BITS_SECTION,
			&section.to_le_bytes(),
		],
		program_id,
	)
}
