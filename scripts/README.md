# ZDK Scripts

Utility scripts for ZDK development and testing.

## 📜 Available Scripts

### `test_examples.sh`

Comprehensive testing script for all ZDK examples.

**Usage:**
```bash
# Test all examples
./scripts/test_examples.sh

# Test specific example
./scripts/test_examples.sh quickstart

# Verbose output
./scripts/test_examples.sh --verbose

# Show help
./scripts/test_examples.sh --help
```

**Features:**
- ✅ Tests all examples systematically
- ✅ Categorizes examples (Core, Workflow, Storage, Integration, Server)
- ✅ Colored output for easy reading
- ✅ Timeout handling for long-running examples
- ✅ Detailed error reporting
- ✅ CI/CD friendly

**Exit Codes:**
- `0` - All tests passed
- `1` - Some tests failed
- `2` - Configuration error

**Example Output:**
```
╔════════════════════════════════════════════════════════════════╗
║          ZDK Examples Test Suite                              ║
╚════════════════════════════════════════════════════════════════╝

Checking prerequisites...
✅ config.toml found
✅ cargo found

Testing examples...

--- Core Examples ---
config_usage              : ✅ PASS
quickstart                : ✅ PASS
tool_usage                : ✅ PASS

--- Workflow Examples ---
workflow_agents           : ✅ PASS

--- Storage Examples ---
artifact_usage            : ✅ PASS
memory_usage              : ✅ PASS
database_session          : ✅ PASS

--- Integration Examples ---
telemetry_usage           : ✅ PASS
openapi_usage             : ✅ PASS
web_tools_usage           : ✅ PASS

--- Server Examples ---
websocket_usage           : ⏭️  SKIP (needs external service)

╔════════════════════════════════════════════════════════════════╗
║                         Summary                                ║
╚════════════════════════════════════════════════════════════════╝

✅ Passed:  10 / 11
❌ Failed:  0 / 11
⏭️  Skipped: 1 / 11

🎉 All testable examples passed!
```

## 🔧 Requirements

**Required:**
- Rust toolchain (cargo)
- config.toml (copy from config.toml.example)

**Optional:**
- `timeout` or `gtimeout` for timeout handling
- API keys in config.toml for examples that make real API calls

## 💡 Tips

### For CI/CD

Add to your CI pipeline:
```yaml
- name: Test examples
  run: |
    cp config.toml.example config.toml
    ./scripts/test_examples.sh
```

### For Local Development

```bash
# Quick test before committing
./scripts/test_examples.sh

# Debug a specific example
./scripts/test_examples.sh --verbose quickstart

# Test without API keys (only local examples)
./scripts/test_examples.sh --timeout 5
```

### Adding New Examples

The script automatically detects examples in the `examples/` directory. To add a new example to the categorization, edit the `EXAMPLES` array in `test_examples.sh`:

```bash
EXAMPLES=(
    ["your_new_example"]="Category"
    # ...
)
```

## 📊 Example Categories

- **Core**: Basic functionality (config, quickstart, tools)
- **Workflow**: Agent orchestration patterns
- **Storage**: Artifact and session storage
- **Integration**: External service integration
- **Server**: Client-server examples

## 🐛 Troubleshooting

**"config.toml not found"**
```bash
cp config.toml.example config.toml
# Edit config.toml with your API keys
```

**"cargo not found"**
```bash
# Install Rust: https://rustup.rs/
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

**"Some examples failed"**
```bash
# Run with verbose output to see details
./scripts/test_examples.sh --verbose

# Test specific failing example
./scripts/test_examples.sh --verbose failing_example_name
```

## 📝 Notes

- Examples that require external services (like `websocket_usage`) will be skipped
- Examples that make real API calls may timeout or fail due to rate limits
- Set `--timeout` lower for faster CI runs
- Use `--no-color` for log files or non-terminal output

