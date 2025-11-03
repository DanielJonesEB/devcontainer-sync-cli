use crate::error::CliError;
use crate::git::GitExecutor;

#[derive(Debug, Clone)]
pub struct Remote {
    pub name: String,
    pub url: String,
}

pub trait RemoteManager {
    fn add_remote(&self, name: &str, url: &str) -> Result<(), CliError>;
    fn remove_remote(&self, name: &str) -> Result<(), CliError>;
    fn fetch_remote(&self, name: &str) -> Result<(), CliError>;
    fn list_remotes(&self) -> Result<Vec<Remote>, CliError>;
}

pub struct GitRemoteManager<T: GitExecutor> {
    executor: T,
    working_dir: std::path::PathBuf,
}

impl<T: GitExecutor> GitRemoteManager<T> {
    pub fn new(executor: T, working_dir: std::path::PathBuf) -> Self {
        Self {
            executor,
            working_dir,
        }
    }
}

impl<T: GitExecutor> RemoteManager for GitRemoteManager<T> {
    fn add_remote(&self, name: &str, url: &str) -> Result<(), CliError> {
        self.executor
            .execute_git_command(&["remote", "add", name, url], &self.working_dir)?;

        // Verify the remote was added successfully
        if self
            .executor
            .execute_git_command(&["remote", "get-url", name], &self.working_dir)
            .is_err()
        {
            return Err(CliError::GitOperation {
                message: format!("Failed to add remote '{}' with URL '{}'", name, url),
                suggestion: "Check that the remote name is valid and the URL is accessible"
                    .to_string(),
            });
        }

        Ok(())
    }

    fn remove_remote(&self, name: &str) -> Result<(), CliError> {
        // Check if remote exists first
        if self
            .executor
            .execute_git_command(&["remote", "get-url", name], &self.working_dir)
            .is_err()
        {
            return Err(CliError::GitOperation {
                message: format!("Remote '{}' does not exist", name),
                suggestion: "Use 'git remote -v' to list existing remotes".to_string(),
            });
        }

        self.executor
            .execute_git_command(&["remote", "remove", name], &self.working_dir)?;

        Ok(())
    }

    fn fetch_remote(&self, name: &str) -> Result<(), CliError> {
        // Check if remote exists first
        if self
            .executor
            .execute_git_command(&["remote", "get-url", name], &self.working_dir)
            .is_err()
        {
            return Err(CliError::GitOperation {
                message: format!("Remote '{}' does not exist", name),
                suggestion: "Add the remote first using 'git remote add'".to_string(),
            });
        }

        self.executor
            .execute_git_command(&["fetch", name], &self.working_dir)?;

        Ok(())
    }

    fn list_remotes(&self) -> Result<Vec<Remote>, CliError> {
        let output = self
            .executor
            .execute_git_command(&["remote", "-v"], &self.working_dir)?;

        let mut remotes = Vec::new();
        let mut seen_names = std::collections::HashSet::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 2 {
                let name = parts[0].to_string();
                let url = parts[1].to_string();

                // Only add each remote name once (git remote -v shows fetch and push URLs)
                if seen_names.insert(name.clone()) {
                    remotes.push(Remote { name, url });
                }
            }
        }

        Ok(remotes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::SystemGitExecutor;
    use std::fs;
    use std::process::Command;
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
    fn test_add_remote_success() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitRemoteManager::new(executor, repo_path);

        let result = manager.add_remote("test", "https://github.com/test/repo.git");
        assert!(result.is_ok());

        // Verify remote was added
        let remotes = manager.list_remotes().unwrap();
        assert!(remotes.iter().any(|r| r.name == "test"));
    }

    #[test]
    fn test_list_remotes_empty() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitRemoteManager::new(executor, repo_path);

        let remotes = manager.list_remotes().unwrap();
        assert!(remotes.is_empty());
    }

    #[test]
    fn test_remove_remote_not_exists() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitRemoteManager::new(executor, repo_path);

        let result = manager.remove_remote("nonexistent");
        assert!(result.is_err());

        if let Err(CliError::GitOperation { message, .. }) = result {
            assert!(message.contains("does not exist"));
        } else {
            panic!("Expected GitOperation error");
        }
    }

    #[test]
    fn test_fetch_remote_not_exists() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitRemoteManager::new(executor, repo_path);

        let result = manager.fetch_remote("nonexistent");
        assert!(result.is_err());

        if let Err(CliError::GitOperation { message, .. }) = result {
            assert!(message.contains("does not exist"));
        } else {
            panic!("Expected GitOperation error");
        }
    }
}
