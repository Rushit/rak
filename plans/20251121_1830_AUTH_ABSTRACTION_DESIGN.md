# Authentication Abstraction Design

**Created:** 2025-11-21 18:30  
**Last Updated:** 2025-11-21 18:30  
**Status:** Planning  
**Author(s):** RAK Team  
**Type:** DESIGN

## Purpose

Design a clean authentication abstraction that allows users to configure their preferred authentication method (API key or gcloud) in `config.toml`, eliminating the need for programmatic auth detection in every example.

## Current Problems

### 1. Repetitive Code
Every example has this pattern:
```rust
let model = if let Ok(token) = gcloud_helper::get_gcloud_access_token() {
    // 10 lines of gcloud setup
} else {
    // 5 lines of API key setup
};
```

### 2. No User Control
Users can't explicitly choose their auth method - it's always "try gcloud first"

### 3. Configuration Gaps
`config.toml` only supports API key:
```toml
[model]
api_key = "your-key"
model_name = "gemini-1.5-flash"
```

No way to configure gcloud preferences!

## Proposed Solution

### Architecture Overview

```
┌─────────────────────────────────────────────────────┐
│                   config.toml                        │
│  [auth]                                             │
│  provider = "gcloud" | "api_key"                    │
│  [auth.gcloud]                                      │
│    project_id = "my-project"  # optional            │
│    location = "us-central1"   # optional            │
│  [auth.api_key]                                     │
│    key = "${GOOGLE_API_KEY}"  # or direct value     │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│              rak-core/src/auth.rs                   │
│                                                      │
│  pub enum AuthProvider {                            │
│      ApiKey { key: String },                        │
│      GCloud { project: String, location: String },  │
│  }                                                   │
│                                                      │
│  pub trait AuthConfig {                             │
│      fn load() -> Result<AuthProvider>;             │
│      fn create_model(&self) -> Result<Arc<Model>>;  │
│  }                                                   │
└─────────────────────────────────────────────────────┘
                         │
                         ▼
┌─────────────────────────────────────────────────────┐
│              Examples (Simplified)                   │
│                                                      │
│  let config = RakConfig::load()?;                   │
│  let model = config.create_model()?;  // That's it! │
│                                                      │
│  // No more if/else auth detection!                 │
└─────────────────────────────────────────────────────┘
```

## Detailed Design

### 1. Configuration Schema

**File**: `config.toml.example`

```toml
# Authentication Configuration
[auth]
# Choose your authentication provider: "api_key" or "gcloud"
provider = "gcloud"

# API Key Authentication (used when provider = "api_key")
[auth.api_key]
# Can use environment variable or direct value
key = "${GOOGLE_API_KEY}"

# Google Cloud Authentication (used when provider = "gcloud")
[auth.gcloud]
# Optional: If not specified, uses `gcloud config get-value project`
project_id = "my-gcp-project"

# Optional: If not specified, defaults to "us-central1"
location = "us-central1"

# Optional: Custom Vertex AI endpoint
# endpoint = "https://us-central1-aiplatform.googleapis.com"

# Model Configuration
[model]
model_name = "gemini-1.5-flash"

# Optional: Override for specific provider
# For API key: uses generativelanguage.googleapis.com
# For gcloud: uses Vertex AI endpoint
```

**Alternative minimal configs**:

```toml
# Option 1: API Key (simplest)
[auth]
provider = "api_key"

[auth.api_key]
key = "${GOOGLE_API_KEY}"

[model]
model_name = "gemini-1.5-flash"
```

```toml
# Option 2: GCloud (auto-detect project)
[auth]
provider = "gcloud"

# Project and location auto-detected from:
# - gcloud config get-value project
# - Default: us-central1

[model]
model_name = "gemini-1.5-flash"
```

### 2. Core Auth Module

**File**: `crates/rak-core/src/auth.rs` (NEW)

