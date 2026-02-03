# Chore 4: Transition from xt/ to tspec

## Context

The `xt/` build system has been extracted and enhanced as a standalone project:
- Location: `../tspec` (https://github.com/winksaville/tspec)
- Enhancements: POP (Plain Old Package) support, `tspec clean`, version 0.2.0

## Goal

Delete `xt/` from rlibc-x and use external tspec instead.

## Approaches

### 1. Global Install (User Responsibility)

**Setup:** User installs tspec from repo or crates.io.

```bash
cargo install --path ../tspec   # from local clone
cargo install tspec             # from crates.io (future)
```

**Usage:** `tspec build`, `tspec run`, etc.

**Pros:**
- Simplest for this repo (just delete xt/, update docs)
- Single tspec binary serves all projects
- Clean separation of concerns

**Cons:**
- Requires user action before building
- Version mismatch possible if tspec evolves
- `cargo xt` workflow breaks (doc changes needed)

---

### 2. Companion Project (Clone Locally)

**Setup:** Clone tspec alongside rlibc-x, optionally alias.

```bash
cd ~/projects
git clone https://github.com/winksaville/tspec
cd rlibc-x
alias xt='cargo run --manifest-path ../tspec/Cargo.toml --'
```

**Usage:** `xt build`, `xt run` (via alias) or `cargo run -p tspec --manifest-path ../tspec/Cargo.toml -- build`

**Pros:**
- Easy to hack on tspec while using it
- Can pin to specific commit/branch
- Alias preserves `xt` muscle memory

**Cons:**
- Requires specific directory layout
- Manual version management
- Not self-contained

---

### 3. Global Install + Cargo Alias

**Setup:** Install globally, add cargo alias for familiar invocation.

```bash
cargo install --path ../tspec

# In ~/.cargo/config.toml:
[alias]
xt = "tspec"
# or for subcommand style:
# xt = ["--", "tspec"]  # doesn't work - cargo aliases are for cargo subcommands
```

**Note:** Cargo aliases only work for cargo subcommands, not external binaries. So `cargo xt` can't directly invoke an external `tspec` binary.

**Alternative:** Shell alias in `.bashrc`/`.zshrc`:
```bash
alias 'cargo xt'='tspec'
```

**Pros:**
- Preserves `cargo xt` or `xt` invocation style
- Single global install

**Cons:**
- Shell alias is per-user, not in repo
- Cargo alias limitation is confusing

---

### 4. Git Submodule

**Setup:** Add tspec as a git submodule.

```bash
git submodule add https://github.com/winksaville/tspec tools/tspec
```

**Usage:**
```bash
cargo run --manifest-path tools/tspec/Cargo.toml -- build
# Or with alias:
alias xt='cargo run --manifest-path tools/tspec/Cargo.toml --'
```

**Pros:**
- Self-contained (cloning rlibc-x gets tspec too)
- Pinned to specific tspec commit
- Can update submodule pointer as tspec evolves

**Cons:**
- Submodule complexity (init, update, etc.)
- Still need alias for ergonomic usage
- Two git repos to manage

---

### 5. Thin xtask Wrapper

**Setup:** Keep minimal `xt/` that just calls installed tspec.

```rust
// xt/src/main.rs
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let status = std::process::Command::new("tspec")
        .args(&args)
        .status()
        .expect("tspec not found - install with: cargo install tspec");
    std::process::exit(status.code().unwrap_or(1));
}
```

**Usage:** `cargo xt build` (unchanged!)

**Pros:**
- Zero doc changes for rlibc-x
- Familiar `cargo xt` workflow preserved
- Delegates all logic to external tspec
- Clear error if tspec not installed

**Cons:**
- Still have an xt/ directory (though tiny)
- Requires tspec pre-installed
- Extra indirection

---

### 6. Path Dependency (Development Only)

**Setup:** Add tspec as workspace member via path.

```toml
# Cargo.toml
[workspace]
members = ["libs/*", "apps/*", "tools/*", "tspec"]

# Requires symlink or expects ../tspec
[patch.crates-io]
tspec = { path = "../tspec" }
```

**Note:** This gets complicated because tspec isn't published and path dependencies outside workspace are tricky.

**Verdict:** Not recommended - too fragile.

---

## Recommendation

**Option 5 (Thin Wrapper)** offers the best balance:
- Preserves existing `cargo xt` workflow and all documentation
- Minimal code to maintain (~10 lines)
- Clear dependency on external tspec
- Easy upgrade path (just update tspec install)

**Runner-up: Option 1 (Global Install)** if we're okay updating all docs to use `tspec` instead of `cargo xt`.

## Decision

**Option 1: Global Install**

User installs tspec, invokes as `tspec xxx` instead of `cargo xt xxx`.

Rationale:
- Cleanest separation - tspec is its own project
- No vestigial wrapper code in rlibc-x
- Doc updates are straightforward
- If customization needed later, can pivot to git submodule (Option 4)

## Actions

- [ ] Delete xt/ directory
- [ ] Remove xt from workspace members in Cargo.toml
- [ ] Update CLAUDE.md (`cargo xt` → `tspec`)
- [ ] Update notes/todo.md if it references xt
- [ ] Check other notes/*.md for xt references
- [ ] Test documented workflows with tspec
