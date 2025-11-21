# Testing and Configuration Guide

**Date**: 2025-11-19 22:00  
**Purpose**: Explain how configuration works for tests vs examples vs production

---

## 🔑 Quick Answer: Tests Don't Need config.toml!

**RAK's automated tests use MOCKS, not real APIs.**

| Scenario | Needs config.toml? | Needs API Key? | Why? |
|----------|-------------------|----------------|------|
| **Unit Tests** | ❌ No | ❌ No | Use mocks |
| **Integration Tests** | ❌ No | ❌ No | Use mocks |
| **Doc Tests** | ❌ No | ❌ No | Compile only |
| **Examples** | ✅ Yes | ✅ Yes | Call real APIs |
| **Production** | ✅ Yes | ✅ Yes | Call real APIs |

---

## 🧪 Running Tests (No Configuration Needed)

### 1. Run All Tests
```bash
# No setup needed - tests use mocks!
cargo test --workspace

# Output: ~83+ tests pass, all using mock LLMs
```

**What happens**: Tests use `TestLLM` mock instead of real Gemini API.

### 2. Run Specific Test Suite
```bash
# Run integration tests
cargo test --test integration_test

# Run unit tests for a crate
cargo test --package rak-web-tools

# Run doc tests
cargo test --doc
```

**No API key required** - all tests are deterministic and fast!

---

## 📝 How Tests Work (Mock Pattern)

### Example from integration_test.rs

```rust
// Tests use a mock LLM, NOT real Gemini API
struct TestLLM {
    responses: Vec<String>,  // Predefined responses
}

#[async_trait]
impl LLM for TestLLM {
    async fn generate_content(&self, ...) -> ... {
        // Returns mock responses, no API call!
        yield Ok(LLMResponse {
            content: Some(Content { /* mock data */ }),
            ...
        })
    }
}

// In test:
let test_llm = Arc::new(TestLLM::new(vec![
    "Hello! I'm a test response.",
]));

let agent = LLMAgent::builder()
    .model(test_llm)  // ← Mock, not real Gemini!
    .build()?;
```

**Benefits**:
- ✅ Fast (no network calls)
- ✅ Free (no API charges)
- ✅ Deterministic (same results every time)
- ✅ Works offline
- ✅ No API keys needed

---

## 🚀 Running Examples (Configuration Required)

### Setup: Choose One Method

#### Method 1: Environment Variable (Recommended)
```bash
# Set your API key
export GEMINI_API_KEY="your-actual-api-key"

# Run any example
cargo run --example quickstart
cargo run --example web_tools_usage
```

#### Method 2: config.toml File
```bash
# 1. Copy template
cp config.toml.example config.toml

# 2. Edit config.toml with your key
[model]
api_key = "your-actual-api-key"  # Or "${GEMINI_API_KEY}"

# 3. Run example
cargo run --example quickstart
```

#### Method 3: .env File
```bash
# 1. Create .env file
echo "GEMINI_API_KEY=your-actual-api-key" > .env

# 2. Load it
source .env

# 3. Run example
cargo run --example quickstart
```

---

## 📂 Configuration Priority

RAK loads configuration in this order (highest to lowest priority):

```
1. Environment Variables (GEMINI_API_KEY)
   ↓ if not found
2. config.toml file
   ↓ if not found
3. Error: "GEMINI_API_KEY not set"
```

### Example Code

```rust
use std::env;

// Priority 1: Try environment variable
let api_key = env::var("GEMINI_API_KEY")
    .or_else(|_| {
        // Priority 2: Try config.toml (if implemented)
        load_from_config_file()
    })
    .expect("GEMINI_API_KEY not set");
```

---

## 🎯 When Do You Need config.toml?

### ❌ You DON'T Need It For:

1. **Running Tests**
   ```bash
   cargo test --workspace  # Works without any config!
   ```

2. **Building the Project**
   ```bash
   cargo build --workspace  # No API key needed
   ```

3. **Running Clippy/Fmt**
   ```bash
   cargo clippy
   cargo fmt
   ```

### ✅ You DO Need It For:

1. **Running Examples**
   ```bash
   cargo run --example quickstart  # Needs API key
   ```

2. **Running the Server**
   ```bash
   cargo run --bin rak-server  # Needs API key
   ```

3. **Production Deployment**
   - Use environment variables (preferred)
   - Or config.toml (if you prefer)

---

## 🔒 Security: config.toml vs .gitignore

### What's Protected

```gitignore
# In .gitignore (never committed):
config.toml      # ← Your actual config with real API keys
.env             # ← Your environment variables
.env.local       # ← Local overrides
```

### What's In Git

```
# In git (safe to commit):
config.toml.example  # ← Template with placeholders
.env.example         # ← Template with placeholders
```

### Verification

```bash
# Check what Git sees
git status

# config.toml should NOT appear here!
# If it does, you need to:
git reset config.toml
```

---

## 📋 Configuration for Different Scenarios

### Scenario 1: Local Development (Examples)

**Best Practice**: Use environment variable

