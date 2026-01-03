# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

rlibc-x is a minimal, educational Rust libc implementation for Linux x86_64. It provides all runtime functionality needed to build standalone Rust binaries with zero external dependencies, including direct syscalls, memory allocation, and program startup/exit.

## Build Commands

```bash
# Build the workspace (both library and app)
cargo build

# Build release
cargo build --release

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

- **panic="abort"** - No unwinding support (configured in workspace Cargo.toml for both dev and release profiles)
- **-nostartfiles** - App uses custom `_start` entry point instead of system startup files (set in `app/build.rs`)
- **Bump allocator** - Simple linear allocator where `free()` is a no-op; memory grows via `brk()` syscall

### rlibc-x1 Library (rlibc-x1/src/lib.rs)

**Syscall Interface** (lines 19-150):
- `syscall0` through `syscall6` - Inline assembly wrappers for x86_64 Linux syscalls
- Follows System V ABI: rax=syscall number, rdi/rsi/rdx/r10/r8/r9=args

**Syscall Constants**:
- `SYS_READ` (0), `SYS_WRITE` (1), `SYS_BRK` (12), `SYS_EXIT` (60)

**Public API** (extern "C" for libc compatibility):
- `exit(code)` - Process exit
- `read(fd, buf, count)` / `write(fd, buf, count)` - Basic I/O
- `malloc(size)` / `realloc(ptr, size)` / `calloc(nmemb, size)` / `free(ptr)` - Memory allocation

**Runtime**:
- `_start()` - Naked assembly entry point that receives argc/argv from kernel, calls main(), then exit()
- `panic_handler` - Routes panics to exit(101)
- `rust_eh_personality()` - Empty stub required by compiler

### Creating New Applications

Applications using rlibc-x1 must:
1. Use `#![no_std]` and `#![no_main]`
2. Define `#[unsafe(no_mangle)] fn main()` (called by rlibc-x1's `_start`)
3. Add `-nostartfiles` linker flag via build.rs
4. Call `rlibc_x1::exit()` to terminate (returning from main goes to exit with return value)

## Platform Support

Currently **Linux x86_64 only**. Syscall numbers and ABI are architecture-specific.
