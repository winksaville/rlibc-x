# Optimizing Rust Binaries with build-std

## Key Finding

Rust binaries using std can be dramatically reduced using nightly's `build-std` feature with `panic=immediate-abort`:

| Target | Baseline | Optimized | Reduction |
|--------|----------|-----------|-----------|
| glibc (dynamic) | 298 KB | 9.3 KB | 97% |
| musl (static) | 381 KB | 22.7 KB | 94% |

**The bloat isn't libc - it's Rust's panic formatting machinery.**

## Size Comparison (stripped)

| Binary | Size | Text |
|--------|------|------|
| ex-x1 (no_std, rlibc-x1) | 1.4 KB | 217 B |
| ex-x2 (std, rlibc-x2, -opt) | 6.4 KB | 3.6 KB |
| ex-glibc (dynamic, -opt) | 9.3 KB | 5.4 KB |
| ex-musl (static, -opt) | 22.7 KB | 11 KB |
| | | |
| ex-glibc (dynamic, baseline) | 298 KB | 284 KB |
| ex-musl (static, baseline) | 381 KB | 344 KB |

## Why This Works

### The Real Win: Eliminating Panic Machinery

Both glibc and musl baselines have ~280-340 KB of text. That's almost entirely `core::fmt` infrastructure for panic messages. Standard panic handlers pull in formatting code, which brings in large portions of the formatting machinery.

Using `-C panic=immediate-abort` with `-Z build-std` rebuilds the standard library to immediately abort on panic without any formatting.

### Dynamic vs Static After Optimization

- **glibc (dynamic, 9.3 KB)** is smaller than **musl (static, 22.7 KB)**
- Makes sense: musl code is embedded in the binary, glibc code lives in system's `libc.so.6`
- Both achieve massive reductions because the optimization targets Rust's std, not the libc

### Musl Already Uses Function Sections

The musl `libc.a` bundled with Rust is compiled with `-ffunction-sections -fdata-sections`:

```
$ ar x libc.a exit.lo
$ readelf -S exit.lo | grep .text
  [ 5] .text.libc_e[...] PROGBITS  ...  # .text.libc_exit_fini
  [ 7] .text.exit        PROGBITS  ...
```

This enables `--gc-sections` to remove unused functions, though LTO already handles most dead code elimination.

## Optimization Commands

### For glibc (dynamic linking)

```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort" \
  cargo +nightly build --release \
  --target x86_64-unknown-linux-gnu \
  -Z build-std=std,core,panic_abort
```

### For musl (static linking)

```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort" \
  cargo +nightly build --release \
  --target x86_64-unknown-linux-musl \
  -Z build-std=std,core,panic_abort
```

### Then strip

```bash
strip target/<target>/release/<binary>
```

## Using xtask

The `-opt` flag in xtask enables these optimizations:

```bash
# For x2 crates (rlibc-x2 based)
cargo xtask build ex-x2 -r -opt -s

# For glibc crates
cargo xtask build ex-glibc -r -opt -s

# For musl crates
cargo xtask build ex-musl -r -opt -s
```

## API Change Note

The `panic_immediate_abort` feature changed in recent nightly Rust. It's now a proper panic strategy:

**Old (no longer works):**
```bash
-Z build-std-features=panic_immediate_abort
```

**New:**
```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort"
```

## Why rlibc-x2 is Still Smaller

Even with full optimization:
- ex-x2 (rlibc-x2): 6.4 KB
- ex-glibc: 9.3 KB
- ex-musl: 22.7 KB

rlibc-x2 is purpose-built to be minimal, while glibc/musl are complete, production-ready implementations with more infrastructure.

## Requirements

- Rust nightly toolchain
- `rust-src` component: `rustup component add rust-src --toolchain nightly`
- For musl: `rustup target add x86_64-unknown-linux-musl`
