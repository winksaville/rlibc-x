# func-analysis

Analyze libc function sizes and reference counts in ELF binaries.

## Features

- **Static binaries** (musl, rlibc-x2): Shows both app functions and embedded libc functions with sizes and call counts
- **Dynamic binaries** (glibc): Shows app functions + imported libc functions (sizes auto-detected from system libc)
- **Compare mode**: Compare function sizes between two binaries (e.g., rlibc-x2 vs musl)
- **Multiple formats**: Table (default), JSON, CSV

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
# Compare rlibc-x2 vs musl (rlibc-funcs.txt is at workspace root)
cargo run -p func-analysis -- compare rlibc-funcs.txt \
    target/release/ex-x2 \
    target/x86_64-unknown-linux-musl/release/ex-musl
```

The function list file (e.g., `rlibc-funcs.txt` at workspace root) contains function names (one per line):
```
abort
calloc
free
malloc
write
...
```

## Output Examples

### Static binary (musl)

Shows both Rust app functions and embedded musl libc functions:

```
Binary: target/x86_64-unknown-linux-musl/release/ex-musl
Type: static
Functions: 73
Functions size: 9215 bytes
.text size: 9357 bytes
Coverage: 98.5%

FUNCTION                                       SIZE      ADDRESS     REFS
--------------------------------------------------------------------------
__syscall_ret                                    48       0x21e5       17
__errno_location                                 14       0x1f2d        6
__syscall_cp                                      5       0x2d58        5
...
write                                            46       0x2da1        0
__stdio_close                                    35       0x32f9        0
lseek                                            21       0x3427        0
```

### Dynamic binary (glibc)

Shows app functions (from `.symtab`) + imported libc functions (sizes from system libc):

```
Binary: target/x86_64-unknown-linux-gnu/release/ex-glibc
Type: dynamic
Functions: 38
Functions size: 7284 bytes
.text size: 2470 bytes
Coverage: 294.9%

FUNCTION                                       SIZE      ADDRESS     REFS
--------------------------------------------------------------------------
_RINvNtCslLel16OqMrf_4core3ptr13drop_...         88       0x1f29        3
free                                            445       0x3a20        2
_RNvMs0_NtNtNtCskm7OexE29z0_3std2io5e...         75       0x2300        2
...
pause                                            35       0x3aa0        0
dup                                              37       0x3a60        0
poll                                             32       0x3a50        0
```

Coverage >100% because "Functions size" includes libc function sizes (external), while ".text size" is local code only.

See [notes/opt-notes.md](../../notes/opt-notes.md) for details on preserving symbols for analysis.

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
   - Static: All FUNC symbols with size > 0 from `.symtab` and `.dynsym`
   - Dynamic: Local functions from `.symtab` + imported functions via PLT/GOT relocations
3. **Disassemble**: Uses `iced-x86` to disassemble executable sections
4. **Count references**: Identifies `call` instructions (direct and indirect) and maps targets to known functions
5. **Libc sizes**: For dynamic binaries, auto-detects system libc via `ldd` and extracts function sizes
6. **Report**: Aggregates and formats the results

See [notes/plt-less-linking.md](../../notes/plt-less-linking.md) for details on how PLT vs GLOB_DAT relocations are handled.

## Module Structure

```
src/
├── main.rs        # CLI parsing and orchestration
├── analyze.rs     # Static/dynamic binary analysis
├── compare.rs     # Binary comparison functionality
├── disasm.rs      # Disassembly and reference counting (iced-x86)
├── elf_utils.rs   # ELF utilities (text size, PLT map, libc sizes)
├── output.rs      # Output formatting (table, JSON, CSV)
├── types.rs       # Core data types (FunctionInfo, AnalysisResult)
└── test_utils.rs  # Shared test helpers
```

The disassembly logic is isolated in `disasm.rs`, making it straightforward to swap disassemblers (e.g., switch from capstone to iced-x86).

## Limitations

- Indirect calls (`call *%rax`) cannot be resolved statically (except RIP-relative GOT calls)
- Tail calls (`jmp` used as call) are not counted
- Inlined functions won't appear in symbol table
- Stripped binaries show only `.dynsym` symbols (use `cargo xtask build <app> -r -opt` without `-s` for full symbols)

## Dependencies

- `goblin` - ELF parsing
- `iced-x86` - Disassembly (pure Rust, x86/x86-64)
- `clap` - CLI argument parsing
- `serde`/`serde_json` - JSON output
- `anyhow` - Error handling
- `is-libc-used` - Detect static vs dynamic linking
