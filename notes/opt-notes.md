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

If not using xt, here are the raw cargo commands:

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

### Limitation: Optimized Builds Are Incompatible with Tests

**`cargo xt test` with `tspec-opt.xt.toml` does not work.** This is a fundamental limitation, not a bug.

#### What Works

| Command | Works? | Notes |
|---------|--------|-------|
| `cargo xt build X -t tspec-opt.xt.toml` | ✓ | Build optimized binary |
| `cargo xt run X -t tspec-opt.xt.toml` | ✓ | Build and run optimized binary |
| `cargo xt test X` | ✓ | Test with standard toolchain |
| `cargo xt test X -r` | ✓ | Test with release profile |
| `cargo xt test X -t tspec-opt.xt.toml` | ✗ | Always fails |

#### Why It Fails

The optimized tspec uses `-Z build-std=std,core,panic_abort` to rebuild the standard library with `panic=immediate-abort`. This creates a custom-built `core` crate.

When running tests, cargo also needs the `test` crate (Rust's test harness), which depends on the **toolchain's prebuilt** `core`. Now there are two versions of `core`:

```
test -opt build:
  ├── Your binary
  │    └── rebuilt core (from -Z build-std)
  │
  └── Test harness
       └── test crate
            └── prebuilt core (from toolchain)

Error: duplicate lang item `sized` in crate `core`
```

The linker sees two different `core` crates and fails with "duplicate lang item" errors.

#### This Affects ALL Targets

The failure is not specific to rlibc-x2. It happens with glibc, musl, and any target:

```bash
cargo xt test hw-x2 -t tspec-opt.xt.toml     # fails - duplicate core
cargo xt test hw-musl -t tspec-opt.xt.toml   # fails - duplicate core
cargo xt test hw-glibc -t tspec-opt.xt.toml  # fails - duplicate core
```

#### Workaround

Test without optimized tspec, then build the final optimized binary:

```bash
# Run tests (standard toolchain)
cargo xt test hw-x2

# Build optimized binary (no tests)
cargo xt build hw-x2 -r -t tspec-opt.xt.toml
```

The assumption is: if tests pass with the standard build, the optimized build will behave correctly. The optimized tspec only changes panic handling and binary size, not program logic.

## 20260120 - Extended -opt to all apps

### Size Comparison

All apps built with `cargo xt build <app> -r -t tspec-opt.xt.toml`:

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

The `-x1` apps don't benefit from optimized tspec since they're already `no_std`.

### Using xt

The `tspec-opt.xt.toml` files enable these optimizations for x2, glibc, and musl crates:

```bash
cargo xt build ex-x2 -r -t tspec-opt.xt.toml     # rlibc-x2 based
cargo xt build ex-glibc -r -t tspec-opt.xt.toml  # glibc (dynamic)
cargo xt build ex-musl -r -t tspec-opt.xt.toml   # musl (static)
cargo xt build hw-glibc -r -t tspec-opt.xt.toml  # hello world variants too
```

The `-x1` crates are already `no_std`, so optimized tspec has no effect on them.

## 20260120 - Preserving Symbols for Analysis

### The Problem

When using `-Z build-std`, the `.symtab` section was being stripped even with `strip = false` in Cargo.toml. This made `func-analysis` unable to show app function sizes for optimized builds.

### The Solution

Use the explicit string form `strip = "none"` in Cargo.toml:

```toml
[profile.release]
strip = "none"    # Preserves .symtab for analysis
```

This ensures symbols survive the build-std + LTO pipeline.

### Workflow

Two build modes for different purposes:

```bash
# For analysis - preserves symbols
cargo xt build ex-musl -r -t tspec-opt.xt.toml
func-analysis analyze target/x86_64-unknown-linux-musl/release/ex-musl

# For production - strips symbols, smallest size (add strip to tspec or use strip command)
cargo xt build ex-musl -r -t tspec-opt.xt.toml && strip target/x86_64-unknown-linux-musl/release/ex-musl
```

### func-analysis Output

With symbols preserved, `func-analysis` shows both app and library functions:

**Static binaries (musl, x2):** Shows local Rust functions + embedded libc functions
```
Functions: 73
Functions size: 9215 bytes
.text size: 9357 bytes
Coverage: 98.5%
```

**Dynamic binaries (glibc):** Shows local Rust functions + imported libc functions (sizes from system libc)
```
Functions: 38
Functions size: 7284 bytes
.text size: 2470 bytes
Coverage: 294.9%   # >100% because libc sizes are external
```

### Why Coverage Can Exceed 100%

For dynamic binaries, "Functions size" sums both:
- Local function sizes (from the binary's `.symtab`)
- Imported libc function sizes (from system's `libc.so.6`)

But `.text size` only measures local code, so coverage exceeds 100% when libc function sizes are included.

## 20260120 - Version Scripts and Dead Code Elimination

### The Problem

When building an executable (not a shared library), there's no good reason for internal symbols to be GLOBAL/exported. The only entry point is `_start` - nothing external can call internal functions at runtime. However, the linker keeps GLOBAL symbols by default, preventing `--gc-sections` from removing unreferenced code.

### The Solution: Version Scripts

We use a linker "version script" to make all symbols LOCAL except `_start`:

```
{ global: _start; local: *; };
```

Combined with `--gc-sections`, this allows the linker to remove unreferenced functions. For ex-x2, this reduced the stripped binary from ~13KB to ~6KB.

### Why This Is Frustrating

The linker should have a simple flag like `--executable-hide-symbols` that automatically makes all symbols local except the entry point. This would:

1. Enable dead code elimination via `--gc-sections`
2. Improve security by hiding internal implementation details
3. Reduce binary size by eliminating `.dynsym` bloat

Instead, we must create a version script file with arcane syntax. A short-term improvement would be accepting the configuration directly:

```bash
-Wl,--version-script="{ global: _start; local: *; }"
```

But this doesn't work - the linker requires an actual file.

### Current Implementation

The `tspec-opt.xt.toml` files include a version script configuration:

```toml
[[linker]]
version_script = { global = ["_start"], local = "*" }
```

This generates the version script and passes it to the linker.

### Impact on Real Applications

For minimal apps (ex-*), the version script provides significant savings. For larger applications like func-analysis, LTO already handles most dead code elimination, so the version script has minimal additional impact:

| Build | Stripped Size |
|-------|---:|
| `cargo build --release && strip` | 886 KB |
| `cargo xt build -r -t tspec-opt.xt.toml && strip` | 683 KB |
| **Reduction** | **23%** |

## 20260120 - Does using -no-pie reduce of a binary

### I tried adding `profile.release-no-pie`

```
[profile.release-no-pie]
inherits = "release"
strip = "symbols"
link-args = ["-no-pie"]
```

The using that didn't change the size

### Experimentation with -no-pie

Added `-C link-args=-no-pie` to rustflags and compared builds with and without:

```
wink@3900x 26-01-21T02:38:24.386Z:~/data/prgs/rust/rlibc-x (main)
$ ls -l fa-r*
-rwxr-xr-x 1 wink users 698656 Jan 20 18:24 fa-r-opt-s
-rwxr-xr-x 1 wink users 688080 Jan 20 18:24 fa-r-opt-s-no-pie
wink@3900x 26-01-21T02:38:28.232Z:~/data/prgs/rust/rlibc-x (main)
```

### Result 1.5% for this app

Difference is 10,576 or 1.5% savings off 689,656
