# API Key Security Best Practices

**Date**: 2025-11-19 21:50  
**Purpose**: Guide for securely managing API keys in ZDK

---

## 🔒 Security First

**NEVER commit API keys to git!** This document explains how to keep your keys safe.

---

## Quick Start: 3 Steps

### 1. Use Environment Variables (Recommended)

```bash
# Set environment variable
export GEMINI_API_KEY="your-actual-api-key"

# Run your application
cargo run --example quickstart
```

### 2. Use config.toml (Local Development)

```bash
# Copy example file
cp config.toml.example config.toml

# Edit with your keys (config.toml is in .gitignore)
vim config.toml

# Run your application
cargo run --example quickstart
```

### 3. Use .env file (Alternative)

```bash
# Copy example file
cp .env.example .env

# Edit with your keys (.env is in .gitignore)
vim .env

# Load variables
source .env

# Run your application
cargo run --example quickstart
```

---

## ✅ What's Protected

ZDK's `.gitignore` protects these files from being committed:

```gitignore
# Configuration files with API keys
config.toml         # ← Your actual config (never committed)
.env                # ← Your actual env vars (never committed)
.env.local          # ← Local overrides (never committed)
```

Only these example files are in git:
- `config.toml.example` - Template, no real keys
- `.env.example` - Template, no real keys

---

## 🔑 API Keys in ZDK

### Current Keys (Phase 8.3)

| Tool | API Key Required | Notes |
|------|------------------|-------|
| **Gemini Model** | ✅ Yes - `GEMINI_API_KEY` | Required for all LLM operations |
| **GeminiGoogleSearchTool** | ✅ Uses Gemini key | No additional key needed! |
| **GeminiUrlContextTool** | ✅ Uses Gemini key | No additional key needed! |
| **WebScraperTool** | ✅ No key needed | Direct HTTP, no API |
| **OpenAPI Tools** | ⚠️ Depends on API | Each API has its own key |

### Future Keys (Phase 8.3.1+)

| Tool | API Key | Optional |
|------|---------|----------|
| Google Custom Search API | `GOOGLE_CUSTOM_SEARCH_API_KEY` + `GOOGLE_CUSTOM_SEARCH_ENGINE_ID` | Yes |
| Database (PostgreSQL) | `DATABASE_URL` | Yes |
| OpenTelemetry | `OTEL_EXPORTER_OTLP_ENDPOINT` | Yes |

---

## 📋 Best Practices

### ✅ DO

1. **Use environment variables in production**
   ```bash
   export GEMINI_API_KEY="..."
   ```

2. **Use config.toml for local development**
   ```toml
   api_key = "${GEMINI_API_KEY}"  # Reference env var
   ```

3. **Keep example files generic**
   ```toml
   api_key = "your-api-key-here"  # Generic placeholder
   ```

4. **Check .gitignore before committing**
   ```bash
   git status  # Make sure config.toml is NOT listed
   ```

5. **Use secrets management in production**
   - AWS Secrets Manager
   - Google Secret Manager
   - HashiCorp Vault
   - Kubernetes Secrets

### ❌ DON'T

1. **Never commit real API keys**
   ```toml
   # DON'T DO THIS in committed files:
   api_key = "AIzaSyC..."  # ← REAL KEY, NEVER COMMIT!
   ```

2. **Never commit config.toml**
   ```bash
   # If you accidentally stage it:
   git reset config.toml
   ```

3. **Never share keys in issues/PRs**
   - Redact keys in logs
   - Mask keys in screenshots
   - Use placeholders in examples

4. **Never hardcode keys in source**
   ```rust
   // DON'T DO THIS:
   let api_key = "AIzaSyC...";  // ← NEVER HARDCODE!
   
   // DO THIS:
   let api_key = env::var("GEMINI_API_KEY")?;
   ```

---

## 🚨 If You Accidentally Commit a Key

### Immediate Actions

1. **Revoke the key immediately**
   - Gemini: https://aistudio.google.com/app/apikey
   - Generate a new key

2. **Remove from git history** (if just committed)
   ```bash
   # If not pushed yet:
   git reset HEAD~1
   git add .gitignore config.toml.example
   git commit -m "Add config with template"
   
   # If already pushed (DANGEROUS):
   # Contact your team before doing this!
   git filter-branch --force --index-filter \
     "git rm --cached --ignore-unmatch config.toml" \
     --prune-empty --tag-name-filter cat -- --all
   ```

