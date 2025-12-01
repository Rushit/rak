# GCloud Authentication for Testing

**Date:** 2025-11-21 17:00  
**Type:** IMPLEMENTATION GUIDE  
**Status:** Complete

## Overview

Updated ZDK testing infrastructure to support gcloud authentication for local testing with Google Cloud APIs (Gemini/Vertex AI) instead of requiring hardcoded API keys.

## Changes Made

### 1. Converted OpenAPI Example to Optional Test

**File:** `tests/openapi_usage_test.rs` (moved from `examples/openapi_usage.rs`)

- Converted from example to test with `#[ignore]` attribute
- Now runs only when explicitly requested: `cargo test openapi_usage_test -- --ignored --nocapture`
- Added assertions to verify tool generation works correctly
- Demonstrates OpenAPI toolset generation with various auth methods

### 2. Added GCloud Auth Helper Module

**File:** `tests/common.rs`

Provides helper functions for getting gcloud credentials:

```rust
pub fn get_gcloud_access_token() -> anyhow::Result<String>
pub fn get_gcloud_project() -> anyhow::Result<String>
```

These functions:
- Execute `gcloud` CLI commands to get credentials
- Provide clear error messages if gcloud is not configured
- Can be used in any test that needs GCP authentication

### 3. Enhanced Gemini Model for Bearer Token Auth

**File:** `crates/zdk-model/src/gemini.rs`

Added `GeminiAuth` enum to support two authentication modes:

```rust
pub enum GeminiAuth {
    /// API Key authentication (for generativelanguage.googleapis.com)
    ApiKey(String),
    /// Bearer token authentication (for Vertex AI via gcloud)
    BearerToken(String),
}
```

New constructor for gcloud auth:

```rust
GeminiModel::with_bearer_token(
    access_token: String,
    model_name: String,
    project_id: String,
    location: String,
) -> Self
```

Key improvements:
- Supports both API key (original) and Bearer token authentication
- Automatically constructs Vertex AI endpoint URL
- Properly adds Authorization header for Bearer tokens
- Backward compatible with existing API key usage

### 4. Created Gemini gcloud Usage Example

**File:** `examples/gemini_gcloud_usage.rs`

Complete working example showing:
- How to get gcloud credentials programmatically
- How to create GeminiModel with Bearer token
- How to use it with ZDK agent system
- Error handling and configuration

Run with:
```bash
cargo run --example gemini_gcloud_usage
```

### 5. Updated Project Configuration

**File:** `Cargo.toml`

- Added `[[test]]` entry for `openapi_usage_test`
- Added `[[example]]` entry for `gemini_gcloud_usage`

**File:** `crates/zdk-model/src/lib.rs`

- Exported `GeminiAuth` enum publicly

## Usage Guide

### Running Optional OpenAPI Test

```bash
# Run the openapi usage test (ignored by default)
cargo test openapi_usage_test -- --ignored --nocapture
```

### Using GCloud Auth in Tests

```rust
mod common;

#[tokio::test]
#[ignore] // Optional - requires gcloud setup
async fn test_with_gcloud_auth() {
    let token = common::get_gcloud_access_token()
        .expect("Failed to get gcloud token");
    
    let project = common::get_gcloud_project()
        .expect("Failed to get project");
    
    let model = GeminiModel::with_bearer_token(
        token,
        "gemini-1.5-flash".to_string(),
        project,
        "us-central1".to_string(),
    );
    
    // Use model in test...
}
```

### Running Gemini Example with GCloud

```bash
# Prerequisites
gcloud auth login
gcloud config set project YOUR_PROJECT_ID

# Optional: Set custom location (default: us-central1)
export GCP_LOCATION=us-central1

# Run example
cargo run --example gemini_gcloud_usage
```

## Prerequisites

For tests/examples using gcloud auth:

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

3. **Set Default Project**
   ```bash
   gcloud config set project YOUR_PROJECT_ID
   ```

4. **Enable APIs** (for Vertex AI)
   ```bash
   gcloud services enable aiplatform.googleapis.com
   ```

## Benefits

### For Development
- No need to hardcode API keys in examples/tests
- Uses existing gcloud credentials developers already have
- More secure - credentials managed by gcloud
- Easier onboarding for new developers

### For Testing
- Tests can run locally with real APIs (when needed)
- Optional tests don't break CI/CD if gcloud not configured
- Clear separation between unit tests (mock) and integration tests (real API)

### For Production
- Bearer token pattern matches production GCP authentication
- Easier to understand service account usage
- Token refresh patterns are clearer

## Token Management

**Important:** Access tokens from `gcloud auth print-access-token` expire after 1 hour.

For long-running applications:
- Use service account keys with automatic refresh
- Implement token refresh logic
- Use Google auth libraries (`google-auth` crate)

For testing:
- Tokens valid for 1 hour is usually sufficient
- Re-run `gcloud auth application-default login` if expired

## Migration Path

### Old Pattern (API Key)
```rust
let model = GeminiModel::new(
    api_key,
    "gemini-1.5-flash".to_string(),
);
```

### New Pattern (GCloud Auth)
```rust
let token = common::get_gcloud_access_token()?;
let project = common::get_gcloud_project()?;

let model = GeminiModel::with_bearer_token(
    token,
    "gemini-1.5-flash".to_string(),
    project,
    "us-central1".to_string(),
);
```

**Note:** Old pattern still works! Backward compatible.

## Testing Strategy

### Unit Tests (Mock)
- Use mock LLMs (existing pattern)
- No network calls
- Fast and reliable
- Run in CI/CD always

```rust
#[tokio::test]
async fn test_agent_logic() {
    let mock_llm = Arc::new(MockLLM::new());
    // Test with mock...
}
```

### Integration Tests (Optional, Real API)
- Use `#[ignore]` attribute
- Require gcloud auth
- Test real API integration
- Run manually or in special CI jobs

```rust
#[tokio::test]
#[ignore] // Run with: cargo test -- --ignored
async fn test_real_gemini_api() {
    let token = common::get_gcloud_access_token()?;
    // Test with real API...
}
```

## Future Improvements

1. **Token Refresh**
   - Add automatic token refresh logic
   - Use `google-auth` crate for better credential management

2. **Service Account Support**
   - Add direct service account key support
   - Better for CI/CD environments

3. **Credential Caching**
   - Cache tokens to reduce gcloud calls
   - Share tokens across multiple tests

4. **Configuration**
   - Read GCP project/location from config files
   - Support multiple environments (dev/staging/prod)

## Files Changed

### New Files
- `tests/openapi_usage_test.rs` - Optional test for OpenAPI toolset
- `tests/common.rs` - GCloud auth helper utilities
- `examples/gemini_gcloud_usage.rs` - Example using gcloud auth
- `plans/20251121_1700_GCLOUD_AUTH_TESTING.md` - This document

### Modified Files
- `crates/zdk-model/src/gemini.rs` - Added Bearer token auth support
- `crates/zdk-model/src/lib.rs` - Exported GeminiAuth enum
- `Cargo.toml` - Added test and example entries

### Removed Files
- `examples/openapi_usage.rs` - Converted to test

## Summary

Successfully implemented gcloud authentication support for ZDK testing infrastructure:

✅ OpenAPI example converted to optional test  
✅ GCloud auth helper module created  
✅ Gemini model supports Bearer token authentication  
✅ Working example with gcloud auth  
✅ Backward compatible with existing API key usage  
✅ Documentation and usage guide complete  

Developers can now run tests locally using their gcloud credentials instead of managing API keys, making development and testing easier and more secure.

