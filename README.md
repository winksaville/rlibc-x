# rlibc-x

A minimal, educational Rust libc implementation for Linux x86_64.

## Goals

- **Educational** - Understand how Rust programs start, allocate memory, and interact with the kernel
- **Minimal binaries** - Explore whether simpler implementations can produce smaller binaries
- **Eventually transparent** - Long-term goal is to not require `#![no_std]` and `#![no_main]`

## Results

The rlibc-x1 + no_std approach produces a **1,480 byte** statically-linked executable:

```
$ cargo build -p ex-x1 --release && ls -la target/release/ex-x1
-rwxr-xr-x 2 wink users 1480 Jan 15 11:52 target/release/ex-x1

$ file target/release/ex-x1
ex-x1: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, stripped
```

This is achieved through:
- **`#![no_std]`** - No Rust standard library overhead
- **Direct syscalls** - `_start` → `_start_rust()` → `main()` → `exit()`
- **Bump allocator** - Simple `malloc`/`realloc` via `brk()`, `free()` is a no-op
- **`panic="abort"`** - No unwinding machinery
- **Aggressive optimization** - `opt-level='z'`, LTO, single codegen unit, stripped symbols

For comparison, rlibc-x2 (which supports `std`) produces **~6 KB** with optimized tspec (nightly).

See [apps/](apps/README.md#size-comparison) for the full size comparison.

## Workspace Structure

| Directory | Description |
|-----------|-------------|
| [libs/](libs/) | Runtime libraries ([rlibc-x1](libs/rlibc-x1/), [rlibc-x2](libs/rlibc-x2/)) |
| [apps/](apps/) | Example and comparison apps |
| [tools/](tools/) | Development tools (`is-libc-used`, `func-analysis`) |
| [notes/](notes/) | Technical notes and findings |
| [xt/](xt/) | Build automation (spec-driven via tspec.ts.toml) |

## Two Approaches

**rlibc-x1** (no_std): App uses `#![no_std]` and `#![no_main]`. Minimal, but requires explicit annotations.

**rlibc-x2** (std): App uses normal `fn main()` with Rust std. Replaces glibc while keeping std functionality.

## Optimized Builds

The `tspec-opt.ts.toml` specs dramatically reduce binary sizes by eliminating Rust's panic formatting machinery:

```bash
cargo xt build -p ex-x2 -r -t tspec-opt.ts.toml     # ~6 KB (vs ~41 KB)
cargo xt build -p ex-glibc -r -t tspec-opt.ts.toml  # ~9 KB (vs ~298 KB)
cargo xt build -p ex-musl -r -t tspec-opt.ts.toml   # ~23 KB (vs ~381 KB)
```

See [notes/opt-notes.md](notes/opt-notes.md) for details.

## Quick Start

```bash
# Build and run
cargo xt run -p hw-x1       # hello world with rlibc-x1
cargo xt run -p hw-x2       # hello world with rlibc-x2

# Test
cargo xt test               # run all tests
cargo xt test -p xt         # test specific package

# Build release
cargo xt build -r           # release builds (all packages)
cargo xt build -a -r        # force all packages (even from inside a package dir)
```

The `-p` flag specifies a package (defaults to current directory if in a package, otherwise all packages). Use `-a, --all` to force all-packages mode. The `-t` flag selects a tspec; if omitted and a package has `tspec.ts.toml`, it's used automatically.

## tspec Management (ts)

The `ts` subcommand manages translation spec files:

```bash
# List and inspect
cargo xt ts list [-p PKG] [-a]                # List tspec files
cargo xt ts show [-p PKG] [-t spec]           # Show tspec contents
cargo xt ts hash [-p PKG] [-t spec]           # Show content hash

# Create and modify
cargo xt ts new [name] [-p PKG] [-f source]   # Create new spec
cargo xt ts set key=value [-p PKG] [-t spec]  # Set value (creates versioned file)

# Compare builds
cargo xt compare -p PKG [-r]                  # Compare binary sizes across all tspecs
```

Example workflow:
```bash
cargo xt ts new opt -p ex-x2                  # Create opt.ts.toml
cargo xt ts set strip=symbols -p ex-x2 -t opt # Set strip option
cargo xt compare -p ex-x2 -r                  # Compare sizes
```

For more information see [xt/README.md](xt/README.md)

## Verifying No libc Usage

The `is-libc-used` tool checks if a binary uses libc:

```bash
cargo run -p is-libc-used -- ./target/release/ex-x1
cargo run -p is-libc-used -- -v ./target/release/ex-x1  # verbose
```

All apps include libc usage tests that run with `cargo xt test`.

## Status

This is an experimental/educational project. For production use, consider:

- [relibc](https://gitlab.redox-os.org/redox-os/relibc) - Redox OS's portable POSIX C library in Rust
- [c-ward](https://github.com/aspect-build/c-ward) - A libc implementation in Rust

## Claude Code Sessions

This project stores `.claude/` session files in the repo. A symlink in `~/.claude/projects/` points to it:
```
$ ls -l ~/.claude/projects/
lrwxrwxrwx 1 wink users 41 Jan 15 17:44 -home-wink-data-prgs-rust-rlibc-x -> /home/wink/data/prgs/rust/rlibc-x/.claude
```
This allows all Claude Code prompts to be saved in git, providing a rich history of conversations.

However, there's a circular reference issue - Claude cannot commit `.claude/` changes because the session file updates as Claude works:
1. **Linear commits:** User should amend every commit performed by Claude, adding `.claude/*` changes. See [xt/README.md](xt/README.md#claude-code--git) for workflow details.
2. **Merge commits:** Trickier - Claude would need to stash changes, but popping the stash causes merge conflicts in `.claude/`. The current solution is to `/exit` the Claude session and do the merge yourself.

## Origin

Started with this prompt for Claude Code:

> "I want to create a rust app that simply returns a number, something like `fn main -> i32 { 2 }`.
> I want to supply all code including the "standard libraries". I don't want the complication of
> code unwinding so panic="abort". So the minimum set of functions in lib.rs is _start, panic,
> exit, free, malloc, realloc, exit and probably a few others."

Initial versions created with Claude Code and ChatGPT. See [this ChatGPT conversation](https://chatgpt.com/share/695b4ae1-d84c-800c-8d09-34cff3de3b33) for the custom target approach.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
