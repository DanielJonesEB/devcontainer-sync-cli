# GitHub Actions Workflow Testing Guide

## Overview

This document provides instructions for testing GitHub Actions workflows locally and validating changes before pushing to the repository.

## Local Testing with Act

### Prerequisites

1. Install [act](https://github.com/nektos/act):
   ```bash
   # macOS
   brew install act

   # Linux
   curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
   ```

2. Install Docker (required by act)

### Testing CI Workflow

Test the continuous integration workflow locally:

```bash
# Test the CI workflow on push event
act push

# Test specific job
act push -j test

# Test with specific event file
act push --eventpath .github/workflows/test-events/push.json
```

### Testing Release Workflow

Test the release workflow with a simulated tag:

```bash
# Create a test event file for tag push
mkdir -p .github/workflows/test-events
cat > .github/workflows/test-events/tag.json << 'EOF'
{
  "ref": "refs/tags/v1.0.0",
  "ref_type": "tag",
  "ref_name": "v1.0.0"
}
EOF

# Test release workflow
act push --eventpath .github/workflows/test-events/tag.json -W .github/workflows/release.yml
```

## Workflow Validation Checklist

### Before Committing Changes

- [ ] Workflow YAML syntax is valid
- [ ] All required secrets and permissions are configured
- [ ] Action versions are pinned to specific commits
- [ ] Build matrix covers all target platforms
- [ ] Caching keys are properly configured

### CI Workflow Validation

- [ ] Workflow triggers on push to main/develop branches
- [ ] Workflow triggers on pull requests to main
- [ ] Rust toolchain setup works correctly
- [ ] Cargo dependencies are cached properly
- [ ] All tests pass (`cargo test --all-features`)
- [ ] Clippy linting passes with deny warnings
- [ ] Code formatting check passes (`cargo fmt --check`)
- [ ] Workflow completes within 10 minutes

### Release Workflow Validation

- [ ] Workflow triggers only on version tags (v*)
- [ ] Workflow runs only on main branch
- [ ] Cross-platform build matrix works for all targets:
  - [ ] Linux x86_64 (`x86_64-unknown-linux-gnu`)
  - [ ] macOS x86_64 (`x86_64-apple-darwin`)
  - [ ] macOS ARM64 (`aarch64-apple-darwin`)
- [ ] Binaries are built with release optimization
- [ ] Artifacts are uploaded with correct naming convention
- [ ] GitHub release is created with proper metadata
- [ ] Release notes are generated from git history
- [ ] All build artifacts are attached to the release

### Security Validation

- [ ] All GitHub Actions are pinned to specific commit hashes
- [ ] Minimal required permissions are configured
- [ ] Only official GitHub-hosted runners are used
- [ ] Cargo.lock verification is enabled
- [ ] Build artifact validation is performed
- [ ] No sensitive information is exposed in logs

## Testing Scenarios

### 1. Feature Branch Testing

```bash
# Test CI workflow for feature branch
git checkout -b feature/test-ci
git commit --allow-empty -m "Test CI workflow"
act push
```

### 2. Release Testing

```bash
# Test release workflow without actually creating a release
act push --eventpath .github/workflows/test-events/tag.json --dryrun
```

### 3. Matrix Build Testing

```bash
# Test specific platform builds
act push -W .github/workflows/release.yml --matrix os:ubuntu-latest
act push -W .github/workflows/release.yml --matrix os:macos-latest
```

## Troubleshooting

### Common Issues

1. **Act fails with permission errors**
   - Ensure Docker is running and accessible
   - Run act with `--privileged` flag if needed

2. **Workflow fails on missing secrets**
   - Create `.secrets` file for local testing
   - Use `--secret-file .secrets` flag with act

3. **Cache-related failures**
   - Act doesn't support GitHub Actions cache
   - Use `--rm` flag to start with clean containers

4. **Platform-specific build failures**
   - Test cross-compilation locally first:
     ```bash
     cargo build --target x86_64-unknown-linux-gnu
     cargo build --target x86_64-apple-darwin
     cargo build --target aarch64-apple-darwin
     ```

### Debug Mode

Run workflows in debug mode for detailed output:

```bash
act push --verbose
```

## Integration Testing

### End-to-End Release Testing

1. Create a test tag in a fork:
   ```bash
   git tag v0.0.1-test
   git push origin v0.0.1-test
   ```

2. Verify the release workflow:
   - Check that binaries are built for all platforms
   - Download and test each binary
   - Verify release notes are generated correctly

3. Clean up test releases:
   ```bash
   git tag -d v0.0.1-test
   git push origin :refs/tags/v0.0.1-test
   ```

## Continuous Monitoring

### Workflow Health Checks

- Monitor workflow run times and success rates
- Review dependency cache hit rates
- Check for security vulnerabilities in dependencies
- Validate that all target platforms remain supported

### Performance Optimization

- Review build times and identify bottlenecks
- Optimize caching strategies
- Consider parallel job execution improvements
- Monitor resource usage and costs
