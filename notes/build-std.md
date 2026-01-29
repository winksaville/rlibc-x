# Implementing build-std Support in xt

Development notes on adding `-Z build-std` support to the xt build system.

## Goal

Enable xt to produce optimized builds via `tspec-opt.xt.toml`, achieving
85-97% binary size reduction through nightly's `-Z build-std` feature.

## The Optimization Stack

The optimized tspec files combine several nightly features:

| Feature | Flag | Effect |
|---------|------|--------|
| Rebuild std | `-Z build-std=std,core,panic_abort` | Compile std from source with our settings |
| Panic immediate abort (cargo) | `-Z panic-immediate-abort` | Tell cargo to use immediate abort |
| Panic immediate abort (rustc) | `-C panic=immediate-abort` | Eliminate panic formatting machinery |
| GC sections | `-Wl,--gc-sections` | Remove unused sections |
| Version script | `-Wl,--version-script=...` | Make symbols LOCAL for better GC |

The combination eliminates Rust's panic formatting code, which is the primary source
of binary bloat (not libc itself).

## Implementation Steps

### 1. Type System Extensions

Added to `types.rs`:

```rust
// Cargo -Z flags (nightly only)
enum CargoParam {
    // ... existing
    Unstable(String),  // -Z flag passthrough
}

// Panic strategy now includes nightly option
enum PanicStrategy {
    Abort,
    Unwind,
    ImmediateAbort,  // -C panic=immediate-abort (nightly)
}

// Version script for symbol visibility
struct VersionScript {
    global: Vec<String>,  // symbols to export
    local: String,        // pattern for local (usually "*")
}

enum LinkerParam {
    Args(Vec<String>),
    VersionScript(VersionScript),
}
```

### 2. Nightly Detection

In `cargo_build.rs`, added automatic nightly toolchain selection:

```rust
fn requires_nightly(spec: &Spec) -> bool {
    let has_build_std = spec.rustc.iter()
        .any(|p| matches!(p, RustcParam::BuildStd(_)));
    let has_unstable = spec.cargo.iter()
        .any(|p| matches!(p, CargoParam::Unstable(_)));
    has_build_std || has_unstable
}

fn build_cargo_command(spec: &Spec) -> Command {
    let mut cmd = Command::new("cargo");
    if requires_nightly(spec) {
        cmd.arg("+nightly");  // Use nightly toolchain
    }
    cmd
}
```

### 3. Flag Application

BuildStd and Unstable are cargo flags (not RUSTFLAGS):

```rust
RustcParam::BuildStd(crates) => {
    // -Z build-std is a cargo flag
    cmd.arg("-Z").arg(format!("build-std={}", crates.join(",")));
}

CargoParam::Unstable(flag) => {
    cmd.arg("-Z").arg(flag);
}
```

### 4. Version Script Generation

Creates a linker version script file for symbol visibility:

```rust
LinkerParam::VersionScript(vs) => {
    let path = workspace.join("target/xt-version.script");
    // Format: { global: _start; local: *; };
    let content = format!("{{ global: {}; local: {}; }};",
        vs.global.join("; "), vs.local);
    fs::write(&path, content)?;
    rustc_flags.push(format!("-C link-arg=-Wl,--version-script={}", path.display()));
}
```

## The Critical Discovery: panic=abort vs panic=immediate-abort

Initial implementation used `panic = "abort"` which generates `-C panic=abort`.
This produced 37KB binaries instead of the expected 6KB.

**The difference:**
- `-C panic=abort` - Use abort strategy, but panic formatting code is still compiled
- `-C panic=immediate-abort` - Nightly flag that eliminates ALL panic formatting

This single flag accounts for ~30KB of the size difference. The fix was adding
`ImmediateAbort` to `PanicStrategy` enum.

## Final tspec-opt.toml

```toml
# ex-x2 optimized spec
# Usage: cargo xt build ex-x2 -r --tspec tspec-opt.xt.toml

[[cargo]]
target_json = "x86_64-unknown-linux-rlibcx2.json"

[[cargo]]
unstable = "panic-immediate-abort"

[[rustc]]
build_std = ["std", "core", "panic_abort"]

[[rustc]]
panic = "immediate-abort"

[[rustc]]
flag = "-Z unstable-options"

[[rustc]]
flag = "-C link-arg=-Wl,--gc-sections"

[[linker]]
args = ["-static", "-nostdlib", "-nodefaultlibs", "-e_start",
        "-Wl,--undefined=_start", "-Wl,--undefined=__libc_start_main"]

[[linker]]
version_script = { global = ["_start"], local = "*" }
```

## Results

| Build | Binary Size (stripped) |
|-------|------------------------|
| cargo xt build ex-x2 -r (baseline) | ~40 KB |
| cargo xt build ex-x2 -r --tspec tspec-opt.xt.toml | 6,416 bytes |

## Lessons Learned

1. **The exact flags matter** - `-C panic=abort` and
   `-C panic=immediate-abort` are completely different.

2. **Version scripts are important** - They enable `--gc-sections` to remove more
   dead code by making symbols LOCAL.

3. **Nightly has two panic-abort flags** - The cargo `-Z panic-immediate-abort` and
   the rustc `-C panic=immediate-abort` work together.

4. **Debug by comparing outputs** - When sizes don't match, something is different.
   The only way to find it is careful comparison of the exact commands.

## Future Improvements

1. **Presets** - A single `optimized = true` flag that applies all these settings
2. **Target validation** - Warn if build_std is used without appropriate target
3. **Strip integration** - Add strip option to tspec
