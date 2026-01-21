use crate::error::CliError;
use regex::Regex;
use std::path::{Path, PathBuf};

/// Trait for customizing devcontainer configurations
pub trait DevcontainerCustomizer {
    /// Strip firewall features from devcontainer directory
    fn strip_firewall_features(
        &self,
        devcontainer_path: &Path,
    ) -> Result<FirewallRemovalResult, CliError>;

    /// Detect firewall scripts using flexible pattern matching
    fn detect_firewall_scripts(&self, devcontainer_path: &Path) -> Result<Vec<PathBuf>, CliError>;

    /// Strip firewall configurations from devcontainer.json
    fn strip_devcontainer_json_firewall(&self, json_path: &Path) -> Result<Vec<String>, CliError>;

    /// Strip firewall configurations from Dockerfile
    fn strip_dockerfile_firewall(&self, dockerfile_path: &Path) -> Result<Vec<String>, CliError>;

    /// Validate firewall removal results and generate warnings
    fn validate_firewall_removal(&self, removal_result: &FirewallRemovalResult) -> Vec<String>;

    /// Commit customizations to git with descriptive message
    fn commit_customizations(&self, changes: &[String], message: &str) -> Result<(), CliError>;
}

/// Result of firewall removal operation
#[derive(Debug, Clone, Default)]
pub struct FirewallRemovalResult {
    pub files_modified: Vec<PathBuf>,
    pub files_removed: Vec<PathBuf>,
    pub dockerfile_changes: Vec<String>,
    pub json_changes: Vec<String>,
    pub warnings: Vec<String>,
    pub patterns_not_found: Vec<String>,
}

impl FirewallRemovalResult {
    pub fn new() -> Self {
        Self {
            files_modified: Vec::new(),
            files_removed: Vec::new(),
            dockerfile_changes: Vec::new(),
            json_changes: Vec::new(),
            warnings: Vec::new(),
            patterns_not_found: Vec::new(),
        }
    }

    pub fn add_modified_file(&mut self, path: PathBuf) {
        self.files_modified.push(path);
    }

    pub fn add_removed_file(&mut self, path: PathBuf) {
        self.files_removed.push(path);
    }

    pub fn add_dockerfile_change(&mut self, change: String) {
        self.dockerfile_changes.push(change);
    }

    pub fn add_json_change(&mut self, change: String) {
        self.json_changes.push(change);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_pattern_not_found(&mut self, pattern: String) {
        self.patterns_not_found.push(pattern);
    }

    pub fn has_changes(&self) -> bool {
        !self.files_modified.is_empty() || !self.files_removed.is_empty()
    }

    pub fn has_warnings(&self) -> bool {
        !self.warnings.is_empty() || !self.patterns_not_found.is_empty()
    }
}

/// Default implementation of DevcontainerCustomizer
///
/// This implementation uses pattern-based detection to identify and remove firewall
/// configurations from devcontainer files. The pattern-based approach makes it resilient
/// to upstream changes in the Claude Code devcontainer implementation.
///
/// ## Pattern-Based Detection Strategy
///
/// Instead of hardcoded strings, this implementation uses regex patterns to detect:
/// - Firewall packages in Dockerfile apt install commands
/// - Firewall capabilities in devcontainer.json runArgs
/// - Firewall scripts and setup sections
///
/// This approach allows the tool to continue working even when the upstream
/// implementation changes, and provides graceful degradation with warning messages
/// when expected patterns aren't found.
///
/// ## Future Maintenance
///
/// If upstream changes break the pattern detection, the patterns can be updated
/// in the constants section without changing the core logic. This design anticipates
/// future AI-assisted maintenance where patterns could be automatically updated
/// based on upstream changes.
pub struct DefaultDevcontainerCustomizer {
    working_dir: PathBuf,
    verbose: bool,
}

impl DefaultDevcontainerCustomizer {
    pub fn new(working_dir: PathBuf, verbose: bool) -> Self {
        Self {
            working_dir,
            verbose,
        }
    }

    /// Create regex patterns for firewall detection
    ///
    /// These patterns are designed to be flexible and resilient to upstream changes.
    /// They use regex syntax to match variations in formatting and structure.
    ///
    /// If upstream changes break detection, these patterns can be updated without
    /// changing the core logic, making maintenance easier.
    fn create_firewall_patterns() -> Result<Vec<Regex>, CliError> {
        let patterns = [
            r"iptables\s*\\?",
            r"ipset\s*\\?",
            r"iproute2\s*\\?",
            r"dnsutils\s*\\?",
            r"aggregate\s*\\?",
            r"--cap-add=NET_ADMIN",
            r"--cap-add=NET_RAW",
            r"init-firewall\.sh",
            r"firewall.*\.sh",
            r"postStartCommand.*firewall",
            r"waitFor.*postStartCommand",
        ];

        patterns
            .iter()
            .map(|pattern| {
                Regex::new(pattern).map_err(|e| CliError::Repository {
                    message: format!("Invalid regex pattern '{}': {}", pattern, e),
                    suggestion: "This is a bug in the firewall pattern configuration".to_string(),
                })
            })
            .collect()
    }

