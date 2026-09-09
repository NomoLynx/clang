use std::fmt::{self, Display, Formatter};
use rust_macro::*;

#[derive(Debug, Clone, PartialEq, Eq, GenIsEnumVariant)]
pub enum SymbolType {
    Unknown,
    Var,
    Function,
    Declaration,
    CompoundStatement,
    ReturnType,
    Parameters,
    Program,
    Struct,
    Union,
    StructUnionItem,
    Enum,
    EnumConstant,
    TypeAlias,
}

impl Default for SymbolType {
    fn default() -> Self {
        SymbolType::Unknown
    }
}

impl From<&str> for SymbolType {
    fn from(s: &str) -> Self {
        match s {
            "VAR" => SymbolType::Var,
            "FUNCTION" => SymbolType::Function,
            "DECLARATION" => SymbolType::Declaration,
            "COMPOUND" => SymbolType::CompoundStatement,
            "RETURN_TYPE" => SymbolType::ReturnType,
            "PARAMETERS" => SymbolType::Parameters,
            "PROGRAM" => SymbolType::Program,
            "STRUCT" => SymbolType::Struct,
            "UNION" => SymbolType::Union,
            "ENUM" => SymbolType::Enum,
            "ENUM_CONSTANT" => SymbolType::EnumConstant,
            "STRUCT_UNION_ITEM" => SymbolType::StructUnionItem,
            "TYPE_ALIAS" => SymbolType::TypeAlias,
            _ => {
                todo!("Unsupported symbol type: {}", s)
            }
        }
    }
}

impl From<String> for SymbolType {
    fn from(s: String) -> Self {
        SymbolType::from(s.as_str())
    }
}

impl Display for SymbolType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let s = match self {
            SymbolType::Unknown => "UNKNOWN",
            SymbolType::Var => "VAR",
            SymbolType::Function => "FUNCTION",
            SymbolType::Declaration => "DECLARATION",
            SymbolType::CompoundStatement => "COMPOUND",
            SymbolType::ReturnType => "RETURN_TYPE",
            SymbolType::Parameters => "PARAMETERS",
            SymbolType::Program => "PROGRAM",
            SymbolType::Struct => "STRUCT",
            SymbolType::Union => "UNION",
            SymbolType::Enum => "ENUM",
            SymbolType::EnumConstant => "ENUM_CONSTANT",
            SymbolType::StructUnionItem => "STRUCT_UNION_ITEM",
            SymbolType::TypeAlias => "TYPE_ALIAS",
        };
        write!(f, "{}", s)
    }
}