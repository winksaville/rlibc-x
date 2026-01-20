# Optimizing Statically Linked musl Binaries

## Key Finding

Musl binaries can be reduced from **381 KB to 22 KB** (94% reduction) using nightly Rust's `build-std` feature with immediate abort panic strategy.

## Size Comparison

| Binary | Stripped Size | Text Section |
|--------|---------------|--------------|
| ex-x1 (no_std, rlibc-x1) | 1.4 KB | 217 B |
| ex-x2 (std, rlibc-x2) | 6.4 KB | 3.6 KB |
| ex-musl (std, optimized) | 22.7 KB | 11 KB |
| ex-musl (std, baseline) | 381 KB | 344 KB |

## Why This Works

### Musl Already Uses Function Sections

The musl `libc.a` bundled with Rust is already compiled with `-ffunction-sections -fdata-sections`. Extracting an object file shows individual sections:

```
$ ar x libc.a exit.lo
$ readelf -S exit.lo | grep .text
  [ 1] .text             PROGBITS  ...
  [ 5] .text.libc_e[...] PROGBITS  ...  # .text.libc_exit_fini
  [ 7] .text.exit        PROGBITS  ...
```

This means the linker *can* perform dead code elimination with `--gc-sections`.

### The Real Win: Eliminating Panic Machinery

The bulk of the size reduction comes from eliminating Rust's panic formatting infrastructure. Standard panic handlers pull in formatting code, which brings in large portions of `core::fmt`.

Using `-C panic=immediate-abort` with `-Z build-std` rebuilds the standard library to immediately abort on panic without any formatting.

### Why gc-sections and Version Scripts Don't Help Further

With LTO enabled (default in release profile), the linker already performs aggressive dead code elimination. Adding explicit `--gc-sections` or version scripts doesn't reduce size further.

## The Optimization Command

```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort" \
  cargo +nightly build --release \
  --target x86_64-unknown-linux-musl \
  -Z build-std=std,core,panic_abort
```

Then strip:
```bash
strip target/x86_64-unknown-linux-musl/release/<binary>
```

## API Change Note

The `panic_immediate_abort` feature changed in recent nightly Rust. It's now a proper panic strategy rather than a build-std feature:

**Old (no longer works):**
```bash
-Z build-std-features=panic_immediate_abort
```

**New:**
```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort"
```

## Why musl is Still Larger Than rlibc-x2

Even with full optimization, musl produces binaries ~3.5x larger than rlibc-x2 for equivalent functionality. This is expected because:

1. musl is a complete, production-ready libc with more infrastructure
2. rlibc-x2 is purpose-built to be minimal
3. musl may have internal dependencies that prevent complete dead code elimination

## Potential Further Optimization

To get even smaller musl binaries, one could:

1. Rebuild musl from source with additional optimizations
2. Use a linker version script to hide internal symbols (may help in some cases)
3. Investigate which musl components are being pulled in and why

## Requirements

- Rust nightly toolchain
- `rust-src` component: `rustup component add rust-src --toolchain nightly`
- musl target: `rustup target add x86_64-unknown-linux-musl`
