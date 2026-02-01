# Done / Todo

Working document for tracking progress. Items reference discussions in goals files.

## Todo

- `--comment` flag for ts set [1]
- ts add command (append to lists)
- ts remove command (remove from lists)
- lto high-level option [2]
- Make xt/README.md more generic (remove repo-specific .claude/ content)

## Done

- Unified `-p` option for all commands (ts, build, run, test, compare), default to cwd [1,7]
- Added `-a, --all` flag to build/run/test to force all-packages mode [7]
- ts set command with versioned snapshots [3]
- strip high-level option [3,4]
- ts new command [5]
- Rename `.xt.toml` → `.ts.toml` [6]
- Refactor ts commands to ts_cmd/ directory [5]

## References

[1]: goals-20260130.md#20260130-6---open-issues
[2]: goals-20260130.md#potential-high-level-options
[3]: goals-20260130.md#20260130-5---strip-and-ts-set-implementation
[4]: goals-20260130.md#20260130-4---add-strip-support
[5]: goals-20260130.md#20260130-3---session-summary
[6]: goals-20260130.md#decision-file-suffix-xttomml-vs-tstoml
[7]: goals-20260130.md#20260131-7---refactor-to--p---package-option
