#!/bin/bash
# verify-no-libc.sh - Verify binary doesn't use libc
#
# Usage: ./verify-no-libc.sh <binary>
#
# Checks performed:
#   1. Not dynamically linked (ldd)
#   2. No interpreter section (no dynamic linker needed)
#   3. No NEEDED libraries
#   4. No strong undefined symbols
#   5. No GLIBC version references
#   6. Contains direct syscall instructions
#   7. Runtime: no libc/ld-linux files opened
#   8. Runtime: full strace shows no dynamic loader activity

if [ -z "$1" ]; then
    echo "Usage: $0 <binary>"
    echo "Example: $0 ./target/release/app-x1"
    exit 1
fi

BINARY="$1"

if [ ! -f "$BINARY" ]; then
    echo "Error: File not found: $BINARY"
    exit 1
fi

FAILED=0
WARNINGS=0

echo "=== Verifying: $BINARY ==="
echo

# 1. Check not dynamically linked
echo -n "1. Dynamic linking check... "
LDD_OUTPUT=$(ldd "$BINARY" 2>&1)
if echo "$LDD_OUTPUT" | grep -qE "not a dynamic executable|statically linked"; then
    echo "PASS ($(echo "$LDD_OUTPUT" | tr -d '\t'))"
else
    echo "FAIL (dynamically linked)"
    echo "$LDD_OUTPUT" | head -5
    FAILED=1
fi

# 2. Check no INTERP section (no dynamic linker)
echo -n "2. Interpreter (INTERP) check... "
if ! readelf -l "$BINARY" 2>/dev/null | grep -q INTERP; then
    echo "PASS (no INTERP section)"
else
    echo "FAIL (has INTERP section - needs dynamic linker)"
    readelf -l "$BINARY" 2>/dev/null | grep -A1 INTERP
    FAILED=1
fi

# 3. Check no NEEDED libraries
echo -n "3. NEEDED libraries check... "
NEEDED=$(readelf -d "$BINARY" 2>/dev/null | grep NEEDED || true)
if [ -z "$NEEDED" ]; then
    echo "PASS (no NEEDED libraries)"
else
    echo "FAIL (has NEEDED libraries)"
    echo "$NEEDED"
    FAILED=1
fi

# 4. Check no strong undefined symbols
echo -n "4. Undefined symbols check... "
# Get undefined symbols that are not weak and not local
STRONG_UNDEF=$(readelf --dyn-syms "$BINARY" 2>/dev/null | awk '
    /UND/ && !/WEAK/ && !/LOCAL/ && $8 != "" { print $8 }
' || true)
WEAK_UNDEF=$(readelf --dyn-syms "$BINARY" 2>/dev/null | awk '
    /UND/ && /WEAK/ && $8 != "" { print $8 }
' || true)

if [ -z "$STRONG_UNDEF" ]; then
    if [ -z "$WEAK_UNDEF" ]; then
        echo "PASS (no undefined symbols)"
    else
        echo "WARN (weak undefined symbols - acceptable)"
        echo "   Weak symbols: $WEAK_UNDEF"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    echo "FAIL (has strong undefined symbols)"
    echo "   Strong undefined: $STRONG_UNDEF"
    FAILED=1
fi

# 5. Check no GLIBC version references
echo -n "5. GLIBC version references check... "
if ! objdump -T "$BINARY" 2>/dev/null | grep -qi "@glibc"; then
    echo "PASS (no @GLIBC version symbols)"
else
    echo "FAIL (has GLIBC version references)"
    objdump -T "$BINARY" 2>/dev/null | grep -i "@glibc"
    FAILED=1
fi

# 6. Verify direct syscalls exist
echo -n "6. Direct syscall instructions check... "
SYSCALL_COUNT=$(objdump -d "$BINARY" 2>/dev/null | grep -c "syscall" || echo "0")
if [ "$SYSCALL_COUNT" -gt 0 ]; then
    echo "PASS ($SYSCALL_COUNT syscall instructions)"
else
    echo "WARN (no syscall instructions found)"
    WARNINGS=$((WARNINGS + 1))
fi

# 7. Runtime check - no libc files opened
echo -n "7. Runtime library file check... "
LIBC_LOAD=$(strace -e openat "$BINARY" 2>&1 | grep -E "libc|ld-linux|ld.so" || true)
if [ -z "$LIBC_LOAD" ]; then
    echo "PASS (no libc/ld-linux files opened)"
else
    echo "FAIL (opens libc files at runtime)"
    echo "$LIBC_LOAD"
    FAILED=1
fi

# 8. Runtime check - verify no dynamic loading via full strace
echo -n "8. Runtime syscall trace check... "
STRACE_OUTPUT=$(strace "$BINARY" 2>&1)
SYSCALL_COUNT=$(echo "$STRACE_OUTPUT" | grep -cE "^\w+\(" || echo "0")
# Check for signs of dynamic linker activity
DYN_LOAD=$(echo "$STRACE_OUTPUT" | grep -E "openat.*\.so|mmap.*\.so|ld-linux" || true)
if [ -z "$DYN_LOAD" ]; then
    echo "PASS ($SYSCALL_COUNT syscalls, no dynamic loader activity)"
else
    echo "FAIL (dynamic loading detected)"
    echo "$DYN_LOAD"
    FAILED=1
fi

# Summary
echo
echo "========================================"
if [ $FAILED -eq 0 ]; then
    if [ $WARNINGS -eq 0 ]; then
        echo "RESULT: PASS - No libc code used"
    else
        echo "RESULT: PASS with $WARNINGS warning(s)"
    fi
    exit 0
else
    echo "RESULT: FAIL - Binary uses libc"
    exit 1
fi
