# Interactive tspec Management

Design thoughts for Phase 3 - enabling fast iteration on build specifications
without manual TOML editing.

## Motivation

Currently, experimenting with build configurations requires:
1. Open tspec.toml in editor
2. Manually write TOML syntax
3. Save file
4. Run build
5. Check results
6. Repeat

This friction slows down exploration. An interactive CLI would enable:
```bash
cargo xt tspec add myapp --rustc build_std=std,core,panic_abort
cargo xt build myapp -r
# Check size, try another option...
cargo xt tspec add myapp --linker version_script.global=_start
cargo xt build myapp -r
```

## Proposed Commands

### List and Show

```bash
cargo xt tspec list                    # All tspec.toml files in workspace
cargo xt tspec list ex-x2-xt           # All tspec*.toml for a crate
cargo xt tspec show ex-x2-xt           # Show default tspec.toml
cargo xt tspec show ex-x2-xt -t opt    # Show tspec-opt.toml
```

### Create

```bash
cargo xt tspec new myapp               # Create empty tspec.toml
cargo xt tspec new myapp --from ex-x2-xt  # Copy from another crate
cargo xt tspec new myapp -t experiment    # Create tspec-experiment.toml
```

### Add Options

```bash
# Cargo params
cargo xt tspec add myapp --cargo profile=release
cargo xt tspec add myapp --cargo target_triple=x86_64-unknown-linux-musl
cargo xt tspec add myapp --cargo unstable=panic-immediate-abort

# Rustc params
cargo xt tspec add myapp --rustc build_std=std,core,panic_abort
cargo xt tspec add myapp --rustc panic=immediate-abort
cargo xt tspec add myapp --rustc lto=true
cargo xt tspec add myapp --rustc flag="-Z unstable-options"

# Linker params
cargo xt tspec add myapp --linker args=-static,-nostdlib
cargo xt tspec add myapp --linker version_script.global=_start
```

### Remove Options

```bash
cargo xt tspec remove myapp --rustc panic
cargo xt tspec remove myapp --linker version_script
cargo xt tspec remove myapp --cargo unstable  # Remove all unstable flags
```

### Diff and Compare

```bash
cargo xt tspec diff ex-x2-xt tspec.toml tspec-opt.toml
cargo xt tspec diff ex-x1-xt ex-x2-xt  # Compare default specs
```

## Alternative Syntax: Inline Modifiers

More fluid, git-like syntax:

```bash
cargo xt tspec ex-x2-xt +rustc.build_std=std,core,panic_abort
cargo xt tspec ex-x2-xt +cargo.unstable=panic-immediate-abort
cargo xt tspec ex-x2-xt -rustc.panic
cargo xt tspec ex-x2-xt +linker.args=-static
```

Pros:
- Concise, single command for multiple changes
- Familiar +/- pattern from git, diff tools

Cons:
- More complex parsing
- Harder to discover (less --help friendly)
- Quoting gets tricky with shell

## Snapshot Integration

Each modification auto-creates a snapshot for undo/history:

```
apps/ex-x2-xt/
  tspec.toml                      # Current working spec
  .tspec/
    ex-x2-xt-001-abc123.toml      # Initial
    ex-x2-xt-002-def456.toml      # After adding build_std
    ex-x2-xt-003-789abc.toml      # After adding panic
```

Commands to work with snapshots:

```bash
cargo xt tspec history ex-x2-xt           # List snapshots
cargo xt tspec restore ex-x2-xt 002       # Restore snapshot 002
cargo xt tspec compare ex-x2-xt 001 003   # Compare two snapshots
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
$ cargo xt tspec add myapp --rustc panic=invalid
error: invalid panic strategy 'invalid'
  valid options: abort, unwind, immediate-abort
```

## Build Integration

Could combine tspec modification with immediate build:

```bash
cargo xt tspec add myapp --rustc lto=true --build -r
# Equivalent to:
#   cargo xt tspec add myapp --rustc lto=true
#   cargo xt build myapp -r
```

Or even inline temporary modifications (don't save):

```bash
cargo xt build myapp -r --with "rustc.lto=true"
# Builds with lto=true but doesn't modify tspec.toml
```

## Open Questions

1. **Subcommand vs inline syntax?** `tspec add` is more discoverable, `+/-` is more fluid
2. **Auto-snapshot?** Every change, or only on explicit save?
3. **Snapshot location?** `.tspec/` subdir, or `tspec-snapshots/` at workspace root?
4. **Conflict handling?** Error on duplicate params, or replace silently?
5. **Tab completion?** Generate shell completions for param names and valid values?

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
