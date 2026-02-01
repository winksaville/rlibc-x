# Notes Directory

Technical documentation and session notes for the rlibc-x project.

## Conventions

### File Types

- **Technical docs** (`*.md`) - Deep dives on specific topics (e.g., `build-std.md`, `plt-less-linking.md`)
- **Design logs** - Append-only dated sections tracking design evolution (e.g., `flatten-translation-spec.md`)
- **Progress tracking** (`todo.md`) - Current Todo/Done status

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

### Workflow

See [xt/README.md](../xt/README.md#workflow) for:
- **Progress Tracking** - todo.md usage
- **Completing Changes** - end-of-session checklist
- **Claude Code & Git** - handling .claude/ files and merges
