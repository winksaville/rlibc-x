# rlibc-x2

A libc replacement that allows using Rust's standard library without linking to glibc.

Produces statically-linked binaries that use `std` but make no glibc calls:
- **~41 KB** with stable Rust
- **~6 KB** with nightly optimizations

## How It Works

```
_start (asm) → __libc_start_main() → Rust's main → user's main()
```

- **`_start`** - Assembly entry point
- **`__libc_start_main()`** - Initializes TLS, environment, then calls Rust's generated main
- **Rust's main** - Standard library initialization
- **User's `main()`** - Normal Rust main function

## Provided Symbols

| Category | Symbols |
|----------|---------|
| Process | `_start`, `__libc_start_main`, `exit`, `abort`, `__libc_stack_end` |
| Environment | `environ`, `getenv` |
| Memory | `malloc`, `realloc`, `calloc`, `free`, `memcpy`, `memset`, `memmove`, `memcmp` |
| I/O | `read`, `write`, `writev` |
| Threading | `pthread_*` stubs, TLS initialization |
| Other | `__errno_location`, signal stubs |

## Usage

1. Add dependency:
   ```toml
   [dependencies]
   rlibc-x2 = { path = "../../libs/rlibc-x2" }
   ```

2. Force linking in your app:
   ```rust
   extern crate rlibc_x2;

   fn main() {
       std::process::exit(42);
   }
   ```

3. Add linker flags in `build.rs`:
   ```rust
   fn main() {
       println!("cargo:rustc-link-arg-bin=myapp=-static");
       println!("cargo:rustc-link-arg-bin=myapp=-nostdlib");
       println!("cargo:rustc-link-arg-bin=myapp=-nodefaultlibs");
       println!("cargo:rustc-link-arg-bin=myapp=-e_start");
       println!("cargo:rustc-link-arg-bin=myapp=-Wl,--undefined=_start");
       println!("cargo:rustc-link-arg-bin=myapp=-Wl,--undefined=__libc_start_main");
   }
   ```

## The "Dynamically Linked" Discrepancy

`file` reports "dynamically linked" but `ldd` says "statically linked" - both are correct:

- **`file`** sees `.dynamic` and `.dynsym` ELF sections (required by Rust's std)
- **`ldd`** sees no INTERP section, so no dynamic linker is invoked

Use `strace` to verify - only ~7 syscalls, no library loading.

## Integration Tests

Tests are in `tests/` and built as separate binaries:

```bash
cargo xtask test rlibc-x2    # runs rlibc-x2-tests binaries
```

## Build Modes

### Stable (default)

Works with stable Rust, produces ~41 KB binaries:

```bash
cargo xtask build ex-x2 -r -s
```

### Optimized (nightly)

Uses nightly features for ~6 KB binaries:

```bash
cargo xtask build ex-x2 -r -opt -s
```

The `-opt` flag enables:
- `-Z build-std` - Rebuild std from source for better LTO
- `-Z panic-immediate-abort` - Eliminate panic formatting code
- Linker version script - Enable dead code elimination
- Custom target - `x86_64-unknown-linux-rlibcx2.json`

## Example

See [apps/ex-x2](../../apps/ex-x2) and [apps/hw-x2](../../apps/hw-x2) for complete examples.
