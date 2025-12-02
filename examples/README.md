# ZDK Examples

This directory contains end-to-end examples demonstrating various features and capabilities of the ZDK framework.

## The `common.rs` Module

**All examples use `common.rs` to avoid code duplication.** This shared module provides:

- **Configuration**: Load config with helpful error messages
- **Authentication**: Create providers with proper auth
- **Validation**: Consistent validation helpers with exit codes
- **Response handling**: Collect and print responses from streams
- **Error messages**: User-friendly error messages with setup instructions

**Why use common?**
- ✅ **DRY Principle**: Write code once, use everywhere
- ✅ **Consistency**: All examples behave the same way
- ✅ **Maintainability**: Fix bugs in one place
- ✅ **Better UX**: Helpful error messages for users

When writing examples, **always check `common.rs` first** before writing any helper code!

## Running Examples

### Run a Specific Example

```bash
cargo run --example <example_name>
```

For example:
```bash
cargo run --example quickstart
cargo run --example tool_usage
cargo run --example web_tools_usage
```

### Run All Examples

Use the test script to run all examples with proper validation:

```bash
./scripts/test_examples.sh
```

Options:
- `./scripts/test_examples.sh --verbose` - Show full output from each example
- `./scripts/test_examples.sh --help` - See all available options
- `./scripts/test_examples.sh quickstart` - Run a specific example

### Build All Examples

```bash
cargo build --examples
```

## Important: DRY Principle

**All examples MUST use the `common` module** to avoid code duplication. The `common.rs` file provides:

- ✅ Configuration loading with helpful error messages
- ✅ Provider creation and authentication
- ✅ Response collection and streaming
- ✅ Validation helpers with consistent error formatting
- ✅ Exit code handling

**Before writing any boilerplate code, check if it exists in `common.rs` first!**

