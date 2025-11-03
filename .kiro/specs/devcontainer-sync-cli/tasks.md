# Implementation Plan

- [x] 1. Set up project structure and acceptance testing framework
  - Create Rust project with Cargo.toml including clap, thiserror, tempfile, rstest, spectral dependencies
  - Set up basic CLI structure with main.rs and lib.rs
  - Define project module structure (cli, git, error, config modules)
  - **Expected test results: 0/15 acceptance tests pass (binary doesn't exist yet)**
  - _Requirements: 2.1_

- [x] 2. Create acceptance test fixtures and write failing tests
  - [x] 2.1 Create test fixtures and helpers
    - Implement temp_git_repo_with_commits fixture using tempfile
    - Create temp_git_repo_without_commits fixture for empty repositories
    - Add temp_non_git_dir fixture for non-git directories
    - Write compiled_binary fixture with binary compilation and caching
    - _Requirements: 4.1, 4.2_

  - [x] 2.2 Implement command execution test helpers
    - Write run_command helper function for executing compiled binary
    - Create CommandResult struct for capturing exit codes and output
    - Add assertion helpers using spectral for readable test expectations
    - _Requirements: 4.1, 4.3_

  - [x] 2.3 Write the required acceptance tests
    - Implement test for non-git repository error case
    - Write test for git repository without commits error case
    - Create test for successful execution with valid git repository
    - Add test for devcontainer directory creation
    - Add test for minimal logging output (without --verbose)
    - Add test for detailed logging output (with --verbose)
    - **Expected test results: 0/15 acceptance tests pass (no CLI implementation yet)**
    - _Requirements: 4.1, 4.2_

- [x] 3. Create minimal CLI structure to make first test pass
  - [x] 3.1 Implement basic error handling and core types
    - Create CliError enum with thiserror derive
    - Define Repository, Network, GitOperation, and FileSystem error variants
    - Implement exit_code method for each error type
    - Add suggestion field to provide actionable error messages
    - _Requirements: 4.1, 4.4_

  - [x] 3.2 Set up clap CLI framework with basic structure
    - Define main command structure with init, update, remove subcommands
    - Add global flags for verbose and help options
    - Implement command parsing and validation
    - **Expected test results: 0/15 acceptance tests pass (no repository validation yet)**
    - _Requirements: 2.1, 4.4_

- [x] 4. Implement repository validation to make error tests pass
  - [x] 4.1 Create repository validation functionality
    - Implement RepositoryValidator trait and struct
    - Write validate_git_repository method to check for .git directory
    - Implement validate_has_commits method to ensure repository has commits
    - Add check_existing_remote and check_existing_branch methods
    - **Expected test results: 3/15 acceptance tests pass (validation tests work, init functionality and update/remove tests fail)**
    - _Requirements: 4.2, 1.1_

  - [x] 4.2 Write unit tests for repository validation
    - Test validation with valid git repositories
    - Test error cases for non-git directories and empty repositories
    - _Requirements: 4.2_

- [x] 5. Implement Git command execution layer
  - [x] 5.1 Create GitExecutor trait and implementation
    - Write execute_git_command method with timeout handling
    - Implement command output parsing and error handling
    - Add working directory and environment variable support
    - _Requirements: 2.2, 2.4_

  - [x] 5.2 Define core data structures
    - Implement CommandContext struct with working directory and options
    - Create OperationResult struct for command execution results
    - Define GitCommand wrapper for git process execution
    - _Requirements: 4.3_

- [x] 6. Implement Git operation managers
  - [x] 6.1 Implement RemoteManager for git remote operations
    - Write add_remote method to add Claude Code repository
    - Implement remove_remote method for cleanup operations
    - Add fetch_remote method with progress reporting
    - _Requirements: 1.1, 1.2, 5.1_

  - [x] 6.2 Implement BranchManager for git branch operations
    - Write create_branch method for tracking branches
    - Implement delete_branch method for cleanup
    - Add checkout_branch method for branch switching
    - _Requirements: 1.3, 1.4, 5.2_

  - [x] 6.3 Implement SubtreeManager for git subtree operations
    - Write split_subtree method to extract .devcontainer directory
    - Implement add_subtree method to integrate devcontainer files
    - Add update_subtree method for synchronization
    - _Requirements: 1.4, 1.5, 3.2_

- [x] 7. Implement init command to make success test pass
  - [x] 7.1 Add configuration and constants
    - Set Claude repository URL and remote name constants
    - Define branch names and directory paths
    - Add default timeout and configuration values
    - _Requirements: 1.1, 2.2_

  - [x] 7.2 Implement init command with specific Git command sequence
    - Write command handler that validates repository state (Requirements: 1.1, 4.2, 4.3)
    - Execute "git remote add claude https://github.com/anthropics/claude-code.git" (Requirements: 1.2)
    - Execute "git fetch claude" to retrieve latest changes (Requirements: 1.3)
    - Execute "git branch -f claude-main claude/main" to create tracking branch (Requirements: 1.4)
    - Execute "git checkout claude-main" to switch to Claude branch (Requirements: 1.5)
    - Execute "git subtree split --prefix=.devcontainer -b devcontainer claude-main" (Requirements: 1.6)
    - Execute "git checkout master" to return to main branch (Requirements: 1.7)
    - Execute "git subtree add --prefix=.devcontainer devcontainer --squash" (Requirements: 1.8)
    - Add progress reporting and success/error messaging
    - **Expected test results: 9/15 acceptance tests pass (init functionality complete, update/remove tests fail until implemented)**
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5, 1.6, 1.7, 1.8_

- [x] 8. Add comprehensive acceptance test coverage
  - [x] 8.1 Write additional acceptance tests for update and remove commands
    - Write tests for update command scenarios
    - Add tests for remove command functionality
    - Create tests for error handling and recovery scenarios
    - **Expected test results: 9/15 acceptance tests pass (6 update/remove tests fail until implemented)**
    - _Requirements: 3.1, 3.2, 5.1, 5.2_

- [x] 9. Implement remaining CLI commands
  - [x] 9.1 Implement update command with specific Git command sequence
    - Write command handler for updating existing devcontainer configurations
    - Execute "git fetch claude" to retrieve latest changes (Requirements: 3.1)
    - Execute "git checkout claude-main" and "git reset --hard claude/main" (Requirements: 3.2)
    - Execute "git subtree split --prefix=.devcontainer -b devcontainer-updated claude-main" (Requirements: 3.3)
    - Execute "git checkout master" and "git subtree pull --prefix=.devcontainer devcontainer-updated --squash" (Requirements: 3.4)
    - Add backup creation before updates (Requirements: 3.6)
    - Handle merge conflicts with clear error messages (Requirements: 3.5)
    - **Expected test results: 12/15 acceptance tests pass (update tests now passing, remove tests still fail)**
    - _Requirements: 3.1, 3.2, 3.3, 3.4, 3.5, 3.6_

  - [x] 9.2 Implement remove command with specific Git command sequence
    - Write command handler for cleaning up Claude Code integration
    - Execute "git remote remove claude" to remove Claude remote (Requirements: 5.1)
    - Execute "git branch -D claude-main" to delete tracking branch (Requirements: 5.2)
    - Execute "git branch -D devcontainer" and "git branch -D devcontainer-updated" (Requirements: 5.3)
    - Execute "rm -rf .devcontainer" with user confirmation (Requirements: 5.4)
    - Add confirmation prompts for destructive operations
    - Display confirmation of all cleaned resources (Requirements: 5.5)
    - **Expected test results: 15/15 acceptance tests pass (all functionality complete)**
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [x] 10. Finalize implementation and add advanced features
  - [x] 10.1 Add progress reporting and user experience features
    - Implement progress indicators for long-running operations (Requirements: 2.5)
    - Add verbose logging options for debugging (Requirements: 4.5)
    - Create clear success and error messaging (Requirements: 4.4)
    - Add confirmation prompts for overwriting existing devcontainer configurations (Requirements: 4.6)
    - Display summary of changes made when operations complete successfully (Requirements: 4.4)
    - _Requirements: 2.5, 4.4, 4.5, 4.6_

  - [x] 10.2 Performance optimization and final testing
    - Optimize git command execution for speed
    - Add timeout handling for network operations
    - Run full end-to-end testing suite
    - **Expected test results: All acceptance tests pass consistently**
    - _Requirements: 2.2_
