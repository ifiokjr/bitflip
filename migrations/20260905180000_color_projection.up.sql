-- Immutable, finalized color events received from the Bitflip program.
CREATE TABLE color_flip_events (
	signature TEXT NOT NULL,
	instruction_index INTEGER NOT NULL CHECK (instruction_index >= 0),
	slot INTEGER NOT NULL CHECK (slot >= 0),
	transaction_index INTEGER NOT NULL CHECK (transaction_index >= 0),
	player_pubkey TEXT NOT NULL,
	game_index INTEGER NOT NULL CHECK (game_index BETWEEN 0 AND 255),
	section_index INTEGER NOT NULL CHECK (section_index BETWEEN 0 AND 255),
	array_index INTEGER NOT NULL CHECK (array_index BETWEEN 0 AND 255),
	bit_offset INTEGER NOT NULL CHECK (bit_offset BETWEEN 0 AND 15),
	value INTEGER NOT NULL CHECK (value IN (0, 1)),
	color INTEGER NOT NULL CHECK (color BETWEEN 0 AND 7),
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY (signature, instruction_index)
);

CREATE INDEX color_flip_events_section_order ON color_flip_events (
	game_index,
	section_index,
	slot,
	transaction_index,
	instruction_index
);

-- Latest finalized color state for every pixel. Rows with value = 0 are kept
-- as ordering tombstones so late, older events cannot repaint cleared pixels.
CREATE TABLE section_color_pixels (
	game_index INTEGER NOT NULL CHECK (game_index BETWEEN 0 AND 255),
	section_index INTEGER NOT NULL CHECK (section_index BETWEEN 0 AND 255),
	array_index INTEGER NOT NULL CHECK (array_index BETWEEN 0 AND 255),
	bit_offset INTEGER NOT NULL CHECK (bit_offset BETWEEN 0 AND 15),
	value INTEGER NOT NULL CHECK (value IN (0, 1)),
	color INTEGER NOT NULL CHECK (color BETWEEN 0 AND 7),
	player_pubkey TEXT NOT NULL,
	slot INTEGER NOT NULL CHECK (slot >= 0),
	transaction_index INTEGER NOT NULL CHECK (transaction_index >= 0),
	instruction_index INTEGER NOT NULL CHECK (instruction_index >= 0),
	signature TEXT NOT NULL,
	updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY (
		game_index,
		section_index,
		array_index,
		bit_offset
	)
);

CREATE INDEX section_color_pixels_visible ON section_color_pixels (game_index, section_index, value);
