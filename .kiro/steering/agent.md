---
inclusion: always
---

# Devcontainer Sync CLI Project Guidelines

## Project Overview
This is a Rust CLI tool (`devcontainer-sync-cli`) that syncs devcontainer configurations from the Claude Code repository. The tool provides init, update, and remove operations for managing devcontainer setups.

## Code Style & Conventions

### Rust Standards
- Follow Rust 2021 edition conventions
- Use `thiserror` for structured error handling with suggestions
- Implement `clap` derive macros for CLI argument parsing
- Use `tokio` for async operations when needed
- Prefer explicit error types over generic errors

### Error Handling Pattern
- All errors should use the `CliError` enum with descriptive messages and actionable suggestions
- Include exit codes for different error categories (Repository=1, Network=2, GitOperation=3, FileSystem=4)
- Provide convenience constructors for common error scenarios

### CLI Design
- Use global flags for `--verbose` and `--dry-run` across all commands
- Structure commands as subcommands (init, update, remove) with specific options
- Always provide helpful descriptions and version information

## Architecture Patterns

### Module Organization
- `cli/` - CLI application logic and command handlers
- `git/` - Git operations (branch, remote, subtree, validator, executor)
- `config.rs` - Configuration management
- `error.rs` - Centralized error handling
- `types.rs` - Shared type definitions

### Key Types
- `CommandContext` - Context for command execution
- `OperationResult` - Standard result type for operations
- `GitCommand` - Git command abstraction

## Development Guidelines

### Testing
- Use `rstest` for parameterized tests
- Use `spectral` for fluent assertions
- Use `tempfile` for temporary file operations in tests
- Place acceptance tests in `tests/` directory

### Git Operations
- Validate git repository state before operations
- Use structured git command execution through the `git` module
- Handle git subtree operations for devcontainer syncing

## Implementation Notes
- The tool manages devcontainer configurations by syncing from a remote Claude Code repository
- Support backup creation before updates
- Provide force options for conflict resolution
- Allow cleanup operations while preserving files when requested

## Communication Style

### Direct Communication
- Be direct and concise in responses
- Avoid excessive praise, enthusiasm, or sycophantic language
- Focus on technical accuracy and practical solutions
- Don't use unnecessary superlatives or overly positive language
- State facts and provide solutions without emotional embellishment

### Professional Tone
- Maintain a professional, matter-of-fact tone
- Acknowledge issues and limitations honestly
- Provide clear, actionable feedback without sugar-coating
- Use technical language appropriately for the development context
