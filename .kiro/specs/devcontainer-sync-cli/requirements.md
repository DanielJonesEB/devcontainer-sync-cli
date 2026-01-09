# Requirements Document

## Introduction

A fast, portable CLI tool that automates the process of setting up and maintaining Git subtree tracking for Claude Code's devcontainer best practices. The main purpose of the tool is to run the equivalent of the following Git command sequence:

```bash
git remote add claude https://github.com/anthropics/claude-code.git
git fetch claude
git branch -f claude-main claude/main
git checkout claude-main
git subtree split --prefix=.devcontainer -b devcontainer claude-main
git checkout master
git subtree add --prefix=.devcontainer devcontainer --squash
```

The tool eliminates the need to manually execute these multiple Git commands by providing a single command interface that handles remote addition, fetching, branching, and subtree operations.

## Glossary

- **CLI Tool**: The command-line interface application being developed
- **Source Repository**: The Claude Code repository (https://github.com/anthropics/claude-code.git)
- **Target Repository**: The user's Git repository where devcontainer configurations will be integrated
- **Git Subtree**: A Git feature that allows embedding one repository inside another as a subdirectory
- **Devcontainer Configuration**: Files in the .devcontainer directory that define development environment settings
- **Firewall Features**: Components like iptables firewall configurations that provide container security but may not be desired in all development environments
- **Feature Stripping**: The process of selectively removing specific functionality from devcontainer configurations while preserving core development environment setup
- **Pattern-Based Detection**: A resilient approach to identifying firewall components using flexible patterns rather than hardcoded strings, allowing the tool to adapt to upstream changes

## Requirements

### Requirement 1

**User Story:** As a developer, I want to initialize devcontainer tracking from Claude Code repository, so that I can quickly set up best practice devcontainer configurations in my project.

#### Acceptance Criteria

1. WHEN the CLI tool is executed with an init command, THE CLI Tool SHALL validate that the Git repository has existing commits before proceeding
2. WHEN validation passes, THE CLI Tool SHALL execute "git remote add claude https://github.com/anthropics/claude-code.git"
3. WHEN the remote is added, THE CLI Tool SHALL execute "git fetch claude" to retrieve the latest changes
4. WHEN the fetch completes, THE CLI Tool SHALL execute "git branch -f claude-main claude/main" to create a tracking branch
5. WHEN the tracking branch is created, THE CLI Tool SHALL execute "git checkout claude-main" to switch to the Claude branch
6. WHEN on the Claude branch, THE CLI Tool SHALL execute "git subtree split --prefix=.devcontainer -b devcontainer claude-main" to extract the devcontainer directory
7. WHEN the subtree is split, THE CLI Tool SHALL execute "git checkout master" to return to the main branch
8. WHEN back on master, THE CLI Tool SHALL execute "git subtree add --prefix=.devcontainer devcontainer --squash" to integrate the devcontainer files

### Requirement 2

**User Story:** As a developer, I want the CLI tool to be portable and fast, so that I can use it across different environments without performance issues.

#### Acceptance Criteria

1. THE CLI Tool SHALL be implemented as a single executable binary
2. THE CLI Tool SHALL execute all Git operations within 30 seconds on a standard development machine
3. THE CLI Tool SHALL require only Git as an external dependency
4. THE CLI Tool SHALL provide clear progress indicators during execution

### Requirement 3

**User Story:** As a developer, I want to update my devcontainer configurations, so that I can stay current with Claude Code best practices.

#### Acceptance Criteria

1. WHEN the CLI tool is executed with an update command, THE CLI Tool SHALL execute "git fetch claude" to retrieve the latest changes
2. WHEN the fetch completes, THE CLI Tool SHALL execute "git checkout claude-main" and "git reset --hard claude/main" to update the tracking branch
3. WHEN the tracking branch is updated, THE CLI Tool SHALL execute "git subtree split --prefix=.devcontainer -b devcontainer-updated claude-main" to create an updated subtree
4. WHEN the updated subtree is ready, THE CLI Tool SHALL execute "git checkout master" and "git subtree pull --prefix=.devcontainer devcontainer-updated --squash" to merge updates
5. WHEN conflicts occur during update, THE CLI Tool SHALL provide clear error messages with resolution guidance
6. THE CLI Tool SHALL create a backup of existing configurations before updating

### Requirement 4

**User Story:** As a developer, I want clear feedback and error handling, so that I can understand what the tool is doing and troubleshoot issues.

#### Acceptance Criteria

1. WHEN any Git operation fails, THE CLI Tool SHALL display the specific error message and suggested resolution
2. WHEN the tool starts execution, THE CLI Tool SHALL validate that the current directory is a Git repository
3. WHEN the init command is executed, THE CLI Tool SHALL validate that the Git repository has existing commits before executing any Git operations
4. WHEN operations complete successfully, THE CLI Tool SHALL display a summary of changes made
5. THE CLI Tool SHALL provide verbose logging options for debugging purposes
6. IF the target repository already has devcontainer configurations, THEN THE CLI Tool SHALL prompt for confirmation before overwriting

### Requirement 5

**User Story:** As a developer, I want to customize devcontainer configurations during sync, so that I can exclude unwanted features like security policies that don't fit my development environment.

#### Acceptance Criteria

1. WHEN the CLI tool is executed with init or update commands and the --strip-firewall flag, THE CLI Tool SHALL remove iptables firewall configurations from devcontainer files
2. WHEN firewall features are stripped, THE CLI Tool SHALL remove or comment out iptables-related commands from Dockerfile and docker-compose files
3. WHEN firewall features are stripped, THE CLI Tool SHALL remove firewall-related environment variables and configuration sections
4. WHEN modifications are made to devcontainer files, THE CLI Tool SHALL create a git commit with a descriptive message indicating the customizations applied
5. WHEN the --strip-firewall flag is used, THE CLI Tool SHALL log all modifications made to devcontainer files for user review
6. THE CLI Tool SHALL preserve all other devcontainer functionality while removing only the specified firewall features
7. WHEN expected firewall patterns are not found in devcontainer files, THE CLI Tool SHALL continue processing other patterns and report warnings about missing patterns rather than failing completely

### Requirement 6

**User Story:** As a developer, I want to remove devcontainer tracking, so that I can clean up my repository if I no longer need the Claude Code configurations.

#### Acceptance Criteria

1. WHEN the CLI tool is executed with a remove command, THE CLI Tool SHALL execute "git remote remove claude" to remove the Claude remote
2. WHEN the remote is removed, THE CLI Tool SHALL execute "git branch -D claude-main" to delete the tracking branch
3. WHEN branches are cleaned up, THE CLI Tool SHALL execute "git branch -D devcontainer" and "git branch -D devcontainer-updated" to remove subtree branches
4. WHERE the user confirms removal, THE CLI Tool SHALL execute "rm -rf .devcontainer" to optionally remove the devcontainer directory
5. WHEN removal completes, THE CLI Tool SHALL display a confirmation of all cleaned resources
