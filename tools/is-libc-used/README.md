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
