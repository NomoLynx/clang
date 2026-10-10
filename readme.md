# CLang compiler summary

This directory contains the project’s C-language compiler implementation for the CLR toolchain. The compiler is implemented in Rust and is designed to parse a C-like source program, analyze its types and symbols, resolve semantic links, and emit RISC-V assembly that can then be assembled into a binary.

## Purpose

The compiler pipeline is centered around `CLangProgram` and the build entry point in `mod.rs`:

- `parse_clang(...)` parses the input text using the Pest grammar in `clang.pest`.
- `parse_clang_2nd(...)` runs the semantic and code-generation preparation passes.
- `build_clang(...)` reads a source file, compiles it, emits assembly, and builds the final output.

This is not a full general-purpose ISO C compiler; it is a focused compiler for a subset of C that targets the project’s internal assembly and RISC-V toolchain.

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
