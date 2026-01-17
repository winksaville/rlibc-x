# rlibc-x

A minimal, educational Rust libc implementation for Linux x86_64.
I briefly explored with Claude-code and
[ChatGPT](https://chatgpt.com/share/695efdde-3c88-800c-b661-bbe5c24a9b94)
the option of supporting dynamic linking with rblic-xX, but
for now decided to just stick static linking.

## Goals

- **Educational** - Understand how Rust programs start, allocate memory, and interact with the
  kernel
- **Minimal binaries** - Explore whether simpler, focused implementations can produce smaller
  binaries than full-featured libcs
- **Eventually transparent** - Long-term goal is to not require `#![no_std]` and `#![no_main]` in
  applications

One hypothesis: if a CLI or GUI app doesn't parse command line arguments, a minimal runtime could
skip that machinery entirely, potentially reducing binary size significantly.

## Example and Comparison Apps

The `apps/` directory contains example apps for comparing binary sizes across different runtimes:

| App | Runtime | Linking | Size | Description |
|-----|---------|---------|-----:|-------------|
| ex-x1 | rlibc-x1 | static | 1.4 KB | Minimal exit-only (no_std + no_main) |
| ex-x2 | rlibc-x2 | static | 41.4 KB | Minimal exit-only (std with custom libc) |
| ex-glibc | glibc | dynamic | 283.7 KB | Minimal exit-only |
| ex-musl | musl | static | 372.6 KB | Minimal exit-only |
| hw-x1 | rlibc-x1 | static | 1.5 KB | Hello world |
| hw-x2 | rlibc-x2 | static | 45.8 KB | Hello world |
| hw-glibc | glibc | dynamic | 286.7 KB | Hello world |
| hw-musl | musl | static | 376.6 KB | Hello world |

Build and run:

These all use "--release", dropping the "--release"
generates a debug as this is the default for rust.
```bash
# rlibc-x1 / rlibc-x2 (default target, no alias needed)
cargo build -p ex-x1 --release
cargo build -p ex-x2 --release
cargo build -p hw-x1 --release
cargo build -p hw-x2 --release
cargo run -p ex-x1 --release
cargo run -p ex-x2 --release
cargo run -p hw-x1 --release
cargo run -p hw-x2 --release

# glibc (requires --target, use alias)
cargo b-glibc -p ex-glibc --release
cargo b-glibc -p hw-glibc --release
cargo r-glibc -p hw-glibc --release

# musl (requires --target, use alias)
cargo b-musl -p ex-musl --release
cargo b-musl -p hw-musl --release
cargo r-musl -p hw-musl --release
```

## Results

The `ex-x1` + `rlibc-x1` combination produces a **1,480 byte** statically-linked executable:

```
$ cargo build -p ex-x1 --release
$ ls -la target/release/ex-x1
-rwxr-xr-x 2 wink users 1480 Jan 15 11:52 target/release/ex-x1

$ file target/release/ex-x1
target/release/ex-x1: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, stripped

$ size target/release/ex-x1
   text    data     bss     dec     hex filename
    284       0      24     308     134 target/release/ex-x1
```

This is achieved through:
- **`#![no_std]`** - No Rust standard library overhead
- **Direct syscalls** - `_start` → `_start_rust()` → `main()` → `exit()` with no glibc
- **Bump allocator** - Simple `malloc`/`realloc` via `brk()`, `free()` is a no-op
- **`panic="abort"`** - No unwinding machinery
- **Aggressive optimization** - `opt-level='z'`, LTO, single codegen unit, stripped symbols

For comparison, `ex-x2` using `rlibc-x2` (which supports Rust's std library) is ~41KB.

## Verifying No libc Usage

The `is-libc-used` tool checks if a binary uses libc by inspecting the INTERP and NEEDED ELF headers:

```bash
# Check a specific binary
cargo run -p is-libc-used -- ./target/release/ex-x1

# Verbose output
cargo run -p is-libc-used -- -v ./target/release/ex-x1
```

All apps include libc usage tests that run automatically with `cargo test`. The `test-repo` tool runs the complete test suite including these checks.

For additional diagnostic checks (strace, syscall inspection, etc.), see the reference script at `tools/sh/verify-no-libc.sh`.

### The "Dynamically Linked" Discrepancy

You may notice that `file` and `ldd` report different things for `ex-x2`:

```
$ file target/release/ex-x2
target/release/ex-x2: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, ...

$ ldd target/release/ex-x2
	statically linked
```

**Both are correct!**

- **`file`** reports "dynamically linked" because `ex-x2` has `.dynamic` and `.dynsym` ELF sections (required by Rust's std for symbol exports)
- **`ldd`** reports "statically linked" because there's no INTERP section, so no dynamic linker is invoked

The definitive proof is **`strace`** - it shows exactly what happens at runtime:

```
$ strace ./target/release/ex-x2
execve("./target/release/ex-x2", ...) = 0
brk(NULL)                               = 0x...
brk(0x...)                              = 0x...
arch_prctl(ARCH_SET_FS, 0x...)          = 0
poll([...], 3, 0)                       = 0 (Timeout)
gettid()                                = ...
exit(42)                                = ?
```

Only 7 syscalls, no library loading. Compare to a typical glibc-linked binary which would show:
- `openat("/etc/ld.so.cache", ...)`
- `openat("/usr/lib/libc.so.6", ...)`
- Multiple `mmap()` calls to load shared libraries

### Weak Undefined Symbols

`ex-x2` has one weak undefined symbol (`gettid`) from Rust's std library. This is acceptable because:
1. Weak symbols resolve to NULL if not provided
2. Rust's std has fallback code that uses `syscall(SYS_gettid)` when `gettid` is unavailable
3. No actual libc code is called

## Architecture

### Workspace Structure

- **rlibc-x1/** - `#![no_std]` library for apps that don't use Rust std
- **rlibc-x2/** - Library for apps that use Rust std (replaces glibc)
- **apps/** - Example and comparison apps (ex-x1, ex-x2, hw-x1, hw-x2, etc.)
- **tools/** - Development tools:
  - **is-libc-used/** - Library and binary to check if an ELF uses libc
  - **test-repo/** - Runs all repository tests

### Two Approaches

**rlibc-x1** (no_std path): App uses `#![no_std]` and `#![no_main]`. Library provides `_start` → `_start_rust()` → user's `main()`. Minimal, but requires explicit no_std annotations.

**rlibc-x2** (std path): App uses normal `fn main()` with Rust std. Library provides `_start` → `__libc_start_main()` → Rust's generated main. Replaces glibc while keeping std functionality.

### Key Design Decisions

- **panic="abort"** - No unwinding support
- **Bump allocator** - Simple linear allocator where `free()` is a no-op; memory grows via `brk()` syscall
- **TLS initialization** - rlibc-x2 sets up FS segment register for thread-local storage before calling main
- **Linux x86_64 only** - Syscall numbers and ABI are architecture-specific

### Creating Applications with rlibc-x1

1. Use `#![no_std]` and `#![no_main]`
2. Define `#[unsafe(no_mangle)] fn main()` (called by rlibc-x1's `_start_rust`)
3. Add linker flags via build.rs:
   ```rust
   println!("cargo:rustc-link-arg-bin=myapp=-nostartfiles");
   println!("cargo:rustc-link-arg-bin=myapp=-static");
   ```
4. Call `rlibc_x1::exit()` to terminate

### Creating Applications with rlibc-x2

1. Add `extern crate rlibc_x2;` to force linking
2. Add linker flags via build.rs:
   ```rust
   println!("cargo:rustc-link-arg-bin=myapp=-static");
   println!("cargo:rustc-link-arg-bin=myapp=-nostdlib");
   println!("cargo:rustc-link-arg-bin=myapp=-nodefaultlibs");
   println!("cargo:rustc-link-arg-bin=myapp=-e_start");
   println!("cargo:rustc-link-arg-bin=myapp=-Wl,--undefined=_start");
   println!("cargo:rustc-link-arg-bin=myapp=-Wl,--undefined=__libc_start_main");
   ```

Note: Use `rustc-link-arg-bin=<name>=` to apply flags only to the binary, not tests.

## rlibc-x2 Symbols

Key symbols provided by rlibc-x2 for std compatibility:

| Category | Symbols |
|----------|---------|
| Process | `_start`, `__libc_start_main`, `exit`, `abort`, `__libc_stack_end` |
| Environment | `environ`, `getenv` - environment variable support for `std::env` |
| Memory | `malloc`, `realloc`, `calloc`, `free`, `memcpy`, `memset`, `memmove`, `memcmp` |
| I/O | `read`, `write`, `writev` |
| Threading | `pthread_*` stubs, TLS initialization |
| Other | `__errno_location`, signal stubs |

## Testing

### Running All Tests

Use the `test-repo` tool to run the full test suite:

```bash
# Run all tests
cargo run -p test-repo

# Verbose output
cargo run -p test-repo -- -v

# Stop on first failure
cargo run -p test-repo -- --fail-fast
```

This runs:
1. `cargo test` - default target tests (includes libc usage tests for apps)
2. `cargo test --target x86_64-unknown-linux-musl` - musl-specific tests
3. rlibc-x2-tests binaries - standalone integration tests

### Cargo Test Aliases

```bash
# Run tests with specific target
cargo t-musl -p ex-musl -p hw-musl    # musl target
cargo t-glibc -p ex-glibc -p hw-glibc  # glibc target
```

### rlibc-x2 Integration Tests

Integration tests for rlibc-x2 are in `rlibc-x2/tests/`. These are standalone binaries that link against rlibc-x2 with the proper linker flags.

| Test | Description |
|------|-------------|
| `environ-tests` | Tests raw `environ` pointer and `getenv()` function |
| `std-env-tests` | Tests Rust's `std::env` API (`var()`, `var_os()`, `vars()`) |

```bash
# Run directly via cargo
cargo run -p rlibc-x2-tests --bin environ-tests --release
cargo run -p rlibc-x2-tests --bin std-env-tests --release
```

### Adding a Test

1. Create `rlibc-x2/tests/foo-tests.rs`:
   ```rust
   use std::process::ExitCode;
   extern crate rlibc_x2;

   fn main() -> ExitCode {
       // Test logic - return 0 on success
       ExitCode::from(0)
   }
   ```

2. Add to `rlibc-x2/tests/Cargo.toml`:
   ```toml
   [[bin]]
   name = "foo-tests"
   path = "foo-tests.rs"
   test = false
   ```

## Status

This is an experimental/educational project. For production use, consider mature alternatives:

- [relibc](https://gitlab.redox-os.org/redox-os/relibc) - Redox OS's portable POSIX C library
  written in Rust
- [c-ward](https://github.com/aspect-build/c-ward) - A libc implementation written in Rust

## Origin

I started this with this prompt for Claude Code 4.5:

> "I want to create a rust app that simply returns a number, something like `fn main -> i32 { 2 }`.
> I want to supply all code including the "standard libraries". I don't want the complication of
> code unwinding so panic="abort". So the minimum set of functions in lib.rs is _start, panic,
> exit, free, malloc, realloc, exit and probably a few others. And main.rs is something like
> `fn main() { exit(2) }`."

## Creation

The initial version of ex-x1 (originally app-x1) was created by Claude Code 4.5
and the initial version of ex-x2 where a custom Target is defined
in .cargo/config was suggested by ChatGPT and then Claude Code 4.5
completed the implementation!

Thanks for the help Claude and ChatGPT :)

https://chatgpt.com/share/695b4ae1-d84c-800c-8d09-34cff3de3b33
And use claude-code to see the many conversations with it!


If you'd like to use claude-code and keep the claude sessions locally install
[claude-code](https://code.claude.com/docs/en/setup) and also install
[claude-symlink.sh](https://github.com/winksaville/claude-symlink.sh)

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