```bash
# Add to your shell profile (~/.bashrc or ~/.zshrc)
export GEMINI_API_KEY="your-key"

# Now examples work automatically
cargo run --example web_tools_usage
```

### Scenario 2: Testing a PR

**No configuration needed!**

```bash
# Just clone and test
git clone <repo>
cd rak/rak
cargo test --workspace  # Works immediately!
```

### Scenario 3: CI/CD (GitHub Actions)

**Use repository secrets**

```yaml
# .github/workflows/test.yml
jobs:
  test:
    steps:
      - run: cargo test --workspace
        # No API key needed for tests!
      
      - run: cargo build --example quickstart
        # Also no API key needed to build!
```

### Scenario 4: Production Deployment

**Use environment variables** (most secure)

```bash
# Set in production environment
export GEMINI_API_KEY="prod-key"
export DATABASE_URL="postgresql://..."

# Run your service
cargo run --release
```

---

## 🧪 Creating New Tests

### For New Features

**Always use mocks for tests!**

```rust
// ✅ GOOD: Use a mock
#[tokio::test]
async fn test_agent_response() {
    let mock_llm = Arc::new(TestLLM::new(vec![
        "Expected response",
    ]));
    
    let agent = LLMAgent::builder()
        .model(mock_llm)  // Mock, not real
        .build()
        .unwrap();
    
    // Test passes without API key!
}

// ❌ BAD: Using real API in tests
#[tokio::test]
async fn test_agent_response() {
    let api_key = env::var("GEMINI_API_KEY")?;  // ← BAD!
    let real_model = GeminiModel::new(api_key, ...);
    // This requires API key and makes real API calls
}
```

### For New Examples

**Examples should show real usage**

```rust
// examples/my_example.rs
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Examples can use real APIs
    let api_key = env::var("GEMINI_API_KEY")
        .expect("GEMINI_API_KEY must be set");
    
    let model = GeminiModel::new(api_key, "gemini-2.0-flash-exp");
    // ... rest of example
    
    Ok(())
}
```

---

## 🎓 Best Practices Summary

### For Tests
1. ✅ **Always use mocks** - No API keys
2. ✅ **Make them fast** - No network calls
3. ✅ **Make them deterministic** - Predictable results
4. ❌ **Never require API keys** - Tests should work offline

### For Examples
1. ✅ **Use real APIs** - Show actual usage
2. ✅ **Require API keys** - via env vars
3. ✅ **Document requirements** - Clear setup instructions
4. ✅ **Provide clear errors** - If API key missing

### For Configuration
1. ✅ **Environment variables** - Production & CI/CD
2. ✅ **config.toml** - Local development convenience
3. ✅ **Keep in .gitignore** - Never commit real keys
4. ✅ **Provide examples** - config.toml.example

---

## 📊 Configuration File Contents

### What's in config.toml.example (Template)

```toml
[model]
api_key = "${GEMINI_API_KEY}"  # References env var (safe)
# OR
# api_key = "your-api-key-here"  # Placeholder (safe)

[web_tools]
# NO additional keys needed! Uses [model] api_key above
```

### What's in your config.toml (Private)

```toml
[model]
api_key = "AIzaSyC..."  # Your actual key (NEVER COMMIT!)
# OR
api_key = "${GEMINI_API_KEY}"  # Still references env var (safer)
```

**Remember**: config.toml is in .gitignore and won't be committed!

---

## ❓ FAQ

### Q: Do I need config.toml to run tests?
**A**: No! Tests use mocks and don't need API keys.

### Q: Do I need config.toml to run examples?
**A**: You need an API key (via env var or config.toml), but env var is easier.

### Q: What if I don't have a config.toml file?
**A**: That's fine! Just use environment variables:
```bash
export GEMINI_API_KEY="your-key"
```

### Q: Can I commit config.toml?
**A**: NO! It's in .gitignore. Only commit config.toml.example.

### Q: How do I know if my key is working?
**A**: Run an example:
```bash
export GEMINI_API_KEY="your-key"
cargo run --example quickstart
# If it works, your key is valid!
```

### Q: What about other API keys (for web tools)?
**A**: Web tools need ZERO additional keys! They use your Gemini key.

---

## ✅ Quick Reference

```bash
# Testing (no config needed)
cargo test --workspace              # Run all tests
cargo test --package rak-web-tools  # Test specific crate

# Examples (needs API key)
export GEMINI_API_KEY="your-key"   # Set key first
cargo run --example web_tools_usage # Then run

# Check configuration
git status                          # config.toml should NOT appear
cat .gitignore                      # Should include config.toml

# Verify your key works
cargo run --example quickstart      # Should run without errors
```

---

## 📚 Related Documentation

- API Key Security: `/docs/20251119_2150_API_KEY_SECURITY.md`
- Testing Guide: `/docs/20251119_1425_TESTING_GUIDE.md`
- Web Tools: `/docs/20251119_2130_PHASE8.3_WEB_TOOLS_COMPLETE.md`

---

**Summary**: Tests don't need config.toml (they use mocks). Examples do need API keys (via env vars or config.toml). Never commit config.toml with real keys!


