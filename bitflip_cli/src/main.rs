use std::collections::HashMap;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;
use std::time::SystemTime;

use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bitflip_program::config_initialize;
use bitflip_program::config_update_authority;
use bitflip_program::get_pda_config;
use bitflip_program::get_pda_game;
use bitflip_program::get_pda_mint;
use bitflip_program::get_pda_treasury;
use bitflip_program::GameStatus;
use bitflip_program::ID;
use bitflip_program_test::create_config_accounts;
use bitflip_program_test::create_game_state;
use bitflip_program_test::create_section_state;
use bitflip_program_test::create_token_accounts;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use colored::*;
use solana_sdk::account::AccountSharedData;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::v0::Message;
use solana_sdk::message::VersionedMessage;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Keypair;
use solana_sdk::signer::EncodableKey;
use solana_sdk::signer::Signer;
use solana_sdk::transaction::VersionedTransaction;
use test_utils_keypairs::get_admin_keypair;
use test_utils_keypairs::get_authority_keypair;
use test_utils_keypairs::get_treasury_keypair;
use test_utils_keypairs::get_wallet_keypair;
use test_utils_solana::TestValidatorPorts;
use wasm_client_solana::SolanaRpcClient;

mod emoji;
mod spinner;
use spinner::Spinner;
use wasm_client_solana::VersionedTransactionExtension;

#[derive(Clone, Debug, ValueEnum, PartialEq, Eq)]
enum Cluster {
	/// Mainnet Beta.
	MainnetBeta,
	/// Testnet.
	Testnet,
	/// Devnet.
	Devnet,
	/// Localhost.
	Localhost,
}

impl Cluster {
	fn url(&self) -> &'static str {
		match self {
			Cluster::MainnetBeta => "https://api.mainnet-beta.solana.com",
			Cluster::Testnet => "https://api.testnet.solana.com",
			Cluster::Devnet => "https://api.devnet.solana.com",
			Cluster::Localhost => "http://localhost:8899",
		}
	}
}

#[derive(Default, Clone, Debug, ValueEnum, PartialEq, Eq)]
enum CommandAction {
	/// Generate the message that needs to be signed and write to `stdout` or
	/// write to the provide `--output-file`.
	#[default]
	#[value(name = "message")]
	Message,
	/// Generate the transaction that has been signed with the current blockhash
	/// using the provided `signers` and write to `stdout` or write to the
	/// provide `--output-file`.
	#[value(name = "transaction")]
	Transaction,
	/// This will simulate the transaction and write to `stdout` or write to the
	/// provide `--output-file`.
	#[value(name = "simulate")]
	Simulate,
	/// This will send the transaction to the network and write to `stdout` or
	/// write to the provide `--output-file`.
	#[value(name = "send")]
	Send,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	/// URL for Solana's JSON RPC or moniker (or their first letter):
	/// [mainnet-beta, testnet, devnet, localhost]
	#[arg(short = 'u', long, value_name = "URL_OR_MONIKER", default_value_t = Cluster::Localhost.url().to_string())]
	url: String,

	/// Optional signers that will be used to sign the transaction
	#[arg(short = 's', long = "signer", value_name = "KEYPAIR")]
	signers: Vec<PathBuf>,

	/// The action to perform with the command provided.
	#[arg(short = 'a', long = "action", value_enum, default_value_t = CommandAction::Message)]
	action: CommandAction,

	/// The output file to write to
	#[arg(short = 'o', long = "output", value_name = "FILE")]
	output_file: Option<PathBuf>,

	/// Whether to output as JSON. The default ouput is bincode.
	#[arg(short = 'j', long = "json")]
	json: bool,

	#[command(subcommand)]
	command: Commands,
}

impl Cli {
	fn get_url(&self) -> Result<String> {
		// Check if it's a URL
		if self.url.starts_with("http") {
			return Ok(self.url.clone());
		}

		// Try to match moniker
		let lowercase = self.url.to_lowercase();
		let cluster = match lowercase.as_str() {
			"m" | "mainnet" | "mainnet-beta" => Cluster::MainnetBeta,
			"t" | "testnet" => Cluster::Testnet,
			"d" | "devnet" => Cluster::Devnet,
			"l" | "localhost" => Cluster::Localhost,
			url => {
				let url = url::Url::parse(url).context(format!("invalid url: {url}"))?;
				return Ok(url.to_string());
			}
		};

		Ok(cluster.url().to_string())
	}
}

#[derive(Subcommand)]
enum Commands {
	/// Launch a test validator with the program deployed
	Validator(ValidatorArgs),
	/// Create config related transactions
	#[command(subcommand)]
	Config(ConfigCommands),

