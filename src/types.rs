use std::path::PathBuf;
use std::time::Duration;
use crate::error::CliError;

/// Context for command execution
#[derive(Debug)]
pub struct CommandContext {
    pub working_dir: PathBuf,
    pub verbose: bool,
    pub dry_run: bool,
    pub timeout: Duration,
}

impl CommandContext {
    pub fn new(working_dir: PathBuf, verbose: bool, dry_run: bool) -> Self {
        Self {
            working_dir,
            verbose,
            dry_run,
            timeout: crate::config::default_timeout(),
        }
    }
}

/// Result of a command operation
#[derive(Debug)]
pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub changes: Vec<String>,
    pub warnings: Vec<String>,
    pub errors: Vec<CliError>,
}

impl OperationResult {
    pub fn success(message: String) -> Self {
        Self {
            success: true,
            message,
            changes: Vec::new(),
            warnings: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn failure(message: String, error: CliError) -> Self {
        Self {
            success: false,
            message,
            changes: Vec::new(),
            warnings: Vec::new(),
            errors: vec![error],
        }
    }
}

/// Wrapper for Git command execution
#[derive(Debug)]
pub struct GitCommand {
    pub args: Vec<String>,
    pub working_dir: PathBuf,
    pub timeout: Duration,
    pub output: String,
    pub error: Option<CliError>,
}
