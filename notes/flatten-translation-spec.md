# Flattening the Translation Spec Format

## 20260129 - Investigation and Initial Implementation

### The Problem: Verbose TOML Format

The original tspec format used TOML's array-of-tables syntax (`[[section]]`), which
required repeating section names for each parameter:

```toml
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
args = ["-static", "-nostdlib", ...]

[[linker]]
version_script = { global = ["_start"], local = "*" }
```

This was verbose and the order didn't actually matter for most parameters.

### Design Goals

1. **Easy** - Simple, flat TOML that's quick to write and read
2. **Generality** - Flexible enough to handle various build configurations
3. **Type-checked** - Use Rust enums to catch typos at compile time

### Solution: Flat Struct with High-Level Options

Instead of nested sections with array-of-tables, use a flat structure with
high-level options that expand to multiple flags automatically.

#### Step 1: Flatten `[cargo]` Section

Changed from `Vec<CargoParam>` enum to flat `CargoConfig` struct:

```rust
pub struct CargoConfig {
    pub profile: Option<Profile>,
    pub target_triple: Option<String>,
    pub target_json: Option<PathBuf>,
    pub unstable: Vec<String>,
}
```

TOML changes from:
```toml
[[cargo]]
target_json = "..."

[[cargo]]
unstable = "..."
```

To:
```toml
[cargo]
target_json = "..."
unstable = ["..."]
```

#### Step 2: High-Level `panic` Option

Observed that `panic-immediate-abort` always requires TWO settings:
- Cargo: `-Z panic-immediate-abort` (rebuild std with immediate abort)
- Rustc: `-C panic=immediate-abort` (compile crate with immediate abort)

These were always used together but specified separately - redundant and error-prone.

Created `options.rs` with `PanicMode` enum:

```rust
pub enum PanicMode {
    Unwind,          // default
    Abort,           // rustc -C panic=abort
    ImmediateAbort,  // BOTH cargo -Z AND rustc -C (nightly)
}
```

The enum has methods that know which flags to generate:
- `requires_nightly()` - true only for `ImmediateAbort`
- `cargo_z_flag()` - returns cargo `-Z` flag if needed
- `rustc_panic_value()` - returns rustc `-C panic=` value

Now a single top-level field:
```toml
panic = "immediate-abort"
```

Replaces:
```toml
[cargo]
unstable = ["panic-immediate-abort"]

[[rustc]]
panic = "immediate-abort"
```

### Current tspec-opt.xt.toml

After these changes:

```toml
# High-level panic mode (sets both cargo -Z and rustc -C flags)
panic = "immediate-abort"

[cargo]
target_json = "x86_64-unknown-linux-rlibcx2.json"

[[rustc]]
build_std = ["std", "core", "panic_abort"]

[[rustc]]
flag = "-Z unstable-options"

[[rustc]]
flag = "-C link-arg=-Wl,--gc-sections"

[[linker]]
args = ["-static", "-nostdlib", "-nodefaultlibs", "-e_start", ...]

[[linker]]
version_script = { global = ["_start"], local = "*" }
```

### Future Work

Continue flattening toward fully flat format:

```toml
panic = "immediate-abort"
target_json = "x86_64-unknown-linux-rlibcx2.json"
build_std = ["std", "core", "panic_abort"]
flags = ["-Z unstable-options", "-C link-arg=-Wl,--gc-sections"]
linker_args = ["-static", "-nostdlib", ...]
version_script = { global = ["_start"], local = "*" }
```

No more `[cargo]`, `[[rustc]]`, `[[linker]]` sections - just fields.

### Branch

Work is on `refactor/tspec-flat-types` branch with commits:
1. `refactor(tspec): Flatten [cargo] section from array to struct`
2. `feat(tspec): Add top-level panic field with PanicMode enum`

---

## 20260130 - Flatten `[rustc]` Section

Continued from [20260129](#20260129---investigation-and-initial-implementation).

### Changes

Changed `Vec<RustcParam>` enum to flat `RustcConfig` struct:

```rust
pub struct RustcConfig {
    pub opt_level: Option<OptLevel>,
    pub panic: Option<PanicStrategy>,
    pub lto: Option<bool>,
    pub codegen_units: Option<u32>,
    pub build_std: Vec<String>,
    pub flags: Vec<String>,
}
```

TOML format changes from:
```toml
[[rustc]]
build_std = ["std", "core", "panic_abort"]

[[rustc]]
flag = "-Z unstable-options"

[[rustc]]
flag = "-C link-arg=-Wl,--gc-sections"
```

To:
```toml
[rustc]
build_std = ["std", "core", "panic_abort"]
flags = ["-Z unstable-options", "-C link-arg=-Wl,--gc-sections"]
```

### Files Modified

- `xt/src/types.rs` - `RustcParam` enum → `RustcConfig` struct
- `xt/src/cargo_build.rs` - Use struct fields instead of enum matching
- `xt/src/testing.rs` - Update `requires_nightly()` check
- `xt/src/tspec.rs` - Update tests
- `xt/tests/tspec_test.rs` - Update test assertions
- `xt/tests/data/minimal.toml` - New format
- `apps/ex-x2/tspec-opt.xt.toml` - New format
- `apps/hw-x2/tspec-opt.xt.toml` - New format

### Current tspec-opt.xt.toml

```toml
panic = "immediate-abort"

[cargo]
target_json = "x86_64-unknown-linux-rlibcx2.json"

[rustc]
build_std = ["std", "core", "panic_abort"]
flags = ["-Z unstable-options", "-C link-arg=-Wl,--gc-sections"]

[[linker]]
args = ["-static", "-nostdlib", ...]

[[linker]]
version_script = { global = ["_start"], local = "*" }
```

### Next

~~Flatten `[[linker]]` section → `[linker]` struct.~~ Done below.

---

## 20260130 - Flatten `[linker]` Section

Continued from [above](#20260130---flatten-rustc-section).

### Changes

Changed `Vec<LinkerParam>` enum to flat `LinkerConfig` struct:

```rust
pub struct LinkerConfig {
    pub args: Vec<String>,
    pub version_script: Option<VersionScript>,
}
```

TOML format changes from:
```toml
[[linker]]
args = ["-static", "-nostdlib", ...]

[[linker]]
version_script = { global = ["_start"], local = "*" }
```

To:
```toml
[linker]
args = ["-static", "-nostdlib", ...]
version_script = { global = ["_start"], local = "*" }
```

### Files Modified

- `xt/src/types.rs` - `LinkerParam` enum → `LinkerConfig` struct
- `xt/src/cargo_build.rs` - Use struct fields
- `xt/src/testing.rs` - Update linker args check
- `xt/src/tspec.rs` - Update tests
- `xt/tests/tspec_test.rs` - Update test assertions
- All `tspec*.xt.toml` files - New `[linker]` format

### Final tspec-opt.xt.toml Format

All sections now flat:

```toml
panic = "immediate-abort"

[cargo]
target_json = "x86_64-unknown-linux-rlibcx2.json"

[rustc]
build_std = ["std", "core", "panic_abort"]
flags = ["-Z unstable-options", "-C link-arg=-Wl,--gc-sections"]

[linker]
args = ["-static", "-nostdlib", ...]
version_script = { global = ["_start"], local = "*" }
```

### What's Next

All three sections are now flat structs. Possible future work:
- Lift fields to top level (eliminate sections entirely)
- Add more high-level options like `panic` (e.g., `strip`, `lto`)
