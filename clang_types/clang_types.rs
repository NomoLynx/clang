use std::fmt::{self, Display, Formatter};

use pest::Parser;
use rust_macro_internal::state_type_mapping;

use crate::ast::clr_type::{TypeBase, TypeDef};
use crate::languages::clang::symbol_table::SymbolTable;
use core_utils::debug::*;
use crate::languages::clang::clang_asm_code_gen::CodeGenConfiguration;
use crate::languages::clang::clang_pest::{self, TypeName, UnaryOperator};
use crate::languages::clang::{CLangItem, enum_type::*};
use crate::languages::clang::struct_union::*;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use crate::languages::clang::type_specifier::TypeSpecifier;
use crate::languages::clang::clang_types::*;
use rust_macro::*;

pub type ArrayName = String;

state_type_mapping!("src/languages/clang/type_conversions.mmd", CLangTypes);

#[derive(Debug, Clone, PartialEq, Eq, GenIsEnumVariant)]
pub enum CLangTypes {
    Invalid, 
    None,
    Void,
    Char,
    UChar,
    Int32,
    UInt32,
    Int16,
    UInt16,
    Int64,
    UInt64,
    Float32,
    Float64,
    Byte,
    UByte,
    Enum(EnumSpecifier, Option<CLangItem>),
    Struct(StructUnionSpecifier, Option<CLangItem>),
    Union(StructUnionSpecifier, Option<CLangItem>),
    TypeAlias(String, Option<Box<CLangTypes>>),

    Array(Box<CLangTypes>, ArrayName, ArrayDimension), // base type and array size

    Pointer(u8, Box<CLangTypes>),   // pointer level and base type

    FunctionPointer(String, Box<CLangTypes>, Vec<CLangTypes>), //function name, return type and parameter types
}

impl CLangTypes {

    /// if type is struct or union
    pub fn is_composite(&self) -> bool {
        match self {
            CLangTypes::Struct(_, _) |
            CLangTypes::Union(_, _) => true,
            CLangTypes::TypeAlias(_, n) => 
                if let Some(real_type) = n {
                    real_type.is_composite()
                }
                else {
                    false
                },
            _ => false,
        }
    }

    /// if type is pointer type and the base type is struct or union
    pub fn is_composite_pointer(&self) -> bool {
        match self {
            CLangTypes::Pointer(_, base_type) => base_type.is_composite(),
            _ => false,
        }
    }

    /// if type is enum type pointer
    pub fn is_enum_pointer(&self) -> bool {
        match self {
            CLangTypes::Pointer(_, base_type) => base_type.is_enum(),
            _ => false,
        }
    }

    /// get the symbol table of the struct or union type, 
    /// it returns None if it is not struct or union type or the symbol table is not set
    pub fn get_composite_type_symbol_table(&self) -> Option<&SymbolTable> {
        self.get_struct_union()
            .map(|x| x.get_definition_symbol_table())
    }

    /// get name of the struct or union type, it returns None if it is not struct or union type or the name is not set
    pub fn get_composite_type_name(&self) -> Option<String> {
        self.get_struct_union()
            .map(|su| su.get_type_name())
    }

    pub fn get_struct(&self) -> Option<&StructUnionSpecifier> {
        self.get_struct_union()
            .and_then(|x| if x.is_struct() { Some(x) } else { None })
    }

    /// get the real type of the type alias, if it is not type alias, it returns itself
    pub fn get_real_type(&self) -> &CLangTypes {
        match self {
            Self::TypeAlias(_, n) => {
                if let Some(real_type) = n {
                    real_type.get_real_type()
                }
                else {
                    self
                }
            }
            _ => self,
        }
    }

    pub fn get_type_alias_real_type(&self) -> Option<&CLangTypes> {
        match self {
            Self::TypeAlias(_, n) => n.as_deref(),
            _ => None,
        }
    }

