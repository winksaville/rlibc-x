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
- **Library specs** - Each library defines its build requirements in `libs/<lib>/tspec.toml`
- **App config** - Per-app compat/incompat lists (planned: `apps/<app>/tspec.toml`)
- **Target dir** - `target/{spec-name}-{hash}/` for isolation and reproducibility

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
4. Implement build command - Get `cargo xt build rlibc-x1 -t rlibc-x1` working

### File Structure

```
xt/
  src/
    lib.rs          # Library root, exposes modules
    main.rs         # Entry point, dispatch
    cli.rs          # Clap CLI definitions
    types.rs        # Spec parameter types
    tspec.rs        # Spec loading/saving/hashing
    commands/       # (planned)
  tests/
    data/           # Test fixtures (TOML specs)
    tspec_test.rs   # Integration tests
```
