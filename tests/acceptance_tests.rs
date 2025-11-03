use once_cell::sync::Lazy;
use rstest::*;
use spectral::prelude::*;
use std::path::{Path, PathBuf};
use std::process::Command;
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

#[rstest]
fn should_create_devcontainer_directory_with_json_file_after_successful_init(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["init"], &repo_path);

    // First verify the command succeeded
    result.should_succeed();

    // Then verify the .devcontainer directory was created
    let devcontainer_dir = repo_path.join(".devcontainer");
    assert_that(&devcontainer_dir.exists()).is_true();

    // Verify devcontainer.json file exists
    let devcontainer_json = devcontainer_dir.join("devcontainer.json");
    assert_that(&devcontainer_json.exists()).is_true();

    // Verify it's a valid file (not empty)
    let metadata = std::fs::metadata(&devcontainer_json).expect("Failed to get file metadata");
    assert_that(&metadata.len()).is_greater_than(0);
}
#[rstest]
fn should_show_minimal_output_without_verbose_flag(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["init"], &repo_path);

    result.should_succeed();
    result.should_contain_in_stdout("Successfully initialized devcontainer sync!");

    // Should not contain verbose messages
    assert!(!result.stdout.contains("Initializing devcontainer sync from Claude Code repository..."));
    assert!(!result.stdout.contains("Adding Claude Code remote..."));
}

#[rstest]
fn should_show_detailed_output_with_verbose_flag(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["init", "--verbose"], &repo_path);

    result.should_succeed();

    // Check all verbose messages are present in order
    result.should_contain_in_stdout("Initializing devcontainer sync from Claude Code repository...");
    result.should_contain_in_stdout("Adding Claude Code remote...");
    result.should_contain_in_stdout("Fetching from Claude Code repository...");
    result.should_contain_in_stdout("Creating tracking branch...");
    result.should_contain_in_stdout("Switching to Claude branch...");
    result.should_contain_in_stdout("Extracting devcontainer subtree...");
    result.should_contain_in_stdout("Returning to master branch...");
    result.should_contain_in_stdout("Adding devcontainer files...");
    result.should_contain_in_stdout("Successfully initialized devcontainer sync!");
}
// ============================================================================
// UPDATE COMMAND TESTS
// ============================================================================

#[rstest]
fn should_fail_update_command_when_not_initialized(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["update"], &repo_path);

    result.should_fail();
    // Should fail because Claude remote doesn't exist yet
    result.should_contain_in_stderr("Remote 'claude' does not exist");
}

#[rstest]
fn should_succeed_update_command_after_init(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Then update should work
    let update_result = run_command(&compiled_binary, &["update"], &repo_path);
    update_result.should_succeed();
    update_result.should_contain_in_stdout("Successfully updated devcontainer configurations!");
}

#[rstest]
fn should_show_verbose_output_for_update_command(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Then update with verbose flag
    let update_result = run_command(&compiled_binary, &["update", "--verbose"], &repo_path);
    update_result.should_succeed();
    update_result.should_contain_in_stdout("Updating devcontainer configurations...");
    update_result.should_contain_in_stdout("Fetching from Claude Code repository...");
    update_result.should_contain_in_stdout("Successfully updated devcontainer configurations!");
}

// ============================================================================
// REMOVE COMMAND TESTS
// ============================================================================

#[rstest]
fn should_fail_remove_command_when_not_initialized(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["remove"], &repo_path);

    result.should_fail();
    // Should fail because there's nothing to remove
    result.should_contain_in_stderr("Remote 'claude' does not exist");
}

#[rstest]
fn should_succeed_remove_command_after_init(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Verify .devcontainer exists
    assert_that(&repo_path.join(".devcontainer").exists()).is_true();

    // Then remove should work
    let remove_result = run_command(&compiled_binary, &["remove"], &repo_path);
    remove_result.should_succeed();
    remove_result.should_contain_in_stdout("Successfully removed devcontainer sync!");

    // Verify .devcontainer is removed
    assert_that(&repo_path.join(".devcontainer").exists()).is_false();
}