    /// Return a computation/codegen-friendly clone where resolved typedef wrappers
    /// are collapsed recursively, while unresolved aliases are preserved.
    pub fn collapse_type_alias_for_usage(&self) -> CLangTypes {
        match self {
            CLangTypes::TypeAlias(_, Some(real_type)) => real_type.collapse_type_alias_for_usage(),
            CLangTypes::Pointer(level, base_type) => {
                CLangTypes::Pointer(*level, Box::new(base_type.collapse_type_alias_for_usage()))
            }
            CLangTypes::Array(base_type, name, dimension) => {
                CLangTypes::Array(
                    Box::new(base_type.collapse_type_alias_for_usage()),
                    name.clone(),
                    dimension.clone(),
                )
            }
            CLangTypes::FunctionPointer(function_name, ret_type, param_types) => {
                CLangTypes::FunctionPointer(
                    function_name.clone(),
                    Box::new(ret_type.collapse_type_alias_for_usage()),
                    param_types.iter()
                        .map(|param_type| param_type.collapse_type_alias_for_usage())
                        .collect(),
                )
            }
            _ => self.clone(),
        }
    }

    /// checkf if the type is type alias (typedef) and it has real type defined.
    pub fn has_real_type(&self) -> bool {
        match self {
            Self::TypeAlias(_, n) => n.is_some(),
            _ => false,
        }
    }

    pub fn get_struct_mut(&mut self) -> Option<&mut StructUnionSpecifier> {
        match self {
            CLangTypes::Struct(m, n) => {
                if let Some(ty) = n {
                    ty.get_struct_or_union().or(Some(m))
                }
                else {
                    Some(m)
                }
            }
            _ => None
        }
    }

    pub fn get_union_mut(&mut self) -> Option<&mut StructUnionSpecifier> {
        match self {
            CLangTypes::Union(m, n) => {
                if let Some(ty) = n {
                    ty.get_struct_or_union().or(Some(m))
                }
                else {
                    Some(m)
                }
            }
            _ => None
        }
    }

    pub fn get_union(&self) -> Option<&StructUnionSpecifier> {
        self.get_struct_union()
            .and_then(|x| if x.is_union() { Some(x) } else { None })
    }

    pub fn get_struct_union(&self) -> Option<&StructUnionSpecifier> {
        match self {
            CLangTypes::Struct(m, n) |
            CLangTypes::Union(m, n) => {
                if let Some(ty) = n {
                    ty.get_struct_or_union().map(|x| &*x).or(Some(m))
                }
                else {
                    Some(m)
                }
            }
            CLangTypes::TypeAlias(_, ty) => {
                ty.as_deref()
                  .and_then(|x| x.get_struct_union())
            }
            _ => None
        }
    }

    pub fn get_struct_union_mut(&mut self) -> Option<&mut StructUnionSpecifier> {
        match self {
            CLangTypes::Struct(m, n) |
            CLangTypes::Union(m, n) => {
                if let Some(ty) = n {
                    ty.get_struct_or_union().or(Some(m))
                }
                else {
                    Some(m)
                }
            }
            CLangTypes::TypeAlias(_, ty) => {
                if let Some(real_type) = ty {
                    real_type.get_struct_union_mut()
                }
                else {
                    None
                }
            }
            _ => None
        }
    }

    pub fn get_enum(&self) -> Option<&EnumSpecifier> {
        match self {
            CLangTypes::Enum(m, n) => 
                if let Some(ty) = n { ty.get_enum().map(|x| &*x) } 
                else { Some(m) },
            CLangTypes::TypeAlias(_, n) => 
                n.as_deref()
                 .and_then(|x| x.get_enum()),
            _ => None
        }
    }

    pub fn get_enum_mut(&mut self) -> Option<&mut EnumSpecifier> {
        match self {
            CLangTypes::Enum(m, n) => 
                if let Some(ty) = n { ty.get_enum() } 
                else { Some(m) },
            CLangTypes::TypeAlias(_, n) => 
                n.as_deref_mut()
                 .and_then(|x| x.get_enum_mut()),
            _ => None
        }
    }

