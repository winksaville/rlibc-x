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
