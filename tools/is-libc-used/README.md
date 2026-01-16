# is-libc-used

Check if an ELF binary uses libc.

## CLI Usage

```bash
# Check a binary (quiet mode, uses exit code)
is-libc-used ./target/debug/ex-x1
echo $?  # 0 = no libc, 1 = uses libc, 2 = error

# Verbose mode shows details
is-libc-used -v /usr/bin/ls
# INTERP: /lib64/ld-linux-x86-64.so.2 (needs dynamic linker)
# NEEDED: libcap.so.2, libc.so.6
# Result: uses libc

is-libc-used -v ./target/debug/ex-x1
# INTERP: none (no dynamic linker needed)
# NEEDED: none
# Result: no libc
```

## Library Usage

```rust
use is_libc_used::is_libc_used;
use std::path::Path;

let result = is_libc_used(Path::new("./my-binary"))?;

if result.uses_libc {
    println!("Binary uses libc");
} else {
    println!("Binary is libc-free");
}

// Details available in result.info
for line in &result.info {
    println!("{}", line);
}
```

## Checks Performed

Two authoritative checks determine if a binary uses libc:

1. **INTERP** - Checks for PT_INTERP program header. If present, the binary requires a dynamic linker (e.g., `/lib64/ld-linux-x86-64.so.2`).

2. **NEEDED** - Checks for DT_NEEDED entries in the dynamic section. These are shared libraries the binary links against (e.g., `libc.so.6`).

A binary is considered libc-free only if both checks pass (no INTERP and no NEEDED).

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Binary does NOT use libc |
| 1 | Binary DOES use libc |
| 2 | Error (file not found, parse error, etc.) |

## Testing Apps for libc Usage

To verify an app binary doesn't use libc via `cargo test -p <app>`:

### For no_std apps (rlibc-x1)

Apps using rlibc-x1 define a `#[panic_handler]`, which conflicts with std's panic handler.
You **cannot** use the library as a dev-dependency. Instead, invoke the binary via `Command`.

**Cargo.toml:**
```toml
[[bin]]
name = "my-app"
test = false  # Prevent cargo from compiling binary in test mode

[[test]]
name = "no_libc"
harness = false  # Custom test binary, not std test harness
```

**build.rs** - Use bin-specific linker args so test binaries aren't affected:
```rust
fn main() {
    // Only applies to the binary, not tests
    println!("cargo:rustc-link-arg-bin=my-app=-nostartfiles");
    println!("cargo:rustc-link-arg-bin=my-app=-static");
}
```

**tests/no_libc.rs:**
```rust
use std::path::Path;
use std::process::{Command, ExitCode};

fn main() -> ExitCode {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().parent().unwrap();

    let is_libc_used = ["target/release/is-libc-used", "target/debug/is-libc-used"]
        .iter().map(|p| workspace_root.join(p)).find(|p| p.exists())
        .expect("is-libc-used not found");

    let binary = ["target/release/my-app", "target/debug/my-app"]
        .iter().map(|p| workspace_root.join(p)).find(|p| p.exists())
        .expect("my-app not found");

    let output = Command::new(&is_libc_used).arg(&binary).output().unwrap();
    if output.status.success() { ExitCode::SUCCESS } else { ExitCode::FAILURE }
}
```

### For std apps (rlibc-x2)

Apps using rlibc-x2 can use the library directly as a dev-dependency.
However, you still need `test = false` on the binary and bin-specific linker args
in build.rs, otherwise the test binary gets the wrong linker flags applied.

**Cargo.toml:**
```toml
[dev-dependencies]
is-libc-used = { path = "../../tools/is-libc-used" }

[[bin]]
name = "my-app"
test = false  # Prevent cargo from compiling binary in test mode
```

**build.rs** - Use bin-specific linker args so test binaries aren't affected.

The syntax is `cargo:rustc-link-arg-bin=<BIN_NAME>=<LINKER_ARG>` where:
- `<BIN_NAME>` is your binary name (e.g., `my-app`)
- `<LINKER_ARG>` is the linker flag (e.g., `-static`, `-nostdlib`, ...)

```rust
fn main() {
    // All of these link-arg only apply to the "my-app" binary, not the tests
    println!("cargo:rustc-link-arg-bin=my-app=-static");
    println!("cargo:rustc-link-arg-bin=my-app=-nostdlib");
    println!("cargo:rustc-link-arg-bin=my-app=-nodefaultlibs");
    println!("cargo:rustc-link-arg-bin=my-app=-e_start");
    println!("cargo:rustc-link-arg-bin=my-app=-Wl,--undefined=_start");
    println!("cargo:rustc-link-arg-bin=my-app=-Wl,--undefined=__libc_start_main");
}
```

**tests/no_libc.rs:**
```rust
use is_libc_used::is_libc_used;
use std::path::Path;

#[test]
fn binary_does_not_use_libc() {
    let binary = env!("CARGO_BIN_EXE_my-app");
    let result = is_libc_used(Path::new(binary)).unwrap();
    assert!(!result.uses_libc, "should not use libc: {:?}", result.info);
}
```