See the [Common Utilities](#4-common-utilities-follow-dry) section below for full details.

## Available Examples

### Core Examples
- **config_usage** - Configuration and settings management
- **quickstart** - Basic ZDK setup and usage
- **tool_usage** - Working with tools and function calling

### Provider Examples
- **provider_discovery** - Discovering and using different LLM providers
- **multi_capability_usage** - Using multiple provider capabilities
- **embedding_usage** - Working with embeddings
- **gemini_gcloud_usage** - Google Cloud authentication with Gemini
- **openai_usage** - OpenAI provider integration

### Workflow Examples
- **workflow_agents** - Multi-agent workflows and orchestration

### Storage Examples
- **artifact_usage** - Working with artifacts and file storage
- **memory_usage** - Memory management and context
- **database_session** - Database-backed sessions

### Integration Examples
- **database_tools_usage** - Database tools and operations
- **mcp_toolset_usage** - Model Context Protocol (MCP) tools
- **telemetry_usage** - Telemetry and monitoring
- **web_tools_usage** - Web scraping and browser automation

### Server Examples
- **server_usage** - Running ZDK as a server
- **websocket_usage** - WebSocket communication

## Adding New Examples

When creating a new example, follow these guidelines:

### 1. File Structure

Create a new file in `examples/` directory:
```bash
touch examples/my_new_example.rs
```

### 2. Example Template

Use this template structure following DRY principle with `common` module utilities:

```rust
//! Description of what this example demonstrates
//!
//! Prerequisites:
//! - List any required setup (e.g., API keys, services)

use anyhow::Result;

#[path = "common.rs"]
mod common;

#[tokio::main]
async fn main() -> Result<()> {
    // Use common utilities - don't repeat yourself!
    common::print_header("My New Example");
    
    // Load config with helpful error messages
    let config = common::load_config()?;
    common::show_auth_info(&config)?;
    
    // Create provider from config
    let provider = common::create_gemini_model(&config)?;
    
    // Your example implementation here
    run_example(provider).await?;
    
    // Use common validation functions
    common::validation_passed("All checks completed successfully");
    std::process::exit(0);
}

async fn run_example(provider: std::sync::Arc<dyn zdk_core::Provider>) -> Result<()> {
    // 1. Set up your agent/runner
    println!("\n📝 Setting up example...");
    
    // Your setup code here
    
    // 2. Execute your example logic
    println!("\n🚀 Running example...");
    
    // Your execution code here
    
    // 3. Validate results using common helpers
    println!("\n✓ Validating results...");
    
    // Example validations:
    // common::validate_response_not_empty(&response, "agent response");
    // common::validate_response_contains(&response, "expected", "response");
    // common::validate_response_min_length(&response, 20, "response");
    
    // Or fail validation with clear message:
    // if !some_condition {
    //     common::validation_failed("Specific reason for failure");
    // }
    
    Ok(())
}
```

### 3. Key Requirements

#### Exit Codes
Examples **MUST** use proper exit codes:
- **Exit 0**: Example ran successfully and validation passed
- **Exit 1**: Example failed validation or encountered an error

```rust
// Good - Use common validation helpers
common::validate_response_not_empty(&response, "agent response");
common::validation_passed("All checks successful");
std::process::exit(0);

// Good - Explicit failure with context
if !some_check {
    common::validation_failed("Specific reason for failure");  // Exits with code 1
}

// Bad - Just returning without validation
Ok(())  // Don't do this!
```

#### End-to-End Testing
Examples should be **complete, runnable demonstrations**:
- Include all necessary setup and teardown
- Validate the actual behavior, not just that code runs
- Test the full workflow from start to finish
- Include meaningful assertions

```rust
// Good - Use common validation helpers (DRY)
let response = collect_and_print_response(&mut stream, "agent execution").await?;
common::validate_response_min_length(&response, 20, "agent response");
common::validate_response_contains(&response, "expected", "response");
common::validation_passed("All validations passed");

// Bad - Repeating validation logic (violates DRY)
if response.is_empty() {
    eprintln!("❌ VALIDATION FAILED: Response empty");
    std::process::exit(1);
}
// Don't repeat what's already in common!
```

#### Error Handling
- Use `anyhow::Result` for error propagation
- Use `common::validation_failed()` for validation errors (auto-exits with code 1)
- Provide helpful error messages with context

```rust
// Good - Let common module handle error formatting
common::validation_failed("Expected tool was not called");

// Good - Error context with anyhow
let provider = common::create_gemini_model(&config)
    .context("Failed to initialize provider")?;
```

#### Documentation
- Add a doc comment at the top explaining what the example demonstrates
- List any prerequisites (API keys, running services, etc.)
- Include comments explaining non-obvious code
- Add the example to this README

### 4. Common Utilities (Follow DRY!)

**Always use the `common` module** to avoid repeating code across examples:

```rust
#[path = "common.rs"]
mod common;
```

Available utilities:

#### Configuration & Setup
- `load_config()` - Load config with helpful error messages
- `show_auth_info(&config)` - Display authentication status
- `create_gemini_model(&config)` - Create authenticated provider from config
- `print_header("Title")` - Print formatted example header

#### Response Collection
- `collect_text_response(&mut stream, "context")` - Collect text from event stream
- `collect_and_print_response(&mut stream, "context")` - Collect and print text as it arrives

#### Validation Helpers
- `validate_response_not_empty(&response, "context")` - Check response is not empty
- `validate_response_contains(&response, "expected", "context")` - Check for expected text
- `validate_response_min_length(&response, min_len, "context")` - Check minimum length
- `validation_passed("message")` - Print success message
- `validation_failed("message")` - Print error and exit with code 1

#### Tool Checking
- `tool_was_called(&events, "tool_name")` - Check if a tool was invoked

**Example Usage:**

```rust
// Setup (replaces ~15 lines of boilerplate)
let config = common::load_config()?;
let provider = common::create_gemini_model(&config)?;

// Collection (handles errors and formatting)
let response = common::collect_and_print_response(&mut stream, "agent").await?;

// Validation (consistent error messages)
common::validate_response_min_length(&response, 20, "response");
common::validate_response_contains(&response, "expected", "response");

// Success
common::validation_passed("All checks completed");
std::process::exit(0);
```

**Benefits of using `common`:**
- ✅ Consistent error messages across examples
- ✅ Reduces code duplication (DRY principle)
- ✅ Easier to maintain examples
- ✅ Better user experience with helpful setup instructions
- ✅ Automatic exit code handling

### 5. Testing Your Example

Before submitting, test your example:

```bash
# 1. Check for code duplication
# Review your code and replace any repeated logic with common utilities

# 2. Format and lint
cargo fmt
cargo clippy --example my_new_example

# 3. Run directly
cargo run --example my_new_example

# 4. Check exit code (should be 0 on success, 1 on failure)
echo $?

# 5. Test with the test script
./scripts/test_examples.sh my_new_example

# 6. Test with verbose output
./scripts/test_examples.sh my_new_example --verbose
```

**Checklist before submitting:**
- [ ] No code duplication - all common logic uses `common` module
- [ ] Proper validation with `common::validate_*` functions
- [ ] Exits with correct codes (0 for pass, 1 for fail)
- [ ] Passes `cargo clippy` with no warnings
- [ ] Formatted with `cargo fmt`
- [ ] Doc comments explain what the example does
- [ ] Prerequisites listed in doc comments

### 6. Add to Test Script

Add your example to the list in `scripts/test_examples.sh`:

```bash
local examples=(
    # ... existing examples ...
    "my_new_example"
)
```

And add it to the appropriate category in the `get_category()` function.

### 7. Configuration

**Always use `common::load_config()`** instead of writing custom config loading:

```rust
// Good - Use common helper (provides helpful error messages)
let config = common::load_config()?;

// Bad - Don't repeat config loading logic
let config = ZConfig::load()?;  // Missing helpful error messages!
```

The `common::load_config()` function:
- Searches for `config.toml` in current and parent directories
- Provides helpful setup instructions if config is missing
- Supports both API key and gcloud authentication
- Handles environment variable substitution

## Best Practices

1. **Follow DRY principle** - Always use `common` module utilities instead of repeating code
2. **Keep examples focused** - Each example should demonstrate one main concept
3. **Make them runnable** - Examples should work out of the box with minimal setup
4. **Validate everything** - Use `common::validate_*` functions to check expected behavior
5. **Use proper exit codes** - Exit 0 for success, 1 for failure (use `common::validation_passed/failed`)
6. **Handle errors gracefully** - Use `anyhow::Result` and `.context()` for helpful messages
7. **Document prerequisites** - List required setup, API keys, services in doc comments
8. **Clean up resources** - Close connections, clean up temp files
9. **Follow project style** - Use `rustfmt` and pass `clippy` checks

### DRY Examples

```rust
// ✅ GOOD - Uses common utilities
let config = common::load_config()?;
let provider = common::create_gemini_model(&config)?;
let response = common::collect_and_print_response(&mut stream, "agent").await?;
common::validate_response_min_length(&response, 20, "response");
common::validation_passed("Example completed");

// ❌ BAD - Repeats code that exists in common
let config = ZConfig::load().map_err(|e| anyhow!("Config error: {}", e))?;
let provider = config.create_provider()?;
let mut response = String::new();
while let Some(result) = stream.next().await {
    // ... manually collecting response ...
}
if response.len() < 20 {
    eprintln!("❌ VALIDATION FAILED: Response too short");
    std::process::exit(1);
}
println!("✅ Example completed");
```

## Troubleshooting

### Example Times Out
- Check if it requires external services (database, API)
- Ensure API keys are configured
- Add appropriate timeout handling in your code

### Example Fails Validation
- Check the error message with `--verbose` flag
- Verify all prerequisites are met
- Ensure services are running and accessible

### Build Fails
- Run `cargo check` to see detailed errors
- Ensure all dependencies are in `Cargo.toml`
- Run `cargo clippy` to check for common issues

## Contributing

When contributing examples:
1. **Use `common` module** - Check `common.rs` before writing any helper code
2. Follow the template and requirements above
3. Test your example thoroughly (see checklist in section 5)
4. Add clear documentation with prerequisites
5. Update this README (add to example list)
6. Run `./scripts/test_examples.sh` to ensure all examples still work
7. Run `cargo fmt` and `cargo clippy`

### Why DRY Matters

**Before (Violating DRY):** Each example has ~50 lines of boilerplate
```rust
// 20+ lines of config loading
let config = ZConfig::load().map_err(|e| anyhow!("Error: {}", e))?;

// 15+ lines of response collection
let mut response = String::new();
while let Some(result) = stream.next().await {
    match result {
        Ok(event) => { /* ... */ }
        Err(e) => { /* ... */ }
    }
}

// 10+ lines of validation
if response.is_empty() {
    eprintln!("❌ VALIDATION FAILED: Empty response");
    std::process::exit(1);
}
```

**After (Following DRY):** Same functionality in ~3 lines
```rust
let config = common::load_config()?;
let response = common::collect_and_print_response(&mut stream, "agent").await?;
common::validate_response_not_empty(&response, "agent response");
```

**Benefits:**
- 🎯 **Maintainability**: Fix bugs once in `common.rs`, not in 18 examples
- 📚 **Readability**: Examples focus on what they demonstrate, not boilerplate
- 🚀 **Velocity**: Write new examples faster
- ✨ **Consistency**: All examples behave the same way

For more information, see [CONTRIBUTING.md](../CONTRIBUTING.md) in the project root.

