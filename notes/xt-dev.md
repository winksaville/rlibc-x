# xt Development Notes

Development notes for the `xt-dev` branch - building a translation spec based build system.

## The Goal

Create a tool (`xt`) that:
1. Reads build specifications from `tspec.toml` files
2. Applies compiler/linker flags consistently
3. Enables easy comparison of different build configurations
4. Eventually supports maximal optimization (`-Z build-std`)

## The Linker Flag Scoping Problem

This was the central challenge of xt development. Understanding it requires knowing
how cargo applies flags.

### The Three Ways to Pass Linker Flags

| Method | Scope | Works with tests? |
|--------|-------|-------------------|
| `RUSTFLAGS="-C link-arg=-nostdlib"` | ALL targets | No |
| `.cargo/config.toml` rustflags | ALL targets | No |
| `cargo:rustc-link-arg-bin=X=flag` in build.rs | Binary X only | Yes |

### Why RUSTFLAGS Doesn't Work

When you set `RUSTFLAGS`, cargo applies those flags to EVERY compilation:
- Your binary
- Your dependencies
- **Build scripts** (build.rs files)
- Test harnesses

For a flag like `-nostdlib` (don't link standard startup code), this is fatal:

```
$ RUSTFLAGS="-C link-arg=-nostdlib" cargo build
   Compiling serde v1.0.228
error: failed to run custom build command for `serde v1.0.228`
  process didn't exit successfully (signal: 11, SIGSEGV)
```

Serde's build.rs is compiled with `-nostdlib`, producing a binary without `_start`.
When cargo tries to run it → SIGSEGV.

### Why We Need Scoped Flags

For rlibc-x2 apps, we need flags like:
- `-static` - static linking
- `-nostdlib` - no standard library
- `-nostartfiles` - no startup files (we provide our own `_start`)
- `-e_start` - entry point is `_start`

These must apply ONLY to the final binary, not to:
- Dependency build scripts (they need normal linking to run)
- Test harnesses (they're separate binaries that need normal linking)

### The Only Solution: build.rs with Scoped Directives

Cargo's `cargo:rustc-link-arg-bin=BINARY=FLAG` directive is the ONLY way to scope
linker flags to a specific binary:

```rust
// build.rs
fn main() {
    println!("cargo:rustc-link-arg-bin=ex-x2=-static");
    println!("cargo:rustc-link-arg-bin=ex-x2=-nostdlib");
}
```

Now:
- `ex-x2` binary gets `-static -nostdlib`
- Serde's build.rs compiles normally
- Test harness links normally
- Tests pass!

### How xt Handles This

Since tspec.toml is meant to be the source of truth, but we need build.rs for
scoping, xt generates a temporary build.rs:

1. Read `tspec.toml` linker args
2. Generate `build.rs` with scoped `cargo:rustc-link-arg-bin` directives
3. Run `cargo build/test`
4. Delete the generated `build.rs`

```
cargo xt build ex-x2-xt:
  1. Generate apps/ex-x2-xt/build.rs from tspec.toml
  2. cargo build -p ex-x2-xt
  3. rm apps/ex-x2-xt/build.rs
```

This keeps tspec.toml as the single source of truth while using cargo's scoping
mechanism under the hood.

### What About rustc Flags?

Flags that affect compilation (not linking) CAN use RUSTFLAGS safely:
- `-C opt-level=z` - optimization level
- `-C panic=abort` - panic strategy
- `-C lto=true` - link-time optimization

These don't break build scripts because they're compilation flags, not linker flags.
xt applies these via RUSTFLAGS, and linker flags via generated build.rs.

## The -opt / build-std Limitation

xtask's `-opt` flag uses `-Z build-std` to rebuild the standard library with
`panic=immediate-abort`. This provides 85-97% binary size reduction.

However, `-Z build-std` is incompatible with tests:

```
cargo xtask test hw-x2 -opt
error[E0152]: duplicate lang item in crate `core`: `sized`
  = note: first definition in `core` loaded from .../libcore-43d09347.rmeta
  = note: second definition in `core` loaded from .../libcore-cf1041a6.rmeta
```

The test harness (`test` crate) depends on the toolchain's prebuilt `core`.
When we rebuild `core` with `-Z build-std`, there are now TWO versions of `core`,
causing the conflict.

**Workaround:** Test without `-opt`, build final binary with `-opt`.

## Apps Created

| App | Runtime | tspec.toml | Notes |
|-----|---------|------------|-------|
| ex-x1-xt | rlibc-x1 | Yes | Linker flags for no_std |
| ex-x2-xt | rlibc-x2 | Yes | Linker flags for std-compatible |
| ex-glibc | glibc | No | Plain cargo build works |
| ex-musl | musl | Yes | target_triple for musl |

## Key Insights

1. **Cargo's scoping is limited** - Only build.rs can scope linker flags to a binary
2. **RUSTFLAGS is dangerous** - Applies to everything, including build scripts
3. **Generated build.rs is a workaround** - Not ideal, but necessary
4. **-Z build-std breaks tests** - Fundamental limitation of rebuilding std
5. **tspec.toml remains the source of truth** - Even though build.rs is generated

## Future Work

1. **build_std support** - Add to tspec.toml for maximal optimization
2. **Spec comparison** - Compare build outputs between different specs
3. **Eventually replace xtask** - Once xt has feature parity