    /// check if it is a string type (char pointer with level 1)
    pub fn is_string(&self) -> bool {
        match self {
            CLangTypes::Pointer(m, n)
                if *m == 1 && n.as_ref() == &CLangTypes::Char => true,
            CLangTypes::TypeAlias(_, n) => 
                n.as_deref()
                 .map(|x | x.is_string())
                 .unwrap_or(false),
            _ => false,
        }
    }

    /// make pointer type based on the type and pointer level
    pub fn make_pointer_type(t: CLangTypes, pointer_level: usize) -> CLangTypes {
        Self::Pointer(pointer_level as u8, Box::new(t))
    }

    /// if the type is built-in types, which is not array, pointer, struct, union, or enum
    pub fn is_built_in(&self) -> bool {
        match self {
            CLangTypes::Char | CLangTypes::UChar
            | CLangTypes::Int32 | CLangTypes::UInt32 | CLangTypes::Int16 | CLangTypes::UInt16 | CLangTypes::Int64 | CLangTypes::UInt64 
            | CLangTypes::Float32 | CLangTypes::Float64 
            | CLangTypes::Byte | CLangTypes::UByte
            | CLangTypes::Void => true,
            | CLangTypes::TypeAlias(_, n) => 
                n.as_deref()
                 .map(|x| x.is_built_in())
                 .unwrap_or(false),
            _ => false,
        }
    }

    pub fn resolve_typealias(&self) -> Option<&CLangTypes> {
        match self {
            CLangTypes::TypeAlias(_, n) => n.as_deref(),
            _ => None,
        }
    }

    pub fn is_invalid_type(&self) -> bool {
        self.is_invalid()
    }

    pub fn is_none_type(&self) -> bool {
        self.is_none()
    }

    pub fn can_default_to_int_for_rv5(&self) -> bool {
        match self {
            CLangTypes::Byte |
            CLangTypes::Int16 |
            CLangTypes::Int32 |
            CLangTypes::UByte |
            CLangTypes::UInt16 |
            CLangTypes::UInt32 => true,
            CLangTypes::TypeAlias(_, n) => 
                n.as_deref()
                 .map(|x| x.can_default_to_int_for_rv5())
                 .unwrap_or(false),
            _ => false,
        }
    }

    pub fn set_struct_union_clang_item(&mut self, clang_item: CLangItem) {
        match self {
            CLangTypes::Struct(_, n) |
            CLangTypes::Union(_, n) => *n = Some(clang_item),
            _ => {}
        }
    }

    pub fn set_enum_clang_item(&mut self, clang_item: CLangItem) {
        match self {
            CLangTypes::Enum(_, n) => *n = Some(clang_item),
            _ => {}
        }
    }

    pub fn apply_unary_operator(&self, op:&UnaryOperator) -> Self {
        let new_type = match op {
            UnaryOperator::AddressOf => {
                self.to_pointer_type()
            }
            UnaryOperator::Dereference(n) => {
                if let Some(new_type) = self.dereference_pointer_type(*n) {
                    new_type
                }
                else {
                    panic!("cannot dereference type {self:?} with level {n}");
                }
            }
            _ => self.clone(),
        };

        new_type
    }

    /// convert the type to pointer type by increasing the pointer level by 1 or making it pointer type
    pub fn to_pointer_type(&self) -> CLangTypes {
        match self {
            CLangTypes::Pointer(level, base_type) => {
                CLangTypes::Pointer(level + 1, base_type.clone())
            }
            _ => Self::make_pointer_type(self.clone(), 1)
        }
    }

    /// dereference the pointer type by decreasing the pointer level by 1 or getting the base type
    pub fn dereference_pointer_type(&self, n:u8) -> Option<CLangTypes> {
        match self {
            CLangTypes::Pointer(level, base_type) => {
                if *level > 1 {
                    if *level < n {
                        return None;
                    }
                    
                    if *level == n {
                        Some(*(*base_type).clone())
                    }
                    else {
                        Some(CLangTypes::Pointer(level - n, base_type.clone()))
                    }
                }
                else {
                    Some(*(*base_type).clone())
                }
            }
            _ => None,
        }
    }

