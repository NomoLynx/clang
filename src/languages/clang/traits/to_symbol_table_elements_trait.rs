use crate::languages::clang::clang_asm_code_gen::CodeGenConfiguration;
use crate::languages::clang::clang_program::CLangProgram;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::compound_statement::CompoundStatement;
use crate::languages::clang::declaration::fix_struct_union_link_in_type;
use crate::languages::clang::declaration::Declaration;
use crate::languages::clang::struct_union::StructUnionSpecifier;
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::{CLangError, enum_type::*};
use crate::languages::clang::function_definition::FunctionDefinition;
use crate::languages::clang::parameter_declaration::ParameterDeclaration;
use crate::languages::clang::symbol_table::*;
use crate::languages::clang::traits::to_symbol_table_element_trait::ToSymbolTableElement;

/// trait to provide multiple symbol table elements
/// each implementer can provide multiple symbol table elements
/// the elmement itself must implement ToSymbolTableElement trait because it will be included into the parent symbol table as an entry
pub trait ToSymbolTableElements where Self : ToSymbolTableElement {
    fn get_symbol_table_elements(&self) -> Vec<SymbolEntry> {
        let id = self.get_symbol_entry_id();
        let mut r = SymbolEntry::new(id, &self.get_symbol_entry_name(), self.get_sybmol_entry_type(), &self.get_sybmol_entry_scope());
        let attributes = self.get_symbol_entry_attributes();
        r.set_attributes(attributes.into_iter().map(|(k, v)| (k, v)).collect());
        vec![r]
    }

    /// default behavior is do nothing,
    /// but for some clang item like declaration, 
    /// it may need to setup symbol table for its inner type like enum or struct/union
    fn perform_additional_setup(&mut self, _config:&CodeGenConfiguration) -> Result<(), CLangError> {
        Ok(())
    }
}

/// implement ToSymbolTableElements for Declaration
impl ToSymbolTableElements for Declaration {
    fn get_symbol_table_elements(&self) -> Vec<SymbolEntry> {
        let mut result = vec![];

        // add enum into current symbol table
        if self.get_type().is_enum() {
            let ty = self.get_type().get_enum().unwrap();
            if ty.has_enum_items() {
                let mut entry = SymbolEntry::from_to_symbol_table_element(ty);
                entry.set_clang_item_link(Some(ty.get_ast_item().get_current_node().unwrap().clone()));
                result.push(entry);
            }
        }
        
        let declaration_type = self.get_type_name_string();
        for item in self.get_variable_init_values() {
            let var_name = item.get_id_name();
            let entry_type = self.get_sybmol_entry_type();
            let entry_scope = self.get_sybmol_entry_scope();
            let id = self.get_symbol_entry_id();
            let mut r = if self.is_extern() {
                SymbolEntry::new_external_item(id, &var_name, entry_type, &entry_scope)
            } 
            else {
                SymbolEntry::new(id, &var_name, entry_type, &entry_scope)
            };

            // if function, need add function pararmeters
            if self.is_function_def() && self.is_extern() {
                // function return type as the first attribute
                r.add_attribute(&FunctionDefinition::get_symbol_table_attribute_key_return_type().to_string(), &declaration_type);

                let def_body= self.get_variable_init_values().first().unwrap();
                let parameter_def = def_body.get_declarator().get_parameters();
                assert!(parameter_def.len() == 0 || parameter_def.len() == 1);   // can be 0 or 1+ parameters 
                let mut param_types = Vec::new();
                if parameter_def.len() == 1 {
                    let param_type = parameter_def[0];
                    let param_list = param_type.get_parameters();
                    let param_str = param_list.iter()
                                        .map(|x| x.get_type_name_string())
                                        .collect::<Vec<_>>()
                                        .join(", ");
                    r.add_attribute(&FunctionDefinition::get_symbol_table_attribute_key_parameters().to_string(), &param_str);

                    param_types = param_list.iter()
                        .map(|x| {
                            let mut ty = x.get_symbol_table_clang_type();
                            if ty.is_typealias() || ty.contains_invalid_clang_item_link() {
                                fix_struct_union_link_in_type(&mut ty, self.get_ast_item());
                            }
                            ty
                        })
                        .collect();

                    if param_type.get_eclipse() {
                        r.add_attribute(&FunctionDefinition::get_symbol_table_attribute_key_parameters().to_string(), "...");
                    }
                }

                let mut return_type = self.get_type().clone();
                if return_type.is_typealias() || return_type.contains_invalid_clang_item_link() {
                    fix_struct_union_link_in_type(&mut return_type, self.get_ast_item());
                }
                r.set_data_type(Some(CLangTypes::FunctionPointer(var_name, Box::new(return_type), param_types)));
            }
            else {
                let mut data_type = self.get_declarator_data_type(item);
                if data_type.contains_invalid_clang_item_link() {
                    fix_struct_union_link_in_type(&mut data_type, self.get_ast_item());
                }
                r.set_data_type(Some(data_type));
            }

            result.push(r);
        }

        result
    }

