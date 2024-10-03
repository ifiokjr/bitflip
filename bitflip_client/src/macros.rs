#[allow(unused_macro_rules)]
macro_rules! bitflip_request_builder {
	($name_prefix:ident, $accounts:ident,"optional:args") => {
		::wasm_client_anchor::create_request_builder!(
			bitflip_program,
			BitflipProgram,
			$name_prefix,
			$accounts,
			"optional:args"
		);
	};
	($name_prefix:ident, $accounts:ident,"required:args") => {
		::wasm_client_anchor::create_request_builder!(
			bitflip_program,
			BitflipProgram,
			$name_prefix,
			$accounts,
			"required:args"
		);
	};
	($name_prefix:ident, $accounts:ident) => {
		::wasm_client_anchor::create_request_builder!(
			bitflip_program,
			BitflipProgram,
			$name_prefix,
			$accounts,
			"required:args"
		);
	};
	($name_prefix:ident,"optional:args") => {
		::wasm_client_anchor::create_request_builder!(
			bitflip_program,
			BitflipProgram,
			$name_prefix,
			$name_prefix,
			"optional:args"
		);
	};
	($name_prefix:ident) => {
		::wasm_client_anchor::create_request_builder!(
			bitflip_program,
			BitflipProgram,
			$name_prefix,
			$name_prefix,
			"required:args"
		);
	};
}

pub(crate) use bitflip_request_builder;