```rust
//! Authentication abstraction for RAK
//!
//! Provides a unified interface for different authentication methods.

use serde::{Deserialize, Serialize};
use std::process::Command;

/// Authentication provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
pub enum AuthProvider {
    /// API Key authentication
    #[serde(rename = "api_key")]
    ApiKey {
        #[serde(flatten)]
        config: ApiKeyAuth,
    },
    
    /// Google Cloud authentication (gcloud)
    #[serde(rename = "gcloud")]
    GCloud {
        #[serde(flatten)]
        config: GCloudAuth,
    },
}

/// API Key authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAuth {
    pub key: String,
}

/// Google Cloud authentication configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCloudAuth {
    /// GCP project ID (optional - auto-detected if not specified)
    pub project_id: Option<String>,
    
    /// GCP location/region (optional - defaults to us-central1)
    pub location: Option<String>,
    
    /// Custom endpoint (optional)
    pub endpoint: Option<String>,
}

impl AuthProvider {
    /// Get credentials for this auth provider
    pub fn get_credentials(&self) -> Result<AuthCredentials> {
        match self {
            AuthProvider::ApiKey { config } => {
                Ok(AuthCredentials::ApiKey {
                    key: config.key.clone(),
                })
            }
            AuthProvider::GCloud { config } => {
                let token = get_gcloud_access_token()?;
                let project = config.project_id.clone()
                    .or_else(|| get_gcloud_project().ok())
                    .ok_or_else(|| Error::Config(
                        "GCloud project not specified and could not auto-detect".into()
                    ))?;
                let location = config.location.clone()
                    .unwrap_or_else(|| "us-central1".to_string());
                
                Ok(AuthCredentials::GCloud {
                    token,
                    project,
                    location,
                    endpoint: config.endpoint.clone(),
                })
            }
        }
    }
}

/// Authentication credentials (resolved from provider)
#[derive(Debug, Clone)]
pub enum AuthCredentials {
    ApiKey {
        key: String,
    },
    GCloud {
        token: String,
        project: String,
        location: String,
        endpoint: Option<String>,
    },
}

// Helper functions (moved from gcloud_helper.rs)
fn get_gcloud_access_token() -> Result<String> {
    let output = Command::new("gcloud")
        .args(&["auth", "print-access-token"])
        .output()
        .map_err(|e| Error::Auth(format!(
            "Failed to run gcloud command. Is gcloud CLI installed? Error: {}", e
        )))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(Error::Auth(format!(
            "gcloud auth failed: {}. Run 'gcloud auth login'", stderr
        )));
    }

    let token = String::from_utf8(output.stdout)
        .map_err(|e| Error::Auth(format!("Invalid gcloud output: {}", e)))?
        .trim()
        .to_string();

    if token.is_empty() {
        return Err(Error::Auth(
            "Empty token from gcloud. Run 'gcloud auth login'".into()
        ));
    }

    Ok(token)
}

fn get_gcloud_project() -> Result<String> {
    let output = Command::new("gcloud")
        .args(&["config", "get-value", "project"])
        .output()
        .map_err(|e| Error::Auth(format!("Failed to get project: {}", e)))?;

    if !output.status.success() {
        return Err(Error::Auth("Failed to get gcloud project".into()));
    }

    let project = String::from_utf8(output.stdout)
        .map_err(|e| Error::Auth(format!("Invalid project output: {}", e)))?
        .trim()
        .to_string();

    if project.is_empty() || project == "(unset)" {
        return Err(Error::Auth(
            "No gcloud project set. Run 'gcloud config set project PROJECT_ID'".into()
        ));
    }

    Ok(project)
}
```

### 3. Enhanced RakConfig

**File**: `crates/rak-core/src/config.rs`

**Add**:
```rust
use crate::auth::{AuthProvider, AuthCredentials};

#[derive(Debug, Deserialize)]
pub struct RakConfig {
    pub auth: AuthProvider,  // NEW: Required auth config
    pub model: ModelConfig,
    // ... existing fields
}

impl RakConfig {
    /// Create a Gemini model using the configured authentication
    pub fn create_gemini_model(&self) -> Result<Arc<GeminiModel>> {
        let credentials = self.auth.get_credentials()?;
        
        match credentials {
            AuthCredentials::ApiKey { key } => {
                Ok(Arc::new(GeminiModel::new(
                    key,
                    self.model.model_name.clone(),
                )))
            }
            AuthCredentials::GCloud { token, project, location, .. } => {
                Ok(Arc::new(GeminiModel::with_bearer_token(
                    token,
                    self.model.model_name.clone(),
                    project,
                    location,
                )))
            }
        }
    }
    
    /// Get authentication credentials (for advanced usage)
    pub fn auth_credentials(&self) -> Result<AuthCredentials> {
        self.auth.get_credentials()
    }
}
```

