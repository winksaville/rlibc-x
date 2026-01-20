# Example and Comparison Apps

Binary size comparison across different runtimes.

## Size Comparison

All sizes are release builds, stripped.

| App | Runtime | Linking | Stable | Nightly | Description |
|-----|---------|---------|-------:|--------:|-------------|
| ex-x1 | rlibc-x1 | static | 1.4 KB | - | Minimal exit-only (no_std) |
| ex-x2 | rlibc-x2 | static | 41 KB | 6 KB | Minimal exit-only (std) |
| ex-glibc | glibc | dynamic | 284 KB | - | Minimal exit-only |
| ex-musl | musl | static | 373 KB | - | Minimal exit-only |
| hw-x1 | rlibc-x1 | static | 1.5 KB | - | Hello world |
| hw-x2 | rlibc-x2 | static | 46 KB | 9 KB | Hello world |
| hw-glibc | glibc | dynamic | 287 KB | - | Hello world |
| hw-musl | musl | static | 377 KB | - | Hello world |

**Nightly** column shows sizes with `-opt` flag (nightly + build-std + panic-immediate-abort).

## App Naming Convention

- **ex-** = exit-only (just returns an exit code)
- **hw-** = hello world (prints to stdout)
- **-x1** = uses rlibc-x1 (no_std)
- **-x2** = uses rlibc-x2 (std)
- **-glibc** = uses system glibc
- **-musl** = uses musl libc

## Building and Running

```bash
# Using xtask (recommended)
cargo xtask build ex-x1        # debug build
cargo xtask build ex-x1 -r     # release build
cargo xtask build ex-x1 -r -s  # release + stripped
cargo xtask run hw-x1          # build and run

# Optimized x2 builds (nightly required)
cargo xtask build ex-x2 -r -opt -s  # ~6KB instead of ~41KB

# Direct cargo commands
cargo build -p ex-x1 --release
cargo run -p hw-x1 --release

# glibc/musl targets (use aliases)
cargo b-glibc -p ex-glibc --release
cargo b-musl -p ex-musl --release
```

## Testing

Each app includes a test that verifies it doesn't use libc (for x1/x2 variants):

```bash
cargo xtask test ex-x1 hw-x1   # test specific apps
cargo xtask test               # test all
```
