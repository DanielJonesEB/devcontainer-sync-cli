use crate::error::CliError;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;

pub trait GitExecutor {
    fn execute_git_command(&self, args: &[&str], working_dir: &Path) -> Result<String, CliError>;
    fn execute_git_command_with_timeout(
        &self,
        args: &[&str],
        working_dir: &Path,
        timeout: Duration,
    ) -> Result<String, CliError>;
}

pub struct SystemGitExecutor;

impl SystemGitExecutor {
    pub fn new() -> Self {
        Self
    }
}

impl GitExecutor for SystemGitExecutor {
    fn execute_git_command(&self, args: &[&str], working_dir: &Path) -> Result<String, CliError> {
        self.execute_git_command_with_timeout(args, working_dir, Duration::from_secs(30))
    }

    fn execute_git_command_with_timeout(
        &self,
        args: &[&str],
        working_dir: &Path,
        _timeout: Duration,
    ) -> Result<String, CliError> {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(working_dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // Execute the command
        let output = command.output().map_err(|e| CliError::GitOperation {
            message: format!("Failed to execute git command: {}", e),
            suggestion: "Make sure git is installed and available in PATH".to_string(),
        })?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() {
            return Err(CliError::GitOperation {
                message: format!(
                    "Git command failed: git {}\nError: {}",
                    args.join(" "),
                    stderr
                ),
                suggestion: format!(
                    "Check the git command syntax and repository state. Command: git {}",
                    args.join(" ")
                ),
            });
        }

        Ok(stdout)
    }
}

impl Default for SystemGitExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_git_repo() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();

        // Initialize git repository
        Command::new("git")
            .args(["init"])
            .current_dir(&path)
            .output()
            .expect("Failed to initialize git repository");

        // Configure git user
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(&path)
            .output()
            .expect("Failed to configure git user name");

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(&path)
            .output()
            .expect("Failed to configure git user email");

        // Create a test file and make initial commit
        fs::write(path.join("test.txt"), "test content").expect("Failed to create test file");

        Command::new("git")
            .args(["add", "test.txt"])
            .current_dir(&path)
            .output()
            .expect("Failed to add file to git");

        Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(&path)
            .output()
            .expect("Failed to make initial commit");

        (temp_dir, path)
    }

    #[test]
    fn test_execute_git_command_success() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();

        let result = executor.execute_git_command(&["status", "--porcelain"], &repo_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute_git_command_failure() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();
        let executor = SystemGitExecutor::new();

        // Use a command that will definitely fail - invalid git subcommand
        let result = executor.execute_git_command(&["this-is-not-a-valid-git-command"], &path);

        // Print result for debugging
        match &result {
            Ok(output) => println!("Unexpected success with output: '{}'", output),
            Err(e) => println!("Expected git command error: {:?}", e),
        }

        assert!(
            result.is_err(),
            "Expected git command to fail with invalid subcommand"
        );

        match result {
            Err(CliError::GitOperation { message, .. }) => {
                assert!(message.contains("Git command failed"));
            }
            _ => panic!("Expected GitOperation error"),
        }
    }

    #[test]
    fn test_execute_git_command_with_timeout() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();

        let result = executor.execute_git_command_with_timeout(
            &["log", "--oneline"],
            &repo_path,
            Duration::from_secs(5),
        );
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Initial commit"));
    }
}
