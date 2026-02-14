#!/bin/bash

###############################################################################
# Visual Regression Test Runner for INT3RCEPTOR UI
# This script runs visual regression tests using Playwright
###############################################################################

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Default values
UPDATE_BASELINES=false
COMPARE_MOCKUPS=false
GENERATE_REPORT=false
HEADLESS=true
BROWSER="chromium"
PROJECT="chromium"
VERBOSE=false

# Function to print usage
usage() {
    cat << EOF
Usage: $0 [OPTIONS]

Visual Regression Test Runner for INT3RCEPTOR UI

OPTIONS:
    --update-baselines    Update baseline screenshots instead of comparing
    --compare-mockups     Compare screenshots against approved mockups
    --generate-report     Generate HTML diff report after tests
    --headed               Run tests in headed mode (show browser)
    --browser BROWSER      Browser to use: chromium, firefox, webkit (default: chromium)
    --project PROJECT      Playwright project to run (default: chromium)
    --verbose              Enable verbose output
    --help                 Show this help message

EXAMPLES:
    # Run visual tests against baselines
    $0

    # Update baseline screenshots
    $0 --update-baselines

    # Compare against approved mockups
    $0 --compare-mockups

    # Run tests with report generation
    $0 --generate-report

    # Run tests in headed mode
    $0 --headed

    # Run tests for specific browser
    $0 --browser firefox

EOF
    exit 0
}

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

# Function to run visual tests
run_visual_tests() {
    print_info "Running visual regression tests..."
    
    cd "$PROJECT_ROOT"
    
    local playwright_args=""
    
    if [ "$HEADLESS" = false ]; then
        playwright_args="$playwright_args --headed"
    fi
    
    if [ "$VERBOSE" = true ]; then
        playwright_args="$playwright_args --reporter=list"
    fi
    
    # Set environment variables for test mode
    export PLAYWRIGHT_UPDATE_BASELINES=$UPDATE_BASELINES
    export PLAYWRIGHT_COMPARE_MOCKUPS=$COMPARE_MOCKUPS
    
    # Run Playwright tests
    if [ "$UPDATE_BASELINES" = true ]; then
        print_info "Updating baselines..."
        npx playwright test --project="$PROJECT" --update-snapshots $playwright_args
    elif [ "$COMPARE_MOCKUPS" = true ]; then
        print_info "Comparing against mockups..."
        npx playwright test --project="$PROJECT" --grep="mockup" $playwright_args
    else
        print_info "Running baseline comparison tests..."
        npx playwright test --project="$PROJECT" $playwright_args
    fi
    
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        print_success "All visual tests passed!"
    else
        print_error "Some visual tests failed"
    fi
    
    return $exit_code
}

# Function to generate HTML report
generate_report() {
    print_info "Generating HTML diff report..."
    
    cd "$PROJECT_ROOT"
    
    # Check if Playwright report exists
    if [ -d "playwright-report" ]; then
        print_success "HTML report generated at: playwright-report/index.html"
        
        # Try to open the report if in interactive mode
        if [ -t 1 ] && command_exists python3; then
            print_info "Opening report in browser..."
            python3 -m http.server 9323 --directory playwright-report &
            local server_pid=$!
            sleep 2
            
            if command_exists xdg-open; then
                xdg-open "http://localhost:9323/index.html" 2>/dev/null || true
            elif command_exists open; then
                open "http://localhost:9323/index.html" 2>/dev/null || true
            fi
            
            print_info "Press Ctrl+C to stop the report server"
            wait $server_pid
        fi
    else
        print_warning "No Playwright report found. Tests may not have run yet."
    fi
}

# Function to clean up
cleanup() {
    print_info "Cleaning up..."
    # Kill any background processes if needed
    exit 0
}

# Trap cleanup on exit
trap cleanup EXIT INT TERM

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --update-baselines)
            UPDATE_BASELINES=true
            shift
            ;;
        --compare-mockups)
            COMPARE_MOCKUPS=true
            shift
            ;;
        --generate-report)
            GENERATE_REPORT=true
            shift
            ;;
        --headed)
            HEADLESS=false
            shift
            ;;
        --browser)
            BROWSER="$2"
            PROJECT="$2"
            shift 2
            ;;
        --project)
            PROJECT="$2"
            shift 2
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            usage
            ;;
        *)
            print_error "Unknown option: $1"
            usage
            ;;
    esac
done

# Main execution
main() {
    echo ""
    echo "=========================================="
    echo "  Visual Regression Test Runner"
    echo "  INT3RCEPTOR UI"
    echo "=========================================="
    echo ""
    
    # Check dependencies
    check_dependencies
    
    # Kill any existing dev server
    kill_dev_server
    
    # Run visual tests
    run_visual_tests
    local test_exit_code=$?
    
    # Generate report if requested
    if [ "$GENERATE_REPORT" = true ]; then
        generate_report
    fi
    
    echo ""
    if [ $test_exit_code -eq 0 ]; then
        print_success "Visual regression testing completed successfully!"
    else
        print_error "Visual regression testing completed with failures"
    fi
    echo ""
    
    exit $test_exit_code
}

# Run main function
main "$@"
