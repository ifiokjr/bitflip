-- Enable WAL mode for better concurrency and performance
PRAGMA journal_mode = wal;

-- Ensure foreign key constraints are enforced
PRAGMA foreign_keys = ON;

-- Increase cache size (in KB) for better performance
PRAGMA cache_size = -2000000;

-- Uses 2GB of RAM for cache
-- Enable memory-mapped I/O for faster reads
PRAGMA mmap_size = 30000000000;

-- 30GB
-- Set temp store to memory for faster temp operations
PRAGMA temp_store = memory;

-- Enable auto_vacuum to keep the database file size in check
PRAGMA auto_vacuum = incremental;

-- Set page size (must be power of 2, default is 4096)
PRAGMA page_size = 4096;

-- Create a game table
CREATE TABLE IF NOT EXISTS games (
	id TEXT PRIMARY KEY,
	game_index INTEGER NOT NULL,
	start_time INTEGER NOT NULL,
	duration INTEGER NOT NULL,
	min_lamports INTEGER NOT NULL,
	base_lamports INTEGER NOT NULL,
	max_lamports INTEGER NOT NULL,
	status INTEGER NOT NULL,
	section_index INTEGER NOT NULL,
	temp_signer TEXT NOT NULL,
	game_signer TEXT NOT NULL,
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	UNIQUE (game_index),
	UNIQUE (temp_signer),
	UNIQUE (game_signer)
);

CREATE TRIGGER IF NOT EXISTS games_updated_at AFTER
UPDATE ON games BEGIN
UPDATE games
SET
	updated_at = CURRENT_TIMESTAMP
WHERE
	game_index = new.game_index;

END;

-- Create sections table
CREATE TABLE IF NOT EXISTS sections (
	id TEXT PRIMARY KEY,
	game_index INTEGER NOT NULL,
	section_index INTEGER NOT NULL,
	reward_tokens INTEGER NOT NULL,
	lamports INTEGER NOT NULL,
	data BLOB NOT NULL,
	owner TEXT NOT NULL, -- the pubkey of the player who owns the section
	flips INTEGER NOT NULL DEFAULT 0,
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	UNIQUE (game_index, section_index),
	FOREIGN KEY (game_index) REFERENCES games (game_index) ON DELETE CASCADE
);

CREATE TRIGGER IF NOT EXISTS sections_updated_at AFTER
UPDATE ON sections BEGIN
UPDATE sections
SET
	updated_at = CURRENT_TIMESTAMP
WHERE
	id = new.id;

END;

-- Create section player events table
CREATE TABLE IF NOT EXISTS section_events (
	id TEXT PRIMARY KEY,
	section_id TEXT NOT NULL,
	player_pubkey TEXT NOT NULL,
	event_type INTEGER NOT NULL,
	data BLOB NOT NULL,
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	FOREIGN KEY (section_id) REFERENCES sections (id) ON DELETE CASCADE,
	FOREIGN KEY (player_pubkey) REFERENCES players (pubkey) ON DELETE CASCADE
);

-- Create players table for participation in the game
CREATE TABLE IF NOT EXISTS players (
	pubkey TEXT PRIMARY KEY,
	signer TEXT NOT NULL,
	status INTEGER NOT NULL DEFAULT 0,
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER IF NOT EXISTS players_updated_at AFTER
UPDATE ON players BEGIN
UPDATE players
SET
	updated_at = CURRENT_TIMESTAMP
WHERE
	id = new.id;

END;

-- Create encrypted keypairs table
CREATE TABLE IF NOT EXISTS encrypted_keypairs (
	pubkey TEXT PRIMARY KEY,
	encrypted_keypair BLOB NOT NULL,
	key_index INTEGER NOT NULL,
	nonce BLOB NOT NULL,
	salt TEXT NOT NULL,
	status INTEGER NOT NULL DEFAULT 0,
	created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
	updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE TRIGGER IF NOT EXISTS encrypted_keypairs_updated_at AFTER
UPDATE ON encrypted_keypairs BEGIN
UPDATE encrypted_keypairs
SET
	updated_at = CURRENT_TIMESTAMP
WHERE
	pubkey = new.pubkey;

END;
