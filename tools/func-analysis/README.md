# func-analysis

Analyze libc function sizes and reference counts in ELF binaries.

## Features

- **Static binaries** (rlibc-x1/x2): Extracts function sizes from symbol table and counts call references via disassembly
- **Dynamic binaries** (glibc): Identifies imported libc functions, counts PLT references, optionally looks up sizes from libc.so

## Usage

```bash
# Basic analysis of a static binary
cargo run -p func-analysis -- target/release/ex-x1

# Analyze with JSON output
cargo run -p func-analysis -- -f json target/release/ex-x1

# Filter to specific functions
cargo run -p func-analysis -- -F malloc target/release/ex-x2

# Show only functions called 5+ times, sorted by size
cargo run -p func-analysis -- --min-refs 5 -s size target/release/ex-x2

# Include all symbols (not just libc-related)
cargo run -p func-analysis -- -a target/release/ex-x1

# Dynamic binary with libc size lookup
cargo run -p func-analysis -- --libc-path /lib/x86_64-linux-gnu/libc.so.6 /usr/bin/ls

# Verbose mode (shows each call site)
cargo run -p func-analysis -- -v target/release/ex-x1
```

## Output Formats

### Table (default)
```
Binary: target/release/ex-x1
Type: static
Functions: 12
Total code size: 1480 bytes

FUNCTION                                       SIZE      ADDRESS     REFS
--------------------------------------------------------------------------
write                                            42       0x401100        3
malloc                                          128       0x401200        2
exit                                             18       0x401300        1
```

### JSON (`-f json`)
```json
{
  "binary_path": "target/release/ex-x1",
  "is_dynamic": false,
  "total_functions": 12,
  "total_code_size": 1480,
  "functions": [
    {"name": "write", "size": 42, "address": 4198656, "references": 3, "source": "local"}
  ]
}
```

### CSV (`-f csv`)
```
name,size,address,references,source
write,42,0x401100,3,local
```

## Comparison Workflow

To compare rlibc-x vs glibc for the same application:

```bash
# Build both versions
cargo xtask build -r

# Analyze rlibc-x1 version
cargo run -p func-analysis -- target/release/ex-x1 > ex-x1-analysis.txt

# Analyze glibc version (if you have one)
cargo run -p func-analysis -- --libc-path /lib/x86_64-linux-gnu/libc.so.6 \
    target/release/ex-glibc > ex-glibc-analysis.txt

# Compare
diff ex-x1-analysis.txt ex-glibc-analysis.txt
```

## How It Works

1. **Parse ELF**: Uses `goblin` to parse the binary's symbol tables, section headers, and relocation entries
2. **Collect functions**: 
   - Static: All FUNC symbols with size > 0
   - Dynamic: UND (undefined) symbols that reference glibc
3. **Disassemble**: Uses `capstone` to disassemble executable sections
4. **Count references**: Identifies `call` instructions and maps targets to known functions
5. **Report**: Aggregates and formats the results

## Limitations

- Indirect calls (`call *%rax`) cannot be resolved statically
- Tail calls (`jmp` used as call) are not counted by default
- PLT layout assumptions may not hold for all linkers
- Inlined functions won't appear in symbol table

## Dependencies

- `goblin` - ELF parsing
- `capstone` - Disassembly
- `clap` - CLI argument parsing
- `serde`/`serde_json` - JSON output
- `anyhow` - Error handling