    /// Check if content matches any firewall patterns
    fn matches_firewall_patterns(&self, content: &str) -> Result<Vec<String>, CliError> {
        let patterns = Self::create_firewall_patterns()?;
        let mut matches = Vec::new();

        for pattern in patterns {
            if let Some(mat) = pattern.find(content) {
                matches.push(mat.as_str().to_string());
            }
        }

        Ok(matches)
    }

    /// Log operation if verbose mode is enabled
    fn log_verbose(&self, message: &str) {
        if self.verbose {
            println!("🔧 {}", message);
        }
    }
}

impl DevcontainerCustomizer for DefaultDevcontainerCustomizer {
    fn strip_firewall_features(
        &self,
        devcontainer_path: &Path,
    ) -> Result<FirewallRemovalResult, CliError> {
        let mut result = FirewallRemovalResult::new();

        self.log_verbose("Starting firewall feature stripping...");

        // Detect and remove firewall scripts
        let scripts = self.detect_firewall_scripts(devcontainer_path)?;
        for script in scripts {
            if script.exists() {
                std::fs::remove_file(&script).map_err(|e| CliError::FileSystem {
                    message: format!(
                        "Failed to remove firewall script {}: {}",
                        script.display(),
                        e
                    ),
                    suggestion: "Check file permissions and try again".to_string(),
                })?;
                result.add_removed_file(script.clone());
                self.log_verbose(&format!("Removed firewall script: {}", script.display()));
            }
        }

        // Strip devcontainer.json firewall configurations
        let json_path = devcontainer_path.join("devcontainer.json");
        if json_path.exists() {
            let changes = self.strip_devcontainer_json_firewall(&json_path)?;
            if !changes.is_empty() {
                result.add_modified_file(json_path);
                for change in changes {
                    result.add_json_change(change);
                }
            }
        } else {
            result.add_warning("devcontainer.json not found".to_string());
        }

        // Strip Dockerfile firewall configurations
        let dockerfile_path = devcontainer_path.join("Dockerfile");
        if dockerfile_path.exists() {
            let changes = self.strip_dockerfile_firewall(&dockerfile_path)?;
            if !changes.is_empty() {
                result.add_modified_file(dockerfile_path);
                for change in changes {
                    result.add_dockerfile_change(change);
                }
            }
        } else {
            result.add_warning("Dockerfile not found".to_string());
        }

        // Validate results
        let validation_warnings = self.validate_firewall_removal(&result);
        for warning in validation_warnings {
            result.add_warning(warning);
        }

        self.log_verbose(&format!(
            "Firewall stripping complete: {} files modified, {} files removed, {} warnings",
            result.files_modified.len(),
            result.files_removed.len(),
            result.warnings.len()
        ));

        Ok(result)
    }

    fn detect_firewall_scripts(&self, devcontainer_path: &Path) -> Result<Vec<PathBuf>, CliError> {
        let mut scripts = Vec::new();

        // Check for common firewall script names
        let script_patterns = ["init-firewall.sh", "firewall.sh", "iptables.sh"];

        for pattern in script_patterns {
            let script_path = devcontainer_path.join(pattern);
            if script_path.exists() {
                scripts.push(script_path);
            }
        }

        // Also check for any .sh files that contain firewall-related content
        // but avoid duplicates from the name-based detection above
        if let Ok(entries) = std::fs::read_dir(devcontainer_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("sh")
                    && !scripts.contains(&path)
                {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        let matches = self.matches_firewall_patterns(&content)?;
                        if !matches.is_empty() {
                            scripts.push(path);
                        }
                    }
                }
            }
        }