### 4. Simplified Examples

**Before** (15 lines of auth code):
```rust
let model = if let Ok(token) = gcloud_helper::get_gcloud_access_token() {
    println!("✓ Using gcloud authentication");
    let project = gcloud_helper::get_gcloud_project()?;
    let location = std::env::var("GCP_LOCATION")
        .unwrap_or_else(|_| "us-central1".to_string());
    Arc::new(GeminiModel::with_bearer_token(
        token, "gemini-1.5-flash".to_string(), project, location
    ))
} else {
    println!("✓ Using API key from config");
    let config = RakConfig::load()?;
    let api_key = config.api_key()?;
    Arc::new(GeminiModel::new(api_key, config.model.model_name))
};
```

**After** (2 lines):
```rust
let config = RakConfig::load()?;
let model = config.create_gemini_model()?;
```

**With optional verbose mode**:
```rust
let config = RakConfig::load()?;
println!("✓ Using {} authentication", 
    match &config.auth {
        AuthProvider::ApiKey { .. } => "API key",
        AuthProvider::GCloud { .. } => "gcloud",
    }
);
let model = config.create_gemini_model()?;
```

## Files to Change

### New Files (1)

1. **`crates/rak-core/src/auth.rs`**
   - ~200 lines
   - AuthProvider enum
   - AuthCredentials enum
   - Helper functions
   - Tests

### Modified Files (15)

1. **`crates/rak-core/src/lib.rs`**
   - Add `pub mod auth;`
   - Export auth types

2. **`crates/rak-core/src/config.rs`**
   - Add `auth: AuthProvider` field
   - Add `create_gemini_model()` method
   - Add `auth_credentials()` method
   - Update documentation

3. **`config.toml.example`**
   - Add `[auth]` section
   - Add `[auth.api_key]` section
   - Add `[auth.gcloud]` section
   - Add examples and comments

4. **`config.test.toml`**
   - Add test auth configuration

5. **Examples** (10 files to simplify):
   - `examples/quickstart.rs`
   - `examples/tool_usage.rs`
   - `examples/workflow_agents.rs`
   - `examples/gemini_gcloud_usage.rs` (rename to `gemini_auth_example.rs`?)
   - `examples/server_usage.rs` (new, will use this pattern)
   - `examples/telemetry_usage.rs`
   - `examples/web_tools_usage.rs`
   - `examples/websocket_usage.rs`
   - `examples/memory_usage.rs` (if it uses models)
   - `examples/artifact_usage.rs` (if it uses models)

6. **`examples/_gcloud_helper.rs`**
   - DELETE (functionality moved to rak-core/auth.rs)

7. **`tests/common.rs`**
   - Can reuse `rak_core::auth` functions
   - Or keep as thin wrapper

## Migration Guide

### For Users

**Before** (API key only):
```toml
[model]
api_key = "your-key"
model_name = "gemini-1.5-flash"
```

**After** (API key):
```toml
[auth]
provider = "api_key"

[auth.api_key]
key = "your-key"

[model]
model_name = "gemini-1.5-flash"
```

**After** (gcloud):
```toml
[auth]
provider = "gcloud"

[auth.gcloud]
# Optional fields - auto-detected if omitted
project_id = "my-project"
location = "us-central1"

[model]
model_name = "gemini-1.5-flash"
```

### For Developers

**Before**:
```rust
#[path = "_gcloud_helper.rs"]
mod gcloud_helper;

let model = if let Ok(token) = gcloud_helper::get_gcloud_access_token() {
    // ... 10 lines
} else {
    // ... 5 lines
};
```

**After**:
```rust
let config = RakConfig::load()?;
let model = config.create_gemini_model()?;
```

## Benefits

### ✅ User Experience
1. **Explicit Configuration** - Users choose auth method in config
2. **Better Defaults** - Can set project/location once
3. **No Surprises** - No automatic fallback behavior
4. **Validation** - Config errors caught at load time