    /// check if the type can be defaulted to int64 for rv5 architecture
    /// default case switch to check if it can be defaulted to int32
    pub fn can_default_to_int64_for_rv5(&self) -> bool {
        match self {
            CLangTypes::Int64 |
            CLangTypes::UInt64 => true,
            CLangTypes::TypeAlias(_, n) => 
                n.as_deref()
                 .map(|x| x.can_default_to_int64_for_rv5())
                 .unwrap_or(false),
            _ => self.can_default_to_int_for_rv5(),
        }
    }

    /// check if the type is integer type which
    /// includes all the types can be defaulted to int32 or int64 for rv5 architecture
    pub fn is_integer_type(&self) -> bool {
        self.can_default_to_int64_for_rv5() || 
        self.can_default_to_int_for_rv5()
    }

    /// check if the type is returned in the RISC-V integer return register a0.
    /// This covers integer-like values, enums, pointers, and function pointers,
    /// while leaving floating-point and composite returns to future explicit handling.
    pub fn returns_in_a0(&self) -> bool {
        match self {
            CLangTypes::Void => false,
            CLangTypes::TypeAlias(_, n) => n.as_deref().map(|x| x.returns_in_a0()).unwrap_or(false),
            CLangTypes::Enum(_, _) |
            CLangTypes::Pointer(_, _) |
            CLangTypes::FunctionPointer(_, _, _) => true,
            _ => self.is_integer_type(),
        }
    }

    /// it includes integer types and floating point types, char and byte,
    /// enum is also considered as numeric type because it can be converted to its base type 
    /// which is a integer type. 
    /// it is all the type can be converted to numberic types
    pub fn is_numeric_type(&self) -> bool {
        self.is_char() || 
        self.is_uchar() ||
        self.is_enum() ||
        self.is_floating_point_type() ||
        self.can_default_to_int64_for_rv5() ||
        self.can_default_to_int_for_rv5()
    }

    pub fn is_floating_point_type(&self) -> bool {
        match self {
            CLangTypes::Float32 |
            CLangTypes::Float64 => true,
            CLangTypes::TypeAlias(_, n) => 
                n.as_deref()
                 .map(|x| x.is_floating_point_type())
                 .unwrap_or(false),
            _ => false,
        }
    }

    pub fn get_type_size_in_bytes(&self, context:&CodeGenContext) -> Option<usize> {
        let config = context.get_code_gen_config();
        self.get_type_size_in_bytes2(config)
    }

    pub fn get_type_size_in_bytes2(&self, config:&CodeGenConfiguration) -> Option<usize> {
        match self {
            CLangTypes::Char => Some(1),
            CLangTypes::UChar => Some(1),
            CLangTypes::Byte => Some(1),
            CLangTypes::UByte => Some(1),
            CLangTypes::Int16 => Some(2),
            CLangTypes::UInt16 => Some(2),
            CLangTypes::Int32 => Some(4),
            CLangTypes::UInt32 => Some(4),
            CLangTypes::Int64 => Some(8),
            CLangTypes::UInt64 => Some(8),
            CLangTypes::Float32 => Some(4),
            CLangTypes::Float64 => Some(8),
            CLangTypes::FunctionPointer(_, _, _) |
            CLangTypes::Pointer(_, _) => Some(config.get_addressing_in_bytes() as usize),
            CLangTypes::Struct(_, _) |
            CLangTypes::Union(_, _) => self.get_struct_union()
                                        .and_then(|x| Some(x.get_type_size(config))),
            CLangTypes::Enum(_, _) => self.get_enum().and_then(|x| Some(x.get_type_size(config))),
            CLangTypes::Array(ty, _, dimenstion) => {
                if let Some(n) = ty.get_type_size_in_bytes2(config) {
                    Some(n * dimenstion.get_total_elements())
                }
                else {
                    None
                }
            }
            CLangTypes::TypeAlias(_, n) => {
                n.as_ref().
                  and_then(|x| x.get_type_size_in_bytes2(config))
            }
            CLangTypes::Void => Some(config.get_addressing_in_bytes() as usize),
            CLangTypes::Invalid |
            CLangTypes::None => None,
        }
    }

