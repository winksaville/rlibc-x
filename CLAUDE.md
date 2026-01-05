# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rlibc-x is a minimal, educational Rust libc implementation for Linux x86_64. It provides all runtime functionality needed to build standalone Rust binaries with zero external dependencies, including direct syscalls, memory allocation, and program startup/exit.

## Build Commands

```bash
# Build the workspace (both library and app) - uses default target
cargo build

# Build release
cargo build --release

# Build with custom target (from app directory, requires nightly)
cd app && cargo +nightly build --release -Z build-std

# Run the app (returns exit code 47)
cargo build && ./target/debug/app; echo $?

# Check/lint
cargo check
cargo clippy
```

## Architecture

### Workspace Structure

- **rlibc-x1/** - Core `#![no_std]` library providing libc functionality
- **app/** - Example application demonstrating library usage

### Key Design Decisions

- **panic="abort"** - No unwinding support
- **Identical dev/release profiles** - Both use `opt-level='z'`, LTO, `codegen-units=1`, `debug=2`, `debug-assertions=false`, `overflow-checks=false`, `incremental=false`. Only difference: `strip=false` (dev) vs `strip=true` (release)
- **-nostartfiles -static** - App uses custom `_start` entry point and static linking (set in `app/build.rs`)
- **Custom target** - `app/.cargo/x86_64-unknown-linux-rlibc-x1.json` for builds with `-Z build-std`
- **Bump allocator** - Simple linear allocator where `free()` is a no-op; memory grows via `brk()` syscall
- **Early heap init** - Heap is initialized once in `_start_rust()` before `main()`, not lazily per-malloc

### rlibc-x1 Library (rlibc-x1/src/lib.rs)

**Syscall Interface**:
- `syscall0` through `syscall6` - Inline assembly wrappers for x86_64 Linux syscalls
- Follows System V ABI: rax=syscall number, rdi/rsi/rdx/r10/r8/r9=args

**Syscall Constants**:
- `SYS_READ` (0), `SYS_WRITE` (1), `SYS_BRK` (12), `SYS_EXIT` (60)

**Public API** (extern "C" for libc compatibility):
- `exit(code)` - Process exit
- `read(fd, buf, count)` / `write(fd, buf, count)` - Basic I/O
- `malloc(size)` / `realloc(ptr, size)` / `calloc(nmemb, size)` / `free(ptr)` - Memory allocation

**Runtime**:
- `_start()` (naked) - Entry point; sets up argc/argv/envp in registers, calls `_start_rust()`
- `_start_rust()` - Initializes heap, calls `main()`, then `exit(0)`
- `main()` - User-defined; call `exit()` explicitly for non-zero exit codes
- `panic_handler` - Routes panics to exit(101)
- `rust_eh_personality()` - Empty stub required by compiler

### Creating New Applications

Applications using rlibc-x1 must:
1. Use `#![no_std]` and `#![no_main]`
2. Define `#[unsafe(no_mangle)] fn main()` (called by rlibc-x1's `_start_rust`)
3. Add `-nostartfiles` and `-static` linker flags via build.rs
4. Call `rlibc_x1::exit()` to terminate with non-zero exit code

## Platform Support

Currently **Linux x86_64 only**. Syscall numbers and ABI are architecture-specific.
