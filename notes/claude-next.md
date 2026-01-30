# Claude Code Session Handoff

## Usage

This file is for Claude Code session continuity. Read at session start to resume context.
Overwritten each session - history lives in design logs.

See [Completing Changes](README.md#completing-changes) for end-of-session checklist.

---

## Current State (20260130)

**Branch:** `refactor/tspec-flat-types` - flattening tspec TOML format

**Done:**
- `[cargo]` flattened (struct, not array-of-tables)
- Top-level `panic` field with `PanicMode` enum
- `[rustc]` flattened (struct, not array-of-tables)

**Next:**
- Flatten `[[linker]]` section → `[linker]` struct

**Key files:**
- `xt/src/types.rs` - `Spec`, `CargoConfig`, `RustcConfig`, `LinkerParam`
- `xt/src/options.rs` - `PanicMode` enum
- `xt/src/cargo_build.rs` - applies spec to cargo commands
- `apps/ex-x2/tspec-opt.xt.toml` - example using new format
- `notes/flatten-translation-spec.md` - detailed design log

**Quick test:**
```bash
cargo xt test xt
cargo xt build ex-x2 -r -t tspec-opt.xt.toml
```
