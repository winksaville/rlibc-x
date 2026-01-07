# rlibc-x

A minimal, educational Rust libc implementation for Linux x86_64.

## Goals

- **Educational** - Understand how Rust programs start, allocate memory, and interact with the
  kernel
- **Minimal binaries** - Explore whether simpler, focused implementations can produce smaller
  binaries than full-featured libcs
- **Eventually transparent** - Long-term goal is to not require `#![no_std]` and `#![no_main]` in
  applications

One hypothesis: if a CLI or GUI app doesn't parse command line arguments, a minimal runtime could
skip that machinery entirely, potentially reducing binary size significantly.

## Results

The `app-x1` + `rlibc-x1` combination produces a **1,480 byte** statically-linked executable:

```
$ cargo build --release
$ ls -la target/release/app-x1
-rwxr-xr-x 2 wink users 1480 Jan  6 09:38 target/release/app-x1

$ file target/release/app-x1
target/release/app-x1: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), statically linked, stripped

$ size target/release/app-x1
   text    data     bss     dec     hex filename
    284       0      24     308     134 target/release/app-x1
```

This is achieved through:
- **`#![no_std]`** - No Rust standard library overhead
- **Direct syscalls** - `_start` → `_start_rust()` → `main()` → `exit()` with no glibc
- **Bump allocator** - Simple `malloc`/`realloc` via `brk()`, `free()` is a no-op
- **`panic="abort"`** - No unwinding machinery
- **Aggressive optimization** - `opt-level='z'`, LTO, single codegen unit, stripped symbols

For comparison, `app-x2` using `rlibc-x2` (which supports Rust's std library) is ~41KB.

## Verifying No libc Usage

The `verify-no-libc.sh` script validates that a binary doesn't use any C library (glibc, musl, etc.):

```
$ ./verify-no-libc.sh ./target/release/app-x1
=== Verifying: ./target/release/app-x1 ===

1. Dynamic linking check (ldd)... INFO (not a dynamic executable)
2. Interpreter (INTERP) check... PASS (no INTERP program header)
3. NEEDED libraries check... PASS (no NEEDED libraries)
4. Undefined symbols check... PASS (no dynsym section - fully static)
5a. GLIBC dynamic symbols check... PASS (no @GLIBC version symbols)
5b. GLIBC undefined symbols check... PASS (no undefined glibc symbols)
6. Syscall instructions (heuristic)... PASS (2 syscall instructions)
7. Runtime library file check... PASS (no libc/runtime libraries accessed)
8. Runtime syscall trace check... PASS (3 syscalls, no dynamic loader activity)

========================================
RESULT: PASS - No dynamic loader or libc dependency detected
```

### Self-Test Mode

Run the built-in test suite to verify the script works correctly:

```
$ ./verify-no-libc.sh --test
=== verify-no-libc.sh self-test ===

Testing app-x1 (debug)... OK (PASS as expected)
Testing app-x1 (release)... OK (PASS as expected)
Testing app-x2 (debug)... OK (PASS as expected)
Testing app-x2 (release)... OK (PASS as expected)
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
| 5b | No undefined glibc symbols | `nm` | No statically linked glibc (checks for undefined `__libc_start_main`, etc.) |
| 6 | Syscall instructions (heuristic) | `objdump -d` | Looks for `syscall`/`svc`/`int 0x80` - may be hidden by vDSO or LTO |
| 7 | No libc files accessed | `strace -f -e trace=file` | Runtime doesn't access libc.so, ld-linux.so, or runtime libs |
| 8 | No dynamic loader activity | `strace -f` | Full syscall trace shows no library loading (ld.so.cache, etc.) |

### The "Dynamically Linked" Discrepancy

You may notice that `file` and `ldd` report different things for `app-x2`:

```
$ file target/release/app-x2
target/release/app-x2: ELF 64-bit LSB executable, x86-64, version 1 (SYSV), dynamically linked, ...

$ ldd target/release/app-x2
	statically linked
```

**Both are correct!**

- **`file`** reports "dynamically linked" because `app-x2` has `.dynamic` and `.dynsym` ELF sections (required by Rust's std for symbol exports)
- **`ldd`** reports "statically linked" because there's no INTERP section, so no dynamic linker is invoked

The definitive proof is **`strace`** - it shows exactly what happens at runtime:

```
$ strace ./target/release/app-x2
execve("./target/release/app-x2", ...) = 0
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

`app-x2` has one weak undefined symbol (`gettid`) from Rust's std library. This is acceptable because:
1. Weak symbols resolve to NULL if not provided
2. Rust's std has fallback code that uses `syscall(SYS_gettid)` when `gettid` is unavailable
3. No actual libc code is called

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

The initial version of app-x1 was created by Claude Code 4.5
and the initial version of app-x2 where a custom Target is defined
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
