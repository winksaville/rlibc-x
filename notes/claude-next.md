# Claude Code Session Handoff

## Usage

This file is for Claude Code session continuity. Read at session start to resume context.
Overwritten each session - history lives in design logs.

**Remind the user** of these workflows:
- [Completing Changes](README.md#completing-changes) - end-of-session checklist
- [Claude Code & Git](README.md#claude-code--git) - exit Claude before merging!

---

## Current State (20260130)

**Branch:** `main`

**Done this session:**
- Renamed `.xt.toml` → `.ts.toml` (content-centric naming)
- Refactored `tspec_cmd.rs` → `ts_cmd/` directory
- Added `ts new` command with tests
- Added `find_tspec_files` tests

**tspec CLI commands:**
```bash
cargo xt ts list [crate]              # List tspec files
cargo xt ts show <crate> [-t spec]    # Show contents
cargo xt ts hash <crate> [-t spec]    # Show hash
cargo xt ts new <crate> [name] [-f source]  # Create new
```

**Next:**
- `ts set` - Set scalar values
- `ts add` - Append to list values
- `ts remove` - Remove from lists
- File versioning on modification
- High-level options: `strip`, `lto`

**Key files:**
- `xt/src/ts_cmd/` - ts subcommands (list, show, hash, new)
- `xt/src/types.rs` - `Spec` with `PartialEq, Eq`
- `notes/goals-20260130.md` - detailed planning

**Quick test:**
```bash
cargo xt test xt
cargo xt test
cargo xt build ex-x2 -r -t tspec-opt.ts.toml
```
