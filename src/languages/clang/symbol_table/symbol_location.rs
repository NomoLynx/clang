use rust_macro::EnumAccessors;

use crate::languages::clang::clang_asm_code_gen::get_register_names;

use super::*;
use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq, EnumAccessors)]
pub enum SymbolLocation {
    Memory(VarialbeName),
    Register(RegisterName),
    Stack(StackOffset),   // for array and struct/union large data type, this is the memory allocation points. RISCV stack grows downwards, but object in the large data type grow upwards.
    Heap(HeapOffset),

    /// both in register and stack, e.g. when register spilling happens OR function argument passing
    RegisterAndStack(RegisterName, StackOffset),

    /// inside struct/union offset, this is used for struct/union member, and the offset is from the start of the struct/union variable
    StructUnionMemberOffset(MemberOffset),
    NotSet,
}

impl SymbolLocation {
    
    /// get stack offset if the symbol location is stack or register and stack
    pub fn get_stack_offset(&self) -> Option<StackOffset> {
        match self {
            Self::Stack(offset) |
            Self::RegisterAndStack(_, offset) => Some(*offset),
            _ => None,
        }
    }

    /// check if the symbol location is in register or register and stack
    pub fn is_register(&self) -> bool {
        match self {
            Self::Register(_) => true,
            Self::RegisterAndStack(_, _) => true,
            _ => false,
        }
    }

    /// check if the symbol location is in stack or register and stack
    pub fn is_stack(&self) -> bool {
        match self {
            Self::Stack(_) => true,
            _ => false,
        }
    }

    /// get register name, a0, a1, or s11
    pub fn get_register_name(&self) -> Option<&str> {
        match self {
            Self::Register(name) |
            Self::RegisterAndStack(name, _) => Some(name),
            _ => None,
        }
    }

    /// get number in the register name, 0 in a0, 1 in a1, or 11 in s11
    pub fn get_register_offset(&self) -> Option<u32> {
        match self {
            Self::RegisterAndStack(name, _) |
            Self::Register(name) => {
                let mut offset = 0;
                for c in name.chars() {
                    if c.is_digit(10) {
                        offset = offset * 10 + c.to_digit(10).unwrap();
                    }
                }
                Some(offset)
            }
            _ => None,
        }
    }

    /// check if the symbol location is register only
    pub fn is_register_only(&self) -> bool {
        match self {
            Self::Register(_) => true,
            _ => false,
        }
    }

    /// check if symbol location is register and stack
    pub fn is_register_and_stack(&self) -> bool {
        match self {
            Self::RegisterAndStack(_, _) => true,
            _ => false,
        }
    }

    /// get the offset in struct/union if the symbol location is StructUnionMemberOffset
    pub fn get_offset_in_struct_union(&self) -> Option<MemberOffset> {
        match self {
            Self::StructUnionMemberOffset(offset) => Some(*offset),
            _ => None,
        }
    }
}

impl From<SymbolLocation> for String {
    fn from(location:SymbolLocation) -> String {
        (&location).into()
    }
}

impl From<&SymbolLocation> for String {
    fn from(location:&SymbolLocation) -> String {
        match location {
            SymbolLocation::Memory(name) => format!("{}", name),
            SymbolLocation::Register(name) => format!("{}", name),
            SymbolLocation::Stack(offset) => format!("{}", offset),
            SymbolLocation::Heap(offset) => format!("{}", offset),
            SymbolLocation::RegisterAndStack(_, _) => {
                panic!("RegisterAndStack is not supported");
            }
            SymbolLocation::StructUnionMemberOffset(offset) => format!("{}", offset),
            SymbolLocation::NotSet => "<NotSet>".to_string(),
        }
    }
}

impl Display for SymbolLocation {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            SymbolLocation::Memory(name) => write!(f, "Memory({})", name),
            SymbolLocation::Register(name) => {
                if let Some((a,b)) = get_register_names(name) {
                    write!(f, "Register({}/{})", b, a)
                } else {
                    write!(f, "Register({})", name)
                }
            }
            SymbolLocation::Stack(offset) => write!(f, "Stack({})", offset),
            SymbolLocation::Heap(offset) => write!(f, "Heap({})", offset),
            SymbolLocation::RegisterAndStack(name, offset) => {
                if let Some((a,b)) = get_register_names(name) {
                    write!(f, "RegisterAndStack({}/{}, {})", b, a, offset)
                } else {
                    write!(f, "RegisterAndStack({}, {})", name, offset)
                }
            }
            SymbolLocation::StructUnionMemberOffset(offset) => write!(f, "StructUnionMemberOffset({})", offset),
            SymbolLocation::NotSet => write!(f, "<NotSet>"),
        }
    }
}