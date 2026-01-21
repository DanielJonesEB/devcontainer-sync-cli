use clap::{Parser, Subcommand};
use devcontainer_sync_cli::cli::CliApp;
use std::process;

#[derive(Parser)]
#[command(name = "devcontainer-sync")]
#[command(about = "A CLI tool for syncing devcontainer configurations from Claude Code repository")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize devcontainer tracking from Claude Code repository
    Init {
        /// Remove firewall configurations from devcontainer files
        #[arg(long)]
        strip_firewall: bool,
    },
    /// Update existing devcontainer configurations
    Update {
        /// Create backup before updating
        #[arg(long)]
        backup: bool,
        /// Force update even if conflicts exist
        #[arg(long)]
        force: bool,
        /// Remove firewall configurations from devcontainer files
        #[arg(long)]
        strip_firewall: bool,
    },
    /// Remove devcontainer tracking and cleanup
    Remove {
        /// Keep devcontainer files when removing tracking
        #[arg(long)]
        keep_files: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    let app = CliApp::new(cli.verbose);

    let result = match cli.command {
        Commands::Init { strip_firewall } => app.init(strip_firewall),
        Commands::Update {
            backup,
            force,
            strip_firewall,
        } => app.update(backup, force, strip_firewall),
        Commands::Remove { keep_files } => app.remove(keep_files),
    };

    match result {
        Ok(_) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            if cli.verbose {
                eprintln!("Suggestion: {}", e.suggestion());
            }
            process::exit(e.exit_code());
        }
    }
}