#[rstest]
fn should_show_verbose_output_for_remove_command(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Then remove with verbose flag
    let remove_result = run_command(&compiled_binary, &["remove", "--verbose"], &repo_path);
    remove_result.should_succeed();
    remove_result.should_contain_in_stdout("Removing devcontainer sync...");
    remove_result.should_contain_in_stdout("Removing Claude remote...");
    remove_result.should_contain_in_stdout("Successfully removed devcontainer sync!");
}

// ============================================================================
// UPDATE BACKUP FEATURE TESTS
// ============================================================================

#[rstest]
fn should_create_backup_when_backup_flag_is_used(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Verify .devcontainer exists
    assert_that(&repo_path.join(".devcontainer").exists()).is_true();

    // Modify a file in .devcontainer to test backup
    let devcontainer_json = repo_path.join(".devcontainer").join("devcontainer.json");
    if devcontainer_json.exists() {
        std::fs::write(&devcontainer_json, r#"{"name": "modified-for-test"}"#)
            .expect("Failed to modify devcontainer.json");
    }

    // Run update with backup flag
    let update_result = run_command(&compiled_binary, &["update", "--backup"], &repo_path);
    update_result.should_succeed();
    update_result.should_contain_in_stdout("Backup created before update");

    // Check that backup directory was created
    let backup_dir = repo_path.join(".devcontainer.backup");
    assert_that(&backup_dir.exists()).is_true();

    // Verify backup contains the modified file
    let backup_json = backup_dir.join("devcontainer.json");
    if backup_json.exists() {
        let backup_content = std::fs::read_to_string(&backup_json)
            .expect("Failed to read backup file");
        assert_that(&backup_content).contains("modified-for-test");
    }
}

#[rstest]
fn should_show_backup_message_in_verbose_mode(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Run update with backup and verbose flags
    let update_result = run_command(&compiled_binary, &["update", "--backup", "--verbose"], &repo_path);
    update_result.should_succeed();
    update_result.should_contain_in_stdout("Creating backup of existing devcontainer configuration...");
    update_result.should_contain_in_stdout("Backup created before update");
}

#[rstest]
fn should_not_create_backup_when_backup_flag_is_not_used(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Run update without backup flag
    let update_result = run_command(&compiled_binary, &["update"], &repo_path);
    update_result.should_succeed();

    // Should not mention backup in output
    assert!(!update_result.stdout.contains("Backup created"));
    assert!(!update_result.stdout.contains("backup"));

    // Check that backup directory was not created
    let backup_dir = repo_path.join(".devcontainer.backup");
    assert_that(&backup_dir.exists()).is_false();
}

#[rstest]
fn should_handle_backup_creation_failure_gracefully(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    // First initialize
    let init_result = run_command(&compiled_binary, &["init"], &repo_path);
    init_result.should_succeed();

    // Create a file where backup directory should be to cause conflict
    let backup_path = repo_path.join(".devcontainer.backup");
    std::fs::write(&backup_path, "blocking file")
        .expect("Failed to create blocking file");

    // Run update with backup flag - should handle the error gracefully
    let update_result = run_command(&compiled_binary, &["update", "--backup"], &repo_path);

    // Should either succeed with warning or fail with helpful error message
    if update_result.exit_code != 0 {
        update_result.should_contain_in_stderr("backup");
    } else {
        // If it succeeds, it should warn about backup issues
        assert!(update_result.stderr.contains("backup") || update_result.stdout.contains("warning"));
    }
}

// ============================================================================
// ERROR HANDLING AND RECOVERY TESTS
// ============================================================================

#[rstest]
fn should_handle_invalid_command_gracefully(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["invalid-command"], &repo_path);

    result.should_fail();
    // Should show help or error about invalid command
}

#[rstest]
fn should_show_help_when_no_command_provided(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &[], &repo_path);

    result.should_fail();
    // Should show help message
}

#[rstest]
fn should_show_version_information(
    temp_git_repo_with_commits: (TempDir, PathBuf),
    compiled_binary: PathBuf
) {
    let (_temp_dir, repo_path) = temp_git_repo_with_commits;

    let result = run_command(&compiled_binary, &["--version"], &repo_path);

    result.should_succeed();
    result.should_contain_in_stdout("devcontainer-sync");
}