### ✅ Code Quality
1. **DRY Principle** - No repeated auth code
2. **Single Responsibility** - Auth logic in one place
3. **Testability** - Easy to test auth without running examples
4. **Maintainability** - One place to update auth logic

### ✅ Flexibility
1. **Easy to Extend** - Add OAuth2, service accounts, etc.
2. **Environment-Specific** - Different config files for dev/prod
3. **Advanced Options** - Custom endpoints, caching, etc.

## Implementation Phases

### Phase 1: Core Infrastructure ✅
1. Create `rak-core/src/auth.rs`
2. Update `rak-core/src/config.rs`
3. Update `config.toml.example`
4. Write tests for auth module

### Phase 2: Example Updates 📝
1. Update all examples to use new pattern
2. Remove `_gcloud_helper.rs`
3. Update documentation in examples

### Phase 3: Server & Advanced 🚀
1. Create `server_usage.rs` example
2. Update websocket examples
3. Update test infrastructure

### Phase 4: Documentation 📚
1. Update README.md
2. Create migration guide
3. Update QUICK_START_GCLOUD.md
4. Update testing documentation

## Advanced Features (Future)

### Token Caching
```rust
impl AuthProvider {
    pub fn with_cache(&self, cache: TokenCache) -> Self {
        // Cache tokens to avoid repeated gcloud calls
    }
}
```

### Service Account Support
```toml
[auth]
provider = "service_account"

[auth.service_account]
key_file = "path/to/service-account.json"
```

### Multiple Profiles
```toml
[profiles.dev]
auth.provider = "gcloud"
auth.gcloud.project_id = "dev-project"

[profiles.prod]
auth.provider = "service_account"
auth.service_account.key_file = "prod-key.json"
```

### Auto-Refresh
```rust
impl AuthProvider {
    pub async fn with_auto_refresh(&self) -> Result<RefreshableAuth> {
        // Automatically refresh expired tokens
    }
}
```

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_api_key_auth() {
    let auth = AuthProvider::ApiKey {
        config: ApiKeyAuth { key: "test".into() }
    };
    let creds = auth.get_credentials().unwrap();
    assert!(matches!(creds, AuthCredentials::ApiKey { .. }));
}

#[test]
fn test_config_parsing() {
    let toml = r#"
        [auth]
        provider = "api_key"
        [auth.api_key]
        key = "test-key"
    "#;
    let config: AuthProvider = toml::from_str(toml).unwrap();
    // ...
}
```

### Integration Tests
```rust
#[tokio::test]
#[ignore] // Requires gcloud setup
async fn test_gcloud_auth_e2e() {
    let config = RakConfig::load().unwrap();
    let model = config.create_gemini_model().unwrap();
    // Actually call API...
}
```

## Breaking Changes

### Config File Format
❌ **BREAKING**: Old config format won't work

**Migration Path**:
1. Detect old format at load time
2. Show helpful error message with migration example
3. Provide `rak config migrate` command (future)

```rust
impl RakConfig {
    pub fn load() -> Result<Self> {
        match config::Config::builder().build() {
            Ok(cfg) => // new format,
            Err(_) => {
                // Try old format
                eprintln!("Old config format detected!");
                eprintln!("Please update your config.toml:");
                eprintln!("{}", MIGRATION_EXAMPLE);
                Err(Error::Config("Old config format".into()))
            }
        }
    }
}
```

### Example API
✅ **NON-BREAKING**: Examples simplified but old patterns still work

## Summary

This design creates a clean, extensible authentication abstraction that:

1. ✅ **Moves auth config to config.toml** - Single source of truth
2. ✅ **Simplifies all examples** - 15 lines → 2 lines
3. ✅ **Gives users control** - Explicit provider selection
4. ✅ **Enables future features** - Service accounts, caching, profiles
5. ✅ **Improves testability** - Auth logic in one place
6. ✅ **Better error messages** - Validate at config load time

**Estimated Effort**: 
- Core implementation: ~2-3 hours
- Example updates: ~1-2 hours
- Testing & docs: ~1-2 hours
- **Total: ~5-7 hours**

**Risk Level**: Low
- Auth module is isolated
- Examples can be updated incrementally
- Old gcloud_helper still works during transition

