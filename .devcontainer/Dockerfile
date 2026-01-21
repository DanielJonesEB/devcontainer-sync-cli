FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Link git-subtree from system git to the /usr/local git installation
# (The devcontainer base image has a newer git at /usr/local that lacks contrib scripts)
RUN ln -s /usr/lib/git-core/git-subtree /usr/local/libexec/git-core/git-subtree

# Install Rust components
RUN . /usr/local/cargo/env && rustup component add rustfmt clippy rust-analyzer

# Install cargo tools (this is the slow part)
RUN . /usr/local/cargo/env && cargo install cargo-watch cargo-edit cargo-outdated
