# func-analysis

Analyze libc function sizes and reference counts in ELF binaries.

## Features

- **Static binaries** (rlibc-x1/x2): Extracts function sizes from symbol table and counts call references via disassembly
- **Dynamic binaries** (glibc): Identifies imported libc functions, counts PLT references, optionally looks up sizes from libc.so
- **Compare mode**: Compare function sizes between two binaries (e.g., rlibc-x2 vs musl)

## Commands

### analyze

Analyze function sizes and references in a binary.

```bash
# Basic analysis of a static binary
cargo run -p func-analysis -- analyze target/release/ex-x2

# Analyze with JSON output
cargo run -p func-analysis -- -f json analyze target/release/ex-x2

# Filter to specific functions
cargo run -p func-analysis -- -F malloc analyze target/release/ex-x2

# Show only functions called 5+ times, sorted by size
cargo run -p func-analysis -- --min-refs 5 -s size analyze target/release/ex-x2

# Verbose mode (shows each call site)
cargo run -p func-analysis -- -v analyze target/release/ex-x2
```

### compare

Compare function sizes between two binaries using a list of function names.

```bash
# Compare rlibc-x2 vs musl
cargo run -p func-analysis -- compare rlibc-funcs.txt \
    target/release/ex-x2 \
    target/x86_64-unknown-linux-musl/release/ex-musl
```

The `rlibc-funcs.txt` file contains function names (one per line):
```
abort
calloc
free
malloc
write
...
```

## Output Formats

### Table (default)
```
Binary: target/release/ex-x2
Type: static
Functions: 165
.text size: 24541 bytes
Total code size: 23867 bytes (97.3% coverage)

FUNCTION                                       SIZE      ADDRESS     REFS
--------------------------------------------------------------------------
__libc_start_main                               189     0x205fe4        1
malloc                                          108     0x205da0        0
...
```

### Compare output
```
FUNCTION                                        ex-x2      ex-musl      RATIO
----------------------------------------------------------------------------
abort                                              15          150      10.0x
malloc                                            108            5       0.0x
sigaction                                           3          130      43.3x
...
----------------------------------------------------------------------------
TOTAL (41 functions)                             1056         5590       5.3x
```

### JSON (`-f json`)
```json
{
  "binary_path": "target/release/ex-x2",
  "is_dynamic": false,
  "total_functions": 165,
  "total_code_size": 23867,
  "text_section_size": 24541,
  "functions": [
    {"name": "malloc", "size": 108, "address": 2120096, "references": 0, "source": "local"}
  ]
}
```

### CSV (`-f csv`)
```
name,size,address,references,source
malloc,108,0x205da0,0,local
```

## How It Works

1. **Parse ELF**: Uses `goblin` to parse the binary's symbol tables, section headers, and relocation entries
2. **Collect functions**:
   - Static: All FUNC symbols with size > 0 from both `.symtab` and `.dynsym`
   - Dynamic: UND (undefined) symbols that reference glibc
3. **Disassemble**: Uses `capstone` to disassemble executable sections
4. **Count references**: Identifies `call` instructions and maps targets to known functions
5. **Report**: Aggregates and formats the results

## Module Structure

```
src/
├── main.rs        # CLI parsing and orchestration
├── analyze.rs     # Static/dynamic binary analysis
├── compare.rs     # Binary comparison functionality
├── disasm.rs      # Disassembly and reference counting (capstone)
├── elf_utils.rs   # ELF utilities (text size, PLT map, libc sizes)
├── output.rs      # Output formatting (table, JSON, CSV)
├── types.rs       # Core data types (FunctionInfo, AnalysisResult)
└── test_utils.rs  # Shared test helpers
```

The disassembly logic is isolated in `disasm.rs`, making it straightforward to swap disassemblers (e.g., switch from capstone to iced-x86).

## Limitations

- Indirect calls (`call *%rax`) cannot be resolved statically
- Tail calls (`jmp` used as call) are not counted by default
- PLT layout assumptions may not hold for all linkers
- Inlined functions won't appear in symbol table
- Stripped binaries will have low coverage (only `.dynsym` symbols available)

## Dependencies

- `goblin` - ELF parsing
- `capstone` - Disassembly
- `clap` - CLI argument parsing
- `serde`/`serde_json` - JSON output
- `anyhow` - Error handling
- `is-libc-used` - Detect static vs dynamic linking
