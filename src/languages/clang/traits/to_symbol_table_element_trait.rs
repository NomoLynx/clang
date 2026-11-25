use std::collections::HashMap;

use super::*;
use crate::languages::clang::clang_program::CLangProgram;
use crate::languages::clang::compound_statement::CompoundStatement;
use crate::languages::clang::declaration::Declaration;
use crate::languages::clang::enum_type::{EnumSpecifier, EnumSpecifier_Constant};
use crate::languages::clang::function_definition::FunctionDefinition;
use crate::languages::clang::parameter_declaration::ParameterDeclaration;
use crate::languages::clang::declaration::fix_struct_union_link_in_type;
use crate::languages::clang::struct_union::{StructUnionSpecifier, StructUnionSpecifier_Declarator};
use crate::languages::clang::symbol_table::{ATTRIBUTE_KEY_VARTYPE, SymbolType};

pub trait ToSymbolTableElement 
    where Self : ASTItemTrait
{
    fn get_symbol_entry_name(&self) -> String;
    fn get_sybmol_entry_type(&self) -> SymbolType;
    fn get_sybmol_entry_scope(&self) -> String;
    fn get_symbol_entry_attributes(&self) -> HashMap<String, String>;

    fn get_child_symbol_table_elements(&self) -> String {
        let ast = self.get_ast_item();
        let citem = ast.get_current_node().unwrap().clone();
        let child_scopes = citem.get_ast_item_mut().get_scope_children_mut();
        if child_scopes.len() == 0 { return "None".to_string(); }
        let strs = child_scopes.iter().map(|x| format!("1. {}", x.get_name()) ).collect::<Vec<_>>();
        strs.join("\r\n")
    }

    fn get_symbol_entry_id(&self) -> usize {
        let id = self.get_id();
        id
    }
}

impl ToSymbolTableElement for StructUnionSpecifier {
    fn get_symbol_entry_name(&self) -> String {
        self.get_type_name()
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        if self.is_struct_def() {
            SymbolType::Struct
        }
        else {
            SymbolType::Union
        }
    }

    fn get_sybmol_entry_scope(&self) -> String {
        if let Some(scope) = self.get_ast_item().get_scope_parent() {
            scope.get_name()
        }
        else {
            if self.is_struct_def() {
                "StructSpecifier_Global".to_string()
            }
            else {
                "UnionSpecifier_Global".to_string()
            }
        }
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        HashMap::default()
    }
}

/// implement ToSymbolTableElement for StructUnion Item (StructUnionSpecifier_Declarator)
impl ToSymbolTableElement for StructUnionSpecifier_Declarator {
    fn get_symbol_entry_name(&self) -> String {
        self.get_struct_item_name()
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        SymbolType::StructUnionItem
    }

    fn get_sybmol_entry_scope(&self) -> String {
        if let Some(scope) = self.get_ast_item().get_scope_parent() {
            scope.get_name()
        }
        else {
            "Struct".to_string()
        }
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        HashMap::default()
    }
}

/// implement ToSymbolTableElement for EnumSpecifier_Constant
/// each enum constant will be added into symbol table as one entry for Enum's symbol table
impl ToSymbolTableElement for EnumSpecifier_Constant {
    fn get_symbol_entry_name(&self) -> String {
        self.get_enum_item()
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        SymbolType::EnumConstant
    }

    fn get_sybmol_entry_scope(&self) -> String {
        if let Some(scope) = self.get_ast_item().get_scope_parent() {
            scope.get_name()
        }
        else {
            "EnumSpecifier_Constant_Global".to_string()
        }
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        HashMap::default()
    }
}

/// implement ToSymbolTableElement for EnumSpecifier, enum type will be added into symbol table as one entry
impl ToSymbolTableElement for EnumSpecifier {
    fn get_symbol_entry_name(&self) -> String {
        self.get_type_name()
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        SymbolType::Enum
    }

    fn get_sybmol_entry_scope(&self) -> String {
        if let Some(scope) = self.get_ast_item().get_scope_parent() {
            scope.get_name()
        }
        else {
            "EnumSpecifier_Global".to_string()
        }
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        HashMap::default()
    }
}

/// implement ToSymbolTableElement for Declaration
impl ToSymbolTableElement for Declaration {
    fn get_symbol_entry_name(&self) -> String {
        panic!("Declaration should not be one symbol entry");
    }

