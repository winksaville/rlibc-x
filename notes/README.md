# Notes Directory

Technical documentation and session notes for the rlibc-x project.

## Conventions

### File Types

- **Technical docs** (`*.md`) - Deep dives on specific topics (e.g., `build-std.md`, `plt-less-linking.md`)
- **Design logs** - Append-only dated sections tracking design evolution (e.g., `flatten-translation-spec.md`)
- **Session handoff** (`claude-next.md`) - Current state for Claude Code session continuity

### Design Logs

Use dated sections, append-only:

```markdown
## 20260130 - Short Description

What was done, decisions made, code references.

### Next

What's pending.
```

- Don't modify old sections (except marking items done, adding forward links)
- Link between related sections: `See [section](#anchor)`

### Session Handoff (`claude-next.md`)

Current state only, not history. Consumed by Claude Code at session start.

Contents:
- Current branch and what it's for
- What's done (brief)
- What's next (actionable)
- Key files to look at
- Quick test commands

Overwrite entirely each session - history lives in design logs.

### Completing Changes

When finishing a set of changes:

1. Update relevant design log with new dated section
2. Update `claude-next.md` with current state
3. Update `xt-dev.md` if xt tooling changed (link to design log section)
4. Run verification loop:
   ```bash
   cargo xt test xt && cargo xt test
   cargo clippy --workspace --all-targets
   cargo fmt --check
   ```
5. Commit with conventional commit message
