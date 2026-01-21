use crate::config::*;
use crate::customizer::{DefaultDevcontainerCustomizer, DevcontainerCustomizer};
use crate::error::CliError;
use crate::git::{
    BranchManager, GitBranchManager, GitExecutor, GitRemoteManager, GitRepositoryValidator,
    GitSubtreeManager, RemoteManager, RepositoryValidator, SubtreeManager, SystemGitExecutor,
};
use crate::types::CommandContext;
use std::env;

pub struct CliApp {
    context: CommandContext,
}

impl CliApp {
    pub fn new(verbose: bool) -> Self {
        let working_dir = env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let context = CommandContext::new(working_dir, verbose);

        Self { context }
    }

    pub fn init(&self, strip_firewall: bool) -> Result<(), CliError> {
        // Update context with strip_firewall flag
        let context = self.context.clone().with_strip_firewall(strip_firewall);

        if context.verbose {
            println!("Initializing devcontainer sync from Claude Code repository...");
            if strip_firewall {
                println!("Firewall stripping enabled - will remove firewall configurations");
            }
        }

        // Validate that we're in a git repository
        let validator = GitRepositoryValidator::new(context.working_dir.clone());
        validator.validate_git_repository(&context.working_dir)?;

        // Validate that the repository has commits
        validator.validate_has_commits()?;

        // Check if .devcontainer already exists and prompt for confirmation
        let devcontainer_path = context.working_dir.join(DEVCONTAINER_PREFIX);
        if devcontainer_path.exists() {
            println!("Warning: .devcontainer directory already exists.");
            println!("This will overwrite existing devcontainer configurations.");
            print!("Continue? (y/N): ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin()
                .read_line(&mut input)
                .map_err(|e| CliError::FileSystem {
                    message: format!("Failed to read user input: {}", e),
                    suggestion: "Try running the command again".to_string(),
                })?;

            let input = input.trim().to_lowercase();
            if input != "y" && input != "yes" {
                return Err(CliError::Repository {
                    message: "Operation cancelled by user".to_string(),
                    suggestion:
                        "Use --force flag to skip confirmation or backup existing files first"
                            .to_string(),
                });
            }
        }

        // Create Git operation managers
        let executor = SystemGitExecutor::new();
        let remote_manager = GitRemoteManager::new(executor, self.context.working_dir.clone());
        let executor = SystemGitExecutor::new();
        let branch_manager = GitBranchManager::new(executor, self.context.working_dir.clone());
        let executor = SystemGitExecutor::new();
        let subtree_manager = GitSubtreeManager::new(executor, self.context.working_dir.clone());

        // Execute the Git command sequence

        // 1. git remote add claude https://github.com/anthropics/claude-code.git
        if context.verbose {
            println!("Adding Claude Code remote...");
        } else {
            print!("Adding remote... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        remote_manager.add_remote(CLAUDE_REMOTE_NAME, CLAUDE_REPO_URL)?;
        if !context.verbose {
            println!("✓");
        }

        // 2. git fetch claude
        if context.verbose {
            println!("Fetching from Claude Code repository...");
        } else {
            print!("Fetching repository... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        remote_manager.fetch_remote(CLAUDE_REMOTE_NAME)?;
        if !context.verbose {
            println!("✓");
        }

        // 3. git branch -f claude-main claude/main
        if context.verbose {
            println!("Creating tracking branch...");
        } else {
            print!("Creating branch... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        branch_manager.force_create_branch(CLAUDE_BRANCH_NAME, CLAUDE_REMOTE_BRANCH)?;
        if !context.verbose {
            println!("✓");
        }

        // 4. git checkout claude-main
        if context.verbose {
            println!("Switching to Claude branch...");
        } else {
            print!("Switching branches... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        branch_manager.checkout_branch(CLAUDE_BRANCH_NAME)?;
        if !context.verbose {
            println!("✓");
        }

        // 5. git subtree split --prefix=.devcontainer -b devcontainer claude-main
        if context.verbose {
            println!("Extracting devcontainer subtree...");
        } else {
            print!("Extracting devcontainer... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        subtree_manager.split_subtree(DEVCONTAINER_PREFIX, DEVCONTAINER_BRANCH)?;
        if !context.verbose {
            println!("✓");
        }

        // 6. git checkout master
        if context.verbose {
            println!("Returning to master branch...");
        } else {
            print!("Returning to master... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        branch_manager.checkout_branch(MASTER_BRANCH)?;
        if !context.verbose {
            println!("✓");
        }

        // 7. git subtree add --prefix=.devcontainer devcontainer --squash
        if context.verbose {
            println!("Adding devcontainer files...");
        } else {
            print!("Adding devcontainer files... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        subtree_manager.add_subtree(DEVCONTAINER_PREFIX, DEVCONTAINER_BRANCH, true)?;
        if !context.verbose {
            println!("✓");
        }

        // Apply firewall stripping if requested
        if context.strip_firewall {
            if context.verbose {
                println!("Stripping firewall configurations...");
            } else {
                print!("Stripping firewall... ");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }

            let customizer =
                DefaultDevcontainerCustomizer::new(context.working_dir.clone(), context.verbose);
            let devcontainer_path = context.working_dir.join(DEVCONTAINER_PREFIX);

            match customizer.strip_firewall_features(&devcontainer_path) {
                Ok(result) => {
                    if result.has_changes() {
                        // Create a commit for the firewall customizations
                        let commit_message = "Strip firewall configurations from devcontainer";
                        let changes: Vec<String> = result
                            .dockerfile_changes
                            .iter()
                            .chain(result.json_changes.iter())
                            .cloned()
                            .collect();

                        if let Err(e) = customizer.commit_customizations(&changes, commit_message) {
                            if context.verbose {
                                println!(
                                    "Warning: Failed to commit firewall customizations: {}",
                                    e
                                );
                            }
                        }

                        if context.verbose {
                            println!("Firewall stripping completed:");
                            for change in &result.dockerfile_changes {
                                println!("  - Dockerfile: {}", change);
                            }
                            for change in &result.json_changes {
                                println!("  - devcontainer.json: {}", change);
                            }
                            if result.has_warnings() {
                                println!("Warnings:");
                                for warning in &result.warnings {
                                    println!("  ⚠️  {}", warning);
                                }
                            }
                        }
                    } else if context.verbose {
                        println!("No firewall configurations found to strip");
                    }
                }
                Err(e) => {
                    if context.verbose {
                        println!("Warning: Firewall stripping failed: {}", e);
                    } else {
                        println!("⚠️");
                    }
                }
            }

            if !context.verbose {
                println!("✓");
            }
        }

        // Display summary of changes
        println!("\n✅ Successfully initialized devcontainer sync!");
        println!("📁 Created .devcontainer directory with Claude Code configurations");
        if context.strip_firewall {
            println!("🔒 Stripped firewall configurations as requested");
        }
        println!(
            "🔗 Added 'claude' remote pointing to https://github.com/anthropics/claude-code.git"
        );
        println!("🌿 Created tracking branch 'claude-main' for future updates");
        println!("\nNext steps:");
        println!("  • Run 'devcontainer-sync update' to get the latest configurations");
        println!("  • Run 'devcontainer-sync remove' to clean up if no longer needed");
        Ok(())
    }

    pub fn update(&self, backup: bool, _force: bool, strip_firewall: bool) -> Result<(), CliError> {
        // Update context with strip_firewall flag
        let context = self.context.clone().with_strip_firewall(strip_firewall);

        if context.verbose {
            println!("Updating devcontainer configurations...");
            if strip_firewall {
                println!("Firewall stripping enabled - will remove firewall configurations");
            }
        }

        // Validate that we're in a git repository
        let validator = GitRepositoryValidator::new(context.working_dir.clone());
        validator.validate_git_repository(&context.working_dir)?;

        // Create Git operation managers
        let executor = SystemGitExecutor::new();
        let remote_manager = GitRemoteManager::new(executor, context.working_dir.clone());
        let executor = SystemGitExecutor::new();
        let branch_manager = GitBranchManager::new(executor, context.working_dir.clone());
        let executor = SystemGitExecutor::new();
        let subtree_manager = GitSubtreeManager::new(executor, context.working_dir.clone());

        // Create backup if requested
        if backup {
            if context.verbose {
                println!("Creating backup of existing devcontainer configuration...");
            } else {
                print!("Creating backup... ");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }
            self.create_backup()?;
            if !context.verbose {
                println!("✓");
            }
        }

        // Execute the Git command sequence for update

        // 1. git fetch claude
        if context.verbose {
            println!("Fetching from Claude Code repository...");
        } else {
            print!("Fetching updates... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        remote_manager.fetch_remote(CLAUDE_REMOTE_NAME)?;
        if !context.verbose {
            println!("✓");
        }

        // 2. git checkout claude-main && git reset --hard claude/main
        if context.verbose {
            println!("Updating tracking branch...");
        } else {
            print!("Updating tracking branch... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        branch_manager.checkout_branch(CLAUDE_BRANCH_NAME)?;

        // Reset to latest remote state
        let executor = SystemGitExecutor::new();
        executor.execute_git_command(
            &["reset", "--hard", CLAUDE_REMOTE_BRANCH],
            &context.working_dir,
        )?;
        if !context.verbose {
            println!("✓");
        }

        // 3. git subtree split --prefix=.devcontainer -b devcontainer-updated claude-main
        if context.verbose {
            println!("Extracting updated devcontainer subtree...");
        } else {
            print!("Extracting updates... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        subtree_manager.split_subtree(DEVCONTAINER_PREFIX, DEVCONTAINER_UPDATED_BRANCH)?;
        if !context.verbose {
            println!("✓");
        }

        // 4. git checkout master && git subtree pull --prefix=.devcontainer devcontainer-updated --squash
        if context.verbose {
            println!("Returning to master branch...");
        } else {
            print!("Returning to master... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        branch_manager.checkout_branch(MASTER_BRANCH)?;
        if !context.verbose {
            println!("✓");
        }

        if context.verbose {
            println!("Updating devcontainer files...");
        } else {
            print!("Applying updates... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        // Use git subtree merge to update the existing subtree
        let executor = SystemGitExecutor::new();
        executor.execute_git_command(
            &[
                "subtree",
                "merge",
                "--prefix=.devcontainer",
                "--squash",
                DEVCONTAINER_UPDATED_BRANCH,
            ],
            &context.working_dir,
        )?;
        if !context.verbose {
            println!("✓");
        }

        // Apply firewall stripping if requested
        if context.strip_firewall {
            if context.verbose {
                println!("Stripping firewall configurations...");
            } else {
                print!("Stripping firewall... ");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }

            let customizer =
                DefaultDevcontainerCustomizer::new(context.working_dir.clone(), context.verbose);
            let devcontainer_path = context.working_dir.join(DEVCONTAINER_PREFIX);

            match customizer.strip_firewall_features(&devcontainer_path) {
                Ok(result) => {
                    if result.has_changes() {
                        // Create a commit for the firewall customizations
                        let commit_message =
                            "Strip firewall configurations from updated devcontainer";
                        let changes: Vec<String> = result
                            .dockerfile_changes
                            .iter()
                            .chain(result.json_changes.iter())
                            .cloned()
                            .collect();

                        if let Err(e) = customizer.commit_customizations(&changes, commit_message) {
                            if context.verbose {
                                println!(
                                    "Warning: Failed to commit firewall customizations: {}",
                                    e
                                );
                            }
                        }

                        if context.verbose {
                            println!("Firewall stripping completed:");
                            for change in &result.dockerfile_changes {
                                println!("  - Dockerfile: {}", change);
                            }
                            for change in &result.json_changes {
                                println!("  - devcontainer.json: {}", change);
                            }
                            if result.has_warnings() {
                                println!("Warnings:");
                                for warning in &result.warnings {
                                    println!("  ⚠️  {}", warning);
                                }
                            }
                        }
                    } else if context.verbose {
                        println!("No firewall configurations found to strip");
                    }
                }
                Err(e) => {
                    if context.verbose {
                        println!("Warning: Firewall stripping failed: {}", e);
                    } else {
                        println!("⚠️");
                    }
                }
            }

            if !context.verbose {
                println!("✓");
            }
        }

        // Display summary of changes
        println!("\n✅ Successfully updated devcontainer configurations!");
        println!("📁 Updated .devcontainer directory with latest Claude Code configurations");
        if context.strip_firewall {
            println!("🔒 Stripped firewall configurations as requested");
        }
        if backup {
            println!("💾 Backup created before update");
        }
        println!("🔄 Merged latest changes from Claude Code repository");
        println!("\nYour devcontainer is now up to date with the latest configurations.");
        Ok(())
    }

    pub fn remove(&self, keep_files: bool) -> Result<(), CliError> {
        if self.context.verbose {
            println!("Removing devcontainer sync...");
        }

        // Validate that we're in a git repository
        let validator = GitRepositoryValidator::new(self.context.working_dir.clone());
        validator.validate_git_repository(&self.context.working_dir)?;

        // Create Git operation managers
        let executor = SystemGitExecutor::new();
        let remote_manager = GitRemoteManager::new(executor, self.context.working_dir.clone());
        let executor = SystemGitExecutor::new();
        let branch_manager = GitBranchManager::new(executor, self.context.working_dir.clone());
        let executor = SystemGitExecutor::new();
        let subtree_manager = GitSubtreeManager::new(executor, self.context.working_dir.clone());

        // Execute the Git command sequence for remove

        // 1. git remote remove claude
        if self.context.verbose {
            println!("Removing Claude remote...");
        } else {
            print!("Removing remote... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        remote_manager.remove_remote(CLAUDE_REMOTE_NAME)?;
        if !self.context.verbose {
            println!("✓");
        }

        // 2. git branch -D claude-main
        if self.context.verbose {
            println!("Deleting tracking branch...");
        } else {
            print!("Removing branches... ");
            use std::io::{self, Write};
            io::stdout().flush().unwrap();
        }
        branch_manager.delete_branch(CLAUDE_BRANCH_NAME)?;

        // 3. git branch -D devcontainer && git branch -D devcontainer-updated
        if self.context.verbose {
            println!("Cleaning up subtree branches...");
        }
        // These branches might not exist, so we ignore errors
        let _ = branch_manager.delete_branch(DEVCONTAINER_BRANCH);
        let _ = branch_manager.delete_branch(DEVCONTAINER_UPDATED_BRANCH);
        if !self.context.verbose {
            println!("✓");
        }

        // 4. Remove .devcontainer directory if not keeping files
        if !keep_files {
            if self.context.verbose {
                println!("Removing devcontainer directory...");
            } else {
                print!("Removing files... ");
                use std::io::{self, Write};
                io::stdout().flush().unwrap();
            }
            subtree_manager.remove_subtree(DEVCONTAINER_PREFIX)?;

            // Commit the removal
            let executor = SystemGitExecutor::new();
            executor.execute_git_command(
                &["commit", "-m", "Remove devcontainer configuration"],
                &self.context.working_dir,
            )?;
            if !self.context.verbose {
                println!("✓");
            }
        }

        // Display summary of changes
        println!("\n✅ Successfully removed devcontainer sync!");
        println!("🔗 Removed 'claude' remote");
        println!("🌿 Deleted tracking branches");
        if !keep_files {
            println!("📁 Removed .devcontainer directory and files");
            println!("💾 Changes committed to git history");
        } else {
            println!("📁 Kept .devcontainer files (--keep-files specified)");
        }
        println!("\nDevcontainer sync has been completely removed from this repository.");
        Ok(())
    }

    fn create_backup(&self) -> Result<(), CliError> {
        let devcontainer_path = self.context.working_dir.join(DEVCONTAINER_PREFIX);
        let backup_path = self
            .context
            .working_dir
            .join(format!("{}.backup", DEVCONTAINER_PREFIX));

        // Check if .devcontainer exists
        if !devcontainer_path.exists() {
            return Err(CliError::FileSystem {
                message: "No .devcontainer directory found to backup".to_string(),
                suggestion:
                    "Run 'devcontainer-sync init' first to create devcontainer configuration"
                        .to_string(),
            });
        }

        // Remove existing backup if it exists
        if backup_path.exists() {
            std::fs::remove_dir_all(&backup_path).map_err(|e| CliError::FileSystem {
                message: format!("Failed to remove existing backup directory: {}", e),
                suggestion: "Check file permissions and try again".to_string(),
            })?;
        }

        // Copy .devcontainer to .devcontainer.backup
        Self::copy_directory(&devcontainer_path, &backup_path)?;

        if self.context.verbose {
            println!("Backup created at: {}", backup_path.display());
        }

        Ok(())
    }

    fn copy_directory(src: &std::path::Path, dst: &std::path::Path) -> Result<(), CliError> {
        std::fs::create_dir_all(dst).map_err(|e| CliError::FileSystem {
            message: format!("Failed to create backup directory: {}", e),
            suggestion: "Check file permissions and available disk space".to_string(),
        })?;

        for entry in std::fs::read_dir(src).map_err(|e| CliError::FileSystem {
            message: format!("Failed to read .devcontainer directory: {}", e),
            suggestion: "Check file permissions".to_string(),
        })? {
            let entry = entry.map_err(|e| CliError::FileSystem {
                message: format!("Failed to read directory entry: {}", e),
                suggestion: "Check file permissions".to_string(),
            })?;

            let src_path = entry.path();
            let dst_path = dst.join(entry.file_name());

            if src_path.is_dir() {
                Self::copy_directory(&src_path, &dst_path)?;
            } else {
                std::fs::copy(&src_path, &dst_path).map_err(|e| CliError::FileSystem {
                    message: format!("Failed to copy file {}: {}", src_path.display(), e),
                    suggestion: "Check file permissions and available disk space".to_string(),
                })?;
            }
        }

        Ok(())
    }
}