    fn get_sybmol_entry_scope(&self) -> String {
        if let Some(scope) = self.get_ast_item().get_scope_parent() {
            if let Some(function) = scope.get_function_definition() {
                if function.is_main_function() {
                    "GLOBAL".to_string()
                }
                else {
                    function.get_function_name()
                }
            }
            else {
                scope.get_name()
            }
        }
        else {
            "Declaration_Global".to_string()
        }
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        if self.is_function_def() {
            SymbolType::Function
        }
        else if self.is_typedef() {
            SymbolType::TypeAlias
        }
        else {
            if self.get_var_names().len() > 0 {
                Self::get_symbol_table_type_var()
            }
            else {
                Self::get_symbol_table_type_declaration()
            }
        }
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        panic!("Declaration should not be one symbol entry");
    }
}


impl ToSymbolTableElement for CLangProgram {
    fn get_symbol_entry_name(&self) -> String {
        self.get_name()
    }

    fn get_sybmol_entry_scope(&self) -> String {
        if let Some(scope) = self.get_ast_item().get_scope_parent() {
            scope.get_name()
        }
        else {
            "CLangProgram_Global".to_string()
        }
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        SymbolType::Program
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        HashMap::default()
    }
}

impl ToSymbolTableElement for FunctionDefinition {
    fn get_symbol_entry_name(&self) -> String {
        self.get_function_name()
    }

    fn get_sybmol_entry_scope(&self) -> String {
        if let Some(scope) = self.get_ast_item().get_scope_parent() {
            scope.get_name()
        }
        else {
            "FunctionDefinition_Global".to_string()
        }
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        Self::get_symbol_table_type()
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        let mut r = HashMap::default();
        r.insert(Self::get_symbol_table_attribute_key_return_type().to_string(), format!("{}", self.get_return_type_name()));

        let params = self.get_parameters();
        let parameter_key = Self::get_symbol_table_attribute_key_parameters().to_string();

        for p in params {
            let mut p_type = p.get_symbol_table_clang_type();
            if p_type.contains_invalid_clang_item_link() {
                fix_struct_union_link_in_type(&mut p_type, self.get_ast_item());
            }

            r.insert(
                format!("{parameter_key}: {}", p.get_var_name().unwrap_or("<NONAME>".to_string())), 
                format!("{}", p_type)
            );
        }

        r
    }
}

impl ToSymbolTableElement for CompoundStatement {
    fn get_symbol_entry_name(&self) -> String {
        self.get_name()
    }

    fn get_sybmol_entry_scope(&self) -> String {
        "CompoundStatement_Global".to_string()
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        SymbolType::CompoundStatement
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        HashMap::default()
    }
}

impl ToSymbolTableElement for ParameterDeclaration {
    fn get_symbol_entry_name(&self) -> String {
        if let Some(decl_part) = self.get_declartion_part() {
            if let Some(declarator) = decl_part.get_declarator() {
                declarator.get_id_name()
            }
            else {
                let default_name = format!("param_{}", self.get_id());
                default_name
            }
        }
        else {
            let err_msg = format!("ParameterDeclaration id='{}' should have declaration part", self.get_id());
            panic!("{}", err_msg);
        }
    }

    fn get_sybmol_entry_scope(&self) -> String {
        "FUNCTION".to_string()
    }

    fn get_sybmol_entry_type(&self) -> SymbolType {
        Self::get_symbol_table_type_parameter_declaration()
    }

    fn get_symbol_entry_attributes(&self) -> HashMap<String, String> {
        let mut r = HashMap::default();

        let mut ty = self.get_symbol_table_clang_type();
        if ty.contains_invalid_clang_item_link() || ty.is_typealias() {
            let scope_ast = self.get_ast_item().get_scope_parent().map(|x| x.get_ast_item().clone());
            if let Some(scope_ast) = scope_ast.as_ref() {
                fix_struct_union_link_in_type(&mut ty, scope_ast);
            }
            else {
                fix_struct_union_link_in_type(&mut ty, self.get_ast_item());
            }
        }

        r.insert(ATTRIBUTE_KEY_VARTYPE.to_string(), format!("{}", ty));
        r
    }
}