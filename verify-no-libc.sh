#!/bin/bash
set -uo pipefail
# verify-no-libc.sh - Verify binary uses no C library (glibc, musl, etc.)
#
# Usage: ./verify-no-libc.sh <binary>
#        ./verify-no-libc.sh --test    # Run self-tests
#
# Checks performed:
#   1. Not dynamically linked (ldd) - informational
#   2. No interpreter section (no dynamic linker needed) - AUTHORITATIVE
#   3. No NEEDED libraries - AUTHORITATIVE
#   4. No strong undefined symbols
#   5. No GLIBC version references
#   6. Contains direct syscall instructions (heuristic)
#   7. Runtime: no libc/ld-linux files opened (diagnostic)
#   8. Runtime: full strace shows no dynamic loader activity (diagnostic)

# Self-test mode
if [ "${1:-}" = "--test" ]; then
    SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
    SELF="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
    PASS_COUNT=0
    FAIL_COUNT=0

    run_test() {
        local binary="$1"
        local expect_pass="$2"
        local name="$3"

        echo -n "Testing $name... "
        if "$SELF" "$binary" >/dev/null 2>&1; then
            if [ "$expect_pass" = "yes" ]; then
                echo "OK (PASS as expected)"
                PASS_COUNT=$((PASS_COUNT + 1))
            else
                echo "FAILED (expected FAIL, got PASS)"
                FAIL_COUNT=$((FAIL_COUNT + 1))
            fi
        else
            if [ "$expect_pass" = "no" ]; then
                echo "OK (FAIL as expected)"
                PASS_COUNT=$((PASS_COUNT + 1))
            else
                echo "FAILED (expected PASS, got FAIL)"
                FAIL_COUNT=$((FAIL_COUNT + 1))
            fi
        fi
    }

    echo "=== verify-no-libc.sh self-test ==="
    echo

    # Test binaries that should PASS (no libc)
    if [ -f "$SCRIPT_DIR/target/debug/ex-x1" ]; then
        run_test "$SCRIPT_DIR/target/debug/ex-x1" "yes" "ex-x1 (debug)"
    fi
    if [ -f "$SCRIPT_DIR/target/release/ex-x1" ]; then
        run_test "$SCRIPT_DIR/target/release/ex-x1" "yes" "ex-x1 (release)"
    fi
    if [ -f "$SCRIPT_DIR/target/debug/ex-x2" ]; then
        run_test "$SCRIPT_DIR/target/debug/ex-x2" "yes" "ex-x2 (debug)"
    fi
    if [ -f "$SCRIPT_DIR/target/release/ex-x2" ]; then
        run_test "$SCRIPT_DIR/target/release/ex-x2" "yes" "ex-x2 (release)"
    fi
    if [ -f "$SCRIPT_DIR/target/debug/hw-x1" ]; then
        run_test "$SCRIPT_DIR/target/debug/hw-x1" "yes" "hw-x1 (debug)"
    fi
    if [ -f "$SCRIPT_DIR/target/release/hw-x1" ]; then
        run_test "$SCRIPT_DIR/target/release/hw-x1" "yes" "hw-x1 (release)"
    fi
    if [ -f "$SCRIPT_DIR/target/debug/hw-x2" ]; then
        run_test "$SCRIPT_DIR/target/debug/hw-x2" "yes" "hw-x2 (debug)"
    fi
    if [ -f "$SCRIPT_DIR/target/release/hw-x2" ]; then
        run_test "$SCRIPT_DIR/target/release/hw-x2" "yes" "hw-x2 (release)"
    fi

    # Test binaries from rlibc-x2/tests
    if [ -f "$SCRIPT_DIR/target/debug/test-environ" ]; then
        run_test "$SCRIPT_DIR/target/debug/test-environ" "yes" "test-environ (debug)"
    fi
    if [ -f "$SCRIPT_DIR/target/release/test-environ" ]; then
        run_test "$SCRIPT_DIR/target/release/test-environ" "yes" "test-environ (release)"
    fi
    if [ -f "$SCRIPT_DIR/target/debug/test-std-env" ]; then
        run_test "$SCRIPT_DIR/target/debug/test-std-env" "yes" "test-std-env (debug)"
    fi
    if [ -f "$SCRIPT_DIR/target/release/test-std-env" ]; then
        run_test "$SCRIPT_DIR/target/release/test-std-env" "yes" "test-std-env (release)"
    fi

    # Test system binaries that should FAIL (use libc)
    run_test "/usr/bin/ls" "no" "/usr/bin/ls"
    run_test "/usr/bin/true" "no" "/usr/bin/true"

    echo
    echo "========================================"
    echo "Tests passed: $PASS_COUNT"
    echo "Tests failed: $FAIL_COUNT"
    if [ $FAIL_COUNT -eq 0 ]; then
        echo "RESULT: ALL TESTS PASSED"
        exit 0
    else
        echo "RESULT: SOME TESTS FAILED"
        exit 1
    fi
