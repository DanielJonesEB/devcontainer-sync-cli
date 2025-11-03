# devcontainer-sync-cli

A CLI tool for syncing devcontainer configurations from the Claude Code repository.

## Usage

```bash
# Initialize devcontainer sync in your git repository
devcontainer-sync init

# Update to latest devcontainer configurations
devcontainer-sync update

# Remove devcontainer sync (keeps files by default)
devcontainer-sync remove
```

## Options

- `--verbose, -v`: Show detailed output
- `--dry-run`: Preview changes without making them
- `update --backup`: Create backup before updating
- `remove --keep-files`: Keep devcontainer files when removing sync
