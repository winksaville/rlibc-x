# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**rlibc-x** is an experimental, educational Rust project implementing minimal libc replacements for Linux x86_64. The goal is understanding how Rust programs start, allocate memory, and interact with the kernel while achieving extremely small binary sizes.

## Build Commands

All builds use the xt build system. Run from repository root:

```bash
cargo xt build [-p PKG] [-a] [-r] [-t FILE]  # Build package(s)
cargo xt run [-p PKG] [-a] [-r] [-t FILE]    # Build and run
cargo xt test [-p PKG] [-a] [-r]             # Run tests
cargo xt compare -p PKG [-r]                 # Compare all tspec*.ts.toml sizes
```

**Options:**
- `-p, --package PKG` - Target package (defaults to current directory if in a package)
- `-a, --all` - Operate on all packages (even when in a package directory)
- `-r, --release` - Release build
- `-t, --tspec FILE` - Use alternative tspec (default: package's `tspec.ts.toml` if present)

**Examples:**
```bash
cargo xt run -p hw-x1                      # Quick test of hello world
cargo xt build -r                          # Build all packages release
cargo xt build -p ex-x2 -r -t tspec-opt.ts.toml  # Optimized build (~6 KB vs ~41 KB)
cargo xt test                              # Run all tests
cargo xt compare -p ex-x2 -r               # Compare spec sizes
```

**Interactive tspec management:**
```bash
cargo xt ts list [-p PKG] [-a]                # List tspec files
cargo xt ts show [-p PKG] [-a] [-t spec]      # Show contents
cargo xt ts hash [-p PKG] [-a] [-t spec]      # Show content hash
cargo xt ts new [name] [-p PKG] [-f source]   # Create new spec
cargo xt ts set key=value [-p PKG] [-t spec]  # Set value (creates versioned file)
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
- Linker flags configured via `tspec.ts.toml` (static, nostdlib, entry point)

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
xt/                 # Build automation (spec-driven via tspec.ts.toml)
notes/              # Technical documentation (opt-notes.md, plt-less-linking.md)
```

### Key Insight: Binary Size

Rust's panic formatting machinery is the primary source of binary bloat, not libc itself. The `tspec-opt.ts.toml` specs achieve 85-97% size reduction by:
- Rebuilding std with `-Z build-std=std,core,panic_abort`
- Using `-C panic=immediate-abort` to eliminate panic formatting
- Version script to make symbols LOCAL, enabling dead code elimination

## Testing

Tests verify that binaries don't use libc and execute correctly:
```bash
cargo xt test              # All packages
cargo xt test -p rlibc-x2  # Includes rlibc-x2-tests binaries
cargo xt test -p ex-x1     # Single package
```

## Conventions

- **Rust Edition:** 2024 for main crates
- **Toolchain:** Stable (nightly only for `-opt` builds via tspec-opt.ts.toml)
- **Commit style:** Conventional commits (feat:, docs:, refactor:)
- Apps with "musl" in name auto-target `x86_64-unknown-linux-musl`
- Custom target file: `x86_64-unknown-linux-rlibcx2.json` (plt-by-default: false)

## Workflow

**Before committing, run verification:**
```bash
cargo xt test -p xt && cargo xt test
cargo clippy --workspace --all-targets
cargo fmt --check
```

**After committing code, remind about .claude/ files:**
```
Committed abc123.

Remember to commit .claude/ session files.
```

**On next prompt after a commit+reminder:** Check `git log -1 --name-only` to see if `.claude/` was included in a commit after the code commit. If not, ask: "Did you forget to commit .claude sessions?"

**Short-term tasks:** See `notes/done-todo.md` for current Todo/Done status.