    /// for declaration, need setup symbol table for enum and struct/union type if any
    /// it also set the real definition nodes for example struct A { struct B b; }, 
    /// the B need to be fixed to the actual struct definition, the fix happens here
    fn perform_additional_setup(&mut self, config:&CodeGenConfiguration) -> Result<(), CLangError> {
        self.fix_struct_union_declaration()?;

        // for declaration, need setup symbol table for enum and struct/union type if any
        let ty0 = self.get_type_mut();
        match ty0 {
            CLangTypes::Enum(_, _) => { 
                let ty = ty0.get_enum_mut().unwrap();
                if ty.has_enum_items() {
                    ty.setup_symbol_table(config)
                } 
                else {
                    Ok(())
                }
            }
            CLangTypes::Union(_, _) => { 
                let ty = ty0.get_union_mut().unwrap();
                if ty.is_union_def() { 
                    ty.setup_symbol_table(config) 
                } 
                else {
                    Ok(())
                }
            }
            CLangTypes::Struct(_, _) => { 
                let ty = ty0.get_struct_mut().unwrap();
                if ty.is_struct_def() {
                    ty.setup_symbol_table(config)
                }
                else {
                    Ok(())
                }
            }
            _ => { 
                Ok(())
            }
        }
    }
}

/// implement ToSymbolTableElements for EnumSpecifier
/// each enumerator will be a symbol table element, Enum itself is NOT a scope element though
impl ToSymbolTableElements for EnumSpecifier {
    fn get_symbol_table_elements(&self) -> Vec<SymbolEntry> {
        let mut result = vec![];
        for enum_item in self.get_enum_items() {
            let mut entry = SymbolEntry::from_to_symbol_table_element(enum_item);
            entry.set_data_type(Some(enum_item.get_base_type().clone()));
            entry.set_clang_item_link(Some(enum_item.get_ast_item().get_current_node().unwrap().clone()));
            result.push(entry);
        }

        result
    }
}

/// implement ToSymbolTableElements for Struct / Union (StructUnionSpecifier)
/// each struct/union member will be a symbol table element
impl ToSymbolTableElements for StructUnionSpecifier {
    fn get_symbol_table_elements(&self) -> Vec<SymbolEntry> {
        let mut result = vec![];
        for item in self.get_declares() {
            let mut entry = SymbolEntry::from_to_symbol_table_element(item);
            entry.set_data_type(Some(item.get_item_type().clone()));
            entry.set_clang_item_link(Some(item.get_clang_item().clone()));
            result.push(entry);
        }

        result
    }
}

impl ToSymbolTableElements for CLangProgram {
    // use default implementation
}

impl ToSymbolTableElements for FunctionDefinition {
    fn get_symbol_table_elements(&self) -> Vec<SymbolEntry> {
        let id = self.get_symbol_entry_id();
        let mut entry = SymbolEntry::new(id, &self.get_symbol_entry_name(), self.get_sybmol_entry_type(), &self.get_sybmol_entry_scope());

        // Keep attribute order deterministic: return type first, then parameters in declaration order.
        entry.add_attribute(
            &FunctionDefinition::get_symbol_table_attribute_key_return_type().to_string(),
            &self.get_return_type_name(),
        );

        let parameter_key = FunctionDefinition::get_symbol_table_attribute_key_parameters().to_string();
        for p in self.get_parameters() {
            let mut p_type = p.get_symbol_table_clang_type();
            if p_type.is_typealias() || p_type.contains_invalid_clang_item_link() {
                fix_struct_union_link_in_type(&mut p_type, self.get_ast_item());
            }

            entry.add_attribute(
                &format!("{parameter_key}: {}", p.get_var_name().unwrap_or("<NONAME>".to_string())),
                &format!("{}", p_type),
            );
        }

        let mut return_type = self.get_return_type().clone();
        if return_type.is_typealias() || return_type.contains_invalid_clang_item_link() {
            fix_struct_union_link_in_type(&mut return_type, self.get_ast_item());
        }

        let param_types = self.get_parameters().iter()
            .map(|p| {
                let mut p_type = p.get_symbol_table_clang_type();
                if p_type.is_typealias() || p_type.contains_invalid_clang_item_link() {
                    fix_struct_union_link_in_type(&mut p_type, self.get_ast_item());
                }
                p_type
            })
            .collect::<Vec<_>>();

        let function_name = self.get_function_name();
        entry.set_data_type(Some(CLangTypes::FunctionPointer(function_name, Box::new(return_type), param_types)));
        vec![entry]
    }

    fn perform_additional_setup(&mut self, _config:&CodeGenConfiguration) -> Result<(), CLangError> {
        // Function return typedef alias resolution needs scope-parent links,
        // so perform it during symbol-table setup instead of parse-time from_pair.
        let needs_resolution = {
            let return_type = self.get_return_type();
            return_type.is_typealias() || return_type.contains_invalid_clang_item_link()
        };

        if needs_resolution {
            let ast_item = self.get_ast_item().clone();
            let return_type = self.get_return_type_mut();
            fix_struct_union_link_in_type(return_type, &ast_item);
        }

        Ok(())
    }
}

impl ToSymbolTableElements for CompoundStatement {
    // use default implementation
}

impl ToSymbolTableElements for ParameterDeclaration {
    fn get_symbol_table_elements(&self) -> Vec<SymbolEntry> {
        let id = self.get_symbol_entry_id();
        let mut entry = SymbolEntry::new(
            id,
            &self.get_symbol_entry_name(),
            self.get_sybmol_entry_type(),
            &self.get_sybmol_entry_scope(),
        );

        let attributes = self.get_symbol_entry_attributes();
        entry.set_attributes(attributes.into_iter().collect());

        let mut ty = self.get_clang_type();
        if ty.contains_invalid_clang_item_link() || ty.is_typealias() {
            let scope_ast = self.get_ast_item().get_scope_parent().map(|x| x.get_ast_item().clone());
            if let Some(scope_ast) = scope_ast.as_ref() {
                fix_struct_union_link_in_type(&mut ty, scope_ast);
            }
            else {
                fix_struct_union_link_in_type(&mut ty, self.get_ast_item());
            }
        }

        entry.set_data_type(Some(ty));

        vec![entry]
    }
}