	/// Game related commands
	#[command(subcommand)]
	Game(GameCommands),
}

#[derive(Parser)]
struct ValidatorArgs {
	/// Whether this is in test mode.
	#[arg(short = 't', long)]
	test: bool,

	/// The game index which only applies if test is true
	#[arg(long)]
	game: Option<u8>,

	/// The section index which only applies if test is true
	#[arg(long)]
	section: Option<u8>,

	/// Optional pubkeys to fund with SOL
	#[arg(long = "pubkey", value_name = "PUBKEY")]
	pubkeys: Vec<Pubkey>,
}

impl ValidatorArgs {
	fn game_index(&self) -> u8 {
		self.game.unwrap_or_default()
	}

	fn section_index(&self) -> u8 {
		self.section.unwrap_or_default()
	}
}

#[derive(Subcommand)]
enum ConfigCommands {
	/// Initialize the program config
	Initialize(ConfigInitializeArgs),

	/// Update the program authority
	UpdateAuthority {
		/// The new authority's public key
		#[arg(long)]
		new_authority: Pubkey,
		/// The authority's public key
		#[arg(long)]
		authority: Pubkey,
	},
}

#[derive(Parser)]
struct ConfigInitializeArgs {
	/// The admin's keypair path
	#[arg(long)]
	admin: Pubkey,
	/// The authority's public key
	#[arg(long)]
	authority: Pubkey,
}

#[derive(Subcommand)]
enum GameCommands {
	/// Initialize a new game
	Initialize {
		/// The game index
		#[arg(long)]
		index: u8,

		/// Optional start time in seconds since UNIX epoch
		#[arg(long)]
		start_time: Option<i64>,

		/// The authority's public key
		#[arg(long)]
		authority: Pubkey,
	},
}

fn create_validator_accounts(args: &ValidatorArgs) -> Result<HashMap<Pubkey, AccountSharedData>> {
	if !args.test {
		return Ok(HashMap::new());
	}

	let mut accounts = create_config_accounts();
	let game_index = args.game_index();
	let section_index = args.section_index();
	accounts.extend(create_token_accounts(false)?);
	let now = SystemTime::now()
		.duration_since(SystemTime::UNIX_EPOCH)
		.unwrap()
		.as_secs() as i64;
	let game = get_pda_game(game_index).0;
	let create_game_state = create_game_state(0, 0, now - 3600, GameStatus::Running);
	accounts.insert(game, create_game_state.game_state_account);

	let section_accounts = create_section_state(
		Pubkey::new_unique(),
		game_index,
		section_index.saturating_add(1),
		false,
	)?;
	accounts.extend(section_accounts);

	Ok(accounts)
}