fi

if [ -z "${1:-}" ]; then
    echo "Usage: $0 <binary>"
    echo "       $0 --test    # Run self-tests"
    echo "Example: $0 ./target/release/ex-x1"
    exit 1
fi

BINARY="$1"

if [ ! -f "$BINARY" ]; then
    echo "Error: File not found: $BINARY"
    exit 1
fi

# Check required tools (nm is optional - may not work on stripped binaries)
REQUIRED_TOOLS="ldd readelf objdump strace timeout"
TIMEOUT="${TIMEOUT:-5s}"
HAS_NM=0
if command -v nm &>/dev/null; then
    HAS_NM=1
fi
for tool in $REQUIRED_TOOLS; do
    if ! command -v "$tool" &>/dev/null; then
        echo "Error: Required tool '$tool' not found"
        exit 1
    fi
done

FAILED=0
WARNINGS=0

echo "=== Verifying: $BINARY ==="
echo

# 1. Dynamic linking check (informational - ldd can execute code, so checks 2 & 3 are authoritative)
echo -n "1. Dynamic linking check (ldd)... "
LDD_OUTPUT=$(ldd "$BINARY" 2>&1)
if echo "$LDD_OUTPUT" | grep -qE "not a dynamic executable|statically linked"; then
    echo "INFO ($(echo "$LDD_OUTPUT" | tr -d '\t'))"
else
    echo "INFO (ldd reports dynamically linked - see checks 2 & 3)"
    echo "$LDD_OUTPUT" | head -5
fi

# 2. Check no INTERP program header (no dynamic linker needed)
echo -n "2. Interpreter (INTERP) check... "
if ! readelf -lW "$BINARY" 2>/dev/null | grep -q "^ *INTERP"; then
    echo "PASS (no INTERP program header)"
else
    echo "FAIL (has INTERP - needs dynamic linker)"
    readelf -lW "$BINARY" 2>/dev/null | grep -A1 "^ *INTERP"
    FAILED=1
fi

# 3. Check no NEEDED libraries (fully static binaries may have no .dynamic section at all)
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
# Check if dynsym section exists
DYNSYM_OUTPUT=$(readelf --dyn-syms "$BINARY" 2>/dev/null || true)
if [ -z "$DYNSYM_OUTPUT" ] || echo "$DYNSYM_OUTPUT" | grep -q "no dynamic symbol"; then
    echo "PASS (no dynsym section - fully static)"
