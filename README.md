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

```bash
# rlibc-x1 / rlibc-x2 (works on stable Rust)
cargo build -p ex-x1 --release
cargo build -p ex-x2 --release
cargo run -p ex-x1
cargo run -p ex-x2

# musl/glibc using cargo aliases
cargo b-musl -p hw-musl --release
cargo b-gnu -p hw-glibc --release
cargo r-musl -p hw-musl --release
cargo r-gnu -p hw-glibc --release
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

The `verify-no-libc.sh` script validates that a binary doesn't use any C library (glibc, musl, etc.):

```
$ ./verify-no-libc.sh ./target/release/ex-x1
=== Verifying: ./target/release/ex-x1 ===

1. Dynamic linking check (ldd)... INFO (not a dynamic executable)
2. Interpreter (INTERP) check... PASS (no INTERP program header)
3. NEEDED libraries check... PASS (no NEEDED libraries)
4. Undefined symbols check... PASS (no dynsym section - fully static)
5a. GLIBC dynamic symbols check... PASS (no @GLIBC version symbols)
5b. GLIBC entrypoint symbols (heuristic)... PASS (no undefined glibc entrypoint symbols)
6. Syscall instructions (heuristic)... PASS (2 syscall instructions)
7. Runtime library file check... PASS (no libc/runtime libraries accessed)
8. Runtime syscall trace check... PASS (3 syscalls, no dynamic loader activity)

========================================
RESULT: PASS - No INTERP/NEEDED (checks 2 & 3 are authoritative)
```

### Self-Test Mode

Run the built-in test suite to verify the script works correctly:

```
$ ./verify-no-libc.sh --test
=== verify-no-libc.sh self-test ===

Testing ex-x1 (release)... OK (PASS as expected)
Testing ex-x2 (release)... OK (PASS as expected)
Testing hw-x1 (release)... OK (PASS as expected)
Testing hw-x2 (release)... OK (PASS as expected)
Testing /usr/bin/ls... OK (FAIL as expected)
Testing /usr/bin/true... OK (FAIL as expected)

========================================
Tests passed: 6
Tests failed: 0
RESULT: ALL TESTS PASSED
```

### Configuration

- **Timeout**: Runtime checks default to 5 seconds. Override with: `TIMEOUT=10s ./verify-no-libc.sh ./mybinary`

### Checks Performed

| # | Check | Tool | Why |
|---|-------|------|-----|
| 1 | Dynamic linking (info) | `ldd` | Informational only - ldd can execute code, so checks 2 & 3 are authoritative |
| 2 | No INTERP header | `readelf -lW` | No dynamic linker (ld-linux.so) needed - **primary check** |
| 3 | No NEEDED libraries | `readelf -d` | No shared library dependencies - **primary check** |
| 4 | No strong undefined symbols | `readelf --dyn-syms` | All symbols resolved (weak undefined is acceptable) |
| 5a | No @GLIBC dynamic symbols | `objdump -T` | No dynamically linked glibc version-tagged symbols |
| 5b | No glibc entrypoint symbols (heuristic) | `nm` | No undefined `__libc_start_main`, etc. (nm optional - may not work on stripped binaries) |
| 6 | Syscall instructions (heuristic) | `objdump -d` | Looks for `syscall`/`svc`/`int 0x80` - may be hidden by vDSO or LTO |
| 7 | No libc files accessed | `strace -f -e trace=file` | Runtime doesn't access libc.so, ld-linux.so, or runtime libs |
| 8 | No dynamic loader activity | `strace -f` | Full syscall trace shows no library loading (ld.so.cache, etc.) |

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

Integration tests for rlibc-x2 are in `rlibc-x2/tests/`. Tests are standalone binaries that link against rlibc-x2 with the proper linker flags.

### Available Tests

| Test | Description |
|------|-------------|
| `test-environ` | Tests raw `environ` pointer and `getenv()` function |
| `test-std-env` | Tests Rust's `std::env` API (`var()`, `var_os()`, `vars()`) |

```bash
# Run all tests
./rlibc-x2/tests/run.sh

# Run specific test
./rlibc-x2/tests/run.sh environ
./rlibc-x2/tests/run.sh std-env

# Or via cargo
cargo run -p rlibc-x2-tests --bin test-environ --release
cargo run -p rlibc-x2-tests --bin test-std-env --release

# Verbose output (shows debug info)
VERBOSE=1 ./rlibc-x2/tests/run.sh
VERBOSE=1 cargo run -p rlibc-x2-tests --bin test-std-env --release
```

### Adding a Test

1. Create `rlibc-x2/tests/foo.rs`:
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
   name = "test-foo"
   path = "foo.rs"
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