async fn process_transaction(
	cli: &Cli,
	payer: &Pubkey,
	instruction: Instruction,
	rpc: &SolanaRpcClient,
	writer: &mut impl Write,
) -> Result<()> {
	let recent_blockhash = rpc.get_latest_blockhash().await?;
	let message = Message::try_compile(payer, &[instruction], &[], recent_blockhash)?;
	let versioned_message = VersionedMessage::V0(message);

	if cli.action == CommandAction::Message {
		let output = versioned_message.serialize();
		writer.write_all(&output)?;
		return Ok(());
	}

	let keypairs = cli
		.signers
		.iter()
		.map(|path| {
			Keypair::read_from_file(path).map_err(|e| anyhow!("{e}: failed to read keypair"))
		})
		.collect::<Result<Vec<_>>>()?;

	let transaction = VersionedTransaction::new(versioned_message, &keypairs);

	if cli.action == CommandAction::Transaction {
		let output = bincode::serialize(&transaction)?;
		writer.write_all(&output)?;
		return Ok(());
	}

	if cli.action == CommandAction::Simulate {
		let simulation = rpc.simulate_transaction(&transaction).await?;
		eprintln!("simulation: {simulation:#?}");
		return Ok(());
	}

	let signature = rpc.send_and_confirm_transaction(&transaction).await?;
	eprintln!("signature: {signature}");
	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
	// env_logger::init();
	let cli = Cli::parse();

	// Get the URL and authority early to catch any errors
	let url = cli.get_url()?;
	let rpc = SolanaRpcClient::new(&url);
	let mut writer = if let Some(ref output_file) = cli.output_file {
		Box::new(std::fs::File::create(output_file)?) as Box<dyn Write>
	} else {
		Box::new(std::io::stdout()) as Box<dyn Write>
	};

	eprintln!("{} {}", "using rpc url:".bright_blue(), url.yellow());

	match cli.command {
		Commands::Validator(ref args) => {
			let spinner = Spinner::new(
				format!("{} starting validator...", emoji::VALIDATOR),
				"blue",
			);
			let accounts = create_validator_accounts(args)?;
			let devenv_root = std::env::var("DEVENV_ROOT").unwrap();
			let launchpad_program = test_utils_solana::TestProgramInfo::builder()
				.program_id(ID)
				.program_path(format!("{devenv_root}/target/deploy/bitflip_program.so"))
				.build();
			let mut pubkeys = vec![
				get_admin_keypair().pubkey(),
				get_authority_keypair().pubkey(),
				get_treasury_keypair().pubkey(),
				get_wallet_keypair().pubkey(),
			];
			pubkeys.extend(args.pubkeys.clone());

			let props = test_utils_solana::TestValidatorRunnerProps::builder()
				.programs(vec![launchpad_program])
				.ports(TestValidatorPorts::default())
				.pubkeys(pubkeys)
				.commitment(CommitmentLevel::Finalized)
				.accounts(accounts)
				.build();

			let runner = test_utils_solana::TestValidatorRunner::run(props).await;
			spinner.done("validator started successfully");

			eprintln!(
				"\n{}",
				format!("{} test validator launched!", emoji::START)
					.green()
					.bold()
			);
			eprintln!(
				"{} {}",
				format!("{} rpc url:", emoji::INFO).bright_blue(),
				runner.rpc_url().yellow()
			);
			eprintln!(
				"{} {}",
				format!("{} ws url:", emoji::INFO).bright_blue(),
				runner.pubsub_url().yellow()
			);
			eprintln!(
				"{} {}",
				format!("{} program id:", emoji::INFO).bright_blue(),
				ID.to_string().yellow()
			);

			if args.test {
				eprintln!(
					"{} {}",
					format!("{} config account:", emoji::INFO).bright_blue(),
					get_pda_config().0.to_string().yellow()
				);
				eprintln!(
					"{} {}",
					format!("{} treasury account:", emoji::INFO).bright_blue(),
					get_pda_treasury().0.to_string().yellow()
				);
				eprintln!(
					"{} {}",
					format!("{} mint account:", emoji::INFO).bright_blue(),
					get_pda_mint(bitflip_program::TokenMember::Bit)
						.0
						.to_string()
						.yellow()
				);
				eprintln!(
					"{} {}",
					format!("{} game account:", emoji::INFO).bright_blue(),
					get_pda_game(args.game_index()).0.to_string().yellow()
				);
				eprintln!(
					"{} {}",
					format!("{} admin account:", emoji::INFO).bright_blue(),
					get_admin_keypair().pubkey().to_string().yellow()
				);
				eprintln!(
					"{} {}",
					format!("{} authority account:", emoji::INFO).bright_blue(),
					get_authority_keypair().pubkey().to_string().yellow()
				);
			}

			eprintln!(
				"\n{}",
				format!("{} press ctrl+c to stop the validator...", emoji::WARNING).bright_black()
			);
			tokio::signal::ctrl_c().await?;

			let shutdown_spinner =
				Spinner::new(format!("{} shutting down validator...", emoji::STOP), "red");

			// Simulate shutdown time (you might want to actually track the real shutdown)
			tokio::time::sleep(Duration::from_secs(2)).await;

			shutdown_spinner.info("test validator stopped");
		}
		Commands::Config(ref config_cmd) => {
			match config_cmd {
				ConfigCommands::Initialize(args) => {
					eprintln!(
						"{} {}",
						format!("{} initializing config with authority:", emoji::CONFIG)
							.bright_blue(),
						args.authority.to_string().yellow()
					);

					let instruction = config_initialize(&args.admin, &args.authority);
					process_transaction(&cli, &args.authority, instruction, &rpc, &mut writer)
						.await?;
				}
				ConfigCommands::UpdateAuthority {
					new_authority,
					authority,
				} => {
					let instruction = config_update_authority(authority, new_authority);
					process_transaction(&cli, authority, instruction, &rpc, &mut writer).await?;
				}
			}
		}
		Commands::Game(game_cmd) => {
			match game_cmd {
				GameCommands::Initialize {
					index,
					start_time,
					authority,
				} => {
					eprintln!(
						"{} {} {} {}",
						format!("{} initializing game with index:", emoji::GAME).bright_blue(),
						index.to_string().yellow(),
						"and authority:".bright_blue(),
						authority.to_string().yellow()
					);
					if let Some(start_time) = start_time {
						eprintln!(
							"{} {}",
							format!("{} start time set to:", emoji::INFO).bright_blue(),
							start_time.to_string().yellow()
						);
					}
					// TODO: Implement game initialization using bitflip_program
				}
			}
		}
	}

	Ok(())
}
