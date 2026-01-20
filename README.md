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

For comparison, rlibc-x2 (which supports `std`) produces:
- **~41 KB** with stable Rust
- **~6 KB** with nightly optimizations (`-x2` flag)

See [apps/](apps/README.md#size-comparison) for the full size comparison across rlibc-x1, rlibc-x2, glibc, and musl.

## Workspace Structure

| Directory | Description |
|-----------|-------------|
| [rlibc-x1/](rlibc-x1/) | `#![no_std]` runtime (~1.4 KB binaries) |
| [rlibc-x2/](rlibc-x2/) | `std`-compatible libc replacement (~6-41 KB binaries) |
| [apps/](apps/) | Example and comparison apps |
| [tools/](tools/) | Development tools (`is-libc-used`) |
| [xtask/](xtask/) | Project automation |

## Two Approaches

**rlibc-x1** (no_std): App uses `#![no_std]` and `#![no_main]`. Minimal, but requires explicit annotations.

**rlibc-x2** (std): App uses normal `fn main()` with Rust std. Replaces glibc while keeping std functionality.

## Build Modes for rlibc-x2

| Mode | Toolchain | Binary Size | Use Case |
|------|-----------|-------------|----------|
| Stable | `rustc` (stable) | ~41 KB | Compatibility, CI |
| Optimized | `rustc` (nightly) | ~6 KB | Size-critical deployments |

The optimized mode uses nightly features (`-Z build-std`, `-Z panic-immediate-abort`) and linker tricks to achieve much smaller binaries. Use the `-opt` flag with xtask:

```bash
cargo xtask build ex-x2 -r -s        # stable: ~41KB
cargo xtask build ex-x2 -r -opt -s   # nightly: ~6KB
```

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
