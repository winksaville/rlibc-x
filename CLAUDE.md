# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rlibc-x** is an experimental, educational Rust project implementing minimal libc replacements for Linux x86_64. The goal is understanding how Rust programs start, allocate memory, and interact with the kernel while achieving extremely small binary sizes.

## Build Commands

All builds use the xt build system. Run from repository root:

```bash
cargo xt build [crate] [-r] [-t FILE]  # Build crate(s)
cargo xt run [crate] [-r] [-t FILE]    # Build and run
cargo xt test [crate] [-r]             # Run tests
```

**Options:**
- `-r, --release` - Release build
- `-t, --tspec FILE` - Use alternative tspec (default: crate's `tspec.xt.toml` if present)
- Use `.` to operate on crate in current directory

**Examples:**
```bash
cargo xt run hw-x1                         # Quick test of hello world
cargo xt build -r                          # Build all crates release
cargo xt build ex-x2 -r -t tspec-opt.xt.toml  # Optimized build (~6 KB vs ~41 KB)
cargo xt test                              # Run all tests
```

**Verification tools:**
```bash
cargo run -p is-libc-used -- ./target/release/ex-x1     # Check libc usage
cargo run -p func-analysis -- analyze target/release/ex-musl  # Analyze functions
```

## Architecture

### Two Approaches to Libc Replacement

**rlibc-x1 (no_std)** - Minimal ~1.4 KB binaries
- Execution: `_start (asm) → _start_rust() → main() → exit()`
- Apps require `#![no_std]` and `#![no_main]`
- Single-file implementation (`libs/rlibc-x1/src/lib.rs`)
- Direct syscalls via inline assembly, bump allocator using `brk()`

**rlibc-x2 (std-compatible)** - Works with Rust std, ~6-41 KB binaries
- Execution: `_start (asm) → __libc_start_main() → Rust's main → user's main()`
- No special attributes needed in application code
- Modular: `process.rs`, `memory.rs`, `io.rs`, `syscall.rs`, `thread.rs`, `environ.rs`, `errno.rs`, `signal.rs`
- Linker flags configured via `tspec.xt.toml` (static, nostdlib, entry point)

### Workspace Structure

```
libs/
  rlibc-x1/         # no_std runtime library
  rlibc-x2/         # std-compatible runtime library
    tests/          # Integration tests (separate binaries)
apps/               # Example applications
  ex-x1, hw-x1      # rlibc-x1 examples (exit-only, hello world)
  ex-x2, hw-x2      # rlibc-x2 examples
  ex-glibc, hw-glibc  # glibc comparison (dynamic)
  ex-musl, hw-musl    # musl comparison (static)
tools/
  func-analysis/    # ELF function size analyzer (goblin, iced-x86)
  is-libc-used/     # Binary libc detection (object crate)
xt/                 # Build automation (spec-driven via tspec.xt.toml)
notes/              # Technical documentation (opt-notes.md, plt-less-linking.md)
```

### Key Insight: Binary Size

Rust's panic formatting machinery is the primary source of binary bloat, not libc itself. The `tspec-opt.xt.toml` specs achieve 85-97% size reduction by:
- Rebuilding std with `-Z build-std=std,core,panic_abort`
- Using `-C panic=immediate-abort` to eliminate panic formatting
- Version script to make symbols LOCAL, enabling dead code elimination

## Testing

Tests verify that binaries don't use libc and execute correctly:
```bash
cargo xt test              # All crates
cargo xt test rlibc-x2     # Includes rlibc-x2-tests binaries
cargo xt test ex-x1        # Single crate
```

## Conventions

- **Rust Edition:** 2024 for main crates
- **Toolchain:** Stable (nightly only for `-opt` builds via tspec-opt.xt.toml)
- **Commit style:** Conventional commits (feat:, docs:, refactor:)
- Apps with "musl" in name auto-target `x86_64-unknown-linux-musl`
- Custom target file: `x86_64-unknown-linux-rlibcx2.json` (plt-by-default: false)
