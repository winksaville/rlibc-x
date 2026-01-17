#!/bin/bash
#
# Run rlibc-x2 integration tests
#
# Usage:
#   ./run.sh              # run all tests
#   ./run.sh environ      # run specific test
#   ./run.sh env*         # glob pattern

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

# Build all test binaries
cargo build -p rlibc-x2-tests --release 2>&1 | grep -v "Compiling\|Finished\|Fresh" || true

# Target is in workspace root
TARGET_DIR="$WORKSPACE_ROOT/target/release"

# Pattern to match (default: all)
pattern="${1:-*}"

passed=0
failed=0

for bin in "$TARGET_DIR"/$pattern-tests; do
    [[ -x "$bin" ]] || continue
    name=$(basename "$bin")

    if "$bin" 2>&1; then
        passed=$((passed + 1))
    else
        failed=$((failed + 1))
    fi
done

echo ""
echo "=== Results: $passed passed, $failed failed ==="
[[ $failed -eq 0 ]]
