#!/bin/bash

################################################################################
# Master Test Runner for INT3RCEPTOR UI
#
# This script executes all test suites sequentially and generates a consolidated
# report with all findings.
#
# Usage:
#   ./run-all-tests.sh [options]
#
# Options:
#   --skip-cross-browser    Skip cross-browser tests
#   --skip-a11y             Skip accessibility tests
#   --skip-visual           Skip visual regression tests
#   --skip-perf             Skip performance tests
#   --generate-report       Generate consolidated report after tests
#   --summary-only          Print quick summary to console only
#   --help                  Show this help message
#
# Examples:
#   ./run-all-tests.sh
#   ./run-all-tests.sh --generate-report
#   ./run-all-tests.sh --skip-perf
################################################################################

set -e

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Default options
SKIP_CROSS_BROWSER=false
SKIP_A11Y=false
SKIP_VISUAL=false
SKIP_PERF=false
GENERATE_REPORT=true
SUMMARY_ONLY=false

# Test results tracking
declare -A TEST_RESULTS
declare -A TEST_TIMES
declare -A TEST_OUTPUTS

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-cross-browser)
            SKIP_CROSS_BROWSER=true
            shift
            ;;
        --skip-a11y)
            SKIP_A11Y=true
            shift
            ;;
        --skip-visual)
            SKIP_VISUAL=true
            shift
            ;;
        --skip-perf)
            SKIP_PERF=true
            shift
            ;;
        --generate-report)
            GENERATE_REPORT=true
            shift
            ;;
        --summary-only)
            SUMMARY_ONLY=true
            GENERATE_REPORT=false
            shift
            ;;
        --help|-h)
            cat << EOF
Master Test Runner for INT3RCEPTOR UI

Usage:
  ./run-all-tests.sh [options]

Options:
  --skip-cross-browser    Skip cross-browser tests
  --skip-a11y             Skip accessibility tests
  --skip-visual           Skip visual regression tests
  --skip-perf             Skip performance tests
  --generate-report       Generate consolidated report after tests
  --summary-only          Print quick summary to console only
  --help                  Show this help message

Examples:
  ./run-all-tests.sh
  ./run-all-tests.sh --generate-report
  ./run-all-tests.sh --skip-perf

EOF
            exit 0
            ;;
        *)
            echo -e "${RED}Error: Unknown option $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Function to print colored output
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

print_header() {
    echo ""
    echo -e "${CYAN}$1${NC}"
    echo -e "${CYAN}$(printf '=%.0s' {1..60})${NC}"
}

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to check dependencies
check_dependencies() {
    print_info "Checking dependencies..."

    if ! command_exists node; then
        print_error "Node.js is not installed"
        exit 1
    fi

    if ! command_exists npm; then
        print_error "npm is not installed"
        exit 1
    fi

    # Check if Playwright is installed
    if [ ! -d "$PROJECT_ROOT/node_modules/@playwright/test" ]; then
        print_warning "Playwright not found. Installing dependencies..."
        cd "$PROJECT_ROOT"
        npm install
        npx playwright install --with-deps
    fi

    print_success "All dependencies are installed"
}

# Function to kill existing dev server
kill_dev_server() {
    local port=5173
    local pid=$(lsof -ti:$port 2>/dev/null || true)

    if [ -n "$pid" ]; then
        print_info "Killing existing dev server on port $port (PID: $pid)"
        kill -9 $pid 2>/dev/null || true
        sleep 2
    fi
}

# Function to wait for dev server
wait_for_server() {
    local max_attempts=30
    local attempt=0
    local url="http://localhost:5173"

    print_info "Waiting for dev server to be ready..."

    while [ $attempt -lt $max_attempts ]; do
        if curl -s -o /dev/null -w "%{http_code}" "$url" | grep -q "200\|404"; then
            print_success "Dev server is ready"
            return 0
        fi
        attempt=$((attempt + 1))
        sleep 1
    done

    print_error "Dev server failed to start within $max_attempts seconds"
    return 1
}

# Function to run test suite
run_test_suite() {
    local suite_name="$1"
    local suite_command="$2"
    local suite_dir="$3"

    print_header "Running $suite_name Tests"

    cd "$PROJECT_ROOT"

    # Create output directory for this suite
    local output_dir="$PROJECT_ROOT/test-results/$suite_dir"
    mkdir -p "$output_dir"

    # Create log file
    local log_file="$output_dir/test-output.log"

    # Record start time
    local start_time=$(date +%s)

    # Run the test suite
    print_info "Executing: $suite_command"
    eval "$suite_command" > "$log_file" 2>&1 || true
    local exit_code=$?

    # Record end time
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))

    # Store results
    TEST_RESULTS["$suite_name"]=$exit_code
    TEST_TIMES["$suite_name"]=$duration
    TEST_OUTPUTS["$suite_name"]=$log_file

    # Print result
    if [ $exit_code -eq 0 ]; then
        print_success "$suite_name tests passed (took ${duration}s)"
    else
        print_warning "$suite_name tests failed with exit code $exit_code (took ${duration}s)"
        print_info "Check log: $log_file"
    fi

    return $exit_code
}

