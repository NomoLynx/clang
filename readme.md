# CLang compiler summary

This directory contains the project’s C-language compiler implementation for the CLR toolchain. The compiler is implemented in Rust and is designed to parse a C-like source program, analyze its types and symbols, resolve semantic links, and emit RISC-V assembly that can then be assembled into a binary.

More code and components will be gradually **migrated from our local development server to this GitHub repository**.

## Purpose

The compiler pipeline is centered around `CLangProgram` and the build entry point in `mod.rs`:

- `parse_clang(...)` parses the input text using the Pest grammar in `clang.pest`.
- `parse_clang_2nd(...)` runs the semantic and code-generation preparation passes.
- `build_clang(...)` reads a source file, compiles it, emits assembly, and builds the final output.

This is not a full general-purpose ISO C compiler; it is a focused compiler for a subset of C that targets the project’s internal assembly and RISC-V toolchain.

The intended source baseline is K&R/pre-ANSI C. The current grammar also
accepts some later extensions, including `//` comments and `const`/`volatile`;
this is not a claim of strict dialect conformance.

## Compiler pipeline

### 1. Parsing and AST creation

The compiler starts by reading a file and feeding it to the grammar parser. The grammar is defined in `clang.pest`, and the parser creates a `CLangProgram` tree from `external_declaration` items such as:

- function definitions
- declarations
- type/struct/enum definitions
- other top-level C constructs

The AST is represented through items such as `ASTItem`, `CLangItem`, and the declaration/expression nodes in the source tree.

### 2. Symbol-table and semantic analysis

After parsing, the compiler builds symbol tables and semantic linkages. This pass identifies:

- variable declarations and their types
- function declarations and definitions
- pointer, array, struct, and union relationships
- address-taken variables
- references that link a usage site back to its declaration/definition

This is handled by the `symbol_table`, `sematic_linkage`, and `logic` modules and by the `setup_symbol_table` / `setup_semantic_linkages` passes.

### 3. Type system and conversions

The compiler includes a type model for C-like types, including:

- integer types
- floating-point types
- pointers and arrays
- `struct` and `union` types
- enum constants

Type handling is orchestrated through the `type_system`, `type_specifier`, `clang_types`, and `struct_union` modules. It also includes conversion logic and explicit mapping for supported numeric conversions and instruction templates.

### 4. Register allocation and code generation

Once semantic and type checks are complete, the compiler assigns storage/register locations and then generates machine-level code. The `clang_asm_code_gen` subsystem provides the backend for:

- instruction mapping
- code generation context
- register usage tracking
- type conversion snippets
- assembly template generation

The generated assembly is then passed to the project’s assembly tooling (`r5asm`), which creates the final binary or output artifact.

## Supported C-style functionality

The implementation covers the kinds of constructs typically required for a small C compiler target, including:

- declarations and function definitions
- assignments and expressions
- arithmetic and logical operators
- conditional and loop statements
- jump and label statements
- pointer and address-of operations
- array and struct-based data layouts
- enum and constant expression support
- internal code generation for RISC-V-style assembly

## Main submodules

- `declaration/` — declaration parsing and declarator handling
- `struct_union/` — struct and union definitions
- `enum_type/` — enum parsing and constants
- `constant_expression/` — constant expression evaluation and casting
- `symbol_table/` — symbol and type metadata
- `clang_asm_code_gen/` — lowering to assembly and backend mapping
- `type_system/` — semantic type logic
- `logic/` — analysis and linkage helpers
- `traits/` — shared compiler behaviors and schema helpers

## High-level flow

1. Read a C source file.
2. Parse it using the CLang Pest grammar.
3. Build the AST and declaration tree.
4. Build the symbol table and semantic links.
5. Validate and normalize types.
6. Resolve structs, unions, and addresses.
7. Assign registers/storage.
8. Generate assembly snippets and instruction mappings.
9. Assemble the output into executable/binary form.

## Summary

The CLang implementation in this directory is a Rust-based C compiler frontend and backend aimed at compiling a useful subset of C into assembly for the project’s RISC-V ecosystem. It combines parsing, AST modeling, semantic analysis, symbol resolution, type checking, register assignment, and code generation into one pipeline.

## Opt-in RVV integer elementwise operations

Set `CLangBuildParameters.enable_rvv = true` to enable the RV64 + V 1.0
elementwise pilot. It defaults to `false` in all supplied build configurations.
The compiler reports `RVV elementwise lowerings: N` when enabled.

The matrix fixtures use K&R-style source constructs: block comments,
declarations before executable statements, fixed-size arrays and indices
declared outside `for`. It requires no ANSI function prototypes, C99 syntax
or headers, vector intrinsics or inline assembly. The `Int32` restriction is
a target-type requirement, independent of the source-language revision.

The supported ordinary-C pattern is a nest of two or three `for` loops with separately
declared `int` indices, literal positive bounds matching every complete array
dimensions, initialization to zero and increments of the form `i = i + 1`.
The only inner statement must be elementwise `c[...] = a[...] + b[...]` or
`c[...] = a[...] - b[...]`, using three
distinct, unqualified, directly declared local 32-bit integer arrays of the
same shape, with stack offsets proven 4-byte aligned. Globals have no checked
alignment guarantee in the current data emitter and keep scalar lowering,
as do custom data-stack configurations, pointer/parameter
aliases, partial/transposed accesses, casts, qualified objects, additional
side effects, other operations and ranks above three. A function containing
unmodelled inline assembly is also excluded: raw register values and vector
configuration cannot be assumed dead.

