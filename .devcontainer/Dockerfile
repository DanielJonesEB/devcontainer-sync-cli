FROM mcr.microsoft.com/devcontainers/rust:1-bookworm

# Install Rust components
RUN . /usr/local/cargo/env && rustup component add rustfmt clippy rust-analyzer

# Install cargo tools (this is the slow part)
RUN . /usr/local/cargo/env && cargo install cargo-watch cargo-edit cargo-outdated
