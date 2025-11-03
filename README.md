# devcontainer-sync-cli

A CLI tool for syncing devcontainer configurations from the Claude Code repository.

⚠️ At the time of writing, the implementation will include a `iptables`-based firewall.

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
- `update --backup`: Create backup before updating
- `remove --keep-files`: Keep devcontainer files when removing sync

## Why?

The Claude Code Best Practices docs say recommend using devcontainers, and to copy the implementation in the main Claude Code repo. As we all know, whenever you copy/paste from a Git repo a fairy dies, and instead we should be able to pull down updates if Anthropic change their implementation. Doing this 'by hand' is a bit of a faff:

```sh
git remote add claude https://github.com/anthropics/claude-code.git
git fetch claude
git branch -f claude-main claude/main
git checkout claude-main
git subtree split --prefix=.devcontainer -b devcontainer claude-main
git checkout master
# Will fail if there wasn't a commit already
git subtree add --prefix=.devcontainer devcontainer --squash
```

This CLI does all this fiddly-faff for you.
