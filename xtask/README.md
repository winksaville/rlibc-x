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
| `-opt, --optimized` | Use nightly optimizations (x2, glibc, musl) |
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

The `-opt` flag enables nightly optimizations for x2, glibc, and musl crates:

```bash
cargo xtask build ex-x2 -r -opt -s     # ~6 KB (vs ~41 KB)
cargo xtask build ex-glibc -r -opt -s  # ~9 KB (vs ~298 KB)
cargo xtask build ex-musl -r -opt -s   # ~23 KB (vs ~381 KB)
```

See [notes/opt-notes.md](../notes/opt-notes.md) for details on how this works.
