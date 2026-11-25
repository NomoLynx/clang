use crate::languages::clang::CLangItem;
use crate::languages::clang::clang_asm_code_gen::RegisterUsage;
use crate::languages::clang::symbol_table::RegisterName;
use crate::languages::clang::symbol_table::SymbolType;
use core_utils::range::Range;

/// A collection of register use types for tracking register allocations
/// and reusability based on symbol types and variable ranges.
/// the element has symbol type, clang item, register names, and variable range
#[derive(Debug, Clone)]
pub struct RegisterUseTypes {
    register_uses: Vec<RegisterUseType>,
}

impl RegisterUseTypes {
    pub fn push(&mut self, item:RegisterUseType) {
        self.register_uses.push(item)
    }

    pub fn contains_register_name(&self, reg_name:&RegisterName) -> bool {
        for item in self.register_uses.iter() {
            if item.register_names.contains(reg_name) {
                return true;
            }
        }

        false
    }

    pub fn contains_register(&self, reg:&RegisterUsage) -> bool {
        let reg_name = reg.get_reg_name();
        let reg_alias = reg.get_reg_alias();

        self.contains_register_name(&reg_name.to_string()) ||
        self.contains_register_name(&reg_alias.to_string())
    }

    /// Find a reusable register name based on symbol type, variable range, and blacklist
    /// symbol_type is used to match the varialbe scope (e.g. compound or function)
    pub fn get_reusable_register_name(&self, symbol_type:SymbolType, var_range:Option<&Range>, blacklist_reg:&Vec<RegisterName>) -> Option<RegisterName>  {
        if var_range.is_none() {
            return None;
        }

        // reuse register if possible, the symbol_type is used to match the register use type
        for item in self.register_uses.iter() {
            if item.symbol_type != symbol_type {
                continue;
            }

            if item.is_overlap(var_range.unwrap()) {
                continue;
            }

            // get the first register name that is not in blacklist
            for reg_name in item.register_names.iter() {
                if !blacklist_reg.contains(reg_name) {
                    return Some(reg_name.clone());
                }
            }
        }

        None
    }
}

impl Default for RegisterUseTypes {
    fn default() -> Self {
        Self { register_uses: Vec::default(), }
    }
}

#[derive(Debug, Clone)]
pub struct RegisterUseType {
    pub symbol_type: SymbolType,
    pub clang_item: CLangItem,
    pub register_names: Vec<RegisterName>,
    pub range: Range,
}

impl RegisterUseType {
    pub fn new(symbol_type: SymbolType, clang_item: CLangItem, register_names: Vec<RegisterName>, range: Range) -> Self {
        Self {
            symbol_type,
            clang_item,
            register_names,
            range,
        }
    }

    pub fn is_overlap(&self, other:&Range) -> bool {
        self.range.overlaps(other)
    }

    pub fn get_first_register(&self) -> Option<&RegisterName> {
        self.register_names.first()
    }
}

impl From<(SymbolType, CLangItem, Vec<RegisterName>, Range)> for RegisterUseType {
    fn from(value: (SymbolType, CLangItem, Vec<RegisterName>, Range)) -> Self {
        Self::new(value.0, value.1, value.2, value.3)
    }
}