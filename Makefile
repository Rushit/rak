# Makefile for RAK (Rust Agent Kit)
# Provides convenient commands for common development tasks
#
# Usage:
#   make              - Run all tests (default)
#   make help         - Show all available commands
#   make example-NAME - Run a specific example

.PHONY: test check build clippy fmt doc clean help test-verbose release
.PHONY: example-quickstart example-tool_usage example-workflow_agents
.PHONY: example-artifact_usage example-database_session example-memory_usage
.PHONY: example-websocket_usage example-telemetry_usage

# Default target - runs all workspace tests
.DEFAULT_GOAL := test

#------------------------------------------------------------------------------
# Primary Development Commands
#------------------------------------------------------------------------------

# Run all workspace tests (72 tests across all crates)
test:
	@echo "Running all workspace tests..."
	@cargo test --workspace

# Check all workspace crates without building (fast)
check:
	@echo "Checking all workspace crates..."
	@cargo check --workspace

# Build all workspace crates
build:
	@echo "Building all workspace crates..."
	@cargo build --workspace

# Run clippy linter on all workspace crates
clippy:
	@echo "Running clippy on all crates..."
	@cargo clippy --workspace -- -D warnings

# Format all code with rustfmt
fmt:
	@echo "Formatting all code..."
	@cargo fmt --all

#------------------------------------------------------------------------------
# Documentation
#------------------------------------------------------------------------------

# Generate and open documentation
doc:
	@echo "Generating documentation..."
	@cargo doc --workspace --no-deps --open

#------------------------------------------------------------------------------
# Testing Variants
#------------------------------------------------------------------------------

# Run tests with debug logging enabled
test-verbose:
	@echo "Running tests with debug logging..."
	@RUST_LOG=debug cargo test --workspace

#------------------------------------------------------------------------------
# Build Variants
#------------------------------------------------------------------------------

# Build release (optimized) binaries
release:
	@echo "Building release binaries..."
	@cargo build --workspace --release

#------------------------------------------------------------------------------
# Examples
#------------------------------------------------------------------------------

# Generic example runner
example-%:
	@echo "Running example: $*"
	@cargo run --example $*

# Test all examples
.PHONY: test-examples
test-examples:
	@echo "Testing all examples..."
	@./scripts/test_examples.sh

# Specific example targets (for autocomplete and documentation)
example-quickstart:
	@echo "Running quickstart example..."
	@cargo run --example quickstart

example-tool_usage:
	@echo "Running tool usage example..."
	@cargo run --example tool_usage

example-workflow_agents:
	@echo "Running workflow agents example..."
	@cargo run --example workflow_agents

example-artifact_usage:
	@echo "Running artifact usage example..."
	@cargo run --example artifact_usage

example-database_session:
	@echo "Running database session example..."
	@cargo run --example database_session

example-memory_usage:
	@echo "Running memory usage example..."
	@cargo run --example memory_usage

example-websocket_usage:
	@echo "Running websocket usage example..."
	@cargo run --example websocket_usage

example-telemetry_usage:
	@echo "Running telemetry usage example..."
	@RUST_LOG=debug cargo run --example telemetry_usage

#------------------------------------------------------------------------------
# Cleanup
#------------------------------------------------------------------------------

# Clean all build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@cargo clean

#------------------------------------------------------------------------------
# Help
#------------------------------------------------------------------------------

help:
	@echo "RAK (Rust Agent Kit) - Development Commands"
	@echo ""
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "PRIMARY COMMANDS"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "  make test          - Run all workspace tests (default, 72 tests)"
	@echo "  make check         - Check all crates without building (fast)"
	@echo "  make build         - Build all workspace crates"
	@echo "  make clippy        - Run clippy linter on all crates"
	@echo "  make fmt           - Format all code with rustfmt"
	@echo "  make doc           - Generate and open documentation"
	@echo ""
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "EXAMPLES (requires GEMINI_API_KEY env var)"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "  make example-quickstart         - Basic agent example"
	@echo "  make example-tool_usage         - Tool calling example"
	@echo "  make example-workflow_agents    - Workflow orchestration"
	@echo "  make example-artifact_usage     - Artifact storage"
	@echo "  make example-database_session   - Database sessions"
	@echo "  make example-memory_usage       - Memory service"
	@echo "  make example-websocket_usage    - WebSocket client"
	@echo "  make example-telemetry_usage    - Telemetry & tracing"
	@echo ""
	@echo "  Generic: make example-NAME      - Run any example"
	@echo ""
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "ADVANCED COMMANDS"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "  make test-verbose  - Run tests with debug logging"
	@echo "  make release       - Build optimized release binaries"
	@echo "  make clean         - Clean all build artifacts"
	@echo ""
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "QUICK START"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo ""
	@echo "  1. Set API key:     export GEMINI_API_KEY=\"your-key\""
	@echo "  2. Run tests:       make test"
	@echo "  3. Run example:     make example-quickstart"
	@echo "  4. Format code:     make fmt"
	@echo "  5. Check code:      make clippy"
	@echo ""
	@echo "For more details, see README.md and CONTRIBUTING.md"
	@echo ""
	@echo "Cargo aliases available:"
	@echo "  cargo t  = cargo test --workspace"
	@echo "  cargo c  = cargo check --workspace"
	@echo "  cargo b  = cargo build --workspace"
	@echo "  cargo cl = cargo clippy --workspace"
	@echo ""

