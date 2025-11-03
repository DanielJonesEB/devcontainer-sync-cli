use once_cell::sync::Lazy;
use rstest::*;
use spectral::prelude::*;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Mutex;
use tempfile::TempDir;

static BINARY_PATH: Lazy<Mutex<Option<PathBuf>>> = Lazy::new(|| Mutex::new(None));

#[derive(Debug)]
pub struct CommandResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Fixture that creates a temporary directory without git initialization
#[fixture]
fn temp_non_git_dir() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_path_buf();
    (temp_dir, path)
}

/// Fixture that creates a temporary git repository without any commits
#[fixture]
fn temp_git_repo_without_commits() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_path_buf();

    // Initialize git repository but don't make any commits
    let output = Command::new("git")
        .args(&["init"])
        .current_dir(&path)
        .output()
        .expect("Failed to initialize git repository");

    if !output.status.success() {
        panic!("Failed to initialize git repository: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Configure git user for the repository
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

    (temp_dir, path)
}

/// Fixture that creates a temporary git repository with at least one commit
#[fixture]
fn temp_git_repo_with_commits() -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    let path = temp_dir.path().to_path_buf();

    // Initialize git repository
    let output = Command::new("git")
        .args(&["init"])
        .current_dir(&path)
        .output()
        .expect("Failed to initialize git repository");

    if !output.status.success() {
        panic!("Failed to initialize git repository: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Configure git user for the repository
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

    // Create a test file and make initial commit
    std::fs::write(path.join("README.md"), "# Test Repository\n")
        .expect("Failed to create test file");

    Command::new("git")
        .args(&["add", "README.md"])
        .current_dir(&path)
        .output()
        .expect("Failed to add file to git");

    let output = Command::new("git")
        .args(&["commit", "-m", "Initial commit"])
        .current_dir(&path)
        .output()
        .expect("Failed to make initial commit");

    if !output.status.success() {
        panic!("Failed to make initial commit: {}", String::from_utf8_lossy(&output.stderr));
    }

    (temp_dir, path)
}

/// Fixture that compiles the binary and returns its path
#[fixture]
fn compiled_binary() -> PathBuf {
    let mut binary_path = BINARY_PATH.lock().unwrap();

    if let Some(ref path) = *binary_path {
        return path.clone();
    }

    // Compile the binary
    let output = Command::new("cargo")
        .args(&["build", "--bin", "devcontainer-sync"])
        .output()
        .expect("Failed to compile binary");

    if !output.status.success() {
        panic!("Failed to compile binary: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Get the binary path
    let target_dir = std::env::current_dir()
        .expect("Failed to get current directory")
        .join("target")
        .join("debug")
        .join("devcontainer-sync");

    if !target_dir.exists() {
        panic!("Binary not found at expected path: {:?}", target_dir);
    }

    *binary_path = Some(target_dir.clone());
    target_dir
}

/// Helper function to run the compiled binary with given arguments in a specific directory
pub fn run_command(binary_path: &Path, args: &[&str], working_dir: &Path) -> CommandResult {
    let output = Command::new(binary_path)
        .args(args)
        .current_dir(working_dir)
        .output()
        .expect("Failed to execute command");

    CommandResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    }
}

/// Additional assertion helpers for more readable tests
impl CommandResult {
    pub fn should_succeed(&self) -> &Self {
        assert_that(&self.exit_code).is_equal_to(0);
        self
    }

    pub fn should_fail(&self) -> &Self {
        assert_that(&self.exit_code).is_not_equal_to(0);
        self
    }

    pub fn should_contain_in_stderr(&self, text: &str) -> &Self {
        assert_that(&self.stderr).contains(text);
        self
    }

    pub fn should_contain_in_stdout(&self, text: &str) -> &Self {
        assert_that(&self.stdout).contains(text);
        self
    }
}

// ============================================================================
// ACCEPTANCE TESTS
// ============================================================================

#[rstest]
fn should_fail_when_not_a_git_repository(
    temp_non_git_dir: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, dir_path) = temp_non_git_dir;

    let result = run_command(&compiled_binary, &["init"], &dir_path);

    result
        .should_fail()
        .should_contain_in_stderr("not a git repository");
}

#[rstest]
fn should_fail_when_git_repo_has_no_commits(
    temp_git_repo_without_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_without_commits;

    let result = run_command(&compiled_binary, &["init"], &repo_path);

    result
        .should_fail()
        .should_contain_in_stderr("no commits found");
}

#[rstest]
fn should_succeed_when_git_repo_has_commits(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["init"], &repo_path);

    result
        .should_succeed()
        .should_contain_in_stdout("Successfully initialized");
}
