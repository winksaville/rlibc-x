# xt - Translation Spec Build System

A tspec-based build system for comparing target triples and compile/linker commands across apps.

**Status:** In development on `xt-dev` branch.

## Usage

```bash
cargo xt build ex-x1-xt                     # Build with crate's tspec.toml
cargo xt build ex-x1-xt -t tspec-expr.toml  # Build with experimental spec
cargo xt run ex-x1-xt                       # Build and run
cargo xt build rlibc-x1                     # Build library for development
```

The `-t` flag is for experimentation - pointing to alternative spec files.
Future: use `save_spec_snapshot` to create `tspec-001-abc123de.toml` variants.

## Design

See [notes/xt-design.md](../notes/xt-design.md) for full design documentation.

Key concepts:
- **Library specs** - Libraries define build requirements in `libs/<lib>/tspec.toml`
- **App specs** - Apps define their own in `apps/<app>/tspec.toml`
- **Target dir** - `target/{spec-name}-{hash}/` for isolation and reproducibility

### Quirks and Notes

**Duplicate tspec.toml files**: Libraries and apps using them often have identical
tspec.toml content. This is intentional:
- `libs/rlibc-x1/tspec.toml` enables `cargo xt build rlibc-x1` for library development
- `apps/ex-x1-xt/tspec.toml` enables `cargo xt build ex-x1-xt` for app builds
- When building an app, RUSTFLAGS apply to both the app and its dependencies,
  so they need compatible flags

**No build.rs for xt apps**: Apps built with xt (like `ex-x1-xt`) should NOT have
a `build.rs` that sets linker flags. RUSTFLAGS applies to all compilations including
build scripts, causing crashes if `-nostartfiles` is passed to a build script.
Use tspec.toml instead.

### Spec Saving

Two save formats support different workflows:

- **`save_spec(spec, path)`** - Canonical specs with simple names (`rlibc-x1.toml`)
- **`save_spec_snapshot(spec, name, dir)`** - Iteration snapshots as `{name}-{seq:03}-{hash}.toml`

Snapshots allow quick experimentation with automatic breadcrumbs. Instead of
disciplined git commits to preserve each iteration, snapshots accumulate as
`rlibc-x1-001-abc123de.toml`, `rlibc-x1-002-def456gh.toml`, etc. The sequence
number enables chronological sorting; the hash identifies content.

## Testing

```bash
cargo test -p xt              # run all xt tests
cargo test -p xt -- --nocapture  # see println output
cargo test -p xt spec_default    # run specific test
```

- Unit tests live alongside code in each module using `#[cfg(test)]` blocks
- Integration tests in `tests/` use fixtures from `tests/data/`

## Development Plan

### Next Steps

1. ~~Create `types.rs` - Spec parameter enums~~ Done
2. ~~Create `tspec.rs` - Loading and resolving specs from TOML~~ Done
3. ~~Create `libs/rlibc-x1/tspec.toml` - Spec for rlibc-x1~~ Done
4. ~~Implement build command~~ Done - `cargo xt build ex-x1-xt -t rlibc-x1`

### File Structure

```
xt/
  src/
    lib.rs          # Library root, exposes modules
    main.rs         # Entry point, dispatch
    cli.rs          # Clap CLI definitions
    types.rs        # Spec parameter types
    tspec.rs        # Spec loading/saving/hashing
    find_paths.rs   # Workspace/crate/tspec discovery
    build.rs        # Build command implementation
    run.rs          # Run command implementation
  tests/
    data/           # Test fixtures (TOML specs)
    tspec_test.rs   # Integration tests
```
