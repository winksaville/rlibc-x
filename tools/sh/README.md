# Shell Tools (Reference/Historical)

This directory contains shell scripts preserved for reference and historical purposes.

## verify-no-libc.sh

A comprehensive shell script for verifying that a binary doesn't use any C library (glibc, musl, etc.).

**Note:** The authoritative libc detection is now handled by the `is-libc-used` Rust tool, which is integrated into the test suite via `cargo test`. This script is preserved because it provides additional diagnostic checks that can be useful for debugging or educational purposes.

### Additional Checks Beyond is-libc-used

The `is-libc-used` tool checks INTERP and NEEDED ELF headers (the authoritative indicators of libc usage). This script performs several additional checks:

| Check | Tool | Purpose |
|-------|------|---------|
| Dynamic linking info | `ldd` | Informational - shows what ldd reports |
| Undefined symbols | `readelf --dyn-syms` | Detects strong undefined symbols |
| GLIBC version symbols | `objdump -T` | Finds @GLIBC version-tagged symbols |
| GLIBC entrypoint symbols | `nm` | Heuristic - finds `__libc_start_main`, etc. |
| Syscall instructions | `objdump -d` | Heuristic - confirms direct syscall usage |
| Runtime file access | `strace -e trace=file` | Diagnostic - checks no libc files opened |
| Runtime syscall trace | `strace` | Diagnostic - verifies no dynamic loader activity |

### Usage

```bash
# Check a single binary
./tools/sh/verify-no-libc.sh ./target/release/ex-x1

# Run self-tests
./tools/sh/verify-no-libc.sh --test

# With custom timeout (default 5s)
TIMEOUT=10s ./tools/sh/verify-no-libc.sh ./mybinary
```

### Primary Testing Path

For regular development, use the integrated Rust tools:

```bash
# Run all tests (includes libc usage checks for all apps)
cargo run -p test-repo

# Check a specific binary
cargo run -p is-libc-used -- ./target/release/ex-x1
```
