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

For comparison, rlibc-x2 (which supports `std`) produces **~6 KB** with `-opt` (nightly).

See [apps/](apps/README.md#size-comparison) for the full size comparison.

## Workspace Structure

| Directory | Description |
|-----------|-------------|
| [libs/](libs/) | Runtime libraries ([rlibc-x1](libs/rlibc-x1/), [rlibc-x2](libs/rlibc-x2/)) |
| [apps/](apps/) | Example and comparison apps |
| [tools/](tools/) | Development tools (`is-libc-used`, `func-analysis`) |
| [notes/](notes/) | Technical notes and findings |
| [xtask/](xtask/) | Project automation |

## Two Approaches

**rlibc-x1** (no_std): App uses `#![no_std]` and `#![no_main]`. Minimal, but requires explicit annotations.

**rlibc-x2** (std): App uses normal `fn main()` with Rust std. Replaces glibc while keeping std functionality.

## Optimized Builds

The `-opt` flag dramatically reduces binary sizes by eliminating Rust's panic formatting machinery:

```bash
cargo xtask build ex-x2 -r -opt -s     # ~6 KB (vs ~41 KB)
cargo xtask build ex-glibc -r -opt -s  # ~9 KB (vs ~298 KB)
cargo xtask build ex-musl -r -opt -s   # ~23 KB (vs ~381 KB)
```

See [notes/opt-notes.md](notes/opt-notes.md) for details.

## Quick Start

```bash
# Build and run
cargo xtask run hw-x1          # hello world with rlibc-x1
cargo xtask run hw-x2          # hello world with rlibc-x2

# Test
cargo xtask test               # run all tests
cargo xtask test -q            # quiet mode

# Build release
cargo xtask build -r           # release builds
```

For more information see [xtask/README.md](xtask/README.md)

## Verifying No libc Usage

The `is-libc-used` tool checks if a binary uses libc:

```bash
cargo run -p is-libc-used -- ./target/release/ex-x1
cargo run -p is-libc-used -- -v ./target/release/ex-x1  # verbose
```

All apps include libc usage tests that run with `cargo xtask test`.

## Status

This is an experimental/educational project. For production use, consider:

- [relibc](https://gitlab.redox-os.org/redox-os/relibc) - Redox OS's portable POSIX C library in Rust
- [c-ward](https://github.com/aspect-build/c-ward) - A libc implementation in Rust

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
