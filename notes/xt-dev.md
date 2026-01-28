# xt Development Notes

Development notes for the `xt-dev` branch - building a translation spec based build system.

## 20260127 - Compare Glob Patterns and Summary Improvements

Enhanced `compare` command to support multiple specs via glob patterns:

```bash
cargo xt compare ex-x2-xt -r                    # All tspec*.xt.toml in crate dir
cargo xt compare ex-x2-xt -t "*.xt.toml" -r     # Explicit glob pattern
cargo xt compare ex-x2-xt -t a.toml -t b.toml   # Explicit file list
```

Output now shows specs sorted by size (smallest first) with percent change from largest:
```
============================================
              COMPARE SUMMARY
============================================
  Spec                     Size    Change
  tspec-opt.xt.toml        6.4K    -84.8%
  tspec.xt.toml           42.2K     0.0%
============================================
```

Also added:
- Column headers to build/test/run summaries (Crate, Status, Size/Exit)
- Size column in build summary
- `print_hline!` and `print_header!` macros for consistent output formatting

## 20260125 - Compare Command and Path Resolution

Completed Phase 1 (spec comparison) with the `compare` command:

```bash
cargo xt compare ex-x2-xt -r   # Compare all tspec*.xt.toml
```

Output shows size differences sorted smallest first.

Also added path-first resolution for crates and tspec files:
- `cargo xt build apps/ex-x2-xt` - path to crate directory
- `cargo xt build ex-x2-xt -t /full/path/to/spec.toml` - absolute tspec path
- `cargo xt build ex-x2-xt -t ../other/spec.toml` - relative tspec path

The resolution order:
1. Try name as path (check for Cargo.toml or file existence)
2. Fall back to libs/ then apps/ (for crates) or crate_dir (for tspec)

Added 36 unit tests covering binary operations and path resolution.

## The Goal

Create a tool (`xt`) that:
1. Reads build specifications from `tspec.xt.toml` files
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

Since tspec.xt.toml is meant to be the source of truth, but we need build.rs for
scoping, xt generates a temporary build.rs:

1. Read `tspec.xt.toml` linker args
2. Generate `build.rs` with scoped `cargo:rustc-link-arg-bin` directives
3. Run `cargo build/test`
4. Delete the generated `build.rs`

```
cargo xt build ex-x2-xt:
  1. Generate apps/ex-x2-xt/build.rs from tspec.xt.toml
  2. cargo build -p ex-x2-xt
  3. rm apps/ex-x2-xt/build.rs
```

This keeps tspec.xt.toml as the single source of truth while using cargo's scoping
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
cargo xt test hw-x2 --tspec tspec-opt.xt.toml
error[E0152]: duplicate lang item in crate `core`: `sized`
  = note: first definition in `core` loaded from .../libcore-43d09347.rmeta
  = note: second definition in `core` loaded from .../libcore-cf1041a6.rmeta
```

The test harness (`test` crate) depends on the toolchain's prebuilt `core`.
When we rebuild `core` with `-Z build-std`, there are now TWO versions of `core`,
causing the conflict.

**Workaround:** Test without `-opt`, build final binary with `-opt`.

## Apps Created

| App | Runtime | tspec.xt.toml | Notes |
|-----|---------|---------------|-------|
| ex-x1-xt | rlibc-x1 | Yes | Linker flags for no_std |
| ex-x2-xt | rlibc-x2 | Yes | Linker flags for std-compatible |
| ex-glibc | glibc | No | Plain cargo build works |
| ex-musl | musl | Yes | target_triple for musl |

## Key Insights

1. **Cargo's scoping is limited** - Only build.rs can scope linker flags to a binary
2. **RUSTFLAGS is dangerous** - Applies to everything, including build scripts
3. **Generated build.rs is a workaround** - Not ideal, but necessary
4. **-Z build-std breaks tests** - Fundamental limitation of rebuilding std
5. **tspec.xt.toml remains the source of truth** - Even though build.rs is generated

## 20260128 - tspec Naming Convention and CLI Design

Finalized the tspec file naming convention and CLI command structure for Phase 3
(interactive tspec management).

### File Naming Convention

The `.xt.toml` suffix identifies a tspec file. Two forms:

| Form | Pattern | Example |
|------|---------|---------|
| Simple | `<name>.xt.toml` | `opt.xt.toml` |
| Versioned | `<name>.<NNN>-<HHHHHHHH>.xt.toml` | `opt.001-a7f3b2c1.xt.toml` |

- **Seqnum (NNN)**: 3-digit sequence, scoped per-base-name
- **Hash (HHHHHHHH)**: 8 hex chars, content-based (identical content = identical hash)
- **Optional**: If no `*.xt.toml` exists, cargo defaults apply

The versioned form enables CLI-created snapshots with history tracking. The content
hash allows quick comparison without reading file contents.

### Command Alias

Decided on short alias for ergonomics:
- `cargo xt tspec` - full form (discoverability)
- `cargo xt ts` - short form (daily use)

Future enhancement: `xts` shell alias for power users.

### Design Decisions

1. **No "tspec" required in filename** - Any `*.xt.toml` is valid
2. **Seqnum per-base-name** - `opt.001`, `opt.002` separate from `experiment.001`
3. **Content hash** - Identical configs produce identical hashes
4. **Snapshots alongside working spec** - Not in a hidden `.tspec/` subdir

See [notes/interactive-tspec.md](interactive-tspec.md) for the complete design.

## Future Work

1. ~~**build_std support**~~ Done - See [notes/build-std.md](build-std.md)
2. ~~**Spec comparison**~~ Done - `cargo xt compare` (see 20260125 section)
3. **Interactive tspec management** - Design complete, implementation pending
4. ~~**Merge to main**~~ Done - xtask removed, xt is now the primary build system

See [notes/interactive-tspec.md](interactive-tspec.md) for Phase 3 design (naming conventions, CLI commands, snapshots).