The backend linearizes this side-effect-free row-major nest and emits real
`vsetvli`, `vle32.v`, `vadd.vv`/`vsub.vv` and `vse32.v` instructions with `e32,m1,ta,ma`.
Each iteration advances by the returned VL, not a hardcoded lane count.
Dimension products and byte spans are checked before flattening. Every
induction variable retains its original final value. Subtraction keeps the
left input in `vs2` and the right input in `vs1`; it is not reverse subtraction.

[The lowering](clang_asm_code_gen/rvv_elementwise.rs) separates the canonical
nest/shape match, operation, scratch bindings and instruction emission.
Adding another operation requires a justified instruction/oracle and tests;
adding another rank requires complete index/shape matching and final-state checks.

### Local liveness and scratch ownership

Snippet outputs are returned values. Declared temporaries are internal scratch;
impacted registers include semantic writes such as updated induction variables.
`get_contaminated_registers()` conservatively collects these effects, including
child effects. It is **not** a live-out set, and an output in a register need
not mean the snippet wrote that register (a variable read can return its location).

At a producer/consumer boundary, `live_outputs_with(live_through)` protects the
producer's results plus values still owned by an enclosing computation. Dead
producer scratch is not protected merely because it was used. Fixed inputs and
intentional in-place outputs are bound explicitly at their last use. Persistent
C variables use their assigned S/argument registers or memory; their state is
not treated as anonymous, reusable T-register scratch.

[CodeGenContext](clang_asm_code_gen/code_gen_context.rs) provides scoped
`with_live_registers` and atomic `with_scratch_registers` reservations. The RVV
kernel requests five GPR temporaries and three LMUL=1 vector groups, excluding
live values and outstanding reservations. `t0`-`t4` and `v8`-`v10` are only the
deterministic first choices, not ISA requirements or a caller-saved-is-free rule.
Other legal bindings are selected when those registers are live. `v0` is
reserved for masks; group selection accounts for integer LMUL alignment and
whole-group overlap. Group-reservation tests cover LMUL 1/2/4/8, but generated
kernels remain e32/m1. Reservations release on success, error and unwinding.

If a full safe reservation is unavailable, vectorization declines with an
explicit scalar-fallback diagnostic. The kernel does not add stack saves.
It declares its actual owned scratch and legitimate counter writes; `vl` and
`vtype` are clobbered. This is a local composition contract, not global CFG/SSA
liveness analysis, arbitrary inline-assembly support or a vector calling ABI.

The scalar conflict resolver first relocates conflicting result registers into
genuinely free destinations outside the consumer footprint, protected values
and fixed bindings. Remaining scratch conflicts are renamed if possible, or
their original physical registers are saved/restored in a verified frame.
Results are never restored over. If no safe result destination exists, the
compiler reports a register-conflict error rather than an unsafe swap or panic.
Parallel input-renaming cycles use the existing guarded replacement helper.
Disjoint input replacements are applied directly and do not consume guard
registers; only chains/cycles use guarded staging.
Multi-input binding first relocates non-input scratch/results away from the
producer registers. Producer values are not fixed template roles before binding;
only the template's declared placeholder inputs may be rebound in place. This
prevents compound stores from overwriting their address and arithmetic indices
from aliasing the dimension-multiplier scratch.

### Checked function frames

Stack allocation assigns naturally aligned base offsets with checked size and
offset arithmetic. A byte array preceding an Int32 matrix now receives the
necessary padding instead of making that matrix unaligned. Existing struct/union
member layout is unchanged; this does not establish complete external C ABI
layout compatibility.

Before body generation, the compiler verifies every assigned local/parameter
slot's size, alignment, FP-relative bounds and non-overlap across nested scopes.
The body reserves the complete lowest-offset frame, rounded to 16 bytes, rather
than summing only its immediate symbol table. Matrix addresses remain
`fp + SymbolLocation::Stack(offset)`, not `sp` or an assumed argument position.
Generic conflict saves require this verified frame and preserve 16-byte SP
alignment and LIFO restoration. This covers frame ownership for these snippets,
not a proof of the compiler's complete calling convention.
Large in-place address adjustments use bounded `addi` chunks instead of
clobbering their base with `lui`. SP chunks retain 16-byte alignment, including
aliases. Complete frames above 2048 bytes are rejected explicitly: existing
scalar FP-relative address/load templates cannot yet represent wider offsets.
Such rejection is not counted as a passing executable.

Build the resulting assembly through `riscv_asm_lib`, then run its ELF under
an RVV-enabled QEMU CPU. Acceptance requires both observed guest exit 42/142
and evidence that the actual ELF contains and executes the selected vector block.
An exit of 42 from a scalar-only build does not validate this optimization.
Assembly serialization must retain the `vsetvli` configuration, vector masks
and memory operands; the assembler has a parse/print/reparse regression for
these fields. The fixtures in `NomoLynx/test_code_data` cover 2D add (`test74.c`),
asymmetric signed 2D subtraction and byte-prefix alignment (`test75.c`),
nonuniform 2x3x5 add/subtract with partial vector tails (`test76.c`/`test77.c`),
and a call following nested-scope allocation (`test78.c`). Inputs, all results,
final indices and memory guards are checked independently of the vector operation.
The lowering follows the [RISC-V V 1.0 specification](https://docs.riscv.org/reference/isa/v20260120/unpriv/v-st-ext.html).
