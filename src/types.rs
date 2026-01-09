use crate::error::CliError;
use std::path::PathBuf;
use std::time::Duration;

/// Context for command execution
#[derive(Debug, Clone)]
pub struct CommandContext {
    pub working_dir: PathBuf,
    pub verbose: bool,
    pub strip_firewall: bool,
    pub timeout: Duration,
}

impl CommandContext {
    pub fn new(working_dir: PathBuf, verbose: bool) -> Self {
        Self {
            working_dir,
            verbose,
            strip_firewall: false,
            timeout: crate::config::default_timeout(),
        }
    }

    pub fn with_strip_firewall(mut self, strip_firewall: bool) -> Self {
        self.strip_firewall = strip_firewall;
        self
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

    pub fn add_change(&mut self, change: String) {
        self.changes.push(change);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty()
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

impl GitCommand {
    pub fn new(args: Vec<String>, working_dir: PathBuf) -> Self {
        Self {
            args,
            working_dir,
            timeout: crate::config::default_timeout(),
            output: String::new(),
            error: None,
        }
    }

    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    pub fn is_success(&self) -> bool {
        self.error.is_none()
    }
}
