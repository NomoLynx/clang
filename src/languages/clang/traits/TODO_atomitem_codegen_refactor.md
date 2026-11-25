# TODO: Add `AsmCodeGenTrait` for `AtomItem` (refactored, no duplication)

## Problem

`generate_dot_reference_code<T>` requires `T: AsmCodeGenTrait + ASTItemTrait + GetCLangTypeTrait`.

`AtomItem` satisfies `ASTItemTrait` and `GetCLangTypeTrait`, but has **no** `impl AsmCodeGenTrait for AtomItem`,
so it cannot be passed as `item` to `generate_dot_reference_code`.

## Plan

### Step 1 — Extract a shared helper in `asm_codegen_trait.rs`

The `Id` match body in `impl AsmCodeGenTrait for Atom` is long and identical to what
`AtomItem` would need. Extract it into a free function:

```rust
fn generate_atom_item_code(
    item: &AtomItem,
    ast_item: &ASTItem,
    context: &mut CodeGenContext,
) -> Result<(), CLangError> {
    match item {
        AtomItem::BasicTypeObject(_, v) => { /* existing BasicTypeObject logic */ }
        AtomItem::Id(_, v) => { /* existing Id logic (Var / Declaration / EnumConstant / StructUnionItem) */ }
        AtomItem::Expression(_, n) => n.generate_code(context),
        AtomItem::TanryExpression(_, n) => n.generate_code(context),
    }
}
```

### Step 2 — Delegate from `impl AsmCodeGenTrait for Atom`

```rust
impl AsmCodeGenTrait for Atom {
    fn generate_code(&self, context: &mut CodeGenContext) -> Result<(), CLangError> {
        generate_atom_item_code(&**self.get_item(), self.get_ast_item(), context)
    }
}
```

### Step 3 — Add `impl AsmCodeGenTrait for AtomItem`

```rust
impl AsmCodeGenTrait for AtomItem {
    fn generate_code(&self, context: &mut CodeGenContext) -> Result<(), CLangError> {
        generate_atom_item_code(self, self.get_ast_item(), context)
    }
}
```

After these three steps `AtomItem` satisfies all bounds on `generate_dot_reference_code`
with zero logic duplication.

## Files to change

| File | Change |
|------|--------|
| `src/languages/clang/traits/asm_codegen_trait.rs` | Extract `generate_atom_item_code`, update `Atom` impl, add `AtomItem` impl |

## Verification

Run the existing test suite after the change:

```sh
cargo test --all
```

No behaviour should change; `Atom` still delegates to the same code path.
