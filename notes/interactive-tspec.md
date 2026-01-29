# Interactive tspec Management

Design thoughts for Phase 3 - enabling fast iteration on build specifications
without manual TOML editing.

## File Naming Convention

### File Type

The `.xt.toml` suffix identifies a tspec file. Any `*.xt.toml` file is valid.

### Forms

| Form | Pattern | Example |
|------|---------|---------|
| Simple | `<name>.xt.toml` | `opt.xt.toml`, `musl.xt.toml` |
| Versioned | `<name>.<NNN>-<HHHHHHHH>.xt.toml` | `opt.001-a7f3b2c1.xt.toml` |

- **Seqnum (NNN)**: 3-digit sequence number, scoped per-base-name
- **Hash (HHHHHHHH)**: 8 hex characters, content-based (identical content = identical hash)

### Optional

If no `*.xt.toml` exists for a crate, cargo defaults apply.

### Example Directory Evolution

```
apps/ex-x2/
  # Initially empty - uses cargo defaults

  # User creates first spec via CLI
  opt.xt.toml

  # CLI modification creates versioned snapshot
  opt.001-a1b2c3d4.xt.toml

  # Another CLI modification
  opt.002-e5f6a7b8.xt.toml

  # User creates another variant
  experiment.xt.toml
  experiment.001-deadbeef.xt.toml
```

## Command Alias

The tspec subcommand has a short alias:
- `cargo xt tspec` - full form (for discoverability)
- `cargo xt ts` - short form (for daily use)

All examples below use the short form.

## Motivation

Currently, experimenting with build configurations requires:
1. Open `*.xt.toml` in editor
2. Manually write TOML syntax
3. Save file
4. Run build
5. Check results
6. Repeat

This friction slows down exploration. An interactive CLI would enable:
```bash
cargo xt ts add myapp --rustc build_std=std,core,panic_abort
cargo xt build myapp -r
# Check size, try another option...
cargo xt ts add myapp --linker version_script.global=_start
cargo xt build myapp -r
```

## Proposed Commands

### List and Show

```bash
cargo xt ts list                    # All *.xt.toml files in workspace
cargo xt ts list ex-x2              # All *.xt.toml for a crate
cargo xt ts show ex-x2              # Show all *.xt.toml for a crate
cargo xt ts show ex-x2 -t tspec-opt # Show specific tspec
```

### Create

```bash
cargo xt ts new myapp               # Create tspec.xt.toml (conventional default name)
cargo xt ts new myapp --from ex-x2  # Copy from another crate
cargo xt ts new myapp -t experiment # Create experiment.xt.toml
```

### Add Options

```bash
# Cargo params
cargo xt ts add myapp --cargo profile=release
cargo xt ts add myapp --cargo target_triple=x86_64-unknown-linux-musl
cargo xt ts add myapp --cargo unstable=panic-immediate-abort

# Rustc params
cargo xt ts add myapp --rustc build_std=std,core,panic_abort
cargo xt ts add myapp --rustc panic=immediate-abort
cargo xt ts add myapp --rustc lto=true
cargo xt ts add myapp --rustc flag="-Z unstable-options"

# Linker params
cargo xt ts add myapp --linker args=-static,-nostdlib
cargo xt ts add myapp --linker version_script.global=_start
```

### Remove Options

```bash
cargo xt ts remove myapp --rustc panic
cargo xt ts remove myapp --linker version_script
cargo xt ts remove myapp --cargo unstable  # Remove all unstable flags
```

### Diff and Compare

```bash
cargo xt ts diff ex-x2 base.xt.toml opt.xt.toml
cargo xt ts diff ex-x1 ex-x2  # Compare default specs
```

## Alternative Syntax: Inline Modifiers

More fluid, git-like syntax:

```bash
cargo xt ts ex-x2 +rustc.build_std=std,core,panic_abort
cargo xt ts ex-x2 +cargo.unstable=panic-immediate-abort
cargo xt ts ex-x2 -rustc.panic
cargo xt ts ex-x2 +linker.args=-static
```

Pros:
- Concise, single command for multiple changes
- Familiar +/- pattern from git, diff tools

Cons:
- More complex parsing
- Harder to discover (less --help friendly)
- Quoting gets tricky with shell

## Snapshot Integration

Each CLI modification auto-creates a versioned snapshot using the naming convention:

```
apps/ex-x2/
  opt.xt.toml                     # Current working spec
  opt.001-a1b2c3d4.xt.toml        # Initial (content hash: a1b2c3d4)
  opt.002-e5f6a7b8.xt.toml        # After adding build_std
  opt.003-deadbeef.xt.toml        # After adding panic
```

The seqnum (NNN) provides ordering within a base name. The hash (8 hex chars) is
derived from file content, so identical configurations produce identical hashes.

Commands to work with snapshots:

```bash
cargo xt ts history ex-x2                 # List all versioned snapshots
cargo xt ts history ex-x2 -t opt          # List snapshots for opt.xt.toml
cargo xt ts restore ex-x2 opt.002         # Restore opt to snapshot 002
cargo xt ts compare ex-x2 opt.001 opt.003 # Compare two snapshots
```

## Value Parsing

Need to handle various value types from CLI strings:

| Type | CLI Input | Parsed |
|------|-----------|--------|
| String | `target_triple=x86_64-unknown-linux-musl` | `"x86_64-unknown-linux-musl"` |
| Bool | `lto=true` or `lto` | `true` |
| Number | `codegen_units=1` | `1` |
| List | `build_std=std,core,panic_abort` | `["std", "core", "panic_abort"]` |
| List | `args=-static,-nostdlib` | `["-static", "-nostdlib"]` |
| Nested | `version_script.global=_start` | `{global: ["_start"]}` |

## Validation

On add/remove, validate:
- Parameter exists in the type system
- Value parses correctly for the parameter type
- No duplicate conflicting params (e.g., two different panic strategies)

```bash
$ cargo xt ts add myapp --rustc panic=invalid
error: invalid panic strategy 'invalid'
  valid options: abort, unwind, immediate-abort
```

## Build Integration

Could combine tspec modification with immediate build:

```bash
cargo xt ts add myapp --rustc lto=true --build -r
# Equivalent to:
#   cargo xt ts add myapp --rustc lto=true
#   cargo xt build myapp -r
```

Or even inline temporary modifications (don't save):

```bash
cargo xt build myapp -r --with "rustc.lto=true"
# Builds with lto=true but doesn't modify *.xt.toml
```

## Open Questions

1. **Subcommand vs inline syntax?** `ts add` is more discoverable, `+/-` is more fluid
2. **Auto-snapshot?** Every change, or only on explicit save?
3. **Conflict handling?** Error on duplicate params, or replace silently?
4. **Tab completion?** Generate shell completions for param names and valid values?

## Future Enhancements

1. **Shell alias `xts`**: For power users, could add a separate `xts` binary or
   document a shell alias (`alias xts='cargo xt ts'`) for even shorter commands:
   ```bash
   xts add myapp --rustc lto=true
   xts history myapp
   ```

## Implementation Notes

The type system in `types.rs` already encodes valid parameters. Could use it to:
- Generate help text for valid options
- Validate CLI input at parse time
- Auto-complete parameter names

```rust
// Potential trait for CLI integration
trait CliParam {
    fn cli_name(&self) -> &'static str;
    fn cli_help(&self) -> &'static str;
    fn parse_value(s: &str) -> Result<Self>;
}
```

## Related

- [notes/build-std.md](build-std.md) - Build-std implementation details
- [notes/xt-design.md](xt-design.md) - Original xt design
- [notes/xt-dev.md](xt-dev.md) - Development notes
