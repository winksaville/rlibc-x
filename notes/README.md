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

### Workflow

See [xt/README.md](../xt/README.md#workflow) for:
- **Progress Tracking** - done-todo.md usage
- **Completing Changes** - end-of-session checklist
- **Claude Code & Git** - handling .claude/ files and merges
