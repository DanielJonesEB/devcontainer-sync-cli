use crate::error::CliError;
use crate::git::GitExecutor;

#[derive(Debug, Clone)]
pub struct Branch {
    pub name: String,
    pub is_current: bool,
    pub upstream: Option<String>,
}

pub trait BranchManager {
    fn create_branch(&self, name: &str, source: &str) -> Result<(), CliError>;
    fn delete_branch(&self, name: &str) -> Result<(), CliError>;
    fn checkout_branch(&self, name: &str) -> Result<(), CliError>;
    fn list_branches(&self) -> Result<Vec<Branch>, CliError>;
    fn force_create_branch(&self, name: &str, source: &str) -> Result<(), CliError>;
}

pub struct GitBranchManager<T: GitExecutor> {
    executor: T,
    working_dir: std::path::PathBuf,
}

impl<T: GitExecutor> GitBranchManager<T> {
    pub fn new(executor: T, working_dir: std::path::PathBuf) -> Self {
        Self {
            executor,
            working_dir,
        }
    }
}

impl<T: GitExecutor> BranchManager for GitBranchManager<T> {
    fn create_branch(&self, name: &str, source: &str) -> Result<(), CliError> {
        self.executor
            .execute_git_command(&["branch", name, source], &self.working_dir)?;

        Ok(())
    }

    fn force_create_branch(&self, name: &str, source: &str) -> Result<(), CliError> {
        // Use -f flag to force create/update the branch
        self.executor
            .execute_git_command(&["branch", "-f", name, source], &self.working_dir)?;

        Ok(())
    }

    fn delete_branch(&self, name: &str) -> Result<(), CliError> {
        // Use -D flag to force delete the branch
        self.executor
            .execute_git_command(&["branch", "-D", name], &self.working_dir)?;

        Ok(())
    }

    fn checkout_branch(&self, name: &str) -> Result<(), CliError> {
        self.executor
            .execute_git_command(&["checkout", name], &self.working_dir)?;

        Ok(())
    }

    fn list_branches(&self) -> Result<Vec<Branch>, CliError> {
        let output = self
            .executor
            .execute_git_command(&["branch", "-vv"], &self.working_dir)?;

        let mut branches = Vec::new();

        for line in output.lines() {
            if line.trim().is_empty() {
                continue;
            }

            let line = line.trim();
            let is_current = line.starts_with('*');

            // Remove the * prefix if present
            let line = if is_current {
                line.strip_prefix('*').unwrap_or(line).trim()
            } else {
                line
            };

            // Parse branch name (first word)
            if let Some(name) = line.split_whitespace().next() {
                // Extract upstream info if present (between square brackets)
                let upstream = if let Some(start) = line.find('[') {
                    line.find(']').map(|end| line[start + 1..end].to_string())
                } else {
                    None
                };

                branches.push(Branch {
                    name: name.to_string(),
                    is_current,
                    upstream,
                });
            }
        }

        Ok(branches)
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
    fn test_list_branches() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitBranchManager::new(executor, repo_path);

        let branches = manager.list_branches().unwrap();
        assert!(!branches.is_empty());

        // Should have at least the main/master branch
        let current_branch = branches.iter().find(|b| b.is_current);
        assert!(current_branch.is_some());
    }

    #[test]
    fn test_create_branch() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitBranchManager::new(executor, repo_path);

        let result = manager.create_branch("test-branch", "HEAD");
        assert!(result.is_ok());

        // Verify branch was created
        let branches = manager.list_branches().unwrap();
        assert!(branches.iter().any(|b| b.name == "test-branch"));
    }

    #[test]
    fn test_force_create_branch() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitBranchManager::new(executor, repo_path);

        // Create branch first
        manager.create_branch("test-branch", "HEAD").unwrap();

        // Force create it again (should succeed)
        let result = manager.force_create_branch("test-branch", "HEAD");
        assert!(result.is_ok());
    }

    #[test]
    fn test_checkout_branch() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitBranchManager::new(executor, repo_path);

        // Create and checkout a new branch
        manager.create_branch("test-branch", "HEAD").unwrap();
        let result = manager.checkout_branch("test-branch");
        assert!(result.is_ok());

        // Verify we're on the new branch
        let branches = manager.list_branches().unwrap();
        let current_branch = branches.iter().find(|b| b.is_current).unwrap();
        assert_eq!(current_branch.name, "test-branch");
    }

    #[test]
    fn test_delete_branch() {
        let (_temp_dir, repo_path) = create_test_git_repo();
        let executor = SystemGitExecutor::new();
        let manager = GitBranchManager::new(executor, repo_path);

        // Create a branch
        manager.create_branch("test-branch", "HEAD").unwrap();

        // Delete it
        let result = manager.delete_branch("test-branch");
        assert!(result.is_ok());

        // Verify branch was deleted
        let branches = manager.list_branches().unwrap();
        assert!(!branches.iter().any(|b| b.name == "test-branch"));
    }
}