3. **Notify your team**
   - If in a shared repository
   - So others can regenerate their keys

4. **Update .gitignore**
   - Make sure config.toml is listed
   - Commit the updated .gitignore

---

## 🎯 Configuration Priority

ZDK loads configuration in this order (highest priority first):

1. **Environment variables** (highest priority)
   ```bash
   GEMINI_API_KEY="key-from-env"
   ```

2. **config.toml** (if exists)
   ```toml
   api_key = "key-from-config"
   ```

3. **Default values** (lowest priority)
   ```rust
   let api_key = env::var("GEMINI_API_KEY")
       .or_else(|_| load_from_config())
       .expect("GEMINI_API_KEY not found");
   ```

---

## 📖 Loading Keys in Code

### Recommended Pattern

```rust
use std::env;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Try environment variable first
    let api_key = env::var("GEMINI_API_KEY")
        .map_err(|_| anyhow::anyhow!(
            "GEMINI_API_KEY environment variable not set. \
             Set it with: export GEMINI_API_KEY='your-key'"
        ))?;
    
    // Use the key
    let model = GeminiModel::new(api_key, "gemini-2.0-flash-exp".to_string());
    
    Ok(())
}
```

### With dotenvy (Optional)

```rust
use dotenvy::dotenv;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if it exists
    dotenv().ok();  // Ignore error if .env doesn't exist
    
    // Now environment variables are available
    let api_key = env::var("GEMINI_API_KEY")?;
    
    Ok(())
}
```

---

## 🐳 Docker / Container Security

### Using Docker Secrets

```dockerfile
# Dockerfile
FROM rust:1.90 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
# Don't copy config.toml - use env vars or secrets!
COPY --from=builder /app/target/release/zdk-agent /usr/local/bin/
CMD ["zdk-agent"]
```

```bash
# Pass key at runtime
docker run -e GEMINI_API_KEY="your-key" zdk-agent
```

### Using docker-compose

```yaml
# docker-compose.yml
version: '3.8'
services:
  zdk-agent:
    build: .
    env_file:
      - .env  # NOT committed to git
    # Or use environment directly:
    environment:
      - GEMINI_API_KEY=${GEMINI_API_KEY}
```

---

## ☁️ CI/CD Security

### GitHub Actions

```yaml
# .github/workflows/test.yml
name: Tests
on: [push]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
      - name: Run tests
        env:
          GEMINI_API_KEY: ${{ secrets.GEMINI_API_KEY }}
        run: cargo test
```

Set secrets in: Repository Settings → Secrets → Actions

### GitLab CI

```yaml
# .gitlab-ci.yml
test:
  script:
    - cargo test
  variables:
    GEMINI_API_KEY: $GEMINI_API_KEY  # Set in CI/CD Settings
```

---

## ✅ Verification Checklist

Before committing code:

- [ ] config.toml is NOT staged (`git status`)
- [ ] .env is NOT staged
- [ ] No API keys in source code
- [ ] Examples use placeholders or env vars
- [ ] .gitignore includes config.toml and .env
- [ ] Documentation references env vars, not hardcoded keys

---

## 📚 Additional Resources

### Getting API Keys

- **Gemini API Key**: https://aistudio.google.com/app/apikey
- **Google Custom Search**: https://developers.google.com/custom-search/v1/introduction
- **OpenAPI Services**: Check each API's documentation

### Security Best Practices

- **OWASP Secrets Management**: https://cheatsheetseries.owasp.org/cheatsheets/Secrets_Management_Cheat_Sheet.html
- **Git Secrets Tool**: https://github.com/awslabs/git-secrets
- **pre-commit hooks**: https://pre-commit.com/

---

## 🎓 Summary

1. **Use environment variables** - Most secure
2. **Use config.toml locally** - Never commit it
3. **Check .gitignore** - config.toml and .env must be listed
4. **Never hardcode keys** - Always use env vars
5. **Revoke if leaked** - Generate new keys immediately

**Remember**: API keys are like passwords. Treat them with the same security!


