# Implementation Plan

- [ ] 1. Set up project structure and acceptance testing framework
  - Create Rust project with Cargo.toml including clap, thiserror, tempfile, rstest, spectral dependencies
  - Set up basic CLI structure with main.rs and lib.rs
  - Define project module structure (cli, git, error, config modules)
  - **Expected test results: 0/3 acceptance tests pass (binary doesn't exist yet)**
  - _Requirements: 2.1, 2.3_

- [ ] 2. Create acceptance test fixtures and write failing tests
  - [ ] 2.1 Create test fixtures and helpers
    - Implement temp_git_repo_with_commits fixture using tempfile
    - Create temp_git_repo_without_commits fixture for empty repositories
    - Add temp_non_git_dir fixture for non-git directories
    - Write compiled_binary fixture with binary compilation and caching
    - _Requirements: 4.1, 4.2_

  - [ ] 2.2 Implement command execution test helpers
    - Write run_command helper function for executing compiled binary
    - Create CommandResult struct for capturing exit codes and output
    - Add assertion helpers using spectral for readable test expectations
    - _Requirements: 4.1, 4.3_

  - [ ] 2.3 Write the three required acceptance tests
    - Implement test for non-git repository error case
    - Write test for git repository without commits error case
    - Create test for successful execution with valid git repository
    - **Expected test results: 0/3 acceptance tests pass (no CLI implementation yet)**
    - _Requirements: 4.1, 4.2_

- [ ] 3. Create minimal CLI structure to make first test pass
  - [ ] 3.1 Implement basic error handling and core types
    - Create CliError enum with thiserror derive
    - Define Repository, Network, GitOperation, and FileSystem error variants
    - Implement exit_code method for each error type
    - Add suggestion field to provide actionable error messages
    - _Requirements: 4.1, 4.4_

  - [ ] 3.2 Set up clap CLI framework with basic structure
    - Define main command structure with init, update, remove subcommands
    - Add global flags for verbose, dry-run, and help options
    - Implement command parsing and validation
    - **Expected test results: 0/3 acceptance tests pass (no repository validation yet)**
    - _Requirements: 2.1, 4.4_

- [ ] 4. Implement repository validation to make error tests pass
  - [ ] 4.1 Create repository validation functionality
    - Implement RepositoryValidator trait and struct
    - Write validate_git_repository method to check for .git directory
    - Implement validate_has_commits method to ensure repository has commits
    - Add check_existing_remote and check_existing_branch methods
    - **Expected test results: 2/3 acceptance tests pass (error cases work, success case still fails)**
    - _Requirements: 4.2, 1.1_

  - [ ] 4.2 Write unit tests for repository validation
    - Test validation with valid git repositories
    - Test error cases for non-git directories and empty repositories
    - _Requirements: 4.2_

- [ ] 5. Implement Git command execution layer
  - [ ] 5.1 Create GitExecutor trait and implementation
    - Write execute_git_command method with timeout handling
    - Implement command output parsing and error handling
    - Add working directory and environment variable support
    - _Requirements: 2.2, 2.4_

  - [ ] 5.2 Define core data structures
    - Implement CommandContext struct with working directory and options
    - Create OperationResult struct for command execution results
    - Define GitCommand wrapper for git process execution
    - _Requirements: 4.3_

- [ ] 6. Implement Git operation managers
  - [ ] 6.1 Implement RemoteManager for git remote operations
    - Write add_remote method to add Claude Code repository
    - Implement remove_remote method for cleanup operations
    - Add fetch_remote method with progress reporting
    - _Requirements: 1.1, 1.2, 5.1_

  - [ ] 6.2 Implement BranchManager for git branch operations
    - Write create_branch method for tracking branches
    - Implement delete_branch method for cleanup
    - Add checkout_branch method for branch switching
    - _Requirements: 1.3, 1.4, 5.2_

  - [ ] 6.3 Implement SubtreeManager for git subtree operations
    - Write split_subtree method to extract .devcontainer directory
    - Implement add_subtree method to integrate devcontainer files
    - Add update_subtree method for synchronization
    - _Requirements: 1.4, 1.5, 3.2_

- [ ] 7. Implement init command to make success test pass
  - [ ] 7.1 Add configuration and constants
    - Set Claude repository URL and remote name constants
    - Define branch names and directory paths
    - Add default timeout and configuration values
    - _Requirements: 1.1, 2.2_

  - [ ] 7.2 Implement init command
    - Write command handler that validates repository state
    - Execute git remote add, fetch, branch creation, and subtree operations
    - Add progress reporting and success/error messaging
    - **Expected test results: 3/3 acceptance tests pass (all core functionality working)**
    - _Requirements: 1.1, 1.2, 1.3, 1.4, 1.5_

- [ ] 8. Add comprehensive acceptance test coverage
  - [ ] 8.1 Write additional acceptance tests for update and remove commands
    - Write tests for update command scenarios
    - Add tests for remove command functionality
    - Create tests for error handling and recovery scenarios
    - **Expected test results: 3/6+ acceptance tests pass (new tests will fail initially)**
    - _Requirements: 3.1, 3.2, 5.1, 5.2_

- [ ] 9. Implement remaining CLI commands
  - [ ] 9.1 Implement update command
    - Write command handler for updating existing devcontainer configurations
    - Add backup creation before updates
    - Handle merge conflicts with clear error messages
    - **Expected test results: 4-5/6+ acceptance tests pass (update tests now passing)**
    - _Requirements: 3.1, 3.2, 3.3, 3.4_

  - [ ] 9.2 Implement remove command
    - Write command handler for cleaning up Claude Code integration
    - Remove remotes, branches, and optionally devcontainer directory
    - Add confirmation prompts for destructive operations
    - **Expected test results: 6+/6+ acceptance tests pass (all functionality complete)**
    - _Requirements: 5.1, 5.2, 5.3, 5.4, 5.5_

- [ ] 10. Finalize implementation and add advanced features
  - [ ] 10.1 Implement configuration management
    - Create config module for managing application settings
    - Add environment variable support for customization
    - Implement validation for configuration values
    - _Requirements: 2.3, 2.4_

  - [ ] 10.2 Add progress reporting and user experience features
    - Implement progress indicators for long-running operations
    - Add verbose logging options for debugging
    - Create clear success and error messaging
    - _Requirements: 2.5, 4.4, 4.5_

  - [ ] 10.3 Performance optimization and final testing
    - Optimize git command execution for speed
    - Add timeout handling for network operations
    - Run full end-to-end testing suite
    - **Expected test results: All acceptance tests pass consistently**
    - _Requirements: 2.2_
