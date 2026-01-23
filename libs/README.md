# Runtime Libraries

Minimal libc replacements for Rust on Linux x86_64.

| Library | Approach | Binary Size | Description |
|---------|----------|-------------|-------------|
| [rlibc-x1](rlibc-x1/) | `#![no_std]` | ~1.4 KB | Minimal runtime, requires `no_std`/`no_main` |
| [rlibc-x2](rlibc-x2/) | `std`-compatible | ~6-41 KB | Works with Rust std, replaces glibc |
