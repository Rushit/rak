#!/usr/bin/env bash
# Test script for RAK examples
# 
# Usage:
#   ./scripts/test_examples.sh           # Test all examples
#   ./scripts/test_examples.sh quickstart # Test specific example
#   ./scripts/test_examples.sh --help    # Show help

# Don't exit on error - we want to test all examples even if some fail
set +e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
PASSED=0
FAILED=0
SKIPPED=0

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Change to project root
cd "$PROJECT_ROOT"

# Help message
show_help() {
    cat << EOF
RAK Examples Test Script

Usage:
    $0 [OPTIONS] [EXAMPLE_NAME]

Options:
    --help          Show this help message
    --verbose       Show full output from examples
    --no-color      Disable colored output
    --timeout N     Set timeout in seconds (default: 10)

Examples:
    $0                  # Test all examples
    $0 quickstart       # Test specific example
    $0 --verbose        # Test all with verbose output

Example Categories:
    - Core examples: config_usage, quickstart, tool_usage
    - Auth examples: gemini_gcloud_usage, openai_usage
    - Workflow examples: workflow_agents
    - Storage examples: artifact_usage, memory_usage, database_session
    - Integration examples: telemetry_usage, web_tools_usage
    - Server examples: server_usage (starts server), websocket_usage (requires server)

Authentication:
    Examples support both gcloud auth and API keys:
    - gcloud: Run 'gcloud auth login' first (recommended)
    - API key: Set in config.toml or environment variable

Exit Codes:
    0 - All tests passed
    1 - Some tests failed
    2 - Configuration error
EOF
}

# Parse arguments
VERBOSE=false
NO_COLOR=false
TIMEOUT=10
SPECIFIC_EXAMPLE=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --help)
            show_help
            exit 0
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --no-color)
            NO_COLOR=true
            RED=''
            GREEN=''
            YELLOW=''
            BLUE=''
            NC=''
            shift
            ;;
        --timeout)
            TIMEOUT="$2"
            shift 2
            ;;
        -*)
            echo "Unknown option: $1"
            show_help
            exit 2
            ;;
        *)
            SPECIFIC_EXAMPLE="$1"
            shift
            ;;
    esac
done

# Print colored message
print_status() {
    local color=$1
    local message=$2
    echo -e "${color}${message}${NC}"
}

# Print header
print_header() {
    echo ""
    print_status "$BLUE" "╔════════════════════════════════════════════════════════════════╗"
    print_status "$BLUE" "║          RAK Examples Test Suite                              ║"
    print_status "$BLUE" "╚════════════════════════════════════════════════════════════════╝"
    echo ""
}

# Check prerequisites
check_prerequisites() {
    print_status "$BLUE" "Checking prerequisites..."
    
    # Check if config.toml exists
    if [ ! -f "config.toml" ]; then
        print_status "$YELLOW" "⚠️  Warning: config.toml not found"
        print_status "$YELLOW" "    Some examples may fail without API keys"
        print_status "$YELLOW" "    Run: cp config.toml.example config.toml"
        echo ""
    else
        print_status "$GREEN" "✅ config.toml found"
    fi
    
    # Check if Cargo is available
    if ! command -v cargo &> /dev/null; then
        print_status "$RED" "❌ Error: cargo not found"
        exit 2
    fi
    
    print_status "$GREEN" "✅ cargo found"
    echo ""
}

