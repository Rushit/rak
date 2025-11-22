# RAK Testing Guide

## Test Types

RAK uses multiple types of tests to ensure code quality:

### 1. Unit Tests
Fast, isolated tests using mocks. Run by default with `cargo test`.

```bash
cargo test
```

### 2. Integration Tests
End-to-end tests with mock services. Run by default.

```bash
cargo test --test integration_test
cargo test --test tool_test
cargo test --test workflow_agents_test
```

### 3. Optional Tests (Ignored)
Tests requiring external setup (gcloud auth, API keys, etc.). Run explicitly.

```bash
# Run all ignored tests
cargo test -- --ignored

# Run specific ignored test with output
cargo test openapi_usage_test -- --ignored --nocapture
```

## Using GCloud Authentication in Tests

Many tests can use gcloud authentication for Google Cloud APIs instead of API keys.

### Setup

1. **Install gcloud CLI**
   ```bash
   # macOS
   brew install google-cloud-sdk
   
   # Linux
   curl https://sdk.cloud.google.com | bash
   ```

2. **Authenticate**
   ```bash
   gcloud auth login
   gcloud auth application-default login
   ```

3. **Set Project**
   ```bash
   gcloud config set project YOUR_PROJECT_ID
   ```

### Test Helper Functions

Use the common test utilities in `tests/common.rs`:

```rust
mod common;

#[tokio::test]
#[ignore] // Mark as optional
async fn test_with_gcloud() {
    // Get access token
    let token = common::get_gcloud_access_token()
        .expect("Run: gcloud auth login");
    
    // Get project ID
    let project = common::get_gcloud_project()
        .expect("Run: gcloud config set project PROJECT_ID");
    
    // Use with Gemini model
    let model = GeminiModel::with_bearer_token(
        token,
        "gemini-1.5-flash".to_string(),
        project,
        "us-central1".to_string(),
    );
}
```

## Running Examples

### Standard Examples (with API Keys)

Set API keys in `config.toml` or environment:

```bash
export GOOGLE_API_KEY="your-api-key"
export OPENAI_API_KEY="sk-..."
cargo run --example quickstart
```

### GCloud Auth Examples

Use your gcloud credentials:

```bash
gcloud auth login
gcloud config set project YOUR_PROJECT_ID
cargo run --example gemini_gcloud_usage
```

## Environment Variables

- `GOOGLE_API_KEY` - Gemini API key (for generativelanguage.googleapis.com)
- `OPENAI_API_KEY` - OpenAI API key
- `GCP_LOCATION` - GCP region (default: us-central1)
- `RUST_LOG` - Logging level (debug, info, warn, error)

## Writing Tests

### Unit Test Pattern

```rust
#[tokio::test]
async fn test_something() {
    let mock = MockLLM::new();
    // Test with mock...
}
```

### Optional Integration Test Pattern

```rust
#[tokio::test]
#[ignore] // Only run when explicitly requested
async fn test_real_api() {
    let token = common::get_gcloud_access_token()
        .expect("gcloud auth required");
    // Test with real API...
}
```

### Running Optional Tests

```bash
# Run all tests including ignored
cargo test -- --ignored

# Run specific test with output
cargo test test_real_api -- --ignored --nocapture
```

## Continuous Integration

CI/CD pipelines run:
- All unit tests (fast, no external deps)
- All integration tests with mocks
- **Not** ignored tests (require manual setup)

To run the same tests as CI locally:

```bash
cargo test
```

## Test Organization

```
rak/
├── tests/
│   ├── common.rs              # Shared test utilities (gcloud auth, etc.)
│   ├── integration_test.rs    # E2E integration tests
│   ├── tool_test.rs           # Tool execution tests
│   ├── workflow_agents_test.rs # Workflow agent tests
│   └── openapi_usage_test.rs  # OpenAPI toolset test (ignored)
└── examples/
    ├── quickstart.rs          # Basic usage
    ├── gemini_gcloud_usage.rs # GCloud auth example
    └── ...
```

## Troubleshooting

### "gcloud command not found"

Install gcloud CLI:
```bash
brew install google-cloud-sdk  # macOS
```

### "Failed to get gcloud token"

Authenticate:
```bash
gcloud auth login
gcloud auth application-default login
```

### "No default project set"

Set project:
```bash
gcloud config set project YOUR_PROJECT_ID
```

### "Access token expired"

Tokens expire after 1 hour. Re-authenticate:
```bash
gcloud auth application-default login
```

### Test fails with "API key not found"

Either:
1. Set API key in config: `openai_api_key = "sk-..."` in `config.toml`
2. Set environment variable: `export OPENAI_API_KEY="sk-..."`
3. Use gcloud auth instead (see examples)

## Best Practices

1. **Use mocks for unit tests** - Fast, reliable, no setup required
2. **Mark API tests as ignored** - Use `#[ignore]` for tests needing external setup
3. **Use gcloud auth in examples** - Easier for developers than managing API keys
4. **Document prerequisites** - Clear instructions in test/example comments
5. **Provide helpful error messages** - Guide users to fix auth issues

## See Also

- [GCloud Auth Implementation Guide](plans/20251121_1700_GCLOUD_AUTH_TESTING.md)
- [Contributing Guidelines](CONTRIBUTING.md)
- [Documentation Index](plans/20251119_1435_DOCUMENTATION_INDEX.md)

