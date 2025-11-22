# Config-Driven Examples - Complete

**Created:** 2025-11-22 01:00  
**Last Updated:** 2025-11-22 01:00  
**Status:** Complete  
**Author(s):** RAK Team  
**Type:** COMPLETE

## Purpose

Document completion of Phase 3: Converting all RAK examples to use config-driven authentication.

## Summary

All RAK examples now use a shared `common.rs` module that reads authentication from `config.toml`. Users control auth method by editing config, not code.

## What Changed

### Created
- `examples/common.rs` - Shared helper functions (194 lines)
  - `create_gemini_model()` - Creates authenticated models
  - `load_config()` - Loads config with helpful errors
  - `show_auth_info()` - Displays auth details
  - `print_header()` - Consistent styling

### Updated
- `examples/quickstart.rs` - 37% code reduction
- `examples/tool_usage.rs` - 40% code reduction
- `examples/workflow_agents.rs` - 42% code reduction
- `examples/gemini_gcloud_usage.rs` - Rewritten for config-driven approach
- `examples/openai_usage.rs` - Added consistent header
- `Cargo.toml` - Excludes `common.rs` from example builds

### Deleted
- `examples/_gcloud_helper.rs` - No longer needed
- `examples/gcloud_helper.rs` - No longer needed

## Before vs After

### Before (15+ lines per example)
```rust
#[path = "_gcloud_helper.rs"]
mod gcloud_helper;

let model = if let Ok(token) = gcloud_helper::get_gcloud_access_token() {
    let project = gcloud_helper::get_gcloud_project()?;
    let location = std::env::var("GCP_LOCATION")
        .unwrap_or_else(|_| "us-central1".to_string());
    Arc::new(GeminiModel::with_bearer_token(
        token, "gemini-1.5-flash".to_string(), project, location
    ))
} else {
    let config = RakConfig::load()?;
    let api_key = config.api_key()?;
    Arc::new(GeminiModel::new(api_key, config.model.model_name))
};
```

### After (3 lines per example)
```rust
#[path = "common.rs"]
mod common;

let config = common::load_config()?;
let model = common::create_gemini_model(&config)?;
```

## User Experience

Users switch auth methods by editing `config.toml`:

```toml
# Use gcloud
[auth]
provider = "gcloud"

# OR use API key
[auth]
provider = "api_key"
[auth.api_key]
key = "${GOOGLE_API_KEY}"
```

No code changes needed. All examples adapt automatically.

## Benefits

- **Config-Driven**: Users control auth via config, not code
- **Simpler**: ~40% less code per example
- **Consistent**: All examples use same pattern
- **Maintainable**: Update once in `common.rs`, all examples benefit
- **Better Errors**: Clear messages with setup instructions
- **Type-Safe**: Compile-time guarantees via enums

## Testing

All examples compile successfully:
```bash
cargo build --examples
# ✅ Finished `dev` profile [unoptimized + debuginfo]
```

## Related Documents

- [20251121_1830_AUTH_ABSTRACTION_DESIGN.md](20251121_1830_AUTH_ABSTRACTION_DESIGN.md) - Original design
- [20251121_1845_AUTH_ABSTRACTION_CORE_COMPLETE.md](20251121_1845_AUTH_ABSTRACTION_CORE_COMPLETE.md) - Core implementation
- [20251121_1800_GCLOUD_AUTH_IMPLEMENTATION_SUMMARY.md](20251121_1800_GCLOUD_AUTH_IMPLEMENTATION_SUMMARY.md) - Previous auth work

## Next Steps

Phase 4: Documentation & Polish
- Update README with new auth approach
- Add examples to show both auth methods
- Consider adding troubleshooting guide