        Ok(scripts)
    }

    fn strip_devcontainer_json_firewall(&self, json_path: &Path) -> Result<Vec<String>, CliError> {
        let content = std::fs::read_to_string(json_path).map_err(|e| CliError::FileSystem {
            message: format!("Failed to read devcontainer.json: {}", e),
            suggestion: "Check file permissions and ensure the file exists".to_string(),
        })?;

        let mut json: serde_json::Value =
            serde_json::from_str(&content).map_err(|e| CliError::Repository {
                message: format!("Invalid JSON in devcontainer.json: {}", e),
                suggestion: "Fix JSON syntax errors in devcontainer.json".to_string(),
            })?;

        let mut changes = Vec::new();

        // Remove firewall capabilities from runArgs
        if let Some(run_args) = json.get_mut("runArgs").and_then(|v| v.as_array_mut()) {
            let original_len = run_args.len();
            run_args.retain(|arg| {
                if let Some(arg_str) = arg.as_str() {
                    !arg_str.contains("--cap-add=NET_ADMIN")
                        && !arg_str.contains("--cap-add=NET_RAW")
                } else {
                    true
                }
            });
            if run_args.len() < original_len {
                changes.push("Removed NET_ADMIN and NET_RAW capabilities from runArgs".to_string());
            }
        }

        // Remove postStartCommand if it references firewall
        if let Some(post_start) = json.get("postStartCommand").and_then(|v| v.as_str()) {
            if post_start.contains("firewall") {
                json.as_object_mut().unwrap().remove("postStartCommand");
                changes.push("Removed postStartCommand referencing firewall".to_string());
            }
        }

        // Remove waitFor if it references postStartCommand
        if let Some(wait_for) = json.get("waitFor").and_then(|v| v.as_str()) {
            if wait_for == "postStartCommand" && json.get("postStartCommand").is_none() {
                json.as_object_mut().unwrap().remove("waitFor");
                changes.push("Removed waitFor since postStartCommand was removed".to_string());
            }
        }

        // Write back the modified JSON if there were changes
        if !changes.is_empty() {
            let modified_content =
                serde_json::to_string_pretty(&json).map_err(|e| CliError::Repository {
                    message: format!("Failed to serialize modified JSON: {}", e),
                    suggestion: "This is likely a bug in the JSON modification logic".to_string(),
                })?;

            std::fs::write(json_path, modified_content).map_err(|e| CliError::FileSystem {
                message: format!("Failed to write modified devcontainer.json: {}", e),
                suggestion: "Check file permissions and available disk space".to_string(),
            })?;

            self.log_verbose(&format!(
                "Modified devcontainer.json: {}",
                changes.join(", ")
            ));
        }

        Ok(changes)
    }

    fn strip_dockerfile_firewall(&self, dockerfile_path: &Path) -> Result<Vec<String>, CliError> {
        let content =
            std::fs::read_to_string(dockerfile_path).map_err(|e| CliError::FileSystem {
                message: format!("Failed to read Dockerfile: {}", e),
                suggestion: "Check file permissions and ensure the file exists".to_string(),
            })?;

        let lines: Vec<&str> = content.lines().collect();
        let mut modified_lines = Vec::new();
        let mut changes = Vec::new();
        let mut in_firewall_section = false;
        let mut in_apt_install = false;

        for line in lines {
            let mut skip_line = false;

            // Check if we're entering a firewall section
            if line.contains("# Copy and set up firewall script") {
                in_firewall_section = true;
                skip_line = true;
                changes.push("Removed firewall setup section".to_string());
            }

            // Check if we're exiting a firewall section (when we see USER node after firewall setup)
            if in_firewall_section && line.trim() == "USER node" {
                in_firewall_section = false;
                skip_line = true;
            }

            // Skip lines in firewall section
            if in_firewall_section {
                skip_line = true;
            }

            // Handle apt install commands (which can be multi-line)
            if line.contains("apt-get install") || line.contains("apt install") {
                in_apt_install = true;
            }

            if in_apt_install {
                let firewall_packages = ["iptables", "ipset", "iproute2", "dnsutils", "aggregate"];
                let mut modified_line = line.to_string();
                let mut package_removed = false;

                for package in firewall_packages {
                    if modified_line.contains(package) {
                        // Remove the package and any trailing backslash/whitespace
                        modified_line = modified_line.replace(&format!("  {} \\", package), "");
                        modified_line = modified_line.replace(&format!("  {}", package), "");
                        modified_line = modified_line.replace(&format!(" {} \\", package), "");
                        modified_line = modified_line.replace(&format!(" {}", package), "");
                        package_removed = true;
                    }
                }

                if package_removed && !changes.iter().any(|c| c.contains("firewall packages")) {
                    changes.push("Removed firewall packages from apt install".to_string());
                }

                // Check if this line ends the apt install command
                if !line.ends_with('\\') {
                    in_apt_install = false;
                }

                if !skip_line {
                    modified_lines.push(modified_line);
                }
            } else if !skip_line {
                modified_lines.push(line.to_string());
            }
        }

        // Write back the modified Dockerfile if there were changes
        if !changes.is_empty() {
            let modified_content = modified_lines.join("\n");
            std::fs::write(dockerfile_path, modified_content).map_err(|e| {
                CliError::FileSystem {
                    message: format!("Failed to write modified Dockerfile: {}", e),
                    suggestion: "Check file permissions and available disk space".to_string(),
                }
            })?;

            self.log_verbose(&format!("Modified Dockerfile: {}", changes.join(", ")));
        }

        Ok(changes)
    }

    fn validate_firewall_removal(&self, removal_result: &FirewallRemovalResult) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check if we expected to find certain files but didn't
        if removal_result.files_removed.is_empty() {
            warnings.push("No firewall scripts were found to remove".to_string());
        }

        if removal_result.dockerfile_changes.is_empty() {
            warnings.push("No firewall configurations found in Dockerfile".to_string());
        }

        if removal_result.json_changes.is_empty() {
            warnings.push("No firewall configurations found in devcontainer.json".to_string());
        }

        // This is expected behavior - we want to warn when patterns aren't found
        // so users know what wasn't stripped
        warnings
    }

    fn commit_customizations(&self, changes: &[String], message: &str) -> Result<(), CliError> {
        use crate::git::{GitExecutor, SystemGitExecutor};

        let executor = SystemGitExecutor::new();

        // Add all modified files to git
        executor.execute_git_command(&["add", ".devcontainer"], &self.working_dir)?;

        // Create commit with detailed message
        let full_message = if changes.is_empty() {
            message.to_string()
        } else {
            format!("{}\n\nChanges made:\n{}", message, changes.join("\n- "))
        };

        executor.execute_git_command(&["commit", "-m", &full_message], &self.working_dir)?;

        self.log_verbose("Committed firewall customizations to git");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_firewall_scripts() {
        let temp_dir = TempDir::new().unwrap();
        let devcontainer_path = temp_dir.path();

        // Create a firewall script
        let script_path = devcontainer_path.join("init-firewall.sh");
        fs::write(&script_path, "#!/bin/bash\niptables -F\n").unwrap();

        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let scripts = customizer
            .detect_firewall_scripts(devcontainer_path)
            .unwrap();

        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0], script_path);
    }

    #[test]
    fn test_detect_firewall_scripts_by_content() {
        let temp_dir = TempDir::new().unwrap();
        let devcontainer_path = temp_dir.path();

        // Create a script with firewall content but different name
        let script_path = devcontainer_path.join("setup.sh");
        fs::write(
            &script_path,
            "#!/bin/bash\necho 'Setting up iptables'\niptables -A INPUT -j DROP\n",
        )
        .unwrap();

        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let scripts = customizer
            .detect_firewall_scripts(devcontainer_path)
            .unwrap();

        assert_eq!(scripts.len(), 1);
        assert_eq!(scripts[0], script_path);
    }

    #[test]
    fn test_no_firewall_scripts() {
        let temp_dir = TempDir::new().unwrap();
        let devcontainer_path = temp_dir.path();

        // Create a non-firewall script
        let script_path = devcontainer_path.join("setup.sh");
        fs::write(&script_path, "#!/bin/bash\necho 'Hello world'\n").unwrap();

        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let scripts = customizer
            .detect_firewall_scripts(devcontainer_path)
            .unwrap();

        assert_eq!(scripts.len(), 0);
    }

    #[test]
    fn test_firewall_removal_result() {
        let mut result = FirewallRemovalResult::new();

        result.add_modified_file(PathBuf::from("test.json"));
        result.add_removed_file(PathBuf::from("firewall.sh"));
        result.add_warning("Test warning".to_string());

        assert!(result.has_changes());
        assert!(result.has_warnings());
        assert_eq!(result.files_modified.len(), 1);
        assert_eq!(result.files_removed.len(), 1);
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_strip_devcontainer_json_firewall() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("devcontainer.json");

        // Create a devcontainer.json with firewall configurations
        let json_content = r#"{
  "name": "Test Container",
  "runArgs": [
    "--cap-add=NET_ADMIN",
    "--cap-add=NET_RAW",
    "--privileged"
  ],
  "postStartCommand": "sudo /usr/local/bin/init-firewall.sh",
  "waitFor": "postStartCommand",
  "customizations": {
    "vscode": {
      "extensions": ["ms-vscode.vscode-typescript-next"]
    }
  }
}"#;
        fs::write(&json_path, json_content).unwrap();

        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let changes = customizer
            .strip_devcontainer_json_firewall(&json_path)
            .unwrap();

        assert!(!changes.is_empty());
        assert!(changes.iter().any(|c| c.contains("NET_ADMIN")));
        assert!(changes.iter().any(|c| c.contains("postStartCommand")));
        assert!(changes.iter().any(|c| c.contains("waitFor")));

        // Verify the file was actually modified
        let modified_content = fs::read_to_string(&json_path).unwrap();
        let modified_json: serde_json::Value = serde_json::from_str(&modified_content).unwrap();

        // Check that firewall capabilities were removed
        if let Some(run_args) = modified_json.get("runArgs").and_then(|v| v.as_array()) {
            assert!(!run_args
                .iter()
                .any(|arg| arg.as_str().unwrap_or("").contains("NET_ADMIN")));
            assert!(!run_args
                .iter()
                .any(|arg| arg.as_str().unwrap_or("").contains("NET_RAW")));
            // --privileged should remain
            assert!(run_args
                .iter()
                .any(|arg| arg.as_str().unwrap_or("") == "--privileged"));
        }

        // Check that firewall-related keys were removed
        assert!(modified_json.get("postStartCommand").is_none());
        assert!(modified_json.get("waitFor").is_none());

        // Check that other configurations were preserved
        assert!(modified_json.get("name").is_some());
        assert!(modified_json.get("customizations").is_some());
    }

    #[test]
    fn test_strip_devcontainer_json_no_firewall() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("devcontainer.json");

        // Create a devcontainer.json without firewall configurations
        let json_content = r#"{
  "name": "Test Container",
  "image": "node:18",
  "customizations": {
    "vscode": {
      "extensions": ["ms-vscode.vscode-typescript-next"]
    }
  }
}"#;
        fs::write(&json_path, json_content).unwrap();

        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let changes = customizer
            .strip_devcontainer_json_firewall(&json_path)
            .unwrap();

        assert!(changes.is_empty());
    }

    #[test]
    fn test_strip_dockerfile_firewall() {
        let temp_dir = TempDir::new().unwrap();
        let dockerfile_path = temp_dir.path().join("Dockerfile");

        // Create a Dockerfile with firewall configurations similar to the example
        let dockerfile_content = r#"FROM node:20

# Install basic development tools and iptables/ipset
RUN apt-get update && apt-get install -y --no-install-recommends \
  less \
  git \
  iptables \
  ipset \
  iproute2 \
  dnsutils \
  aggregate \
  jq \
  && apt-get clean

# Copy and set up firewall script
COPY init-firewall.sh /usr/local/bin/
USER root
RUN chmod +x /usr/local/bin/init-firewall.sh && \
  echo "node ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/node-firewall
USER node

ENV NPM_CONFIG_PREFIX=/usr/local/share/npm-global
"#;
        fs::write(&dockerfile_path, dockerfile_content).unwrap();

        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let changes = customizer
            .strip_dockerfile_firewall(&dockerfile_path)
            .unwrap();

        assert!(!changes.is_empty());
        assert!(changes.iter().any(|c| c.contains("firewall packages")));
        assert!(changes.iter().any(|c| c.contains("firewall setup section")));

        // Verify the file was actually modified
        let modified_content = fs::read_to_string(&dockerfile_path).unwrap();

        // Check that firewall packages were removed from apt install
        // The packages should be completely removed from the lines
        let apt_lines: Vec<&str> = modified_content
            .lines()
            .filter(|line| {
                line.contains("apt-get install")
                    || (!line.trim().is_empty()
                        && !line.starts_with("RUN")
                        && !line.starts_with("FROM")
                        && !line.starts_with("#")
                        && !line.starts_with("USER")
                        && !line.starts_with("ENV")
                        && !line.starts_with("COPY"))
            })
            .collect();

        for line in apt_lines {
            assert!(
                !line.contains("iptables"),
                "iptables should be removed from: {}",
                line
            );
            assert!(
                !line.contains("ipset"),
                "ipset should be removed from: {}",
                line
            );
            assert!(
                !line.contains("iproute2"),
                "iproute2 should be removed from: {}",
                line
            );
            assert!(
                !line.contains("dnsutils"),
                "dnsutils should be removed from: {}",
                line
            );
            assert!(
                !line.contains("aggregate"),
                "aggregate should be removed from: {}",
                line
            );
        }

        // Check that firewall setup section was removed
        assert!(!modified_content.contains("# Copy and set up firewall script"));
        assert!(!modified_content.contains("COPY init-firewall.sh"));
        assert!(!modified_content.contains("sudoers.d/node-firewall"));

        // Check that other content was preserved
        assert!(modified_content.contains("FROM node:20"));
        assert!(modified_content.contains("less"));
        assert!(modified_content.contains("git"));
        assert!(modified_content.contains("jq"));
        assert!(modified_content.contains("NPM_CONFIG_PREFIX"));
    }

    #[test]
    fn test_strip_dockerfile_no_firewall() {
        let temp_dir = TempDir::new().unwrap();
        let dockerfile_path = temp_dir.path().join("Dockerfile");

        // Create a Dockerfile without firewall configurations
        let dockerfile_content = r#"FROM node:18

RUN apt-get update && apt-get install -y \
  git \
  curl \
  vim \
  && apt-get clean

USER node
WORKDIR /app
"#;
        fs::write(&dockerfile_path, dockerfile_content).unwrap();

        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let changes = customizer
            .strip_dockerfile_firewall(&dockerfile_path)
            .unwrap();

        assert!(changes.is_empty());
    }

    #[test]
    fn test_matches_firewall_patterns() {
        let customizer = DefaultDevcontainerCustomizer::new(PathBuf::from("/tmp"), false);

        // Test positive matches
        let firewall_content = "RUN apt-get install iptables ipset";
        let matches = customizer
            .matches_firewall_patterns(firewall_content)
            .unwrap();
        assert!(!matches.is_empty());
        assert!(matches.iter().any(|m| m.contains("iptables")));
        assert!(matches.iter().any(|m| m.contains("ipset")));

        // Test negative matches
        let clean_content = "RUN apt-get install git vim";
        let matches = customizer.matches_firewall_patterns(clean_content).unwrap();
        assert!(matches.is_empty());

        // Test capability matches
        let capability_content = r#"["--cap-add=NET_ADMIN", "--cap-add=NET_RAW"]"#;
        let matches = customizer
            .matches_firewall_patterns(capability_content)
            .unwrap();
        assert_eq!(matches.len(), 2);
    }

    #[test]
    fn test_validate_firewall_removal() {
        let customizer = DefaultDevcontainerCustomizer::new(PathBuf::from("/tmp"), false);

        // Test with no changes
        let empty_result = FirewallRemovalResult::new();
        let warnings = customizer.validate_firewall_removal(&empty_result);
        assert!(warnings.iter().any(|w| w.contains("No firewall scripts")));
        assert!(warnings
            .iter()
            .any(|w| w.contains("No firewall configurations found in Dockerfile")));
        assert!(warnings
            .iter()
            .any(|w| w.contains("No firewall configurations found in devcontainer.json")));

        // Test with changes
        let mut result_with_changes = FirewallRemovalResult::new();
        result_with_changes.add_removed_file(PathBuf::from("init-firewall.sh"));
        result_with_changes.add_dockerfile_change("Removed firewall packages".to_string());
        result_with_changes.add_json_change("Removed capabilities".to_string());

        let warnings = customizer.validate_firewall_removal(&result_with_changes);
        assert!(warnings.is_empty());
    }

    #[test]
    fn test_strip_devcontainer_json_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let json_path = temp_dir.path().join("devcontainer.json");

        // Test with malformed JSON
        fs::write(&json_path, r#"{ invalid json }"#).unwrap();
        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let result = customizer.strip_devcontainer_json_firewall(&json_path);
        assert!(result.is_err());

        // Test with empty runArgs
        let json_content = r#"{"name": "Test", "runArgs": []}"#;
        fs::write(&json_path, json_content).unwrap();
        let changes = customizer
            .strip_devcontainer_json_firewall(&json_path)
            .unwrap();
        assert!(changes.is_empty());

        // Test with mixed capabilities
        let json_content = r#"{
  "name": "Test",
  "runArgs": ["--privileged", "--cap-add=NET_ADMIN", "--cap-add=SYS_ADMIN", "--cap-add=NET_RAW"]
}"#;
        fs::write(&json_path, json_content).unwrap();
        let changes = customizer
            .strip_devcontainer_json_firewall(&json_path)
            .unwrap();
        assert!(!changes.is_empty());

        // Verify only firewall capabilities were removed
        let modified_content = fs::read_to_string(&json_path).unwrap();
        let modified_json: serde_json::Value = serde_json::from_str(&modified_content).unwrap();
        let run_args = modified_json.get("runArgs").unwrap().as_array().unwrap();

        assert!(run_args
            .iter()
            .any(|arg| arg.as_str().unwrap() == "--privileged"));
        assert!(run_args
            .iter()
            .any(|arg| arg.as_str().unwrap() == "--cap-add=SYS_ADMIN"));
        assert!(!run_args
            .iter()
            .any(|arg| arg.as_str().unwrap().contains("NET_ADMIN")));
        assert!(!run_args
            .iter()
            .any(|arg| arg.as_str().unwrap().contains("NET_RAW")));
    }

    #[test]
    fn test_strip_dockerfile_edge_cases() {
        let temp_dir = TempDir::new().unwrap();
        let dockerfile_path = temp_dir.path().join("Dockerfile");

        // Test with no apt install commands
        let dockerfile_content = r#"FROM node:20
WORKDIR /app
COPY . .
"#;
        fs::write(&dockerfile_path, dockerfile_content).unwrap();
        let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
        let changes = customizer
            .strip_dockerfile_firewall(&dockerfile_path)
            .unwrap();
        assert!(changes.is_empty());

        // Test with firewall section but no packages
        let dockerfile_content = r#"FROM node:20
RUN apt-get update && apt-get install -y git vim

# Copy and set up firewall script
COPY init-firewall.sh /usr/local/bin/
USER root
RUN chmod +x /usr/local/bin/init-firewall.sh
USER node
"#;
        fs::write(&dockerfile_path, dockerfile_content).unwrap();
        let changes = customizer
            .strip_dockerfile_firewall(&dockerfile_path)
            .unwrap();
        assert!(changes.iter().any(|c| c.contains("firewall setup section")));

        // Verify firewall section was removed but other content preserved
        let modified_content = fs::read_to_string(&dockerfile_path).unwrap();
        assert!(!modified_content.contains("# Copy and set up firewall script"));
        assert!(!modified_content.contains("COPY init-firewall.sh"));
        assert!(modified_content.contains("FROM node:20"));
        assert!(modified_content.contains("git vim"));
    }

    #[test]
    fn test_firewall_removal_result_methods() {
        let mut result = FirewallRemovalResult::new();

        // Test initial state
        assert!(!result.has_changes());
        assert!(!result.has_warnings());

        // Test adding changes
        result.add_modified_file(PathBuf::from("test.json"));
        assert!(result.has_changes());

        result.add_removed_file(PathBuf::from("script.sh"));
        assert!(result.has_changes());

        // Test adding warnings
        result.add_warning("Test warning".to_string());
        assert!(result.has_warnings());

        result.add_pattern_not_found("missing pattern".to_string());
        assert!(result.has_warnings());

        // Test change tracking
        result.add_dockerfile_change("Dockerfile change".to_string());
        result.add_json_change("JSON change".to_string());

        assert_eq!(result.dockerfile_changes.len(), 1);
        assert_eq!(result.json_changes.len(), 1);
        assert_eq!(result.files_modified.len(), 1);
        assert_eq!(result.files_removed.len(), 1);
        assert_eq!(result.warnings.len(), 1);
        assert_eq!(result.patterns_not_found.len(), 1);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use std::fs;
    use tempfile::TempDir;

    // Generator for firewall-related content
    fn firewall_content_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            "iptables -F",
            "ipset create test hash:ip",
            "iproute2 command",
            "dnsutils query",
            "aggregate --help",
            "sudo /usr/local/bin/init-firewall.sh",
            "postStartCommand.*firewall",
            "--cap-add=NET_ADMIN",
            "--cap-add=NET_RAW"
        ]
        .prop_map(|s| s.to_string())
    }

    // Generator for non-firewall content
    fn non_firewall_content_strategy() -> impl Strategy<Value = String> {
        prop_oneof![
            "echo hello",
            "npm install",
            "git clone",
            "curl https://example.com",
            "mkdir -p /app",
            "chmod +x script.sh",
            "apt-get install vim"
        ]
        .prop_map(|s| s.to_string())
    }

    // Generator for devcontainer.json with firewall configurations
    fn devcontainer_json_with_firewall_strategy() -> impl Strategy<Value = String> {
        (
            prop::collection::vec(firewall_content_strategy(), 1..3),
            prop::collection::vec(non_firewall_content_strategy(), 1..3)
        ).prop_map(|(firewall_items, _non_firewall_items)| {
            let mut run_args = vec!["--privileged".to_string()];
            run_args.extend(firewall_items.iter().filter(|item| item.contains("--cap-add")).cloned());

            let post_start = if firewall_items.iter().any(|item| item.contains("firewall")) {
                r#","postStartCommand": "sudo /usr/local/bin/init-firewall.sh","waitFor": "postStartCommand""#
            } else {
                ""
            };

            format!(r#"{{
  "name": "Test Container",
  "runArgs": [{}],
  "customizations": {{
    "vscode": {{
      "extensions": ["ms-vscode.vscode-typescript-next"]
    }}
  }}{}
}}"#,
                run_args.iter().map(|s| format!(r#""{}""#, s)).collect::<Vec<_>>().join(","),
                post_start
            )
        })
    }

    // Generator for Dockerfile with firewall configurations
    fn dockerfile_with_firewall_strategy() -> impl Strategy<Value = String> {
        (
            prop::collection::vec(firewall_content_strategy(), 1..3),
            prop::collection::vec(non_firewall_content_strategy(), 1..3),
        )
            .prop_map(|(_firewall_items, _non_firewall_items)| {
                let firewall_packages = ["iptables", "ipset", "iproute2", "dnsutils", "aggregate"];
                let mut apt_packages: Vec<String> =
                    vec!["git".to_string(), "curl".to_string(), "vim".to_string()];
                apt_packages.extend(firewall_packages.iter().map(|s| s.to_string()));

                let firewall_section = r#"
# Copy and set up firewall script
COPY init-firewall.sh /usr/local/bin/
USER root
RUN chmod +x /usr/local/bin/init-firewall.sh && \
  echo "node ALL=(root) NOPASSWD: /usr/local/bin/init-firewall.sh" > /etc/sudoers.d/node-firewall
USER node
"#;

                format!(
                    r#"FROM node:20

RUN apt-get update && apt-get install -y \
  {} \
  && apt-get clean
{}
ENV NODE_ENV=development
"#,
                    apt_packages.join(" \\\n  "),
                    firewall_section
                )
            })
    }

    proptest! {
        // **Feature: devcontainer-sync-cli, Property 1: Firewall pattern detection and removal**
        // **Validates: Requirements 5.1, 5.2**
        #[test]
        fn property_firewall_pattern_detection_and_removal(
            json_content in devcontainer_json_with_firewall_strategy(),
            dockerfile_content in dockerfile_with_firewall_strategy()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let devcontainer_path = temp_dir.path().join(".devcontainer");
            fs::create_dir_all(&devcontainer_path).unwrap();

            // Create files with firewall configurations
            let json_path = devcontainer_path.join("devcontainer.json");
            let dockerfile_path = devcontainer_path.join("Dockerfile");
            let script_path = devcontainer_path.join("init-firewall.sh");

            fs::write(&json_path, &json_content).unwrap();
            fs::write(&dockerfile_path, &dockerfile_content).unwrap();
            fs::write(&script_path, "#!/bin/bash\niptables -F\n").unwrap();

            let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
            let result = customizer.strip_firewall_features(&devcontainer_path).unwrap();

            // Property: All detected firewall patterns should be removed while preserving non-firewall functionality
            if json_content.contains("--cap-add=NET_ADMIN") || json_content.contains("--cap-add=NET_RAW") {
                prop_assert!(result.json_changes.iter().any(|c| c.contains("NET_ADMIN") || c.contains("NET_RAW")));
            }

            if json_content.contains("postStartCommand") {
                prop_assert!(result.json_changes.iter().any(|c| c.contains("postStartCommand")));
            }

            if dockerfile_content.contains("iptables") || dockerfile_content.contains("ipset") {
                prop_assert!(result.dockerfile_changes.iter().any(|c| c.contains("firewall packages")));
            }

            if dockerfile_content.contains("# Copy and set up firewall script") {
                prop_assert!(result.dockerfile_changes.iter().any(|c| c.contains("firewall setup section")));
            }

            // Verify firewall script was removed
            prop_assert!(result.files_removed.iter().any(|f| f.file_name().unwrap() == "init-firewall.sh"));

            // Verify non-firewall content is preserved
            let modified_json = fs::read_to_string(&json_path).unwrap();
            prop_assert!(modified_json.contains("Test Container"));
            prop_assert!(modified_json.contains("customizations"));

            let modified_dockerfile = fs::read_to_string(&dockerfile_path).unwrap();
            prop_assert!(modified_dockerfile.contains("FROM node:20"));
            prop_assert!(modified_dockerfile.contains("NODE_ENV=development"));
        }

        // **Feature: devcontainer-sync-cli, Property 2: Graceful handling of missing patterns**
        // **Validates: Requirements 5.3, 5.5**
        #[test]
        fn property_graceful_handling_of_missing_patterns(
            _non_firewall_json in prop::collection::vec(non_firewall_content_strategy(), 1..5),
            non_firewall_dockerfile in prop::collection::vec(non_firewall_content_strategy(), 1..5)
        ) {
            let temp_dir = TempDir::new().unwrap();
            let devcontainer_path = temp_dir.path().join(".devcontainer");
            fs::create_dir_all(&devcontainer_path).unwrap();

            // Create files without firewall configurations
            let json_content = r#"{
  "name": "Clean Container",
  "image": "node:18",
  "customizations": {
    "vscode": {
      "extensions": ["ms-vscode.vscode-typescript-next"]
    }
  }
}"#;

            let dockerfile_content = format!(r#"FROM node:18
RUN apt-get update && apt-get install -y git vim
{}
WORKDIR /app
"#, non_firewall_dockerfile.join("\nRUN "));

            let json_path = devcontainer_path.join("devcontainer.json");
            let dockerfile_path = devcontainer_path.join("Dockerfile");

            fs::write(&json_path, json_content).unwrap();
            fs::write(&dockerfile_path, &dockerfile_content).unwrap();

            let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
            let result = customizer.strip_firewall_features(&devcontainer_path).unwrap();

            // Property: Tool should continue processing and report what wasn't found rather than failing
            prop_assert!(result.warnings.iter().any(|w| w.contains("No firewall")));
            prop_assert_eq!(result.files_removed.len(), 0);
            prop_assert_eq!(result.json_changes.len(), 0);
            prop_assert_eq!(result.dockerfile_changes.len(), 0);

            // Verify files are unchanged
            let final_json = fs::read_to_string(&json_path).unwrap();
            let final_dockerfile = fs::read_to_string(&dockerfile_path).unwrap();
            prop_assert_eq!(final_json, json_content);
            prop_assert_eq!(final_dockerfile, dockerfile_content);
        }

        // **Feature: devcontainer-sync-cli, Property 5: Non-firewall functionality preservation with validation**
        // **Validates: Requirements 5.6**
        #[test]
        fn property_non_firewall_functionality_preservation(
            firewall_json in devcontainer_json_with_firewall_strategy(),
            firewall_dockerfile in dockerfile_with_firewall_strategy()
        ) {
            let temp_dir = TempDir::new().unwrap();
            let devcontainer_path = temp_dir.path().join(".devcontainer");
            fs::create_dir_all(&devcontainer_path).unwrap();

            let json_path = devcontainer_path.join("devcontainer.json");
            let dockerfile_path = devcontainer_path.join("Dockerfile");
            let script_path = devcontainer_path.join("init-firewall.sh");

            fs::write(&json_path, &firewall_json).unwrap();
            fs::write(&dockerfile_path, &firewall_dockerfile).unwrap();
            fs::write(&script_path, "#!/bin/bash\niptables -F\n").unwrap();

            let customizer = DefaultDevcontainerCustomizer::new(temp_dir.path().to_path_buf(), false);
            let _result = customizer.strip_firewall_features(&devcontainer_path).unwrap();

            // Property: After stripping, all non-firewall functionality should remain intact
            let modified_json = fs::read_to_string(&json_path).unwrap();
            let modified_dockerfile = fs::read_to_string(&dockerfile_path).unwrap();

            // Verify essential non-firewall elements are preserved
            if firewall_json.contains("Test Container") {
                prop_assert!(modified_json.contains("Test Container"));
            }
            if firewall_json.contains("customizations") {
                prop_assert!(modified_json.contains("customizations"));
            }
            if firewall_json.contains("--privileged") {
                prop_assert!(modified_json.contains("--privileged"));
            }

            if firewall_dockerfile.contains("FROM node:20") {
                prop_assert!(modified_dockerfile.contains("FROM node:20"));
            }
            if firewall_dockerfile.contains("NODE_ENV=development") {
                prop_assert!(modified_dockerfile.contains("NODE_ENV=development"));
            }
            if firewall_dockerfile.contains("git") && !firewall_dockerfile.contains("iptables") {
                prop_assert!(modified_dockerfile.contains("git"));
            }

            // Verify firewall elements are removed
            prop_assert!(!modified_json.contains("--cap-add=NET_ADMIN"));
            prop_assert!(!modified_json.contains("--cap-add=NET_RAW"));
            prop_assert!(!modified_dockerfile.contains("# Copy and set up firewall script"));
        }
    }
}
