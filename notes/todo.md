# Todo

Working document for tracking progress. Items reference discussions in chores files.

## Todo

- Discuss xt relationship to cargo and standalone operation [3,4,5]
-- Make xt/README.md more generic
-- Update /README.md to document all xt commands including ts
- xt clean command (clean, build, run, test covers 98% of usage)
- xt should work for any cargo package, not just workspaces
- ts add command (append to lists)
- ts remove command (remove from lists)
- lto high-level option [2]
- Preserve/restore comments in tspec files [1]
- Consider allowing claude to do complete commits by having .claude/* a separate repo may eliminates circular "references"[5]

## Done

## References

[1]: chores-20260201.md#20260201-comments-lost-on-serialization
[2]: chores-20260130.md#potential-high-level-options
[3]: chores-20260201.md#20260201---xt-relationship-to-cargo
[4]: chores-20260201.md#20260201---the-double-cargo-problem
[5]: /README.md#claude-code-sessions