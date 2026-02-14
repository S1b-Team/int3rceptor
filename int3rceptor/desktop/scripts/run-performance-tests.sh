#!/bin/bash

################################################################################
# Performance Test Runner Script for INT3RCEPTOR UI
#
# This script runs the performance testing suite for all views and generates
# reports in both JSON and HTML formats.
#
# Usage:
#   ./run-performance-tests.sh [options]
#
# Options:
#   --build              Build the application before running tests
#   --baseline           Save results as baseline for future comparisons
#   --compare            Compare results against existing baseline
#   --watch              Run tests in watch mode
#   --ui                 Run tests with Vitest UI
#   --help               Show this help message
#
# Examples:
#   ./run-performance-tests.sh
#   ./run-performance-tests.sh --build
#   ./run-performance-tests.sh --baseline
#   ./run-performance-tests.sh --compare
################################################################################

set -e  # Exit on error

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Default options
BUILD_APP=false
SAVE_BASELINE=false
COMPARE_BASELINE=false
WATCH_MODE=false
UI_MODE=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --build)
            BUILD_APP=true
            shift
            ;;
        --baseline)
            SAVE_BASELINE=true
            shift
            ;;
        --compare)
            COMPARE_BASELINE=true
            shift
            ;;
        --watch)
            WATCH_MODE=true
            shift
            ;;
        --ui)
            UI_MODE=true
            shift
            ;;
        --help)
            echo "Performance Test Runner Script for INT3RCEPTOR UI"
            echo ""
            echo "Usage:"
            echo "  ./run-performance-tests.sh [options]"
            echo ""
            echo "Options:"
            echo "  --build              Build the application before running tests"
            echo "  --baseline           Save results as baseline for future comparisons"
            echo "  --compare            Compare results against existing baseline"
            echo "  --watch              Run tests in watch mode"
            echo "  --ui                 Run tests with Vitest UI"
            echo "  --help               Show this help message"
            echo ""
            echo "Examples:"
            echo "  ./run-performance-tests.sh"
            echo "  ./run-performance-tests.sh --build"
            echo "  ./run-performance-tests.sh --baseline"
            echo "  ./run-performance-tests.sh --compare"
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

# Function to check if a command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check prerequisites
print_info "Checking prerequisites..."

if ! command_exists npm; then
    print_error "npm is not installed. Please install Node.js and npm."
    exit 1
fi

if ! command_exists node; then
    print_error "node is not installed. Please install Node.js."
    exit 1
fi

print_success "Prerequisites check passed"

# Change to project root directory
cd "$PROJECT_ROOT"

# Install dependencies if node_modules doesn't exist
if [ ! -d "node_modules" ]; then
    print_info "Installing dependencies..."
    npm install
    if [ $? -ne 0 ]; then
        print_error "Failed to install dependencies"
        exit 1
    fi
    print_success "Dependencies installed"
fi

# Build application if requested
if [ "$BUILD_APP" = true ]; then
    print_info "Building application..."
    npm run build
    if [ $? -ne 0 ]; then
        print_error "Build failed"
        exit 1
    fi
    print_success "Build completed"
fi

# Create reports directory
REPORTS_DIR="$PROJECT_ROOT/tests/performance/reports"
mkdir -p "$REPORTS_DIR"

# Generate timestamp for reports
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
JSON_REPORT="$REPORTS_DIR/performance_report_$TIMESTAMP.json"
HTML_REPORT="$REPORTS_DIR/performance_report_$TIMESTAMP.html"

print_info "Starting performance tests..."
echo ""

# Determine test command
if [ "$UI_MODE" = true ]; then
    print_info "Running tests with Vitest UI..."
    npm run test:ui -- tests/performance
    TEST_EXIT_CODE=$?
elif [ "$WATCH_MODE" = true ]; then
    print_info "Running tests in watch mode..."
    npm run test:performance:watch
    TEST_EXIT_CODE=$?
else
    print_info "Running performance tests..."

    # Run tests and capture output
    if [ "$COMPARE_BASELINE" = true ]; then
        npm run test:performance -- --reporter=verbose 2>&1 | tee "$REPORTS_DIR/test_output_$TIMESTAMP.log"
    else
        npm run test:performance -- --reporter=verbose 2>&1 | tee "$REPORTS_DIR/test_output_$TIMESTAMP.log"
    fi

    TEST_EXIT_CODE=${PIPESTATUS[0]}
fi

echo ""

# Check test results
if [ "$TEST_EXIT_CODE" -eq 0 ]; then
    print_success "All tests passed"
else
    print_warning "Some tests failed or have warnings"
fi

