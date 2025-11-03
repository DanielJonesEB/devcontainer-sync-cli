use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Repository error: {message}")]
    Repository { message: String, suggestion: String },

    #[error("Network error: {message}")]
    Network { message: String, suggestion: String },

    #[error("Git operation error: {message}")]
    GitOperation { message: String, suggestion: String },

    #[error("File system error: {message}")]
    FileSystem { message: String, suggestion: String },
}

impl CliError {
    pub fn exit_code(&self) -> i32 {
        match self {
            CliError::Repository { .. } => 1,
            CliError::Network { .. } => 2,
            CliError::GitOperation { .. } => 3,
            CliError::FileSystem { .. } => 4,
        }
    }

    pub fn suggestion(&self) -> &str {
        match self {
            CliError::Repository { suggestion, .. } => suggestion,
            CliError::Network { suggestion, .. } => suggestion,
            CliError::GitOperation { suggestion, .. } => suggestion,
            CliError::FileSystem { suggestion, .. } => suggestion,
        }
    }

    // Convenience constructors
    pub fn not_git_repository() -> Self {
        CliError::Repository {
            message: "Current directory is not a git repository".to_string(),
            suggestion:
                "Run this command from within a git repository or initialize one with 'git init'"
                    .to_string(),
        }
    }

    pub fn no_commits_found() -> Self {
        CliError::Repository {
            message: "Git repository has no commits found".to_string(),
            suggestion: "Make at least one commit before running this command".to_string(),
        }
    }
}
