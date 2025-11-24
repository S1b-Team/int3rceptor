#!/bin/bash
# Benchmark Concurrency for Int3rceptor
# Uses 'wrk' to test high concurrency handling

PROXY_HOST="127.0.0.1"
PROXY_PORT="8080"
TARGET_URL="http://httpbin.org/get"

# Benchmark settings
THREADS=4
CONNECTIONS_LIST=(100 500 1000 2000)
DURATION=30

echo "=========================================="
echo "Int3rceptor Concurrency Benchmark"
echo "=========================================="

# Check if Int3rceptor is running
if ! nc -z $PROXY_HOST $PROXY_PORT; then
    echo "Error: Int3rceptor is not running on $PROXY_HOST:$PROXY_PORT"
    echo "Please start it first."
    exit 1
fi

# Check for wrk
if ! command -v wrk &> /dev/null; then
    echo "Error: 'wrk' is not installed."
    exit 1
fi

for CONNS in "${CONNECTIONS_LIST[@]}"; do
    echo ""
    echo "Testing with $CONNS concurrent connections..."
    
    wrk -t${THREADS} -c${CONNS} -d${DURATION}s --latency \
        --proxy http://${PROXY_HOST}:${PROXY_PORT} \
        $TARGET_URL
        
    echo "------------------------------------------"
    sleep 5
done

echo "Benchmark complete."
