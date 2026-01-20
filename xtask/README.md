# xtask

Project automation using the [xtask pattern](https://github.com/matklad/cargo-xtask).

## Usage

```bash
cargo xtask <command> [options] [crates...]
```

## Commands

### build

Build crates one at a time.

```bash
cargo xtask build                 # build all crates
cargo xtask build ex-x1           # build specific crate
cargo xtask build ex-x1 -r        # build with release profile
```

### run

Build and run crates, showing exit codes. Library-only crates (no `src/main.rs`) are automatically skipped.

```bash
cargo xtask run hw-x1             # run specific crate
cargo xtask run .                 # run crate in current directory
cargo xtask run                   # run all binary crates
```

### test

Run tests with a summary at the end.

```bash
cargo xtask test                  # test all crates
cargo xtask test ex-x1            # test specific crate
cargo xtask test rlibc-x2         # also runs rlibc-x2-tests binaries
```

## Options

| Option | Description |
|--------|-------------|
| `-q, --quiet` | Suppress cargo output (default is verbose) |
| `-f, --fail-fast` | Stop on first failure |
| `-d, --debug` | Use debug builds (default) |
| `-r, --release` | Use release builds |
| `-opt, --optimized` | Use nightly optimizations for x2 crates (smaller binaries) |
| `-s, --strip` | Strip debug symbols after building |
| `-h, --help` | Show help |

## Features

- **One crate at a time**: Each crate is processed individually for clear output
- **Musl auto-detection**: Crates with "musl" in the name use `--target x86_64-unknown-linux-musl`
- **Current directory**: Use `.` to operate on the crate in the current directory
- **Exit codes**: The `run` command shows exit codes for each binary
- **Binary path**: The `build` command shows the path to the built binary
- **Smart stripping**: With `-s`, `run` only strips if the binary was rebuilt (uses mtime)

## Optimized Builds

The `-opt` flag enables nightly optimizations for x2 crates (those using rlibc-x2):

- **Nightly toolchain** - Required for unstable features
- **`-Z build-std`** - Rebuilds std from source for better optimization
- **`-Z panic-immediate-abort`** - Eliminates panic formatting code
- **Linker version script** - Enables dead code elimination via `--gc-sections`
- **Custom target** - `x86_64-unknown-linux-rlibcx2.json`

```bash
# Stable build (larger, ~41KB stripped)
cargo xtask build ex-x2 -r -s

# Nightly optimized build (smaller, ~6KB stripped)
cargo xtask build ex-x2 -r -opt -s
```

Without `-opt`, x2 crates build with the stable toolchain, producing larger but more portable binaries.
