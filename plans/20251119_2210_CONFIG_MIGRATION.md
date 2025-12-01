# Configuration System Migration Guide

**Date**: 2025-11-19 22:10  
**Status**: Implementation Complete  
**Breaking Change**: Yes (better practice)

---

## 🔄 What Changed

### Old System (Environment Variables First)
```
Priority: Environment Variables > config.toml
```

### New System (Config Files First) ⭐
```
Priority: config.toml > Environment Variables > Defaults
```

---

## 🎯 Why This Change?

### Benefits

1. **Better Testing** - Separate `config.test.toml` for tests
2. **Multi-Environment** - Easy dev/staging/prod configs
3. **Version Control** - Can commit environment templates
4. **Explicit Configuration** - Clear what each environment uses
5. **No Surprises** - Config file takes precedence over shell variables

### Use Cases Enabled

```
config.toml        # Development (your local work)
config.test.toml   # Tests (CI/CD)
config.prod.toml   # Production (deployed service)
config.staging.toml # Staging environment
```

---

## 📦 New Configuration Module

### Location
`zdk-core/src/config.rs` - Centralized configuration management

### Key Features

```rust
use rak_core::RakConfig;

// Load from config.toml (or auto-discover)
let config = RakConfig::load()?;

// Load from specific file
let config = RakConfig::load_from(Some(Path::new("config.prod.toml")))?;

// Load test configuration
let config = RakConfig::load_test()?;  // Uses config.test.toml

// Get API key with helpful error
let api_key = config.api_key()?;
```

---

## 🔧 What Needs to Change

### 1. Update Examples ✅

**Old Way** (env var only):
```rust
let api_key = env::var("GEMINI_API_KEY")
    .expect("GEMINI_API_KEY must be set");
```

**New Way** (config first):
```rust
use rak_core::RakConfig;

let config = RakConfig::load()?;
let api_key = config.api_key()?;
```

### 2. Configuration Files ✅

**Created**:
- `config.test.toml` - For automated tests
- Updated `config.toml.example` - With new priority docs

**Protected in .gitignore**:
```gitignore
config.toml          # Your actual config
config.test.toml     # Test config
config.prod.toml     # Production config (if you create it)
config.dev.toml      # Dev config (if you create it)
```

### 3. Priority Changes ✅

**New Resolution Order**:

1. **Config File** (highest)
   ```toml
   [model]
   api_key = "key-from-config-file"
   ```

2. **Config File with Env Var Reference**
   ```toml
   [model]
   api_key = "${GEMINI_API_KEY}"  # Resolves to env var
   ```

3. **Environment Variable** (fallback if config missing)
   ```bash
   export GEMINI_API_KEY="key-from-env"
   ```

4. **Default** (lowest)
   ```rust
   // Built-in defaults for non-sensitive values
   ```

---

## 📋 Migration Checklist

### For Examples

- [ ] Update to use `RakConfig::load()`
- [ ] Remove direct `env::var()` calls
- [ ] Add clear error messages if config missing
- [ ] Document required config format

### For Tests

- [ ] Create `config.test.toml` with test values
- [ ] Update tests to use `RakConfig::load_test()`
- [ ] Remove env var dependencies from tests
- [ ] Tests should work without any environment setup

### For Documentation

- [ ] Update README with new config priority
- [ ] Update TESTING_AND_CONFIG.md
- [ ] Update API_KEY_SECURITY.md
- [ ] Add migration guide (this document)

---

## 🚀 Usage Examples

### Development (Local)

**Setup**:
```bash
# Copy example
cp config.toml.example config.toml

# Edit with your key
vim config.toml
```

**config.toml**:
```toml
[model]
# Option 1: Direct value (easy)
api_key = "your-actual-api-key"

# Option 2: Reference env var (flexible)
api_key = "${GEMINI_API_KEY}"

model_name = "gemini-2.0-flash-exp"
```

**Code**:
```rust
use rak_core::RakConfig;

let config = RakConfig::load()?;  // Reads config.toml
let api_key = config.api_key()?;
```

### Testing

**config.test.toml** (committed to repo):
```toml
[model]
provider = "test"
api_key = "test-key-not-used"  # Mocks don't need real keys
model_name = "test-model"
```

**Code**:
```rust
let config = RakConfig::load_test()?;  // Uses config.test.toml
// Or for mocks, config isn't even needed!
```

### Production

**Option 1: Environment-Specific Config** (Recommended)

```bash
# Deploy with config file
cp config.prod.toml config.toml
./zdk-server
```

**config.prod.toml**:
```toml
[model]
api_key = "${GEMINI_API_KEY}"  # Resolved from environment
model_name = "gemini-2.0-flash-exp"

[server]
host = "0.0.0.0"
port = 8080
```

**Option 2: Pure Environment Variables**

If `config.toml` doesn't exist, fallback to env vars:
```bash
export GEMINI_API_KEY="prod-key"
./zdk-server  # Falls back to env vars
```

---

## 🔍 Config Resolution Examples

### Example 1: Config File Wins

```toml
# config.toml
[model]
api_key = "config-file-key"
```

```bash
export GEMINI_API_KEY="env-var-key"
```

**Result**: Uses `"config-file-key"` ← Config file wins!

### Example 2: Config References Env Var

```toml
# config.toml
[model]
api_key = "${GEMINI_API_KEY}"
```