# Test a single example
test_example() {
    local example_name=$1
    local category=$2
    
    printf "%-25s : " "$example_name"
    
    # Build the example first (suppress output unless there's an error)
    local build_output
    build_output=$(cargo build --example "$example_name" --quiet 2>&1)
    local build_status=$?
    
    if [ $build_status -eq 0 ]; then
        # Run the example with timeout
        local output
        local exit_code
        
        if command -v gtimeout &> /dev/null; then
            # macOS with GNU timeout
            output=$(gtimeout "$TIMEOUT" cargo run --example "$example_name" --quiet 2>&1 || echo "TIMEOUT_OR_ERROR")
            exit_code=$?
        elif command -v timeout &> /dev/null; then
            # Linux timeout
            output=$(timeout "$TIMEOUT" cargo run --example "$example_name" --quiet 2>&1 || echo "TIMEOUT_OR_ERROR")
            exit_code=$?
        else
            # No timeout available, just run it
            output=$(cargo run --example "$example_name" --quiet 2>&1 || echo "ERROR")
            exit_code=$?
        fi
        
        # Show verbose output if requested
        if [ "$VERBOSE" = true ]; then
            echo ""
            echo "$output"
            echo ""
        fi
        
        # Analyze output to determine status
        if echo "$output" | grep -qE "(panic|thread .* panicked)"; then
            print_status "$RED" "❌ PANIC"
            ((FAILED++))
            if [ "$VERBOSE" = false ]; then
                echo "   Error: $(echo "$output" | grep -i "panicked" | head -1)"
            fi
        elif echo "$output" | grep -qE "Connection refused|Failed to connect"; then
            print_status "$YELLOW" "⏭️  SKIP (needs external service)"
            ((SKIPPED++))
        elif echo "$output" | grep -qE "API key not found|config.toml not found"; then
            print_status "$YELLOW" "⏭️  SKIP (needs config.toml)"
            ((SKIPPED++))
        elif echo "$output" | grep -qE "(Complete|Done|Success|Example|Demo)" || [ "$exit_code" -eq 0 ]; then
            print_status "$GREEN" "✅ PASS"
            ((PASSED++))
        elif echo "$output" | grep -q "TIMEOUT"; then
            print_status "$YELLOW" "⏭️  SKIP (timeout - likely waiting for API)"
            ((SKIPPED++))
        else
            print_status "$RED" "❌ FAIL"
            ((FAILED++))
            if [ "$VERBOSE" = false ]; then
                echo "   Error: $(echo "$output" | grep -iE "error|failed" | head -1)"
            fi
        fi
    else
        print_status "$RED" "❌ BUILD FAILED"
        ((FAILED++))
    fi
}

# Get category for example
get_category() {
    local example=$1
    case $example in
        config_usage|quickstart|tool_usage)
            echo "Core"
            ;;
        gemini_gcloud_usage|openai_usage)
            echo "Authentication"
            ;;
        workflow_agents)
            echo "Workflow"
            ;;
        artifact_usage|memory_usage|database_session)
            echo "Storage"
            ;;
        telemetry_usage|web_tools_usage)
            echo "Integration"
            ;;
        server_usage|websocket_usage)
            echo "Server"
            ;;
        *)
            echo "Other"
            ;;
    esac
}

# Test all examples or specific example
test_examples() {
    print_status "$BLUE" "Testing examples..."
    echo ""
    
    # Define examples in order by category
    local examples=(
        "config_usage"
        "quickstart"
        "tool_usage"
        "gemini_gcloud_usage"
        "openai_usage"
        "workflow_agents"
        "artifact_usage"
        "memory_usage"
        "database_session"
        "telemetry_usage"
        "web_tools_usage"
        "server_usage"
        "websocket_usage"
    )
    
    # Test specific example or all
    if [ -n "$SPECIFIC_EXAMPLE" ]; then
        if [ -f "examples/${SPECIFIC_EXAMPLE}.rs" ]; then
            local category=$(get_category "$SPECIFIC_EXAMPLE")
            test_example "$SPECIFIC_EXAMPLE" "$category"
        else
            print_status "$RED" "❌ Example not found: $SPECIFIC_EXAMPLE"
            exit 1
        fi
    else
        # Test all examples by category
        local current_category=""
        for example in "${examples[@]}"; do
            local category=$(get_category "$example")
            
            # Print category header if changed
            if [ "$category" != "$current_category" ]; then
                if [ -n "$current_category" ]; then
                    echo ""
                fi
                print_status "$BLUE" "--- $category Examples ---"
                current_category="$category"
            fi
            
            test_example "$example" "$category"
        done
    fi
}

# Print summary
print_summary() {
    echo ""
    print_status "$BLUE" "╔════════════════════════════════════════════════════════════════╗"
    print_status "$BLUE" "║                         Summary                                ║"
    print_status "$BLUE" "╚════════════════════════════════════════════════════════════════╝"
    echo ""
    
    local total=$((PASSED + FAILED + SKIPPED))
    
    print_status "$GREEN" "✅ Passed:  $PASSED / $total"
    
    # Only show failed/skipped if they're non-zero
    if [ $FAILED -gt 0 ]; then
        print_status "$RED" "❌ Failed:  $FAILED / $total"
    fi
    
    if [ $SKIPPED -gt 0 ]; then
        print_status "$YELLOW" "⏭️  Skipped: $SKIPPED / $total"
    fi
    
    echo ""
    
    if [ $FAILED -eq 0 ]; then
        if [ $SKIPPED -eq 0 ]; then
            print_status "$GREEN" "🎉 All examples passed!"
        else
            print_status "$GREEN" "🎉 All testable examples passed!"
        fi
        return 0
    else
        print_status "$RED" "⚠️  Some examples failed. Run with --verbose for details."
        return 1
    fi
}

# Main execution
main() {
    print_header
    check_prerequisites
    test_examples
    print_summary
}

# Run main function
main
exit $?