    /// get type size when used as function parameter, it considers the addressing size and round up
    pub fn get_type_size_as_parameter_in_bytes(&self, context:&CodeGenContext) -> Option<usize> {
        let config = context.get_code_gen_config();
        let r = match self {
            CLangTypes::Enum(_, _) |
            CLangTypes::Struct(_, _) |
            CLangTypes::Union(_, _) => Some(config.get_addressing_in_bytes() as usize),
            _ => self.get_type_size_in_bytes(context),
        };

        // set min value to 32 bits (4 or 8 bytes)
        let addressing_unit = context.get_code_gen_config().get_addressing_in_bytes() as usize;
        if let Some(n) = r {
            Some(std::cmp::max(addressing_unit, n))
        }
        else {
            None
        }
    }

    /// create array type based on the base type, array name and dimension
    pub fn create_array_type(base_type:CLangTypes, name:ArrayName, dimension:ArrayDimension) -> CLangTypes {
        CLangTypes::Array(Box::new(base_type), name, dimension)
    }

    /// convert string to CLangTypes, this conversion is case insensitive
    pub fn from_string(s: &str) -> CLangTypes {
        match s.to_lowercase().as_str() {
            "void" => CLangTypes::Void,
            "char" => CLangTypes::Char,
            "int" => CLangTypes::Int32,
            "unsigned int" => CLangTypes::UInt32,
            "short" => CLangTypes::Int16,
            "unsigned short" => CLangTypes::UInt16,
            "long" => CLangTypes::Int64,
            "unsigned long" => CLangTypes::UInt64,
            "float" => CLangTypes::Float32,
            "double" => CLangTypes::Float64,
            "byte" => CLangTypes::Byte,
            "unsigned byte" => CLangTypes::UByte,
            _ => {
                let mut r = clang_pest::CLangParser::parse(clang_pest::Rule::declaration_specifiers, s).unwrap();
                let mut context = CodeGenContext::default();
                let type_specifiers = r.nth(0).unwrap()
                                        .into_inner()
                                        .map(|pair| TypeSpecifier::from_pair(&pair, &mut context).unwrap())
                                        .collect::<Vec<_>>();
                let clang_types = type_specifiers.into();
                clang_types
            }
        }
    }

    fn from_string2(s: String) -> CLangTypes {
        CLangTypes::from_string(s.as_str())
    }

    fn from_option_string(s: Option<String>) -> CLangTypes {
        if let Some(n) = s {
            CLangTypes::from_string(n.as_str())
        }
        else {
            CLangTypes::None
        }
    }

    fn from_type_specifier_ref(ty:&TypeSpecifier) -> CLangTypes {
        match ty {
            TypeSpecifier::Void(_) => CLangTypes::Void,
            TypeSpecifier::Char(_) => CLangTypes::Char,
            TypeSpecifier::Int(_) => CLangTypes::Int32,
            TypeSpecifier::Short(_) => CLangTypes::Int16,
            TypeSpecifier::Long(_) => CLangTypes::Int64,
            TypeSpecifier::Float(_) => CLangTypes::Float32,
            TypeSpecifier::Double(_) => CLangTypes::Float64,
            TypeSpecifier::TypeAlias(_, n) => CLangTypes::TypeAlias(n.clone(), None),
            TypeSpecifier::EnumSpecifier(_, n) => CLangTypes::Enum(n.clone(), None),
            TypeSpecifier::StructOrUnionSpecifier(_, n) => {
                if n.is_struct() {
                    CLangTypes::Struct(n.clone(), None)
                }
                else {
                    CLangTypes::Union(n.clone(), None)
                }
            }
            _ => CLangTypes::Invalid,
        }
    }

