# Example and Comparison Apps

Binary size comparison across different runtimes.

## Size Comparison

All sizes are release builds, stripped.

| App | Runtime | Linking | Stable | Nightly | Description |
|-----|---------|---------|-------:|--------:|-------------|
| ex-x1 | rlibc-x1 | static | 1.4 KB | - | Minimal exit-only (no_std) |
| ex-x2 | rlibc-x2 | static | 41 KB | 6.4 KB | Minimal exit-only (std) |
| ex-glibc | glibc | dynamic | 298 KB | 9.3 KB | Minimal exit-only |
| ex-musl | musl | static | 381 KB | 22.7 KB | Minimal exit-only |
| hw-x1 | rlibc-x1 | static | 1.6 KB | - | Hello world |
| hw-x2 | rlibc-x2 | static | 46 KB | 8.9 KB | Hello world |
| hw-glibc | glibc | dynamic | 287 KB | 11.9 KB | Hello world |
| hw-musl | musl | static | 377 KB | 26.8 KB | Hello world |

**Nightly** column shows sizes with `-opt` flag. See [notes/opt-notes.md](../notes/opt-notes.md) for details.

## App Naming Convention

- **ex-** = exit-only (just returns an exit code)
- **hw-** = hello world (prints to stdout)
- **-x1** = uses rlibc-x1 (no_std)
- **-x2** = uses rlibc-x2 (std)
- **-glibc** = uses system glibc
- **-musl** = uses musl libc

## Building and Running

```bash
cargo xtask build ex-x1 -r -s          # release + stripped
cargo xtask build ex-glibc -r -opt -s  # optimized (nightly)
cargo xtask run hw-x1                  # build and run
```

## Testing

Each app includes a test that verifies it doesn't use libc (for x1/x2 variants):

```bash
cargo xtask test ex-x1 hw-x1   # test specific apps
cargo xtask test               # test all
```
