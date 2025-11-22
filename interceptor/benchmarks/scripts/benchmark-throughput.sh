#!/bin/bash

# Int3rceptor Throughput Benchmark
# Measures requests per second and latency

set -e

PROXY_PORT=8080
DURATION=60
CONNECTIONS=400
THREADS=12

echo "========================================="
echo "Int3rceptor Throughput Benchmark"
echo "========================================="
echo "Duration: ${DURATION}s"
echo "Connections: ${CONNECTIONS}"
echo "Threads: ${THREADS}"
echo ""

# Check if wrk is installed
if ! command -v wrk &> /dev/null; then
    echo "Error: wrk is not installed"
    echo "Install: https://github.com/wg/wrk"
    exit 1
fi

# Check if Int3rceptor is running
if ! curl -s -o /dev/null -w "%{http_code}" --proxy http://127.0.0.1:${PROXY_PORT} http://httpbin.org/get | grep -q "200"; then
    echo "Error: Int3rceptor is not running on port ${PROXY_PORT}"
    echo "Start with: ./target/release/interceptor"
    exit 1
fi

echo "✓ Int3rceptor is running"
echo ""

# Create Lua script for wrk
cat > /tmp/proxy-benchmark.lua << 'EOF'
-- Proxy benchmark script for wrk
request = function()
    return wrk.format("GET", "http://httpbin.org/get")
end
EOF

echo "Running benchmark..."
echo ""

# Run wrk benchmark
wrk -t${THREADS} -c${CONNECTIONS} -d${DURATION}s --latency \
    -s /tmp/proxy-benchmark.lua \
    --proxy http://127.0.0.1:${PROXY_PORT} \
    http://httpbin.org/get

# Cleanup
rm /tmp/proxy-benchmark.lua

echo ""
echo "Benchmark complete!"
echo ""
echo "To compare with other tools, run:"
echo "  ./benchmark-comparison.sh"
