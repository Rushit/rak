# Quick Start with Google Cloud Authentication

Get started with RAK using Google Cloud authentication in under 2 minutes!

## Prerequisites

- Rust toolchain installed
- gcloud CLI installed ([Install guide](https://cloud.google.com/sdk/docs/install))
- Google Cloud project with Vertex AI API enabled

## Setup (One Time)

### 1. Authenticate with Google Cloud

```bash
gcloud auth login
gcloud config set project YOUR_PROJECT_ID
```

### 2. Clone and Build

```bash
git clone https://github.com/your-org/rak.git
cd rak/rak
cargo build
```

## Run Examples

That's it! All examples now work automatically with your gcloud credentials:

```bash
# Basic agent
make example-quickstart

# Tool calling
make example-tool_usage

# Workflow orchestration
make example-workflow_agents

# Dedicated gcloud auth example
make example-gemini_gcloud_usage
```

Each example will show:
```
✓ Using gcloud authentication
```

## Run Tests

```bash
# All unit tests
make test

# Optional integration tests (with real APIs)
cargo test -- --ignored
```

## Alternative: API Key

If you prefer not to use gcloud auth, you can use API keys:

```bash
# Copy example config
cp config.toml.example config.toml

# Edit config.toml and add your GOOGLE_API_KEY
# or set environment variable:
export GOOGLE_API_KEY="your-api-key-here"

# Run examples
make example-quickstart
```

Output will show:
```
✓ Using API key from config
```

## Available Examples

```bash
make help              # See all available commands

# Core examples
make example-quickstart
make example-config_usage
make example-tool_usage
make example-workflow_agents

# Authentication examples
make example-gemini_gcloud_usage
make example-openai_usage

# Storage & services
make example-artifact_usage
make example-memory_usage
make example-database_session

# Advanced
make example-telemetry_usage
make example-web_tools_usage
make example-websocket_usage
```

## Troubleshooting

### "gcloud command not found"

Install gcloud CLI:
```bash
# macOS
brew install google-cloud-sdk

# Linux
curl https://sdk.cloud.google.com | bash
```

### "No default project set"

Set your project:
```bash
gcloud config set project YOUR_PROJECT_ID
```

### "API not enabled"

Enable Vertex AI API:
```bash
gcloud services enable aiplatform.googleapis.com
```

### Token expired (after 1 hour)

Re-authenticate:
```bash
gcloud auth application-default login
```

## Next Steps

- Read the [Testing Guide](README_TESTING.md) for more details
- Check out [examples/](examples/) for all available examples
- See [plans/](plans/) for implementation details
- Join discussions in GitHub Issues

## Documentation

- [GCloud Auth Implementation](plans/20251121_1800_GCLOUD_AUTH_IMPLEMENTATION_SUMMARY.md)
- [Testing Guide](README_TESTING.md)
- [Documentation Index](plans/20251119_1435_DOCUMENTATION_INDEX.md)

## Getting Help

- **Examples not working?** Run with `--help` flag
- **Build errors?** Run `cargo clean && cargo build`
- **Still stuck?** Open an issue on GitHub

Enjoy building with RAK! 🚀

