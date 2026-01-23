# xt Design Notes

Design exploration for `xt` - a translation spec based build system to replace/complement `xtask`.

## Goals

- Compare different target triples and compile/linker commands across multiple apps
- Reproducible builds with fair comparisons
- Explore possibilities by trying specs and recording results
- Type-safe parameter specification leveraging Rust's type system

## Core Concepts

### Translation Specs (tspec)

Two-level specification system:

1. **Global specs** (`tspec/*.toml`) - Define compilation strategies
2. **Local config** (`<crate>/tspec/config.toml`) - Crate compatibility and local modifications

### Directory Structure

```
tspec/                              # Global translation specs
  musl-static.toml
  rlibc-x1.toml
  rlibc-x2.toml
  glibc-dynamic.toml

apps/xyz/
  tspec/
    config.toml                     # Compat/incompat lists + local mods
  src/
  Cargo.toml

target/
  musl-static-{hash}/               # Isolated by spec name + resolved hash
  rlibc-x2-{hash}/
```

### Local Config Schema

```toml
# apps/xyz/tspec/config.toml

compat = ["musl-static", "rlibc-x2"]      # allowlist (optional)
incompat = ["glibc-dynamic"]              # blocklist (optional)

[mods.musl-static]                        # local modifications (optional)
[mods.musl-static.linker.args]
add = ["-Wl,--gc-sections"]
remove = ["-Wl,--build-id"]
```

### Compatibility Logic

- No `tspec/config.toml` → all specs assumed compatible (optimistic default)
- `incompat = [...]` → blocklist, skip these
- `compat = [...]` → allowlist, only these (once curating)
- Both → compat minus incompat

### Target Directory Naming

Format: `{spec-name}-{hash}`

- Hash is computed from the fully resolved spec (global + local mods)
- Same hash suffix = identical resolved config (user can see redundancy)
- Different hash = different config, isolated builds

Example:
```
target/
  musl-static-abc123de/           # original
  musl-experimental-abc123de/     # same hash! user sees they're identical
  musl-tweaked-77889900/          # different hash, actually different
```

### Spec Resolution Flow

```
global:musl-static.toml
        ↓
  + apps/xyz/tspec/config.toml [mods.musl-static] (if exists)
        ↓
  = resolved spec
        ↓
  hash(resolved) → abc123de
        ↓
  --target-dir=target/musl-static-abc123de
```

## Commands

```bash
cargo xt build xyz -t musl-static         # build with spec
cargo xt build xyz -t all                 # all compatible specs
cargo xt run xyz -t musl-static           # build + run (skip if incompat)
cargo xt compat xyz                       # show compat state
cargo xt compat xyz musl-static           # add to compat list
cargo xt incompat xyz glibc-dynamic       # add to incompat list
cargo xt spec list                        # list global specs
cargo xt spec show musl-static            # show global spec
cargo xt spec show musl-static --crate xyz  # show resolved spec with local mods
cargo xt spec hash musl-static --crate xyz  # show hash
```

All commands explicit (no auto-inference from last build - can relax later).

## Parameter Specification Design

### Philosophy

- Leverage Rust's type system to prevent mistakes at compile time where possible
- Mutually exclusive params = single enum (can't have Release + Debug)
- Tool-specific params = separate types (can't pass linker args to rustc)
- Ordering preserved via Vec (reproducibility)
- Validation at load time for constraints that can't be compile-time enforced

### Type Structure

```rust
// Mutually exclusive choices are enums
enum Profile { Debug, Release }
enum OptLevel { O0, O1, O2, O3, Os, Oz }
enum PanicStrategy { Abort, Unwind }

// Each tool's params as enum variants
enum CargoParam {
    Profile(Profile),
    TargetTriple(String),
    TargetJson(PathBuf),
}

enum RustcParam {
    OptLevel(OptLevel),
    Panic(PanicStrategy),
    Lto(bool),
    CodegenUnits(u32),
    BuildStd(Vec<String>),
    Flag(String),
}

enum LinkerParam {
    Static,
    NoStdlib,
    Entry(String),
    GcSections,
    Arg(String),
}

// Spec is ordered Vecs - order preserved, serialization deterministic
struct Spec {
    cargo: Vec<CargoParam>,
    rustc: Vec<RustcParam>,
    linker: Vec<LinkerParam>,
}
```

### Arg Generation

```rust
trait ToArgs {
    fn to_args(&self) -> Vec<String>;
}

impl ToArgs for LinkerParam {
    fn to_args(&self) -> Vec<String> {
        match self {
            LinkerParam::Static => vec!["-static".into()],
            LinkerParam::Entry(e) => vec![format!("-e{e}")],
            LinkerParam::GcSections => vec!["-Wl,--gc-sections".into()],
            LinkerParam::Arg(s) => vec![s.clone()],
            // ...
        }
    }
}
```

### Validation

At construction/load time, validate:
- At most one Profile in cargo params
- At most one OptLevel in rustc params
- At most one Panic in rustc params
- etc.

Tradeoff: lose some compile-time exclusivity for single-choice params, but gain guaranteed iteration order and simpler mental model.

### Reproducibility

Critical for fair comparisons and consistent hashing:

1. Vec preserves insertion order
2. BTreeMap for any maps (not HashMap)
3. Serde serializes struct fields in definition order
4. Generation methods process in defined sequence

Same spec → same hash → same target dir → reproducible builds.

## Auto-generating Parameters

Potential to bootstrap enum definitions from tool help output:

```bash
rustc -C help          # lists all -C (codegen) options with types
rustc -Z help          # lists all -Z (unstable/nightly) options
cargo build --help     # cargo options
ld --help              # linker options
```

`rustc -C help` output is structured enough to parse:
```
-C opt-level=val       -- optimize with possible levels 0-3, s, or z
-C panic=val           -- panic strategy to compile crate with
-C lto=val             -- perform LLVM link-time optimizations
```

Could parse to generate initial enums, then manually curate for exclusivity rules.

## Open Questions

- Global spec TOML schema (how params serialize to/from TOML)
- How nightly-only features are flagged/validated
- Whether notes/ should capture rotating build logs
- Migration path from existing xtask-based workflow
