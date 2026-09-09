pub (crate) mod symbol_table;
pub (crate) mod display_diagram_trait;
pub (crate) mod to_symbol_table_element_trait;
pub (crate) mod to_markdown_table_row_trait;
pub (crate) mod scope_trait;
pub (crate) mod asm_codegen_trait;
pub (crate) mod to_symbol_table_elements_trait;
pub (crate) mod ast_item_trait;
pub (crate) mod get_clangtype_trait;
pub (crate) mod state_diagram_node_trait;

pub use ast_item_trait::*;
pub use asm_codegen_trait::*;
pub use get_clangtype_trait::*;
pub use state_diagram_node_trait::*;
pub use symbol_table::*;
pub use to_markdown_table_row_trait::*;
pub use to_symbol_table_element_trait::*;
pub use to_symbol_table_elements_trait::*;

/// generate RISC-V assembly code to add an offset to a base register and store the result in a destination register
pub fn gen_add_offset(rd: &str, rs1: &str, offset: isize) -> String {
    // addi immediate range
    const ADDI_MIN: isize = -2048;
    const ADDI_MAX: isize = 2047;

    // Try to fit offset into addi
    if offset <= ADDI_MAX && offset >= ADDI_MIN {
        return format!(
            "addi {}, {}, {}",
            rd, rs1, offset
        );
    }

    // Otherwise: use lui + addi
    // RISC-V rule: addi sign-extends, so low 12 bits must be signed
    let offset = offset as isize;

    // Split constant with sign-correct rounding
    let hi = (offset + 0x800) >> 12;
    let lo = offset - (hi << 12);

    debug_assert!(lo >= ADDI_MIN && lo <= ADDI_MAX);

    format!(
        "lui  {rd}, {hi}\n\
         add  {rd}, {rd}, {rs1}\n\
         addi {rd}, {rd}, {lo}",
        rd = rd,
        rs1 = rs1,
        hi = hi,
        lo = lo
    )
}
