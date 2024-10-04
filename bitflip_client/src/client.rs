use wasm_client_anchor::create_program_client;
use wasm_client_anchor::create_program_client_macro;

create_program_client!(bitflip_program::ID_CONST, BitflipProgramClient);
create_program_client_macro!(bitflip_program, BitflipProgramClient);

bitflip_program_client_request_builder!(InitializeConfig);
bitflip_program_client_request_builder!(InitializeToken);
bitflip_program_client_request_builder!(InitializeBitsMeta, "optional:args");
bitflip_program_client_request_builder!(InitializeBitsDataSection);
bitflip_program_client_request_builder!(SetBits);
