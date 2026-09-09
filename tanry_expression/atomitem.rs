use std::fmt::Display;

use rust_macro::GenIsEnumVariant;

use crate::ast::clr_basic_type_object::CLRBasicTypeObject;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::symbol_table::MemberOffset;
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::{ASTItem, CLangItem};
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::declaration::resolve_clang_type_for_usage;
use crate::languages::clang::tanry_expression::*;

#[derive(Debug, Clone, PartialEq, Eq, GenIsEnumVariant)]
pub enum AtomItem {
    BasicTypeObject(ASTItem, CLRBasicTypeObject),
    Id(ASTItem, String),
    Expression(ASTItem, Expression),
    TanryExpression(ASTItem, TanryExpression),
}

impl AtomItem {
    /// get id value for Id case; all other cases are None
    pub fn get_id_value(&self) -> Option<&String> {
        match self {
            Self::Id(_, n) => Some(n),
            _ => None,
        }
    }

    /// get type declaration in declaration or in parameter as CLangItem for Id case; all other cases are None
    pub fn get_declaration_type(&self) -> Option<&CLangItem> {
        match self {
            Self::Id(ast_item, id) => {
                if let Some(entry) = ast_item.get_symbol_recursive_type_definition(&id) {
                    entry.get_clang_item_link()
                }
                else {
                    None
                }
            }
            _ => None,
        }
    }

    /// get string value for BasicTypeObject case; all other cases are None
    /// its string value is n.to_string() where n is the CLRBasicTypeObject
    pub fn get_string_value(&self) -> Option<String> {
        match self {
            Self::BasicTypeObject(_, n) => {
                let r = format!("{n}");  //use Display trait's value as object value's string representation
                Some(r)
            }
            _ => None,
        }
    }

    pub fn set_id_value_changed(&mut self, changed:bool) {
        match self {
            Self::Id(n, _) => {
                n.set_value_changed(changed);
            }
            _ => panic!("set_id_value_changed only support Id AtomItem"),
        }
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }

        match self {
            Self::BasicTypeObject(_, n) => {
                let type_def = n.get_type_def().unwrap();
                let clang_type : CLangTypes = type_def.into();
                clang_type
            },
            Self::Expression(_, n) => n.get_clang_type(),
            Self::Id(ast_item, id) => {
                //query type from symbol table
                if let Some(entry) = ast_item.get_symbol_recursive_all_types(id) {
                    if let Some(ty) = entry.get_enum_type().ok() {
                        ty
                    }
                    else if let Some(ty) = entry.get_struct_union_type().ok() {
                        ty
                    }
                    else if let Some(ty) = entry.get_data_type() {   // variable type can be stored in symbol table
                        resolve_clang_type_for_usage(ty, ast_item)
                    }
                    else if let Some(ty) = entry.get_function_return_type().ok()
                        .or_else(|| entry.get_data_type()) {
                        resolve_clang_type_for_usage(ty, ast_item)
                    }
                    else {
                        CLangTypes::None
                    }
                }
                else { 
                    // look at semantic linkage for type
                    let semantic_links = self.get_ast_item().get_semantic_linkages();
                    for link in semantic_links {
                        if let Some(ty) = link.as_definition() {
                            if let Some(ty_clang_type) = ty.get_declaration() {
                                if let Some(result) = ty_clang_type.get_struct_union_member_type(id) {
                                    return result;
                                }
                            }
                            else if let Some(struct_union) = ty.get_struct_or_union() {
                                if let Some(result) = struct_union.get_member_type(id) {
                                    return result;
                                }
                            }
                        }
                    }

                    CLangTypes::None
                }
            }
            Self::TanryExpression(_, n) => n.get_clang_type(),
        }
    }

    pub fn get_struct_union_member_offset(&self) -> Option<MemberOffset> {
        if self.is_id() {
            let id = self.get_id_value().unwrap().to_string();
            let semantic_links = self.get_ast_item().get_semantic_linkages();
            for link in semantic_links {
                if let Some(ty) = link.as_definition() {
                    if let Some(ty_clang_type) = ty.get_declaration() {
                        if let Some(struct_ty) = ty_clang_type.get_struct() {
                            let symbol_table = struct_ty.get_symbol_table();
                            if let Some(entry) = symbol_table.lookup(&id) {
                                let offset = entry.get_single_storage_location_member_offset();
                                return offset;
                            }
                        }
                        else if let Some(ty_clang_type) = ty_clang_type.get_union() {
                            let symbol_table = ty_clang_type.get_symbol_table();
                            if let Some(entry) = symbol_table.lookup(&id) {
                                let offset = entry.get_single_storage_location_member_offset();
                                return offset;
                            }
                        }
                    }                            
                }
            }

            None
        }
        else {
            None
        }
        
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::BasicTypeObject(_, _) => vec![],
            Self::Expression(_, n) => vec![n.into()],
            Self::Id(_, _) => vec![],
            Self::TanryExpression(_, n) => vec![n.into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::BasicTypeObject(_, _) => (),
            Self::Expression(_, n) => n.access_children(context),
            Self::Id(_, _) => (),
            Self::TanryExpression(_, n) => n.access_children(context),
        }

        context.access_clang_item(CLangItem::AtomItem(CLangItem::new(self)));
    }

    /// get the composite or enum definition for Id case; all other cases are None
    /// the item will have symbol table
    pub fn get_composite_definition_from_id(&self) -> Option<CLangItem> {
        match self {
            Self::Id(ast_item, id) => {
                let type_name = ast_item.get_symbol_recursive_all_types(&id)
                                .and_then(|x| x.get_composite_type_name())
                                .unwrap();
                let type_clang_item = ast_item.get_symbol_recursive_type_definition(&type_name)
                            .and_then(|x| x.get_clang_item_link());

                type_clang_item.map(|x| x.clone())
            }
            _ => None,
        }
    }
}

impl Display for AtomItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::BasicTypeObject(_, n) => format!("{n}"),
            Self::Id(_, n) => n.to_string(),
            Self::Expression(_, n) => format!("{n}"),
            Self::TanryExpression(_, n) => format!("{n}"),
        };

        write!(f, "{}", s)
    }
}

impl ASTItemTrait for AtomItem {
    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::BasicTypeObject(n, _) => n,
            Self::Expression(n, _) => n,
            Self::Id(n, _) => n,
            Self::TanryExpression(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::BasicTypeObject(n, _) => n,
            Self::Expression(n, _) => n,
            Self::Id(n, _) => n,
            Self::TanryExpression(n, _) => n,
        }
    }

    fn get_name(&self) -> String {
        let id = self.get_id();
        match self {
            Self::BasicTypeObject(_, n) => format!("Obj({n}) Id({id})"),
            Self::Expression(_, n) => format!("AtomItem_Expr({id}, {})", n.get_name()),
            Self::Id(_, n) => {
                let value_change_mark = if self.get_ast_item().get_value_changed() { "*" } else { "" };
                format!("Id[{id},{n}]{value_change_mark}")
            }
            Self::TanryExpression(_, n) => format!("Tanry({}, {id})", n.get_name()),
        }
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}