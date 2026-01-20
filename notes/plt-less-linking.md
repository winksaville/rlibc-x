# PLT-less Linking in Modern ELF Binaries

## 20260120 - Discovery: Optimized glibc binaries use GLOB_DAT instead of PLT

When building with `-opt` (nightly + build-std + panic=immediate-abort), the linker
produces binaries without a traditional `.plt` section. Instead of:

```
call printf@plt        ; jump to PLT stub
```

The binary uses direct GOT calls:

```
call *0x1b93(%rip)     ; call through GOT entry directly
```

### Traditional PLT linking

```
.rela.plt:  R_X86_64_JUMP_SLOT relocations
.plt:       PLT stubs (16 bytes each)
.got.plt:   GOT entries for lazy binding

Code: call <plt_stub> -> plt_stub jumps through GOT
```

### PLT-less linking (GLOB_DAT)

```
.rela.dyn:  R_X86_64_GLOB_DAT relocations
.got:       GOT entries (no lazy binding)
No .plt section

Code: call *offset(%rip) -> direct GOT call
```

### Why this happens

The optimized build flags likely trigger the linker to use `-z now` (immediate binding)
which eliminates the need for PLT stubs. The GOT entries are resolved at load time,
so there's no need for the lazy binding infrastructure.

### Impact on func-analysis

The `func-analysis` tool needed updates to handle both styles:

1. `build_plt_map()` now checks `.rela.dyn` for `GLOB_DAT` relocations when no `.plt` exists
2. `extract_call_target()` now resolves RIP-relative indirect calls (`call *offset(%rip)`)
3. Call detection includes `FlowControl::IndirectCall` for indirect calls

### Verification

```bash
# Check if binary has .plt section
readelf -S binary | grep '\.plt'

# Check relocation types
readelf -r binary | grep -E 'JUMP_SLOT|GLOB_DAT'

# Traditional PLT: JUMP_SLOT in .rela.plt
# PLT-less: GLOB_DAT in .rela.dyn
```
