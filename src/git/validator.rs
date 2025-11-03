use crate::error::CliError;
use std::path::Path;
use std::process::Command;

pub trait RepositoryValidator {
    fn validate_git_repository(&self, path: &Path) -> Result<(), CliError>;
    fn check_existing_remote(&self, remote_name: &str) -> Result<bool, CliError>;
    fn check_existing_branch(&self, branch_name: &str) -> Result<bool, CliError>;
    fn validate_has_commits(&self) -> Result<(), CliError>;
}

pub struct GitRepositoryValidator {
    working_dir: std::path::PathBuf,
}

impl GitRepositoryValidator {
    pub fn new(working_dir: std::path::PathBuf) -> Self {
        Self { working_dir }
    }
}

impl RepositoryValidator for GitRepositoryValidator {
    fn validate_git_repository(&self, path: &Path) -> Result<(), CliError> {
        let git_dir = path.join(".git");

        if !git_dir.exists() {
            return Err(CliError::not_git_repository());
        }

        // Also check if git command recognizes this as a valid repository
        let output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(path)
            .output()
            .map_err(|e| CliError::GitOperation {
                message: format!("Failed to execute git command: {}", e),
                suggestion: "Make sure git is installed and available in PATH".to_string(),
            })?;

        if !output.status.success() {
            return Err(CliError::not_git_repository());
        }

        Ok(())
    }

    fn check_existing_remote(&self, remote_name: &str) -> Result<bool, CliError> {
        let output = Command::new("git")
            .args(["remote", "get-url", remote_name])
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| CliError::GitOperation {
                message: format!("Failed to check remote: {}", e),
                suggestion: "Make sure git is installed and available in PATH".to_string(),
            })?;

        Ok(output.status.success())
    }

    fn check_existing_branch(&self, branch_name: &str) -> Result<bool, CliError> {
        let output = Command::new("git")
            .args([
                "show-ref",
                "--verify",
                &format!("refs/heads/{}", branch_name),
            ])
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| CliError::GitOperation {
                message: format!("Failed to check branch: {}", e),
                suggestion: "Make sure git is installed and available in PATH".to_string(),
            })?;

        Ok(output.status.success())
    }

    fn validate_has_commits(&self) -> Result<(), CliError> {
        let output = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(&self.working_dir)
            .output()
            .map_err(|e| CliError::GitOperation {
                message: format!("Failed to check for commits: {}", e),
                suggestion: "Make sure git is installed and available in PATH".to_string(),
            })?;

        if !output.status.success() {
            return Err(CliError::no_commits_found());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    fn create_temp_git_repo(with_commits: bool) -> (TempDir, std::path::PathBuf) {
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

        if with_commits {
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
        }

        (temp_dir, path)
    }

    #[test]
    fn test_validate_git_repository_success() {
        let (_temp_dir, repo_path) = create_temp_git_repo(false);
        let validator = GitRepositoryValidator::new(repo_path.clone());

        let result = validator.validate_git_repository(&repo_path);
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_git_repository_not_git_repo() {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();
        let validator = GitRepositoryValidator::new(path.clone());

        let result = validator.validate_git_repository(&path);
        assert!(result.is_err());

        if let Err(CliError::Repository { message, .. }) = result {
            assert!(message.contains("not a git repository"));
        } else {
            panic!("Expected Repository error");
        }
    }

    #[test]
    fn test_validate_has_commits_success() {
        let (_temp_dir, repo_path) = create_temp_git_repo(true);
        let validator = GitRepositoryValidator::new(repo_path);

        let result = validator.validate_has_commits();
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_has_commits_no_commits() {
        let (_temp_dir, repo_path) = create_temp_git_repo(false);
        let validator = GitRepositoryValidator::new(repo_path);

        let result = validator.validate_has_commits();
        assert!(result.is_err());

        if let Err(CliError::Repository { message, .. }) = result {
            assert!(message.contains("no commits found"));
        } else {
            panic!("Expected Repository error");
        }
    }

    #[test]
    fn test_check_existing_remote_not_exists() {
        let (_temp_dir, repo_path) = create_temp_git_repo(true);
        let validator = GitRepositoryValidator::new(repo_path);

        let result = validator.check_existing_remote("nonexistent");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }

    #[test]
    fn test_check_existing_branch_not_exists() {
        let (_temp_dir, repo_path) = create_temp_git_repo(true);
        let validator = GitRepositoryValidator::new(repo_path);

        let result = validator.check_existing_branch("nonexistent");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