```bash
export GEMINI_API_KEY="env-var-key"
```

**Result**: Uses `"env-var-key"` ← Resolved from env

### Example 3: No Config, Uses Env Var

```bash
# config.toml doesn't exist
export GEMINI_API_KEY="env-var-key"
```

**Result**: Uses `"env-var-key"` ← Fallback to env

### Example 4: Missing API Key

```toml
# config.toml
[model]
# api_key not set
```

```bash
# GEMINI_API_KEY not set
```

**Result**: Clear error message:
```
API key not found. Set it in config.toml:
[model]
api_key = "your-key"

Or set environment variable:
export GEMINI_API_KEY="your-key"
```

---

## 🎓 Best Practices

### DO ✅

1. **Use config files for each environment**
   ```
   config.toml       # Development
   config.test.toml  # Tests
   config.prod.toml  # Production
   ```

2. **Reference env vars in production configs**
   ```toml
   api_key = "${GEMINI_API_KEY}"  # Flexible
   ```

3. **Commit test configs**
   ```bash
   git add config.test.toml  # Safe, no real keys
   ```

4. **Use `.gitignore` for real configs**
   ```gitignore
   config.toml  # Never commit this
   ```

### DON'T ❌

1. **Don't commit real API keys**
   ```toml
   # DON'T commit this:
   api_key = "AIzaSyC..."  # Real key!
   ```

2. **Don't rely solely on environment variables**
   - Harder to manage multiple environments
   - Easy to forget to set
   - Shell-dependent

3. **Don't create config files for every developer**
   - Use `config.toml.example` as template
   - Each developer copies and customizes

---

## 📊 Comparison Table

| Aspect | Old (Env First) | New (Config First) |
|--------|----------------|-------------------|
| **Priority** | Env > Config | Config > Env |
| **Testing** | Hard (need env vars) | Easy (config.test.toml) |
| **Multi-Environment** | Hard | Easy |
| **Debugging** | Unclear what's used | Clear in config file |
| **CI/CD** | Need secrets | Can use config files |
| **Version Control** | Can't commit configs | Can commit templates |

---

## 🔧 Implementation Status

### ✅ Completed

- [x] Create `zdk-core/src/config.rs` module
- [x] Add config loading with priority
- [x] Support `${VAR}` environment variable references
- [x] Create `config.test.toml` for tests
- [x] Update `.gitignore` for config files
- [x] Add helpful error messages
- [x] Support `RakConfig::load_test()` for tests
- [x] Auto-discover config.toml in parent directories

### 📋 TODO (Next Steps)

- [ ] Update all examples to use `RakConfig`
- [ ] Update integration tests to use `config.test.toml`
- [ ] Update documentation (README, guides)
- [ ] Add `cargo run --example config_usage` example
- [ ] Add config validation (schemas)
- [ ] Add config hot-reload (optional)

---

## 📖 Code Examples

### Basic Usage

```rust
use rak_core::RakConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load config (auto-discovers config.toml)
    let config = RakConfig::load()?;
    
    // Get API key with helpful error
    let api_key = config.api_key()?;
    
    // Use configuration
    let model = GeminiModel::new(
        api_key,
        config.model.model_name.clone(),
    );
    
    Ok(())
}
```

### Test Usage

```rust
use rak_core::RakConfig;

#[tokio::test]
async fn test_with_config() {
    // Loads config.test.toml (or falls back to test defaults)
    let config = RakConfig::load_test().unwrap();
    
    // Tests don't need real API keys!
    assert_eq!(config.model.provider, "test");
}
```

### Custom Config File

```rust
use rak_core::RakConfig;
use std::path::Path;

let config = RakConfig::load_from(
    Some(Path::new("config.prod.toml"))
)?;
```

---

## 🚨 Breaking Changes

### What Breaks

If you were relying on environment variables without a config file:

**Old**:
```bash
export GEMINI_API_KEY="key"
cargo run --example quickstart  # Worked
```

**New**:
```bash
export GEMINI_API_KEY="key"
cargo run --example quickstart  # Still works! (fallback)
```

**Actually, nothing breaks!** The new system has fallback to env vars.

### Migration Path

**Phase 1: No Changes Required** (Current)
- Env vars still work as fallback
- Existing examples still work

**Phase 2: Gradual Migration** (Recommended)
- Create `config.toml` for better practice
- Update examples to show config usage
- Documentation shows both methods

**Phase 3: Config-First** (Future)
- Examples prefer config files
- Environment variables as fallback only
- Clearer error messages

---

## ✅ Testing the New System

```bash
# 1. Build with new config module
cargo build --package zdk-core

# 2. Run tests (uses config.test.toml or test defaults)
cargo test --workspace

# 3. Create your config
cp config.toml.example config.toml
# Edit config.toml with your API key

# 4. Run examples (uses config.toml)
cargo run --example quickstart
```

---

## 📚 Related Documentation

- Configuration Guide: `/docs/20251119_2200_TESTING_AND_CONFIG.md`
- API Key Security: `/docs/20251119_2150_API_KEY_SECURITY.md`
- Testing Guide: `/docs/20251119_1425_TESTING_GUIDE.md`

---

**Summary**: Config files now take priority over environment variables, making multi-environment setups easier and testing more reliable. The old way (env vars) still works as fallback!