# Function to run cross-browser tests
run_cross_browser_tests() {
    if [ "$SKIP_CROSS_BROWSER" = true ]; then
        print_warning "Skipping cross-browser tests"
        TEST_RESULTS["cross-browser"]=0
        TEST_TIMES["cross-browser"]=0
        return 0
    fi

    run_test_suite "Cross-Browser" "npm run test:cross-browser" "cross-browser"
}

# Function to run accessibility tests
run_accessibility_tests() {
    if [ "$SKIP_A11Y" = true ]; then
        print_warning "Skipping accessibility tests"
        TEST_RESULTS["accessibility"]=0
        TEST_TIMES["accessibility"]=0
        return 0
    fi

    run_test_suite "Accessibility" "npm run test:a11y" "accessibility"
}

# Function to run visual regression tests
run_visual_tests() {
    if [ "$SKIP_VISUAL" = true ]; then
        print_warning "Skipping visual regression tests"
        TEST_RESULTS["visual"]=0
        TEST_TIMES["visual"]=0
        return 0
    fi

    run_test_suite "Visual Regression" "npm run test:visual" "visual"
}

# Function to run performance tests
run_performance_tests() {
    if [ "$SKIP_PERF" = true ]; then
        print_warning "Skipping performance tests"
        TEST_RESULTS["performance"]=0
        TEST_TIMES["performance"]=0
        return 0
    fi

    run_test_suite "Performance" "npm run test:perf" "performance"
}

# Function to print summary
print_summary() {
    print_header "Test Execution Summary"

    local total_suites=0
    local passed_suites=0
    local failed_suites=0
    local total_time=0

    echo ""
    printf "%-25s %-10s %-10s\n" "Test Suite" "Status" "Duration"
    printf "%-25s %-10s %-10s\n" "-----------" "------" "--------"

    for suite in "cross-browser" "accessibility" "visual" "performance"; do
        if [ -n "${TEST_RESULTS[$suite]}" ]; then
            local status="${TEST_RESULTS[$suite]}"
            local time="${TEST_TIMES[$suite]}"
            local status_text=""

            if [ $status -eq 0 ]; then
                status_text="${GREEN}PASSED${NC}"
                ((passed_suites++))
            else
                status_text="${RED}FAILED${NC}"
                ((failed_suites++))
            fi

            printf "%-25s %-10s %-10ds\n" "$suite" "$status_text" "$time"
            total_time=$((total_time + time))
            ((total_suites++))
        fi
    done

    echo ""
    printf "%-25s %-10s\n" "Total" "$passed_suites/$total_suites passed"
    printf "%-25s %-10ds\n" "Total Duration" "$total_time"
    echo ""

    if [ $failed_suites -eq 0 ]; then
        print_success "All test suites passed!"
    else
        print_warning "$failed_suites test suite(s) failed"
    fi
}

# Function to generate consolidated report
generate_consolidated_report() {
    print_info "Generating consolidated report..."

    cd "$PROJECT_ROOT"

    # Run the Node.js report generator
    node scripts/generate-report.js

    if [ $? -eq 0 ]; then
        print_success "Consolidated report generated at: test-results/consolidated-report.html"
        print_info "Findings JSON: test-results/findings.json"
    else
        print_error "Failed to generate consolidated report"
    fi
}

# Main execution
main() {
    echo ""
    echo "=========================================="
    echo "  Master Test Runner"
    echo "  INT3RCEPTOR UI"
    echo "=========================================="
    echo ""

    # Check dependencies
    check_dependencies

    # Kill any existing dev server
    kill_dev_server

    # Run test suites in optimal order (fastest first)
    run_cross_browser_tests
    run_accessibility_tests
    run_visual_tests
    run_performance_tests

    # Print summary
    print_summary

    # Generate report if requested
    if [ "$GENERATE_REPORT" = true ]; then
        generate_consolidated_report
    fi

    echo ""
    print_info "Test execution completed"
    echo ""

    # Determine overall exit code
    local overall_exit_code=0
    for suite in "cross-browser" "accessibility" "visual" "performance"; do
        if [ -n "${TEST_RESULTS[$suite]}" ] && [ "${TEST_RESULTS[$suite]}" -ne 0 ]; then
            overall_exit_code=1
            break
        fi
    done

    exit $overall_exit_code
}

# Run main function
main "$@"
