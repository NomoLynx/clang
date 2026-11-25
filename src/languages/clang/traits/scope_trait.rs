use super::*;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::{ast_item::ASTItem, clang_item::CLangItem, clang_program::CLangProgram, compound_statement::CompoundStatement, function_definition::FunctionDefinition};
use crate::languages::clang::symbol_table::*;

pub trait ScopeTrait {
    fn get_scope_ast_item(&self) -> &ASTItem;
    fn add_child_scope(&mut self, child:&CLangItem);
    fn set_parent_scope(&mut self, parent:Option<&CLangItem>);
    fn get_symbol_table(&self) -> &SymbolTable;
    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable;

    fn get_symbol_entry(&self, id:&str, symbol_type:&SymbolType) -> Option<&SymbolEntry> {
        if let Some(var) = self.get_symbol_table().lookup_by_type(id, symbol_type) {
            Some(var)
        }
        else {
            if let Some(parent) = self.get_scope_ast_item().get_scope_parent() {
                parent.get_scope_trait()
                        .and_then( |x| x.get_symbol_entry(id, symbol_type) )
            }
            else {
                None
            }
        }
    }

    fn assgin_var_register(&mut self, context:&mut CodeGenContext) {
        let symbol_table = self.get_symbol_table_mut();
        symbol_table.assgin_var_register(context);
    }

    fn assign_parameter_register(&mut self, _context:&mut CodeGenContext) { 
        // default implementation does nothing
    }
}

impl ScopeTrait for CLangProgram {
    fn get_scope_ast_item(&self) -> &ASTItem {
        self.get_ast_item()
    }

    fn add_child_scope(&mut self, child:&CLangItem) {
        self.get_ast_item_mut().add_scope_child(child.clone());
    }

    fn set_parent_scope(&mut self, parent:Option<&CLangItem>) {
        self.get_ast_item_mut().set_scope_parent(parent.cloned());
    }

    fn get_symbol_table(&self) -> &SymbolTable {
        self.get_symbol_table()
    }

    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        self.get_symbol_table_mut()
    }
}

impl ScopeTrait for FunctionDefinition {
    fn get_scope_ast_item(&self) -> &ASTItem {
        self.get_ast_item()
    }

    fn add_child_scope(&mut self, child:&CLangItem) {
        self.get_ast_item_mut().add_scope_child(child.clone());
    }

    fn set_parent_scope(&mut self, parent:Option<&CLangItem>) {
        self.get_ast_item_mut().set_scope_parent(parent.cloned());
    }

    fn get_symbol_table(&self) -> &SymbolTable {
        self.get_symbol_table()
    }

    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        self.get_symbol_table_mut()
    }

    fn assign_parameter_register(&mut self, context:&mut CodeGenContext) {
        let n = self.update_symbol_offset(context.get_parameter_register_number(), 
                    context.get_code_gen_config().get_addressing_in_bytes());
        n.unwrap()
    }
}

impl ScopeTrait for CompoundStatement {
    fn get_scope_ast_item(&self) -> &ASTItem {
        self.get_ast_item()
    }

    fn add_child_scope(&mut self, child:&CLangItem) {
        self.get_ast_item_mut().add_scope_child(child.clone());
    }

    fn set_parent_scope(&mut self, parent:Option<&CLangItem>) {
        self.get_ast_item_mut().set_scope_parent(parent.cloned());
    }

    fn get_symbol_table(&self) -> &SymbolTable {
        self.get_symbol_table()
    }

    fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        self.get_symbol_table_mut()
    }
}