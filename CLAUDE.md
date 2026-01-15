# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rlibc-x is a minimal, educational Rust libc implementation for Linux x86_64. It provides all runtime functionality needed to build standalone Rust binaries with zero external dependencies, including direct syscalls, memory allocation, and program startup/exit.

## Build Commands

```bash
# Build the workspace
cargo build
cargo build --release

# Build/run ex-x1 (works on stable Rust, returns 47)
cargo build -p ex-x1 --release
cargo run -p ex-x1; echo $?

# Build/run ex-x2 (returns 42)
cargo build -p ex-x2 --release
cargo run -p ex-x2; echo $?

# Check/lint
cargo check
cargo clippy
```

### Cargo Aliases

Aliases for building/running with specific targets (defined in `.cargo/config.toml`):

```bash
# Build with musl (static) or glibc (dynamic)
cargo b-musl -p hw-musl --release
cargo b-gnu -p hw-glibc --release

# Run with musl or glibc
cargo r-musl -p hw-musl --release
cargo r-gnu -p hw-glibc --release
```

## Architecture

### Workspace Structure

- **rlibc-x1/** - `#![no_std]` library for apps that don't use Rust std
- **rlibc-x2/** - Library for apps that use Rust std (replaces glibc)
- **apps/** - Example and comparison apps:
  - **ex-x1/** - Minimal exit-only using rlibc-x1 (no_std + no_main)
  - **ex-x2/** - Minimal exit-only using rlibc-x2 (std with custom libc)
  - **ex-glibc/** - Minimal exit-only using glibc (dynamic)
  - **ex-musl/** - Minimal exit-only using musl (static)
  - **hw-x1/** - Hello world using rlibc-x1 (no_std + no_main)
  - **hw-glibc/** - Hello world with glibc (dynamic)
  - **hw-musl/** - Hello world with musl (static)

### Two Approaches

**rlibc-x1** (no_std path): App uses `#![no_std]` and `#![no_main]`. Library provides `_start` → `_start_rust()` → user's `main()`. Minimal, but requires explicit no_std annotations.

**rlibc-x2** (std path): App uses normal `fn main()` with Rust std. Library provides `_start` → `__libc_start_main()` → Rust's generated main. Replaces glibc while keeping std functionality. Uses stub macros for unimplemented functions that print "STUB: funcname" and exit(99).

### Key Design Decisions

- **panic="abort"** - No unwinding support
- **Identical dev/release profiles** - Both use `opt-level='z'`, LTO, `codegen-units=1`, `debug=2`. Only difference: `strip=false` (dev) vs `strip=true` (release)
- **Bump allocator** - Simple linear allocator where `free()` is a no-op; memory grows via `brk()` syscall
- **TLS initialization** - rlibc-x2 sets up FS segment register for thread-local storage before calling main

### rlibc-x2 Module Structure

- **syscall.rs** - `syscall0` through `syscall6` wrappers, syscall constants
- **process.rs** - `_start`, `__libc_start_main`, `exit`, `abort`, `__libc_stack_end`
- **environ.rs** - `environ` global pointer, `getenv` function for `std::env`
- **memory.rs** - `malloc`, `realloc`, `calloc`, `free`, `memcpy`, `memset`, `memmove`, `memcmp`, `posix_memalign`
- **io.rs** - `read`, `write`, `writev`
- **thread.rs** - TLS init, pthread stubs, `poll`, `sysconf`
- **signal.rs** - Signal handling stubs (`signal`, `sigaction`, `sigaltstack`)
- **errno.rs** - `__errno_location`, `strlen`
- **lib.rs** - Module re-exports and stub! macro for unimplemented functions
- **tests/** - Integration test binaries (separate crate)

### Creating Applications with rlibc-x2

Applications using rlibc-x2 need:
1. Add `extern crate rlibc_x2;` to force linking
2. Use build.rs with linker flags:
   ```rust
   println!("cargo:rustc-link-arg=-static");
   println!("cargo:rustc-link-arg=-nostdlib");
   println!("cargo:rustc-link-arg=-nodefaultlibs");
   println!("cargo:rustc-link-arg=-e_start");
   println!("cargo:rustc-link-arg=-Wl,--undefined=_start");
   println!("cargo:rustc-link-arg=-Wl,--undefined=__libc_start_main");
   ```

### Creating Applications with rlibc-x1

Applications using rlibc-x1 must:
1. Use `#![no_std]` and `#![no_main]`
2. Define `#[unsafe(no_mangle)] fn main()` (called by rlibc-x1's `_start_rust`)
3. Add `-nostartfiles` and `-static` linker flags via build.rs
4. Call `rlibc_x1::exit()` to terminate with non-zero exit code

## Testing

Integration tests are in `rlibc-x2/tests/` as a separate crate with proper linker flags.

```bash
# Run all tests
./rlibc-x2/tests/run.sh

# Run specific test
./rlibc-x2/tests/run.sh environ

# Via cargo
cargo run -p rlibc-x2-tests --bin test-environ --release
```

Adding a test: create `foo.rs`, add `[[bin]]` entry to `rlibc-x2/tests/Cargo.toml`.

## Verification

Use `verify-no-libc.sh` to validate binaries don't depend on libc:

```bash
# Verify a binary has no libc dependency
./verify-no-libc.sh ./target/release/ex-x1

# Run self-tests (requires built binaries)
./verify-no-libc.sh --test

# Override timeout for slow/interactive binaries
TIMEOUT=10s ./verify-no-libc.sh ./mybinary
```

Key checks: INTERP header (check 2) and NEEDED libraries (check 3) are authoritative - if both pass, the binary is truly libc-free.

## Platform Support

Currently **Linux x86_64 only**. Syscall numbers and ABI are architecture-specific.