else
    # Get undefined symbols that are not weak and not local
    # Use $NF (last field) for robustness across readelf versions
    STRONG_UNDEF=$(echo "$DYNSYM_OUTPUT" | awk '
        /UND/ && !/WEAK/ && !/LOCAL/ && NF > 0 { print $NF }
    ' | grep -v '^$' || true)
    WEAK_UNDEF=$(echo "$DYNSYM_OUTPUT" | awk '
        /UND/ && /WEAK/ && NF > 0 { print $NF }
    ' | grep -v '^$' || true)

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
fi

# 5a. Check no GLIBC version references in dynamic symbols
# Note: This only catches dynamically linked glibc; check 5b covers statically linked
echo -n "5a. GLIBC dynamic symbols check... "
if ! objdump -T "$BINARY" 2>/dev/null | grep -qi "@glibc"; then
    echo "PASS (no @GLIBC version symbols)"
else
    echo "FAIL (has GLIBC version references)"
    objdump -T "$BINARY" 2>/dev/null | grep -i "@glibc"
    FAILED=1
fi

# 5b. Check for common glibc entrypoint symbols (heuristic - nm may not work on stripped binaries)
# Defined symbols (T/t) are OK - they're our own implementations
echo -n "5b. GLIBC entrypoint symbols (heuristic)... "
if [ "$HAS_NM" -eq 0 ]; then
    echo "WARN (nm not available)"
    WARNINGS=$((WARNINGS + 1))
else
    NM_OUTPUT=$(nm "$BINARY" 2>&1)
    NM_EXIT=$?
    if [ $NM_EXIT -ne 0 ] || [ -z "$NM_OUTPUT" ] || echo "$NM_OUTPUT" | grep -qi "no symbols"; then
        echo "WARN (nm produced no symbols - binary may be stripped)"
        WARNINGS=$((WARNINGS + 1))
    else
        GLIBC_UNDEF=$(echo "$NM_OUTPUT" | grep " U " | grep -E "__libc_start_main|__glibc|__cxa_atexit" || true)
        if [ -z "$GLIBC_UNDEF" ]; then
            echo "PASS (no undefined glibc entrypoint symbols)"
        else
            echo "FAIL (has undefined glibc symbols)"
            echo "$GLIBC_UNDEF" | head -5
            FAILED=1
        fi
    fi
fi

# 6. Heuristic: syscall instruction presence (vDSO, LTO, or stripping may hide them)
echo -n "6. Syscall instructions (heuristic)... "
# Match syscall instructions at end of line: syscall (x86_64), svc (ARM/AArch64), int $0x80 (x86 32-bit)
SYSCALL_PATTERN='(syscall|svc|int.*0x80)\s*$'
SYSCALL_INSTR_COUNT=$(objdump -d "$BINARY" 2>/dev/null | grep -cE "$SYSCALL_PATTERN" || true)
SYSCALL_INSTR_COUNT=${SYSCALL_INSTR_COUNT:-0}
if [ "$SYSCALL_INSTR_COUNT" -gt 0 ]; then
    echo "PASS ($SYSCALL_INSTR_COUNT syscall instructions)"
else
    echo "WARN (no syscall instructions found - may use vDSO or be LTO'd)"
    WARNINGS=$((WARNINGS + 1))
fi

# 7. Runtime check - no libc files opened (diagnostic - checks 2 & 3 are authoritative)
# Use -f to follow forks, trace=file catches open/openat/openat2/access/stat/etc.
echo -n "7. Runtime library file check... "
STRACE_OUT7=$(timeout "$TIMEOUT" strace -f -e trace=file "$BINARY" 2>&1 >/dev/null)
STRACE_EXIT7=$?
if [ $STRACE_EXIT7 -eq 124 ]; then
    echo "WARN (binary timed out after $TIMEOUT - may be interactive)"
    WARNINGS=$((WARNINGS + 1))
else
    # Tighter patterns: libc.so, ld-linux, ld.so, and common C++ runtime libs
    LIBC_PATTERN='ld-linux|ld\.so|libc\.so|libpthread\.so|libm\.so|libgcc_s\.so|libstdc\+\+\.so'
    LIBC_LOAD=$(echo "$STRACE_OUT7" | grep -E "$LIBC_PATTERN" || true)
    if [ -z "$LIBC_LOAD" ]; then
        echo "PASS (no libc/runtime libraries accessed)"
    else
        echo "FAIL (accesses libc/runtime library files)"
        echo "$LIBC_LOAD" | head -5
        FAILED=1
    fi
fi

# 8. Runtime check - verify no dynamic loading via full strace (diagnostic - checks 2 & 3 are authoritative)
# Use -f to follow forks/execs
echo -n "8. Runtime syscall trace check... "
STRACE_OUT8=$(timeout "$TIMEOUT" strace -f "$BINARY" 2>&1 >/dev/null)
STRACE_EXIT8=$?
if [ $STRACE_EXIT8 -eq 124 ]; then
    echo "WARN (binary timed out after $TIMEOUT - may be interactive)"
    WARNINGS=$((WARNINGS + 1))
else
    RUNTIME_SYSCALL_COUNT=$(echo "$STRACE_OUT8" | grep -cE "^\w+\(" || echo "0")
    # Check for known dynamic loader artifacts:
    # - ld.so.cache (loader cache), ld-linux (dynamic linker)
    # - .so files in /lib/ or /usr/lib/ paths
    DYN_PATTERN='ld\.so\.cache|ld-linux|"/lib/.*\.so|"/lib64/.*\.so|"/usr/lib/.*\.so'
    DYN_LOAD=$(echo "$STRACE_OUT8" | grep -E "$DYN_PATTERN" || true)
    if [ -z "$DYN_LOAD" ]; then
        echo "PASS ($RUNTIME_SYSCALL_COUNT syscalls, no dynamic loader activity)"
    else
        echo "FAIL (dynamic loading detected)"
        echo "$DYN_LOAD" | head -5
        FAILED=1
    fi
fi

# Summary
echo
echo "========================================"
if [ $FAILED -eq 0 ]; then
    if [ $WARNINGS -eq 0 ]; then
        echo "RESULT: PASS - No INTERP/NEEDED (checks 2 & 3 are authoritative)"
    else
        echo "RESULT: PASS with $WARNINGS warning(s) - No INTERP/NEEDED (checks 2 & 3 are authoritative)"
    fi
    exit 0
else
    echo "RESULT: FAIL - Dynamic dependency detected (see INTERP/NEEDED checks)"
    exit 1
fi
