use std::collections::HashMap;
use std::time::Duration;

use anyhow::Context;
use anyhow::Result;
use bitflip_program::ID;
use clap::Parser;
use clap::Subcommand;
use clap::ValueEnum;
use colored::*;
use solana_sdk::commitment_config::CommitmentLevel;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signer::Signer;
use test_utils_keypairs::get_admin_keypair;
use test_utils_keypairs::get_authority_keypair;
use test_utils_keypairs::get_treasury_keypair;
use test_utils_keypairs::get_wallet_keypair;
use test_utils_solana::TestValidatorPorts;

mod emoji;
mod spinner;
use spinner::Spinner;

#[derive(Clone, Debug, ValueEnum)]
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
	/// URL for Solana's JSON RPC or moniker (or their first letter):
	/// [mainnet-beta, testnet, devnet, localhost]
	#[arg(short = 'u', long, value_name = "URL_OR_MONIKER")]
	url: Option<String>,

	/// The authority address which will be signing the transaction
	#[arg(short = 'a', long)]
	authority: Option<Pubkey>,

	#[command(subcommand)]
	command: Commands,
}

impl Cli {
	fn get_url(&self) -> Result<String> {
		match &self.url {
			Some(url_or_moniker) => {
				// Check if it's a URL
				if url_or_moniker.starts_with("http") {
					return Ok(url_or_moniker.clone());
				}

				// Try to match moniker
				let lowercase = url_or_moniker.to_lowercase();
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
			None => Ok(Cluster::Localhost.url().to_string()),
		}
	}

	fn get_authority(&self) -> Pubkey {
		self.authority.unwrap_or(get_authority_keypair().pubkey())
	}
}

#[derive(Subcommand)]
enum Commands {
	/// Launch a test validator with the program deployed
	Validator,

	/// Config related commands
	#[command(subcommand)]
	Config(ConfigCommands),

	/// Game related commands
	#[command(subcommand)]
	Game(GameCommands),
}

#[derive(Subcommand)]
enum ConfigCommands {
	/// Initialize the program config
	Initialize,

	/// Update the program authority
	UpdateAuthority {
		/// The new authority's public key
		#[arg(long)]
		new_authority: Pubkey,
	},
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
	},
}

#[tokio::main]
async fn main() -> Result<()> {
	// env_logger::init();
	let cli = Cli::parse();
	let authority = cli.get_authority();

	// Get the URL and authority early to catch any errors
	let url = cli.get_url()?;
	println!("{} {}", "Using RPC URL:".bright_blue(), url.yellow());

	match cli.command {
		Commands::Validator => {
			let spinner = Spinner::new(
				format!("{} Starting validator...", emoji::VALIDATOR),
				"blue",
			);

			let devenv_root = std::env::var("DEVENV_ROOT").unwrap();
			let launchpad_program = test_utils_solana::TestProgramInfo::builder()
				.program_id(ID)
				.program_path(format!("{devenv_root}/target/deploy/bitflip_program.so"))
				.build();

			let props = test_utils_solana::TestValidatorRunnerProps::builder()
				.programs(vec![launchpad_program])
				.ports(TestValidatorPorts::default())
				.pubkeys(vec![
					get_admin_keypair().pubkey(),
					authority,
					get_treasury_keypair().pubkey(),
					get_wallet_keypair().pubkey(),
				])
				.commitment(CommitmentLevel::Finalized)
				.accounts(HashMap::new())
				.build();

			let runner = test_utils_solana::TestValidatorRunner::run(props).await;
			spinner.done("Validator started successfully");

			println!(
				"\n{}",
				format!("{} Test validator launched!", emoji::START)
					.green()
					.bold()
			);
			println!(
				"{} {}",
				format!("{} RPC URL:", emoji::INFO).bright_blue(),
				runner.rpc_url().yellow()
			);
			println!(
				"{} {}",
				format!("{} WS URL:", emoji::INFO).bright_blue(),
				runner.pubsub_url().yellow()
			);
			println!(
				"{} {}",
				format!("{} Program ID:", emoji::INFO).bright_blue(),
				ID.to_string().yellow()
			);

			println!(
				"\n{}",
				format!("{} Press Ctrl+C to stop the validator...", emoji::WARNING).bright_black()
			);
			tokio::signal::ctrl_c().await?;

			let shutdown_spinner =
				Spinner::new(format!("{} Shutting down validator...", emoji::STOP), "red");

			// Simulate shutdown time (you might want to actually track the real shutdown)
			tokio::time::sleep(Duration::from_secs(2)).await;

			shutdown_spinner.info("Test validator stopped");
		}

		Commands::Config(config_cmd) => {
			match config_cmd {
				ConfigCommands::Initialize => {
					println!(
						"{} {}",
						format!("{} Initializing config with authority:", emoji::CONFIG)
							.bright_blue(),
						authority.to_string().yellow()
					);
					// TODO: Implement config initialization using
					// bitflip_program
				}
				ConfigCommands::UpdateAuthority { new_authority } => {
					println!(
						"{} {} {} {}",
						format!("{} Updating authority from", emoji::CONFIG).bright_blue(),
						authority.to_string().yellow(),
						"to:".bright_blue(),
						new_authority.to_string().yellow()
					);
					// TODO: Implement authority update using bitflip_program
				}
			}
		}

		Commands::Game(game_cmd) => {
			match game_cmd {
				GameCommands::Initialize { index, start_time } => {
					println!(
						"{} {} {} {}",
						format!("{} Initializing game with index:", emoji::GAME).bright_blue(),
						index.to_string().yellow(),
						"and authority:".bright_blue(),
						authority.to_string().yellow()
					);
					if let Some(start_time) = start_time {
						println!(
							"{} {}",
							format!("{} Start time set to:", emoji::INFO).bright_blue(),
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
