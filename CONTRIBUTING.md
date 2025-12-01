# Contributing to ZDK

We welcome contributions to ZDK! This document provides guidelines for contributing.

## Development Setup

1. Install Rust 1.90.0 or later
2. Clone the repository
3. Copy `config.toml.example` to `config.toml` and add your API keys
4. Run tests: `cargo test --workspace`
5. Run examples: `make test-examples`

## Pull Request Process

1. Fork the repository and create a feature branch
2. Make your changes with clear commit messages
3. Ensure all tests pass: `cargo test --workspace`
4. Run formatting: `cargo fmt --all`
5. Run linting: `cargo clippy --workspace -- -D warnings`
6. Submit a PR with a clear description of changes

## Questions?

Open an issue or discussion on GitHub for questions or feedback.

