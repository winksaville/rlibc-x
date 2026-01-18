# rlibc-x1

A minimal `#![no_std]` runtime for Rust on Linux x86_64.

Produces tiny binaries (~1.4 KB) by eliminating standard library overhead entirely.

## How It Works

```
_start (asm) → _start_rust() → main() → exit()
```

- **`_start`** - Assembly entry point, sets up stack and calls `_start_rust()`
- **`_start_rust()`** - Calls user's `main()` function
- **`main()`** - User-defined, must call `exit()` to terminate
- **`exit()`** - Direct syscall, no glibc

## Provided Functions

| Category | Functions |
|----------|-----------|
| Process | `exit()` |
| Memory | `malloc()`, `realloc()`, `free()` (no-op) |
| I/O | `read()`, `write()` |
| Syscalls | `syscall0` through `syscall6` |

The allocator is a simple bump allocator using `brk()`. Memory is never freed.

## Usage

1. Add dependency:
   ```toml
   [dependencies]
   rlibc-x1 = { path = "../rlibc-x1" }
   ```

2. Configure your app:
   ```rust
   #![no_std]
   #![no_main]

   #[unsafe(no_mangle)]
   fn main() {
       rlibc_x1::exit(0);
   }
   ```

3. Add linker flags in `build.rs`:
   ```rust
   fn main() {
       println!("cargo:rustc-link-arg-bin=myapp=-nostartfiles");
       println!("cargo:rustc-link-arg-bin=myapp=-static");
   }
   ```

## Example

See [apps/ex-x1](../apps/ex-x1) and [apps/hw-x1](../apps/hw-x1) for complete examples.
