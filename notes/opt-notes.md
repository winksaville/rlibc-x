# Optimizing Rust Binaries with build-std

## 20260119 - Key Finding

Rust binaries using std can be dramatically reduced using nightly's `build-std` feature with `panic=immediate-abort`:

| Target | Baseline | Optimized | Reduction |
|--------|----------|-----------|-----------|
| glibc (dynamic) | ~298 KB | ~9 KB | 97% |
| musl (static) | ~381 KB | ~23 KB | 94% |
| x2 (rlibc-x2) | ~41 KB | ~6 KB | 85% |

**The bloat isn't libc - it's Rust's panic formatting machinery.**

### Why This Works

#### The Real Win: Eliminating Panic Machinery

Both glibc and musl baselines have ~280-340 KB of text. That's almost entirely `core::fmt` infrastructure for panic messages. Standard panic handlers pull in formatting code, which brings in large portions of the formatting machinery.

Using `-C panic=immediate-abort` with `-Z build-std` rebuilds the standard library to immediately abort on panic without any formatting.

#### Dynamic vs Static After Optimization

- **glibc (dynamic, 9.3 KB)** is smaller than **musl (static, 22.7 KB)**
- Makes sense: musl code is embedded in the binary, glibc code lives in system's `libc.so.6`
- Both achieve massive reductions because the optimization targets Rust's std, not the libc

#### Musl Already Uses Function Sections

The musl `libc.a` bundled with Rust is compiled with `-ffunction-sections -fdata-sections`:

```
$ ar x libc.a exit.lo
$ readelf -S exit.lo | grep .text
  [ 5] .text.libc_e[...] PROGBITS  ...  # .text.libc_exit_fini
  [ 7] .text.exit        PROGBITS  ...
```

This enables `--gc-sections` to remove unused functions, though LTO already handles most dead code elimination.

#### Why rlibc-x2 is Still Smaller

Even with full optimization:
- ex-x2 (rlibc-x2): 6.4 KB
- ex-glibc: 9.3 KB
- ex-musl: 22.7 KB

rlibc-x2 is purpose-built to be minimal, while glibc/musl are complete, production-ready implementations with more infrastructure.

### Manual Commands

If not using xtask, here are the raw cargo commands:

#### For glibc (dynamic linking)

```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort" \
  cargo +nightly build --release \
  --target x86_64-unknown-linux-gnu \
  -Z build-std=std,core,panic_abort
strip target/x86_64-unknown-linux-gnu/release/<binary>
```

#### For musl (static linking)

```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort" \
  cargo +nightly build --release \
  --target x86_64-unknown-linux-musl \
  -Z build-std=std,core,panic_abort
strip target/x86_64-unknown-linux-musl/release/<binary>
```

### API Change Note

The `panic_immediate_abort` feature changed in recent nightly Rust. It's now a proper panic strategy:

**Old (no longer works):**
```bash
-Z build-std-features=panic_immediate_abort
```

**New:**
```bash
RUSTFLAGS="-Z unstable-options -C panic=immediate-abort"
```

### Requirements

- Rust nightly toolchain
- `rust-src` component: `rustup component add rust-src --toolchain nightly`
- For musl: `rustup target add x86_64-unknown-linux-musl`

## 20260120 - Extended -opt to all apps

### Size Comparison

All apps built with `cargo xtask build <app> -r -opt -s`:

| App | Runtime | Stable | Nightly | Description |
|-----|---------|-------:|--------:|-------------|
| ex-x1 | rlibc-x1 | 1.4 KB | - | Exit-only (no_std) |
| hw-x1 | rlibc-x1 | 1.6 KB | - | Hello world (no_std) |
| ex-x2 | rlibc-x2 | 41 KB | 6.4 KB | Exit-only (std) |
| hw-x2 | rlibc-x2 | 46 KB | 8.9 KB | Hello world (std) |
| ex-glibc | glibc | 298 KB | 9.3 KB | Exit-only (dynamic) |
| hw-glibc | glibc | 287 KB | 11.9 KB | Hello world (dynamic) |
| ex-musl | musl | 381 KB | 22.7 KB | Exit-only (static) |
| hw-musl | musl | 377 KB | 26.8 KB | Hello world (static) |

The `-x1` apps don't benefit from `-opt` since they're already `no_std`.

### Using xtask

The `-opt` flag in xtask enables these optimizations for x2, glibc, and musl crates:

```bash
cargo xtask build ex-x2 -r -opt -s     # rlibc-x2 based
cargo xtask build ex-glibc -r -opt -s  # glibc (dynamic)
cargo xtask build ex-musl -r -opt -s   # musl (static)
cargo xtask build hw-glibc -r -opt -s  # hello world variants too
```

The `-x1` crates are already `no_std`, so `-opt` has no effect on them.
