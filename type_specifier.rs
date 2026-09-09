use std::fmt::Display;

use pest::iterators::Pair;
use core_utils::debug::*;
use rust_macro::GenIsEnumVariant;

use crate::languages::clang::struct_union::*;
use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule};
use crate::languages::clang::enum_type::*;

#[derive(Debug, Clone, PartialEq, Eq, GenIsEnumVariant)]
pub enum TypeSpecifier {
    Void(ASTItem),
    Char(ASTItem),
    Short(ASTItem),
    Int(ASTItem),
    Long(ASTItem),
    Float(ASTItem),
    Double(ASTItem),
    Signed(ASTItem),
    Unsigned(ASTItem),
    Const(ASTItem),              // access specifier
    Volatile(ASTItem),
    Typedef(ASTItem),            //storage class specifier
    Extern(ASTItem),
    Static(ASTItem),
    Auto(ASTItem),
    Register(ASTItem),
    StructOrUnionSpecifier(ASTItem, StructUnionSpecifier),
    TypeAlias(ASTItem, String),    // type alias
    EnumSpecifier(ASTItem, EnumSpecifier),
    Pointer(ASTItem, u8), // pointer level
}

impl TypeSpecifier {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        let ast_item = ASTItem::new(pair.line_col().into());
        match pair.as_rule() {
            Rule::VOID => Ok(Self::Void(ast_item)),

            Rule::CHAR => Ok(Self::Char(ast_item)),

            Rule::SHORT => Ok(Self::Short(ast_item)),
            Rule::INT => Ok(Self::Int(ast_item)),
            Rule::LONG => Ok(Self::Long(ast_item)),

            Rule::FLOAT => Ok(Self::Float(ast_item)),
            Rule::DOUBLE => Ok(Self::Double(ast_item)),

            Rule::SIGNED => Ok(Self::Signed(ast_item)),
            Rule::UNSIGNED => Ok(Self::Unsigned(ast_item)),

            Rule::CONST => Ok(Self::Const(ast_item)),
            Rule::VOLATILE => Ok(Self::Volatile(ast_item)),

            Rule::TYPEDEF => Ok(Self::Typedef(ast_item)),
            Rule::EXTERN => Ok(Self::Extern(ast_item)),
            Rule::STATIC => Ok(Self::Static(ast_item)),
            Rule::AUTO => Ok(Self::Auto(ast_item)),
            Rule::REGISTER => Ok(Self::Register(ast_item)),

            Rule::struct_or_union_specifier => Ok(Self::StructOrUnionSpecifier(
                ast_item,
                StructUnionSpecifier::from_pair(&pair, context)?,
            )),

            Rule::enum_specifier => Ok(Self::EnumSpecifier(ast_item, EnumSpecifier::from_pair(
                &pair, context
            )?)),
            Rule::pointer => {
                let star_str = pair.as_str().trim().to_string();
                let pointer_level = star_str.len() as u8;
                Ok(Self::Pointer(ast_item, pointer_level))
            }
            Rule::type_alias => {
                let r = pair.as_str().to_string();
                Ok(Self::TypeAlias(ast_item, r))
            }
            Rule::identifier => {
                let r = pair.as_str().to_string();
                Ok(Self::TypeAlias(ast_item, r))
            }
            _ => {
                let str = format!("cannot find {pair:?} in type specifier processing logic");
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn is_struct(&self) -> bool {
        match self {
            Self::StructOrUnionSpecifier(_, _) => true,
            _ => false,
        }
    }

    pub fn is_struct_def(&self) -> bool {
        match self {
            Self::StructOrUnionSpecifier(_, n) => n.is_struct_def(),
            _ => false,
        }
    }

    pub fn is_enum(&self) -> bool {
        match self {
            Self::EnumSpecifier(_, _) => true,
            _ => false,
        }
    }

    pub fn get_pointer_level(&self) -> u8 {
        match self {
            Self::Pointer(_, n) => *n,
            _ => 0,
        }
    }

    pub fn is_builtin_type(&self) -> bool {
        !self.is_enum() && !self.is_struct() && !self.is_storage_class_specifier() && !self.is_type_modifier()
    }

    /// get complicated type name, the name of struct or union or enum
    pub fn get_type_name(&self) -> Option<&String> {
        match self {
            Self::StructOrUnionSpecifier(_, n) => n.get_name_id(),
            Self::EnumSpecifier(_, n) => n.get_name_id(),
            _ => None,
        }
    }

    pub fn get_type_name_string(&self) -> Option<String> {
        match self {
            Self::StructOrUnionSpecifier(_, n) => {
                let tag = if n.is_struct() { "struct" } else { "union" };
                let r = format!("{tag} {name}", tag=tag, name=self.get_type_name().unwrap_or(&"<NONAME>".to_string()));                
                Some(r)
            }
            Self::EnumSpecifier(_, _) => {
                let r = format!("enum {name}", name=self.get_type_name().unwrap_or(&"<NONAME>".to_string()));
                Some(r)
            }
            Self::Char(_) => Some("char".to_string()),
            Self::Double(_) => Some("double".to_string()),
            Self::Float(_) => Some("float".to_string()),
            Self::Int(_) => Some("int".to_string()),
            Self::Long(_)=> Some("long".to_string()),
            Self::Short(_) => Some("short".to_string()),
            Self::Void(_) => Some("void".to_string()),
            Self::Pointer(_, n) => Some("*".repeat((*n).into())),
            _ => None,
        }
    }

    pub fn is_real_type(&self) -> bool {
        match self {
            Self::TypeAlias(_, _) |
            Self::StructOrUnionSpecifier(_, _) |
            Self::EnumSpecifier(_, _) => true,
            Self::Char(_) => true,
            Self::Double(_) => true,
            Self::Float(_) => true,
            Self::Int(_) => true,
            Self::Long(_)=> true,
            Self::Short(_) => true,
            Self::Void(_) => true,
            _ => false,
        }
    }

    pub fn get_struct(&self) -> Option<&StructUnionSpecifier> {
        match self {
            Self::StructOrUnionSpecifier(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_enum(&self) -> Option<&EnumSpecifier> {
        match self {
            Self::EnumSpecifier(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_builtin_type_name(&self) -> Option<String> {
        let r = match self {
            Self::Char(_) => "char",
            Self::Double(_) => "double",
            Self::Float(_) => "float",
            Self::Int(_) => "int",
            Self::Long(_)=> "long",
            Self::Short(_) => "short",
            _ => "",
        };
        
        if r.is_empty() {
            None
        }
        else {
            Some(r.to_string())
        }
    }

    /// if type is Const or Voliatile
    pub fn is_type_modifier(&self) -> bool {
        match self {
            Self::Const(_) |
            Self::Volatile(_) => true,
            _ => false,
        }
    }

    pub fn is_storage_class_specifier(&self) -> bool {
        match self {
            Self::Typedef(_) |
            Self::Extern(_) |
            Self::Static(_) |
            Self::Auto(_) |
            Self::Register(_) => true,
            _ => false,
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Char(n) |
            Self::Double(n) |
            Self::EnumSpecifier(n, _) |
            Self::Float(n) | 
            Self::Int(n) | 
            Self::Long(n) | 
            Self::Short(n) | 
            Self::Signed(n) | 
            Self::StructOrUnionSpecifier(n, _) |
            Self::Unsigned(n) |
            Self::Const(n) |
            Self::Volatile(n) |
            Self::Typedef(n) |
            Self::Extern(n) |
            Self::Static(n) |
            Self::Auto(n) |
            Self::Register(n) |
            Self::Void(n) | 
            Self::TypeAlias(n, _) |
            Self::Pointer(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Char(n) |
            Self::Double(n) |
            Self::EnumSpecifier(n, _) |
            Self::Float(n) | 
            Self::Int(n) | 
            Self::Long(n) | 
            Self::Short(n) | 
            Self::Signed(n) | 
            Self::StructOrUnionSpecifier(n, _) |
            Self::Unsigned(n) | 
            Self::Const(n) |
            Self::Volatile(n) |
            Self::Typedef(n) |
            Self::Extern(n) |
            Self::Static(n) |
            Self::Auto(n) |
            Self::Register(n) |
            Self::Void(n) | 
            Self::TypeAlias(n, _) |
            Self::Pointer(n, _) => n,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Char(_) |
            Self::Double(_) => vec![],
            Self::EnumSpecifier(_, n) => vec![n.into()],
            Self::Float(_) |
            Self::Int(_) | 
            Self::Long(_) | 
            Self::Short(_) | 
            Self::Signed(_) => vec![], 
            Self::StructOrUnionSpecifier(_, n) => vec![n.into()],
            Self::Unsigned(_) | 
            Self::Const(_) |
            Self::Volatile(_) |
            Self::Typedef(_) |
            Self::Extern(_) |
            Self::Static(_) |
            Self::Auto(_) |
            Self::Register(_) |
            Self::Void(_) | 
            Self::TypeAlias(_, _) |
            Self::Pointer(_, _) => vec![],
        }
    }

    pub fn get_name(&self) -> String {
        let type_name = self.get_type_name_string().unwrap_or("<Invalid>".to_string());
        let id = self.get_ast_item().get_sequence_id();
        format!("TypeSpec_{}({})", type_name, id)
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        match self {
            Self::Char(_) |
            Self::Double(_) => (),
            Self::EnumSpecifier(_, n) => n.access_children(context),
            Self::Float(_) |
            Self::Int(_) | 
            Self::Long(_) | 
            Self::Short(_) | 
            Self::Signed(_) => (), 
            Self::StructOrUnionSpecifier(_, n) => n.access_children(context),
            Self::Unsigned(_) | 
            Self::Const(_) |
            Self::Volatile(_) |
            Self::Typedef(_) |
            Self::Extern(_) |
            Self::Static(_) |
            Self::Auto(_) |
            Self::Register(_) |
            Self::Pointer(_, _) |
            Self::TypeAlias(_, _) |
            Self::Void(_) => (),
        } 

        context.access_clang_item(CLangItem::TypeSpecifier(CLangItem::new(self)));
    }
}

impl Display for TypeSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Char(_) => write!(f, "char"),
            Self::Double(_) => write!(f, "double"),
            Self::Float(_) => write!(f, "float"),
            Self::Int(_) => write!(f, "int"),
            Self::Long(_) => write!(f, "long"),
            Self::Short(_) => write!(f, "short"),
            Self::Signed(_) => write!(f, "signed"),
            Self::Unsigned(_) => write!(f, "unsigned"),
            Self::Const(_) => write!(f, "const"),
            Self::Volatile(_) => write!(f, "volatile"),
            Self::Typedef(_) => write!(f, "typedef"),
            Self::Extern(_) => write!(f, "extern"),
            Self::Static(_) => write!(f, "static"),
            Self::Auto(_) => write!(f, "auto"),
            Self::Register(_) => write!(f, "register"),
            Self::Void(_) => write!(f, "void"),
            Self::TypeAlias(_, n) => write!(f, "Alias_{}", n),
            Self::StructOrUnionSpecifier(_, n) => write!(f, "{}", n),
            Self::EnumSpecifier(_, n) => write!(f, "{}", n),
            Self::Pointer(_, n) => write!(f, "{}", "*".repeat((*n).into())),
        }
    }
}