#!/bin/bash
# Qcker Integration Test Script
# This script tests the basic container lifecycle

set -e

echo "=== Qcker Integration Test ==="
echo ""

# Build qcker
echo "1. Building qcker..."
cd "$(dirname "$0")/.."
cargo build --release 2>&1 | tail -5
echo "   Build complete!"
echo ""

# Create test rootfs
echo "2. Preparing test rootfs..."
TEST_ROOTFS="/tmp/qcker-test/rootfs"
mkdir -p "$TEST_ROOTFS"

# Check if we have a rootfs to use
if [ ! -f "$TEST_ROOTFS/bin/sh" ]; then
    echo "   No rootfs found. Creating minimal test rootfs..."
    
    # Create minimal structure
    mkdir -p "$TEST_ROOTFS"/{bin,etc,proc,sys,dev,tmp,root}
    
    # Create a simple test script
    cat > "$TEST_ROOTFS/bin/test.sh" << 'EOF'
#!/bin/sh
echo "Hello from container!"
echo "PID: $$"
echo "User: $(id)"
echo "Hostname: $(hostname)"
EOF
    chmod +x "$TEST_ROOTFS/bin/test.sh"
    
    echo "   Minimal test rootfs created."
else
    echo "   Using existing rootfs."
fi
echo ""

# Test 1: Create container
echo "3. Testing container create..."
CONTAINER_ID=$(./target/release/qcker create --rootfs "$TEST_ROOTFS" --name test-container -- echo "hello" 2>&1 | grep -oP 'Container \K[a-f0-9]+' || echo "test-container")
echo "   Container created: $CONTAINER_ID"
echo ""

# Test 2: Check container state
echo "4. Testing container state..."
./target/release/qcker state "$CONTAINER_ID" 2>&1
echo ""

# Test 3: List containers
echo "5. Testing container list..."
./target/release/qcker ps --all 2>&1
echo ""

# Test 4: Delete container
echo "6. Testing container delete..."
./target/release/qcker delete "$CONTAINER_ID" 2>&1
echo ""

# Test 5: Run container (create + start)
echo "7. Testing container run..."
./target/release/qcker run --rootfs "$TEST_ROOTFS" --name test-run -- /bin/sh -c "echo 'Container executed successfully!'" 2>&1
echo ""

# Cleanup
echo "8. Cleanup..."
./target/release/qcker delete test-run 2>/dev/null || true
echo ""

echo "=== All tests completed! ==="
