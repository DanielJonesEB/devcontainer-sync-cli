use std::time::Duration;

pub const CLAUDE_REMOTE_NAME: &str = "claude";
pub const CLAUDE_REPO_URL: &str = "https://github.com/anthropics/claude-code.git";
pub const CLAUDE_BRANCH_NAME: &str = "claude-main";
pub const CLAUDE_REMOTE_BRANCH: &str = "claude/main";
pub const DEVCONTAINER_BRANCH: &str = "devcontainer";
pub const DEVCONTAINER_UPDATED_BRANCH: &str = "devcontainer-updated";
pub const DEVCONTAINER_PREFIX: &str = ".devcontainer";
pub const MASTER_BRANCH: &str = "master";
pub const DEFAULT_TIMEOUT_SECS: u64 = 30;

pub fn default_timeout() -> Duration {
    Duration::from_secs(DEFAULT_TIMEOUT_SECS)
}