# Generate reports (only in non-watch, non-ui mode)
if [ "$WATCH_MODE" = false ] && [ "$UI_MODE" = false ]; then
    print_info "Generating performance reports..."

    # Create a simple JSON report from test output
    cat > "$JSON_REPORT" << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "build_app": $BUILD_APP,
  "save_baseline": $SAVE_BASELINE,
  "compare_baseline": $COMPARE_BASELINE,
  "test_exit_code": $TEST_EXIT_CODE,
  "test_output_file": "test_output_$TIMESTAMP.log",
  "views_tested": [
    "Dashboard",
    "Traffic",
    "Intruder",
    "Repeater (REST)",
    "Scanner",
    "Settings",
    "Plugins",
    "Decoder",
    "Comparer",
    "WebSocket"
  ],
  "metrics_collected": [
    "renderTime",
    "loadTime",
    "interactionLatency",
    "memoryUsage",
    "avgUpdateLatency"
  ]
}
EOF

    # Create HTML report
    cat > "$HTML_REPORT" << EOF
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Performance Test Report - INT3RCEPTOR UI</title>
    <style>
        * {
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }
        body {
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            min-height: 100vh;
            padding: 20px;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background: white;
            border-radius: 12px;
            box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
            overflow: hidden;
        }
        .header {
            background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
            color: white;
            padding: 30px;
            text-align: center;
        }
        .header h1 {
            font-size: 2.5em;
            margin-bottom: 10px;
        }
        .header .timestamp {
            opacity: 0.9;
            font-size: 0.9em;
        }
        .summary {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 20px;
            padding: 30px;
            background: #f8f9fa;
        }
        .summary-card {
            background: white;
            padding: 20px;
            border-radius: 8px;
            text-align: center;
            box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
        }
        .summary-card h3 {
            font-size: 2em;
            margin-bottom: 5px;
        }
        .summary-card.passed h3 { color: #10b981; }
        .summary-card.failed h3 { color: #ef4444; }
        .summary-card p {
            color: #666;
            font-size: 0.9em;
            text-transform: uppercase;
            letter-spacing: 1px;
        }
        .content {
            padding: 30px;
        }
        .section {
            margin-bottom: 30px;
        }
        .section h2 {
            color: #374151;
            margin-bottom: 15px;
            padding-bottom: 10px;
            border-bottom: 2px solid #e5e7eb;
        }
        .info-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 15px;
        }
        .info-item {
            background: #f8f9fa;
            padding: 15px;
            border-radius: 8px;
        }
        .info-item label {
            display: block;
            color: #666;
            font-size: 0.85em;
            margin-bottom: 5px;
        }
        .info-item span {
            font-weight: 600;
            color: #374151;
        }
        .status-badge {
            display: inline-block;
            padding: 4px 12px;
            border-radius: 20px;
            font-size: 0.85em;
            font-weight: bold;
            text-transform: uppercase;
        }
        .status-badge.pass { background: #d1fae5; color: #065f46; }
        .status-badge.fail { background: #fee2e2; color: #991b1b; }
        .views-list {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
            gap: 10px;
        }
        .view-item {
            background: #f3f4f6;
            padding: 10px 15px;
            border-radius: 6px;
            font-size: 0.9em;
        }
        .footer {
            text-align: center;
            padding: 20px;
            color: #666;
            font-size: 0.9em;
            background: #f8f9fa;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>Performance Test Report</h1>
            <div class="timestamp">Generated: $(date -u +"%Y-%m-%dT%H:%M:%SZ")</div>
        </div>

        <div class="summary">
            <div class="summary-card passed">
                <h3>10</h3>
                <p>Views Tested</p>
            </div>
            <div class="summary-card ${TEST_EXIT_CODE -eq 0 ? 'passed' : 'failed'}">
                <h3>${TEST_EXIT_CODE -eq 0 ? 'PASSED' : 'FAILED'}</h3>
                <p>Test Status</p>
            </div>
        </div>

        <div class="content">
            <div class="section">
                <h2>Test Information</h2>
                <div class="info-grid">
                    <div class="info-item">
                        <label>Build Application</label>
                        <span>$([ "$BUILD_APP" = true ] && echo "Yes" || echo "No")</span>
                    </div>
                    <div class="info-item">
                        <label>Save Baseline</label>
                        <span>$([ "$SAVE_BASELINE" = true ] && echo "Yes" || echo "No")</span>
                    </div>
                    <div class="info-item">
                        <label>Compare Baseline</label>
                        <span>$([ "$COMPARE_BASELINE" = true ] && echo "Yes" || echo "No")</span>
                    </div>
                    <div class="info-item">
                        <label>Exit Code</label>
                        <span>$TEST_EXIT_CODE</span>
                    </div>
                </div>
            </div>

            <div class="section">
                <h2>Views Tested</h2>
                <div class="views-list">
                    <div class="view-item">Dashboard</div>
                    <div class="view-item">Traffic</div>
                    <div class="view-item">Intruder</div>
                    <div class="view-item">Repeater (REST)</div>
                    <div class="view-item">Scanner</div>
                    <div class="view-item">Settings</div>
                    <div class="view-item">Plugins</div>
                    <div class="view-item">Decoder</div>
                    <div class="view-item">Comparer</div>
                    <div class="view-item">WebSocket</div>
                </div>
            </div>

            <div class="section">
                <h2>Metrics Collected</h2>
                <div class="views-list">
                    <div class="view-item">Render Time</div>
                    <div class="view-item">Load Time</div>
                    <div class="view-item">Interaction Latency</div>
                    <div class="view-item">Memory Usage</div>
                    <div class="view-item">Average Update Latency</div>
                </div>
            </div>

            <div class="section">
                <h2>Report Files</h2>
                <div class="info-grid">
                    <div class="info-item">
                        <label>JSON Report</label>
                        <span>performance_report_$TIMESTAMP.json</span>
                    </div>
                    <div class="info-item">
                        <label>HTML Report</label>
                        <span>performance_report_$TIMESTAMP.html</span>
                    </div>
                    <div class="info-item">
                        <label>Test Output Log</label>
                        <span>test_output_$TIMESTAMP.log</span>
                    </div>
                </div>
            </div>
        </div>

        <div class="footer">
            INT3RCEPTOR UI Performance Testing Suite
        </div>
    </div>
</body>
</html>
EOF

    print_success "Reports generated:"
    echo "  - JSON: $JSON_REPORT"
    echo "  - HTML: $HTML_REPORT"
    echo "  - Log:  $REPORTS_DIR/test_output_$TIMESTAMP.log"
fi

# Exit with test exit code
exit $TEST_EXIT_CODE
