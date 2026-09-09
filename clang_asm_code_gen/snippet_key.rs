use core::panic;
use std::fmt::Debug;

use core_utils::debug::error_string;

use crate::languages::clang::clang_types::CLangTypes;

pub const SNIPPET_CATEGORY_INT: &str = "int";
pub const SNIPPET_CATEGORY_UINT: &str = "unsigned int";
pub const SNIPPET_CATEGORY_SHORT: &str = "short";
pub const SNIPPET_CATEGORY_USHORT: &str = "unsigned short";
pub const SNIPPET_CATEGORY_BYTE : &str = "byte";
pub const SNIPPET_CATEGORY_UBYTE : &str = "unsigned byte";
pub const SNIPPET_CATEGORY_FLOAT: &str = "float";
pub const SNIPPET_CATEGORY_DOUBLE: &str = "double";
pub const SNIPPET_CATEGORY_LONG: &str = "long";
pub const SNIPPET_CATEGORY_STRING: &str = "string";

#[derive(Clone, PartialEq, Eq, Hash)]
pub enum SnippetKey {
    Int,
    UInt,
    Short,
    UShort,
    Byte,
    UByte,
    Float,
    Double,
    Long,
    Address,
    String,
}

impl SnippetKey {
    pub fn to_string(&self, addressing_in_bits:u8) -> String {
        match self {
            SnippetKey::Int => SNIPPET_CATEGORY_INT.to_string(),
            SnippetKey::UInt => SNIPPET_CATEGORY_UINT.to_string(),
            SnippetKey::Short => SNIPPET_CATEGORY_SHORT.to_string(),
            SnippetKey::UShort => SNIPPET_CATEGORY_USHORT.to_string(),
            SnippetKey::Byte => SNIPPET_CATEGORY_BYTE.to_string(),
            SnippetKey::UByte => SNIPPET_CATEGORY_UBYTE.to_string(),
            SnippetKey::Float => SNIPPET_CATEGORY_FLOAT.to_string(),
            SnippetKey::Double => SNIPPET_CATEGORY_DOUBLE.to_string(),
            SnippetKey::Long => SNIPPET_CATEGORY_LONG.to_string(),
            SnippetKey::String => SNIPPET_CATEGORY_STRING.to_string(),
            SnippetKey::Address => {
                if addressing_in_bits == 32 {
                    "int".to_string()
                } else {
                    "long".to_string()
                }
            }
        }
    }

    pub fn is_address(&self) -> bool {
        matches!(self, SnippetKey::Address)
    }
}

impl Debug for SnippetKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string(64))
    }
}

impl Default for SnippetKey {
    fn default() -> Self {
        SnippetKey::Int
    }
}

impl From<CLangTypes> for SnippetKey {
    fn from(t: CLangTypes) -> Self {
        (&t).into()
    }
}

impl From<&CLangTypes> for SnippetKey {
    fn from(t: &CLangTypes) -> Self {
        match t {
            CLangTypes::Int32 => SnippetKey::Int,
            CLangTypes::UInt32 => SnippetKey::UInt,
            CLangTypes::Int16 => SnippetKey::Short,
            CLangTypes::UInt16 => SnippetKey::UShort,
            CLangTypes::Byte => SnippetKey::Byte,
            CLangTypes::UByte => SnippetKey::UByte,
            CLangTypes::Float32 => SnippetKey::Float,
            CLangTypes::Float64 => SnippetKey::Double,
            CLangTypes::Int64 => SnippetKey::Long,
            CLangTypes::UInt64 => SnippetKey::Long,
            CLangTypes::Char => SnippetKey::Byte,
            CLangTypes::UChar => SnippetKey::UByte,
            _ if t.is_string() => SnippetKey::String,
            _ if t.is_functionpointer() => SnippetKey::Address,
            _ if t.is_pointer() => SnippetKey::Address,
            _ if t.is_composite() => SnippetKey::Address,
            _ if t.is_array() => SnippetKey::Address,
            _ if t.is_enum() => SnippetKey::Int,
            _ if t.is_typealias() => {
                if let Some(real_type) = t.get_type_alias_real_type() {
                    real_type.into()
                }
                else {
                    error_string(format!("type alias '{t}' does not have real type defined"));
                    panic!("type alias '{t}' does not have real type defined");
                }
            }
            _ => {
                error_string(format!("cannot convert CLangTypes '{}' to SnippetKey", t));
                panic!("cannot convert CLangTypes '{}' to SnippetKey", t)
            }
        }
    }
}

impl From<&str> for SnippetKey {
    fn from(value: &str) -> Self {
        let clang_type = CLangTypes::from_string(value);
        clang_type.into()
    }
}

impl From<String> for SnippetKey {
    fn from(value: String) -> Self {
        value.as_str().into()
    }
}