# Contributing to ZDK (ZAgent Development Kit)

Thank you for your interest in contributing to ZDK! This document provides guidelines and instructions for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Project Structure](#project-structure)
- [Development Workflow](#development-workflow)
- [Code Style Guidelines](#code-style-guidelines)
- [Testing Guidelines](#testing-guidelines)
- [Documentation Guidelines](#documentation-guidelines)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Pull Request Process](#pull-request-process)
- [Release Process](#release-process)

## Code of Conduct

By participating in this project, you agree to maintain a respectful and inclusive environment for all contributors.

## Getting Started

### Prerequisites

- **Rust**: 1.90.0+ (edition 2024)
- **Cargo**: Latest stable version
- **Git**: For version control
- **PostgreSQL** (optional): For database session testing
- **SQLite** (optional): For SQLite session testing

### Quick Start

```bash
# Clone the repository
git clone <repository-url>
cd rak

# Build the project
make build

# Run tests
make test

# Run a specific example
make example-quickstart
```

## Development Setup

### 1. Build Dependencies

```bash
# Install Rust (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Update to latest stable
rustup update stable
```

### 2. Environment Setup

```bash
# Copy example config
cp config.toml.example config.toml

# Set environment variables
export GEMINI_API_KEY="your-api-key-here"
export RUST_LOG=info
```

### 3. IDE Setup

**Recommended**: Use an IDE with Rust support:
- **VSCode**: Install `rust-analyzer` extension
- **IntelliJ IDEA**: Install Rust plugin
- **Cursor**: Built-in Rust support

## Project Structure

```
rak/
├── crates/              # All workspace crates
│   ├── zdk-core/       # Core traits and types
│   ├── zdk-model/      # LLM implementations
│   ├── zdk-session/    # Session management
│   ├── zdk-agent/      # Agent implementations
│   ├── zdk-runner/     # Execution engine
│   ├── zdk-server/     # REST/WebSocket API
│   ├── zdk-tool/       # Tool system
│   ├── zdk-macros/     # Procedural macros
│   ├── zdk-artifact/   # Artifact storage
│   ├── zdk-memory/     # Memory service
│   └── zdk-telemetry/  # Observability
├── docs/               # Documentation
├── examples/           # Usage examples
├── tests/             # Integration tests
└── Makefile           # Development commands
```

### Key Directories

- **`crates/`**: Individual crates that make up ZDK
- **`docs/`**: Architecture docs, phase summaries, guides
- **`examples/`**: Runnable examples demonstrating features
- **`tests/`**: Integration and E2E tests

## Development Workflow

### Primary: Using Make Commands

ZDK uses `make` for all common development tasks. Run `make help` to see all available commands.

```bash
# Run all tests (default and recommended)
make test

# Check code without building (fast)
make check

# Build all workspace crates
make build

# Run clippy linter
make clippy

# Format all code
make fmt

# Generate and open documentation
make doc

# Clean build artifacts
make clean

# Run specific example
make example-quickstart
make example-tool_usage
make example-workflow_agents

# Run tests with debug logging
make test-verbose

# Build release version
make release
```

### Advanced: Using Cargo Directly

For advanced use cases, you can use cargo commands directly:

```bash
# Build specific crate only
cargo build --package zdk-core

# Test specific crate only
cargo test --package zdk-agent

# Run with custom logging
RUST_LOG=debug cargo run --example quickstart

# Check single crate
cargo check --package zdk-session

# Use shorter aliases (configured in .cargo/config.toml)
cargo t    # Test all workspace
cargo c    # Check all workspace
cargo b    # Build all workspace
cargo cl   # Clippy all workspace
```

**Note**: When using cargo directly, always use `--workspace` flag to work with all crates, or use the configured aliases (`cargo t`, `cargo c`, etc.).

## Code Style Guidelines

ZDK follows Rust best practices and conventions defined in `.cursorrules`.

### Formatting

- **Use `rustfmt`** for all code formatting:
  ```bash
  make fmt
  ```

- **Use `clippy`** for linting:
  ```bash
  make clippy
  ```

### Naming Conventions

- **Variables/Functions**: `snake_case`
  ```rust
  fn process_event(event_data: &Event) -> Result<()> { }
  ```

- **Types/Traits**: `PascalCase`
  ```rust
  pub struct LLMAgent { }
  pub trait SessionService { }
  ```

- **Constants**: `UPPERCASE_SNAKE_CASE`
  ```rust
  const MAX_RETRIES: usize = 3;
  ```

- **Modules**: `snake_case`
  ```rust
  mod llm_agent;
  ```

### Code Organization

1. **Use iterators** over explicit loops when appropriate:
   ```rust
   // Prefer this
   let sum: i32 = items.iter().map(|x| x.value).sum();
   
   // Over this
   let mut sum = 0;
   for item in &items {
       sum += item.value;
   }
   ```

2. **Use the `?` operator** for error propagation:
   ```rust
   fn load_session(id: &str) -> Result<Session> {
       let session = session_service.get(id)?;
       Ok(session)
   }
   ```

3. **Prefer `&str` over `String`** when possible:
   ```rust
   pub fn set_name(&mut self, name: &str) {
       self.name = name.to_string();
   }
   ```

### Error Handling

- **Use `anyhow::Result`** for application-level errors:
  ```rust
  use anyhow::Result;
  
  fn run_agent() -> Result<()> {
      // ...
  }
  ```

- **Use `thiserror`** for library-level custom errors:
  ```rust
  use thiserror::Error;
  
  #[derive(Error, Debug)]
  pub enum SessionError {
      #[error("Session not found: {0}")]
      NotFound(String),
  }
  ```

- **Avoid `unwrap()`** in production code:
  ```rust
  // Don't do this in libraries
  let value = some_option.unwrap();
  
  // Do this instead
  let value = some_option.ok_or_else(|| Error::MissingValue)?;
  ```

- **Use `expect()` only** when panic is intentional:
  ```rust
  let config = Config::load().expect("Failed to load required config");
  ```

### Async Code

- **Use `tokio` runtime** for async operations
- **Prefer async/await** over combinators:
  ```rust
  // Prefer this
  async fn fetch_data() -> Result<Data> {
      let response = client.get(url).await?;
      let data = response.json().await?;
      Ok(data)
  }
  ```

- **Use `Arc` for shared state** across async tasks:
  ```rust
  let agent = Arc::new(LLMAgent::new(model));
  ```

### Type Safety

- **Leverage the type system** to prevent bugs
- **Use enums** for state machines and variants
- **Prefer `Option` and `Result`** over null/exception patterns
- **Use newtype pattern** for domain-specific types:
  ```rust
  pub struct SessionId(String);
  pub struct UserId(String);
  ```

## Testing Guidelines

### Test Organization

- **Unit tests**: Place in same file with `#[cfg(test)]` module
  ```rust
  #[cfg(test)]
  mod tests {
      use super::*;
      
      #[test]
      fn test_agent_builder() {
          // ...
      }
  }
  ```

- **Integration tests**: Place in `tests/` directory
  ```rust
  // tests/integration_test.rs
  use rak_agent::LLMAgent;
  
  #[tokio::test]
  async fn test_agent_execution() {
      // ...
  }
  ```

### Test Guidelines

1. **Aim for meaningful coverage**, not just percentage
2. **Test public APIs**, not implementation details
3. **Use descriptive test names**:
   ```rust
   #[test]
   fn test_session_service_creates_new_session_with_unique_id() { }
   ```

4. **Test error conditions**:
   ```rust
   #[tokio::test]
   async fn test_agent_handles_llm_timeout() { }
   ```

5. **Use `#[should_panic]`** for expected panics:
   ```rust
   #[test]
   #[should_panic(expected = "Invalid configuration")]
   fn test_invalid_config_panics() { }
   ```

### Running Tests

```bash
# Run all tests (recommended)
make test

# Run tests with debug logging
make test-verbose

# Run specific test (use cargo directly)
cargo test test_session_creation --workspace

# Run tests with output (use cargo directly)
cargo test --workspace -- --nocapture
```

## Documentation Guidelines

### Code Documentation

- **Add doc comments** (`///`) for all public APIs:
  ```rust
  /// Creates a new LLM agent with the specified model.
  ///
  /// # Arguments
  ///
  /// * `model` - The LLM model to use for generation
  ///
  /// # Examples
  ///
  /// ```
  /// use rak_agent::LLMAgent;
  /// use rak_model::GeminiModel;
  ///
  /// let model = GeminiModel::new(api_key, "gemini-2.0-flash".into());
  /// let agent = LLMAgent::builder()
  ///     .model(Arc::new(model))
  ///     .build()?;
  /// ```
  ///
  /// # Errors
  ///
  /// Returns an error if the agent configuration is invalid.
  pub fn build(self) -> Result<LLMAgent> {
      // ...
  }
  ```

- **Include examples** in doc comments when helpful
- **Document errors** that can be returned
- **Use `#[doc(hidden)]`** for internal-only public items

### Documentation Files

All documentation goes in `docs/` directory with naming convention:
```
docs/YYYYMMDD_HHmm_FEATURE_DOCTYPE.md
```

Examples:
- `20251119_1500_TOOL_SYSTEM.md`
- `20251119_1600_STORAGE_PROVIDERS.md`

### Generating Documentation

```bash
# Generate and open docs
make doc

# Generate docs with private items (use cargo directly)
cargo doc --workspace --document-private-items --open
```

## Commit Message Guidelines

### Format

Use [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Build process, dependencies, etc.

### Examples

```
feat(agent): Add support for parallel agent execution

Implements ParallelAgent that executes multiple sub-agents concurrently
using Tokio. Includes proper error handling and result aggregation.

Closes #123
```

```
fix(session): Fix memory leak in session cleanup

The session service was not properly releasing references to completed
sessions, causing memory growth over time.
```

```
docs(readme): Update quick start guide with new examples
```

### Scope Guidelines

Use the crate name as scope when applicable:
- `core`, `model`, `session`, `agent`, `runner`, `server`, `tool`, `macros`, `artifact`, `memory`, `telemetry`

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass**:
   ```bash
   make test
   ```

2. **Run clippy** and fix warnings:
   ```bash
   make clippy
   ```

3. **Format your code**:
   ```bash
   make fmt
   ```

4. **Update documentation** if needed

5. **Add tests** for new features

### PR Guidelines

1. **One feature per PR** - Keep PRs focused
2. **Descriptive title** - Use conventional commit format
3. **Clear description** - Explain what and why
4. **Link issues** - Reference related issues
5. **Update CHANGELOG** - Add entry for notable changes

### PR Template

```markdown
## Description
Brief description of changes

## Motivation
Why is this change needed?

## Changes
- List of specific changes made

## Testing
How was this tested?

## Checklist
- [ ] Tests pass (`make test`)
- [ ] Clippy passes (`make clippy`)
- [ ] Code is formatted (`make fmt`)
- [ ] Documentation updated
- [ ] CHANGELOG updated (if applicable)
```

### Review Process

1. **Automated checks** must pass (tests, clippy, formatting)
2. **At least one approval** required for merge
3. **Address review comments** before merge
4. **Squash commits** if requested

## Release Process

### Version Bumping

ZDK follows [Semantic Versioning](https://semver.org/):
- `MAJOR`: Breaking changes
- `MINOR`: New features (backward compatible)
- `PATCH`: Bug fixes

### Pre-release Checklist

- [ ] All tests passing
- [ ] Documentation updated
- [ ] CHANGELOG updated
- [ ] Version bumped in all `Cargo.toml` files
- [ ] Examples tested
- [ ] README reflects current state

## Getting Help

- **Documentation**: Check `docs/` directory
- **Examples**: Look at `examples/` directory
- **Issues**: Search existing issues or create a new one
- **Discussions**: Start a discussion for questions

## Project-Specific Guidelines

### Crate Dependencies

- Minimize dependencies between crates
- Use `pub(crate)` for internal-only items
- Keep crates focused and modular

### Feature Flags

When adding new features, consider using feature flags:
```toml
[features]
default = ["sqlite"]
sqlite = ["sqlx/sqlite"]
postgres = ["sqlx/postgres"]
```

### Performance

- Profile before optimizing
- Use `cargo bench` for benchmarks (run directly with cargo)
- Consider zero-copy patterns with `Cow` when needed

### Security

- Never commit secrets or API keys
- Use environment variables for sensitive data
- Review security implications of changes

## Additional Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Tokio Tutorial](https://tokio.rs/tokio/tutorial)

## Quick Reference Card

### Essential Make Commands

```bash
# Daily workflow
make test          # Run all tests
make check         # Fast check without building  
make fmt           # Format code
make clippy        # Lint code

# Building
make build         # Build all crates
make release       # Build optimized release

# Documentation
make doc           # Generate and open docs

# Examples
make example-NAME  # Run specific example (e.g., make example-quickstart)

# Help
make help          # See all available commands
```

### Cargo Aliases (Advanced)

For quick access, use configured aliases in `.cargo/config.toml`:

```bash
cargo t    # cargo test --workspace
cargo c    # cargo check --workspace
cargo b    # cargo build --workspace
cargo cl   # cargo clippy --workspace
```

## License

By contributing to ZDK, you agree that your contributions will be licensed under the Apache 2.0 License.

---

Thank you for contributing to ZDK! 🦀🚀

