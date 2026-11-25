use crate::languages::clang::{ASTItem, clang_asm_code_gen::CodeGenConfiguration, clang_error::CLangError, clang_program::CLangProgram, compound_statement::CompoundStatement, function_definition::FunctionDefinition, symbol_table::{SymbolEntry, SymbolTable, SymbolType}};

use super::scope_trait::ScopeTrait;
use super::*;
use crate::languages::clang::ast_item_iteration_context::*;
use core_utils::debug::*;

fn get_typedef_alias_target(entry:&SymbolEntry) -> Option<(String, String, SymbolType)> {
    if entry.get_symbol_type() != &SymbolType::TypeAlias {
        return None;
    }

    let alias_name = entry.get_name().to_string();
    let data_type = entry.get_data_type()?;

    if let Some(struct_union) = data_type.get_struct_union() {
        let target_name = struct_union.get_type_name();
        let target_type = if struct_union.is_struct() {
            SymbolType::Struct
        }
        else {
            SymbolType::Union
        };

        return Some((alias_name, target_name, target_type));
    }

    if let Some(enum_specifier) = data_type.get_enum() {
        return Some((alias_name, enum_specifier.get_type_name(), SymbolType::Enum));
    }

    None
}

fn add_typedef_alias_to_symbol_entry(ast_item:&ASTItem, target_name:&str, target_type:&SymbolType, alias_name:&str) -> bool {
    let parent_scope = ast_item.get_scope_parent().cloned();

    if let Some(scope_item) = ast_item.get_scope_item() {
        if let Some(entry) = scope_item.get_symbol_table_mut().lookup_by_type_mut(target_name, target_type) {
            entry.add_type_alias(alias_name);
            return true;
        }
    }

    if let Some(parent_scope) = parent_scope {
        return add_typedef_alias_to_symbol_entry(parent_scope.get_ast_item(), target_name, target_type, alias_name);
    }

    false
}

/// trait to create and access symbol table
pub trait AccessSymbolTableTrait {
    fn get_symbol_table(&self) -> &SymbolTable;
    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable;

    fn get_all_symbol_tables(&mut self) -> Result<Vec<&SymbolTable>, CLangError> {
        let t = self.get_symbol_table();
        let mut sub_table_from_t = t.get_sub_tables();
        let mut r = vec![t];
        r.append(&mut sub_table_from_t);

        // get scope element's symbol tables
        let mut child_scope_elements = self.get_symbol_table().get_child_scope_elements();
        for x in child_scope_elements.iter_mut() {
            for table in x.get_all_symbol_tables()? {
                r.push(table);
                let mut tables = table.get_sub_tables();
                r.append(&mut tables);                
            }
        }

        Ok(r)
    }

    fn setup_symbol_table(&mut self, config:&CodeGenConfiguration) -> Result<(), CLangError>
        where Self: ASTItemTrait + ScopeTrait 
    {
        let ast = self.get_ast_item_mut();
        let citem = ast.get_current_node().unwrap().clone();

        // get all symbol entry element in the current scope with preorder traversal
        // this order is important because child's can only reference a type already defined in parent or itself
        // please note that child of 'declaration' and 'scope' will NOT be visited
        let mut context = ASTItemIterationContext::new(citem.clone());
        ast.preorder_tree_mut(&mut context, 
                &mut |x, context| { 
                    let item = x.get_current_node().unwrap().clone();
                    let root = context.current_scope_element.as_ref().unwrap().clone();
                    if item != root && item.is_symbol_entry_element() {
                        context.visited_nodes.push( item.clone() ) 
                    }
                    else if item.is_for_loop() {
                        context.visited_nodes.push( item.clone() )   // var can be defined in for loop init statement
                    }
                }, 
            |n| {                 
                let node = n.get_current_node().unwrap();
                !node.is_declaration() &&    // can't visit declaration and scope's children
                !node.is_scope()
            });

        // context.visited_nodes is in preorder traversal order and all its nodes can generate symbol entry
        // perform_addition_setup is to do additional action including find real struct/union definition 
        // before generate symbol table
        AccessSymbolTableTrait::get_symbol_table_mut(self).set_container(Some(citem.clone()));
        for n in context.visited_nodes.iter_mut() {
            // set the scope parent which is indicate the current node belongs to which scope
            let n_ast = n.get_ast_item_mut();
            n_ast.set_scope_parent(Some(citem.clone()));

            if let Some(x) = n.get_symbol_entry_elements() {
                x.perform_additional_setup(config)?;
                for mut entry in x.get_symbol_table_elements() {
                    if entry.get_clang_item_link().is_none() {
                        entry.set_clang_item_link(Some(n.clone()));
                    }
                    AccessSymbolTableTrait::get_symbol_table_mut(self).insert(entry);
                }
            }
        }

        // add symbol alias to the real type definition, the assumption is that the element should
        // be setup already in the current symbol table or parent symbol table
        let typedef_alias_updates = AccessSymbolTableTrait::get_symbol_table(self)
            .get_all_entries()
            .iter()
            .filter_map(get_typedef_alias_target)
            .collect::<Vec<_>>();

        for (alias_name, target_name, target_type) in typedef_alias_updates {
            if !add_typedef_alias_to_symbol_entry(self.get_ast_item(), &target_name, &target_type, &alias_name) {
                warn_string(format!("Cannot resolve typedef alias '{}' to target '{} ({})'", alias_name, target_name, target_type));
            }
        }
        

        // process the scope elements in current scope
        let mut scope_elements = context.visited_nodes.iter_mut()
                                                .filter_map(|x| x.get_scope_trait())
                                                .collect::<Vec<_>>();
        for n in scope_elements.iter_mut() {
            self.add_child_scope(& n.get_scope_ast_item().get_current_node().unwrap().clone());
        }

        // build up the symbol table for each scope
        for n in context.visited_nodes.iter_mut().filter(|x| x.is_scope()) {
            match n.setup_symbol_table(config) {
                Ok(_) => { }
                Err(ex) => {
                    error_string(format!("setup symbol table error: {ex:?}"));
                    panic!("setup symbol table error");
                }
            }
        }

        // process for FOR node, put the symbol definition to child component's symbol table
        for n in context.visited_nodes.iter_mut().filter(|x| x.is_for_loop()) {
            let for_node = n.get_for_loop().unwrap();
            for_node.process_definitions_in_init_statement(config)?;
        }

        Ok(())
    }
}

impl AccessSymbolTableTrait for CLangProgram {
    fn get_symbol_table(&self) -> &SymbolTable {
        self.get_symbol_table()
    }

    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        self.get_symbol_table_mut()
    }
}

impl AccessSymbolTableTrait for FunctionDefinition {
    fn get_symbol_table(&self) -> &SymbolTable {
        self.get_symbol_table()
    }

    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        self.get_symbol_table_mut()
    }
}

impl AccessSymbolTableTrait for CompoundStatement {
    fn get_symbol_table(&self) -> &SymbolTable {
        self.get_symbol_table()
    }

    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        self.get_symbol_table_mut()
    }
}