    /// check if composite types without a real definition clang item link
    pub fn contains_invalid_clang_item_link(&self) -> bool {
        match self {
            CLangTypes::Struct(m, n) |
            CLangTypes::Union(m, n) => {
                if let Some(n) = n { 
                    if let Some(ty) = n.get_struct_or_union() {
                        ty.get_name_id() != m.get_name_id()
                    }
                    else {
                        true
                    }
                } 
                else { true }
            }
            CLangTypes::Enum(m, n) => {
                if let Some(n) = n {
                    if let Some(ty) = n.get_enum() {
                        ty.get_name_id() != m.get_name_id()
                    }
                    else {
                        true
                    }
                }
                else {
                    true
                }
            }
            CLangTypes::Array(base_type, _, _) => base_type.contains_invalid_clang_item_link(),
            CLangTypes::Pointer(_, base_type) => base_type.contains_invalid_clang_item_link(),
            _ => false,
        }
    }

    /// get the type or embedded type if it is enum
    pub fn get_type_or_embedded_type(&self) -> CLangTypes {
        match self {
            CLangTypes::Enum(_, _) => self.get_enum()
                                                    .and_then(|x| Some(x.get_base_type()))
                                                    .unwrap(),
            _ => self.clone(),
        }
    }

    /// get the array type information if it is array type
    pub fn get_array_type(&self) -> Option<(&CLangTypes, String, &ArrayDimension)> {
        match self {
            CLangTypes::Array(base_type, name, dimension) => 
                Some((base_type.as_ref(), name.to_string(), dimension)),
            _ => None,
        }
    }

    pub fn get_array_element_type(&self) -> Option<&CLangTypes> {
        match self {
            CLangTypes::Array(base_type, _, _) => Some(base_type.as_ref()),
            _ => None,
        }
    }

    /// get the offset in bytes for the given indexes in array type
    pub fn get_array_offset(&self, indexes:&Vec<usize>) -> Option<usize> {
        let (array_type, _name, dimension) = self.get_array_type()?;
        let base_type_size = array_type.get_type_size_in_bytes2(&CodeGenConfiguration::default())?;

        if indexes.len() != dimension.get_dimension_count() {
            return None;
        }

        let mut offset = 0;
        let mut stride = 1; // stride for the last dimension is 1
        for i in (0..indexes.len()).rev() {
            let index = indexes[i];
            let dim_size = dimension[i];
            if index >= dim_size {
                return None;
            }

            offset += index * stride;
            stride *= dim_size;
        }

        Some(offset * base_type_size)
    }

