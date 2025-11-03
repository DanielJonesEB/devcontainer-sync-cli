use crate::error::CliError;
use crate::git::GitExecutor;

pub trait SubtreeManager {
    fn split_subtree(&self, prefix: &str, branch: &str) -> Result<(), CliError>;
    fn add_subtree(&self, prefix: &str, branch: &str, squash: bool) -> Result<(), CliError>;
    fn update_subtree(&self, prefix: &str, branch: &str) -> Result<(), CliError>;
    fn remove_subtree(&self, prefix: &str) -> Result<(), CliError>;
}

pub struct GitSubtreeManager<T: GitExecutor> {
    executor: T,
    working_dir: std::path::PathBuf,
}

impl<T: GitExecutor> GitSubtreeManager<T> {
    pub fn new(executor: T, working_dir: std::path::PathBuf) -> Self {
        Self {
            executor,
            working_dir,
        }
    }
}

impl<T: GitExecutor> SubtreeManager for GitSubtreeManager<T> {
    fn split_subtree(&self, prefix: &str, branch: &str) -> Result<(), CliError> {
        // git subtree split --prefix=<prefix> -b <branch> <source>
        // For our use case, we'll split from the current branch
        let prefix_arg = format!("--prefix={}", prefix);
        self.executor.execute_git_command(
            &["subtree", "split", &prefix_arg, "-b", branch],
            &self.working_dir
        )?;

        Ok(())
    }

    fn add_subtree(&self, prefix: &str, branch: &str, squash: bool) -> Result<(), CliError> {
        let prefix_arg = format!("--prefix={}", prefix);
        let mut args = vec!["subtree", "add", &prefix_arg];

        if squash {
            args.push("--squash");
        }

        args.push(branch);

        self.executor.execute_git_command(&args, &self.working_dir)?;

        Ok(())
    }

    fn update_subtree(&self, prefix: &str, branch: &str) -> Result<(), CliError> {
        // Use subtree pull to update an existing subtree
        let prefix_arg = format!("--prefix={}", prefix);
        self.executor.execute_git_command(
            &["subtree", "pull", &prefix_arg, "--squash", branch],
            &self.working_dir
        )?;

        Ok(())
    }

    fn remove_subtree(&self, prefix: &str) -> Result<(), CliError> {
        // Git doesn't have a built-in subtree remove command
        // We'll remove the directory and commit the change
        use std::fs;

        let subtree_path = self.working_dir.join(prefix);

        if subtree_path.exists() {
            fs::remove_dir_all(&subtree_path).map_err(|e| CliError::FileSystem {
                message: format!("Failed to remove subtree directory '{}': {}", prefix, e),
                suggestion: "Check file permissions and ensure the directory is not in use".to_string(),
            })?;

            // Stage the removal
            self.executor.execute_git_command(
                &["add", prefix],
                &self.working_dir
            )?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::git::SystemGitExecutor;
    use tempfile::TempDir;
    use std::process::Command;
    use std::fs;

    fn create_test_git_repo_with_subtree() -> (TempDir, std::path::PathBuf) {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let path = temp_dir.path().to_path_buf();

        // Initialize git repository
        Command::new("git")
            .args(&["init"])
            .current_dir(&path)
            .output()
            .expect("Failed to initialize git repository");

        // Configure git user
        Command::new("git")
            .args(&["config", "user.name", "Test User"])
            .current_dir(&path)
            .output()
            .expect("Failed to configure git user name");

        Command::new("git")
            .args(&["config", "user.email", "test@example.com"])
            .current_dir(&path)
            .output()
            .expect("Failed to configure git user email");

        // Create a test directory structure that can be used for subtree operations
        fs::create_dir_all(path.join("subdir")).expect("Failed to create subdir");
        fs::write(path.join("subdir/file.txt"), "subtree content")
            .expect("Failed to create subtree file");
        fs::write(path.join("main.txt"), "main content")
            .expect("Failed to create main file");

        Command::new("git")
            .args(&["add", "."])
            .current_dir(&path)
            .output()
            .expect("Failed to add files to git");

        Command::new("git")
            .args(&["commit", "-m", "Initial commit with subtree content"])
            .current_dir(&path)
            .output()
            .expect("Failed to make initial commit");

        (temp_dir, path)
    }

    #[test]
    fn test_split_subtree() {
        let (_temp_dir, repo_path) = create_test_git_repo_with_subtree();
        let executor = SystemGitExecutor::new();
        let manager = GitSubtreeManager::new(executor, repo_path.clone());

        let result = manager.split_subtree("subdir", "subtree-branch");
        assert!(result.is_ok());

        // Verify the branch was created
        let output = Command::new("git")
            .args(&["branch", "--list", "subtree-branch"])
            .current_dir(&repo_path)
            .output()
            .expect("Failed to list branches");

        let branch_list = String::from_utf8_lossy(&output.stdout);
        assert!(branch_list.contains("subtree-branch"));
    }

    #[test]
    fn test_remove_subtree() {
        let (_temp_dir, repo_path) = create_test_git_repo_with_subtree();
        let executor = SystemGitExecutor::new();
        let manager = GitSubtreeManager::new(executor, repo_path.clone());

        let result = manager.remove_subtree("subdir");
        assert!(result.is_ok());

        // Verify the directory was removed
        assert!(!repo_path.join("subdir").exists());
    }

    #[test]
    fn test_remove_nonexistent_subtree() {
        let (_temp_dir, repo_path) = create_test_git_repo_with_subtree();
        let executor = SystemGitExecutor::new();
        let manager = GitSubtreeManager::new(executor, repo_path);

        // Should succeed even if directory doesn't exist
        let result = manager.remove_subtree("nonexistent");
        assert!(result.is_ok());
    }

    // Note: add_subtree and update_subtree tests are more complex as they require
    // actual remote repositories or more sophisticated setup. For now, we'll test
    // the basic functionality that doesn't require network access.
}
