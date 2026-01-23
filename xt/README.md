# xt - Translation Spec Build System

A tspec-based build system for comparing target triples and compile/linker commands across apps.

**Status:** In development on `xt-dev` branch.

## Usage

```bash
cargo xt build rlibc-x1 -t rlibc-x1     # Build with spec
cargo xt build rlibc-x1 -t all          # All compatible specs
cargo xt run hw-x1 -t rlibc-x1          # Build and run
cargo xt compat hw-x1                   # Show compat state
cargo xt compat hw-x1 rlibc-x1          # Add to compat list
cargo xt incompat hw-x1 glibc-dynamic   # Add to incompat list
cargo xt spec list                      # List global specs
cargo xt spec show rlibc-x1             # Show spec details
```

## Design

See [notes/xt-design.md](../notes/xt-design.md) for full design documentation.

Key concepts:
- **Global tspec/** - Translation specs defining compilation strategies
- **Local tspec/config.toml** - Per-crate compat/incompat lists and local modifications
- **Target dir** - `target/{spec-name}-{hash}/` for isolation and reproducibility

## Development Plan

### Next Steps

1. Create `types.rs` - Spec parameter enums (Profile, OptLevel, CargoParam, RustcParam, LinkerParam)
2. Create `tspec.rs` - Loading and resolving specs from TOML
3. Create `tspec/rlibc-x1.toml` - Minimal global spec for rlibc-x1
4. Implement build command - Get `cargo xt build rlibc-x1 -t rlibc-x1` working

### File Structure (planned)

```
xt/src/
  main.rs           # Entry point, dispatch
  cli.rs            # Clap CLI definitions
  types.rs          # Spec parameter types
  tspec.rs          # Spec loading/resolution
  commands/
    build.rs
    run.rs
    compat.rs
    spec.rs
```