    /// get the return type of the function pointer type, 
    /// it returns None if it is not function pointer type
    pub fn get_function_return_type(&self) -> Option<&CLangTypes> {
        match self {
            CLangTypes::FunctionPointer(_, ret_type, _) => Some(ret_type.as_ref()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::CLangTypes;

    #[test]
    fn int_like_types_return_in_a0() {
        assert!(CLangTypes::Int32.returns_in_a0());
        assert!(CLangTypes::UInt64.returns_in_a0());
        assert!(CLangTypes::Pointer(1, Box::new(CLangTypes::Int32)).returns_in_a0());
        assert!(CLangTypes::FunctionPointer("f".to_string(), Box::new(CLangTypes::Int32), vec![]).returns_in_a0());
    }

    #[test]
    fn non_a0_return_types_are_not_marked_as_a0() {
        assert!(!CLangTypes::Void.returns_in_a0());
        assert!(!CLangTypes::Float32.returns_in_a0());
        assert!(!CLangTypes::Float64.returns_in_a0());
        assert!(!CLangTypes::Struct(crate::languages::clang::struct_union::StructUnionSpecifier::default(), None).returns_in_a0());
    }

    #[test]
    fn collapse_type_alias_for_usage_keeps_unresolved_aliases_but_flattens_resolved_ones() {
        let resolved = CLangTypes::FunctionPointer(
            "fn_ptr".to_string(),
            Box::new(CLangTypes::TypeAlias(
                "RetAlias".to_string(),
                Some(Box::new(CLangTypes::Int32)),
            )),
            vec![
                CLangTypes::Pointer(
                    1,
                    Box::new(CLangTypes::TypeAlias(
                        "ParamAlias".to_string(),
                        Some(Box::new(CLangTypes::UInt16)),
                    )),
                ),
                CLangTypes::TypeAlias("Unresolved".to_string(), None),
            ],
        );

        assert_eq!(
            resolved.collapse_type_alias_for_usage(),
            CLangTypes::FunctionPointer(
                "fn_ptr".to_string(),
                Box::new(CLangTypes::Int32),
                vec![
                    CLangTypes::Pointer(1, Box::new(CLangTypes::UInt16)),
                    CLangTypes::TypeAlias("Unresolved".to_string(), None),
                ],
            )
        );
    }
}

impl Display for CLangTypes {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            CLangTypes::Invalid => write!(f, "Invalid"),
            CLangTypes::None => write!(f, "None"),
            CLangTypes::Void => write!(f, "void"),
            CLangTypes::UChar => write!(f, "unsigned char"),
            CLangTypes::Char => write!(f, "char"),
            CLangTypes::Byte => write!(f, "byte"),
            CLangTypes::UByte => write!(f, "unsigned byte"),
            CLangTypes::Int32 => write!(f, "int"),
            CLangTypes::UInt32 => write!(f, "unsigned int"),
            CLangTypes::Int16 => write!(f, "short"),
            CLangTypes::UInt16 => write!(f, "unsigned short"),
            CLangTypes::Int64 => write!(f, "long"),
            CLangTypes::UInt64 => write!(f, "unsigned long"),
            CLangTypes::Float32 => write!(f, "float"),
            CLangTypes::Float64 => write!(f, "double"),
            CLangTypes::Enum(_, n) => {
                let id_str = if let Some(id)= n.as_ref().and_then(|x| Some(x.get_ast_item().get_sequence_id())) {
                    format!("(CI_{id})")
                }
                else {
                    "".to_string()
                };
                let n = self.get_enum().unwrap();
                let enum_name = n.get_type_name();
                write!(f, "enum {enum_name}{id_str}")
            }
            CLangTypes::Struct(_, n) |
            CLangTypes::Union(_, n) => {
                let id_str = if let Some(id)= n.as_ref().and_then(|x| Some(x.get_ast_item().get_sequence_id())) {
                    format!("(CI_{id})")
                }
                else {
                    "(?)".to_string()
                };
                let n = self.get_struct_union().unwrap();
                let data_struct_keyword = if n.is_struct() { "struct" } else { "union" };
                let struct_name = n.get_type_name();
                write!(f, "{data_struct_keyword} {struct_name}{id_str}")
            }
            CLangTypes::TypeAlias(m, n) => { 
                if let Some(real_type) = n {
                    write!(f, "typedef {} as {}", real_type, m)
                }
                else {
                    write!(f, "typedef <Anonymous> {}", m)
                }
            }
            CLangTypes::Array(ty, name, size) => {
                let postfixs = size.get_dimensions_values().iter()
                                        .map(|x| x.to_string())
                                        .collect::<Vec<_>>()
                                        .join("][");
                write!(f, "{ty} {name}[{postfixs}]")
            }
            CLangTypes::Pointer(m, n) => write!(f, "{n}{}", "*".repeat(*m as usize)),
            CLangTypes::FunctionPointer(function_name, ret_type, param_types) => {
                let param_types_str = param_types.iter()
                    .map(|ty| ty.to_string())
                    .collect::<Vec<_>>()
                    .join(", ");
                write!(f, "{} (*{function_name})({})", ret_type, param_types_str)
            }
        }
    }
}

impl From<&mut TypeSpecifier> for CLangTypes {
    fn from(value: &mut TypeSpecifier) -> Self {
        (&*value).into()
    }
}

impl From<&TypeName> for CLangTypes {
    fn from(ty: &TypeName) -> Self {
        ty.get_specifiers_qualifiers().into()
    }
}

impl From<&TypeDef> for CLangTypes {
    fn from(t: &TypeDef) -> Self {
        match t {
            TypeDef::TypeDef(n) => {
                match n {
                    TypeBase::Void => CLangTypes::Void,
                    TypeBase::Char => CLangTypes::Char,
                    TypeBase::Int32 => CLangTypes::Int32,
                    TypeBase::UInt32 => CLangTypes::UInt32,
                    TypeBase::Int16 => CLangTypes::Int16,
                    TypeBase::UInt16 => CLangTypes::UInt16,
                    TypeBase::Int64 => CLangTypes::Int64,
                    TypeBase::UInt64 => CLangTypes::UInt64,
                    TypeBase::Float32 => CLangTypes::Float32,
                    TypeBase::Float64 => CLangTypes::Float64,
                    TypeBase::String => CLangTypes::Pointer(1, Box::new(CLangTypes::Char)),
                    _ => {
                        warn_string(format!("Possible missing a case for type '{n:?}'"));
                        CLangTypes::Invalid
                    }
                }
            }
            _ => CLangTypes::Invalid,
        }
    }
}

impl From<TypeDef> for CLangTypes {
    fn from(t: TypeDef) -> Self {
        (&t).into()
    }
}

impl From<&Vec<TypeSpecifier>> for CLangTypes {
    fn from(value: &Vec<TypeSpecifier>) -> Self {
        let real_types = value.iter().filter(|x| x.is_real_type())
                                                .collect::<Vec<_>>();
        let has_unsigned = value.iter().any(|x| x.is_unsigned());
        let pointer_level = value.iter().filter(|x| x.is_pointer())
                                                        .map(|x| x.get_pointer_level())
                                                        .nth(0)
                                                        .unwrap_or(0);
        let base_type = match real_types.as_slice() {
            [TypeSpecifier::TypeAlias(_, n)] => CLangTypes::TypeAlias(n.clone(), None),
            [TypeSpecifier::Char(_)] => if has_unsigned { CLangTypes::UChar } else { CLangTypes::Char },
            [TypeSpecifier::Int(_)] => if has_unsigned { CLangTypes::UInt32} else { CLangTypes::Int32 }
            [TypeSpecifier::Short(_)] => if has_unsigned { CLangTypes::UInt16 } else { CLangTypes::Int16 },
            [TypeSpecifier::Long(_)] => if has_unsigned { CLangTypes::Int64 } else {  CLangTypes::Int64 },
            [TypeSpecifier::Float(_)] => CLangTypes::Float32,
            [TypeSpecifier::Double(_)] => CLangTypes::Float64,
            [TypeSpecifier::Void(_)] => CLangTypes::Void,
            [TypeSpecifier::EnumSpecifier(_, n)] => CLangTypes::Enum(n.clone(), None),
            [TypeSpecifier::StructOrUnionSpecifier(_, n)] => 
                if n.is_struct() { CLangTypes::Struct(n.clone(), None) } 
                else { CLangTypes::Union(n.clone(), None) },
            [] => CLangTypes::Void,
            _ => {
                for ty in real_types.iter() {
                    warn_string(format!("Unsupported type specifier '{ty:?}'"));
                }
                todo!("not supported type {real_types:?} with unsigned = {has_unsigned} and pointer level = {pointer_level}");
            }
        };

        if pointer_level == 0 {
            base_type
        } else {
            CLangTypes::make_pointer_type(base_type, pointer_level.into())
        }
    }
} 

impl From<Vec<TypeSpecifier>> for CLangTypes {
    fn from(ty: Vec<TypeSpecifier>) -> Self {
        (&ty).into()
    }
}
