
use std::ops::ShlAssign;
use std::vec;

use core_utils::debug::error_string;
use pest::iterators::Pair;

use crate::languages::clang::{sematic_linkage::SemanticLinkage, traits::*};
use crate::languages::clang::struct_union::*;
use crate::languages::clang::declaration::fix_struct_union_link_in_type;
use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule, declaration::Declaration, external_declaration::ExternalDeclaration, function_definition::FunctionDefinition};
use crate::languages::clang::ast_item_iteration_context::*;
use crate::languages::clang::symbol_table::*;
use crate::languages::clang::logic::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CLangProgram {
    items: Vec<ExternalDeclaration>,
    ast_item : ASTItem,
    symbol_table : SymbolTable,
    source_file_path: Option<String>,
}

impl CLangProgram {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::clang_program); //ensure it is rule:clang_program

        let mut clang_program = CLangProgram::default();
        clang_program.ast_item = ASTItem::new(pair.line_col().into());

        let mut inner_pairs = pair.to_owned().into_inner();

        //choose this method to avoid the rule:EOI
        while let Some(_) = inner_pairs
            .peek()
            .filter(|pair| pair.as_rule() == Rule::external_declaration)
        {
            let inner = inner_pairs.next().unwrap(); //need to consume the rule:external_declaration
            let item = ExternalDeclaration::from_pair(&inner, context)?;
            clang_program
                .items
                .push(item);
        }
        Ok(clang_program)
    }

    /// check if current prog is the root program, whose parent is NULL
    pub fn is_root_prog(&self) -> bool {
        let r = self.get_ast_item().get_parent();
        r.is_none()
    }

    /// type normalization now runs in Declaration::from_pair to avoid post-AST pointer invalidation
    pub fn fix_types(&mut self, _context:&mut CodeGenContext) {
    }

    /// fix the struct/union declartion, Struct A { struct B bb }, this B need to link to its definition
     pub fn fix_struct_union_declaration(&mut self, _context:&mut CodeGenContext) -> Result<(), CLangError> {
        let mut iteration_context = self.get_ast_item().into();
        self.get_ast_item_mut().preorder_tree_mut(&mut iteration_context, 
            &mut |n, m| { 
                let clang_item = n.get_current_node().unwrap();
                if clang_item.is_declaration() {
                    m.visited_nodes.push(clang_item.clone());
                } 
            }, 
            |_| true);

        let decls = iteration_context.visited_nodes;
        for n in decls {
            if let Some(decl) = n.get_declaration() {
                if let Some(ty) = decl.get_struct_union() {
                    let ty_clang_item = ty.get_clang_item();
                    decl.get_type_mut().set_struct_union_clang_item(ty_clang_item);
                }
                // Also fix pointer/array-wrapped struct types (e.g. struct S2 *p;)
                let decl_ast_item = decl.get_ast_item().clone();
                fix_struct_union_link_in_type(decl.get_type_mut(), &decl_ast_item);
            }
        }

        Ok(())
    }

    pub fn get_source_file_path(&self) -> Option<&String> {
        self.source_file_path.as_ref()
    }

    pub fn set_source_file_path(&mut self, path:&str) {
        self.source_file_path = Some(path.to_string());
    }

    pub fn get_clang_units(&self) -> &Vec<ExternalDeclaration> {
        &self.items
    }

    pub fn get_clang_functions(&self) -> Vec<&FunctionDefinition> {
        let r = self.get_clang_units().iter()
                .filter_map(|x| x.get_function())
                .collect::<Vec<_>>();

        r
    }

    /// get all child clang programs included in this clang program
    pub fn get_child_clang_progs(&self) -> Vec<&CLangProgram> {
        let r = self.items.iter()
            .filter_map(|x| x.get_clang_program())
            .collect::<Vec<_>>();

        r
    }

    /// get external item from symbol tables from attached clang program
    pub fn get_extern_items(&self, symbol:&str, symbol_type:&SymbolType) -> Option<&SymbolEntry> {
        for clang_prog in self.get_child_clang_progs() {
            let extern_items = clang_prog.get_extern_items_locally(symbol, symbol_type);
            if extern_items.is_some() {
                return extern_items;
            }
        }

        self.get_extern_items_locally(symbol, symbol_type)
    }

    /// get external item from current symbol table only with no recursion
    /// parameters symbol: symbol name
    /// parameters symbol_type: symbol type
    pub fn get_extern_items_locally(&self, symbol:&str, symbol_type:&SymbolType) -> Option<&SymbolEntry> {
        if let Some(n) = self.get_symbol_table().lookup_by_type(symbol, symbol_type) {
            if n.is_extern() {
                return Some(n);
            }
        }

        None  
    }

    /// check if entry point 'main' exists
    pub fn is_entry_point_exists(&self) -> bool {
        let functions = self.get_clang_functions();
        for f in functions {
            if f.is_main_function() {
                return true;
            }
        }

        false
    }

    pub fn get_clang_declarations(&self) -> Vec<&ExternalDeclaration> {
        let r = self.get_clang_units().iter()
            .filter(|x| matches!(x, ExternalDeclaration::Declaration(_, _)))
            .collect::<Vec<_>>();

        r
    }

    pub fn get_declarations(&self) -> Vec<&Declaration> {
        let r = self.get_clang_declarations().iter()
                    .filter_map(|x| x.get_declaration())
                    .collect::<Vec<_>>();

        r
    }

    pub fn get_struct_def(&self, name:&str) -> Option<&StructUnionSpecifier> {
        let dels = self.get_declarations();
        let struct_defs = dels.iter().filter(|x| x.get_type().is_struct() || x.get_type().is_union())
                                                        .collect::<Vec<_>>();
        let r = struct_defs.iter().filter(|x| x.has_var_name(name)).collect::<Vec<_>>();
        if r.len()==0 {
            None
        }
        else {
            let core_types = r.first()?.get_type();
            core_types.get_struct_union()
        }
    }

    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        self.items.iter_mut()
            .flat_map(|x| x.get_children())
            .collect()
    }

    /// set up the semantic linkages between clang items
    /// 1. process address dot and arrow case
    /// 2. process other cases other than address dot and arrow, like variable usage link to declaration or definition
    pub fn setup_semantic_linkages(&mut self) -> ASTItemIterationContext{
        // process address dot and arrow case
        {
            let ast_item = self.get_ast_item_mut();
            let mut context_obj = ast_item.into();
            let context = &mut context_obj;

            let child_can_visit = | n : &mut ASTItem | {
                if let Some(expr) = n.get_current_node().and_then(|x| x.get_expression()) {
                    if expr.is_addressing_expression() {
                        false // can't visit children of addressing expression, which is address dot and arrow
                    }
                    else { true }
                }
                else {  true  }
            };

            let visit = &mut |n:&mut ASTItem, context:&mut ASTItemIterationContext| {
                if let Some(expr) = n.get_current_node().and_then(|x| x.get_expression()) {
                    if expr.is_addressing_expression() {
                        match expr.set_semantic_linkage_for_addressing_expression() {
                            Ok(_) => { expr.set_is_highest_addressing_expression(true); },
                            Err(ex) => {
                                error_string(format!("Failed to set semantic linkage for addressing expression: {ex:?}"));
                                context.add_error_node(n.get_current_node().unwrap().clone(), 
                                    (format!("Failed to set semantic linkage for addressing expression: {ex:?}"), n.get_source_file_location()).into());
                            }
                        }
                    }
                }
            };

            ast_item.preorder_tree_mut(context, visit, child_can_visit);
        }
        
        // process other cases other than address dot and arrow, like variable usage link to declaration or definition
        let ast_item = self.get_ast_item_mut();
        let mut context_obj = ast_item.into();
        let context = &mut context_obj;

        // for atom item, link its id to the declaration or definition
        let child_can_visit = | n : &mut ASTItem | {
            if let Some(expr) = n.get_current_node().and_then(|x| x.get_expression()) {
                if expr.is_addressing_expression() {
                    return false; // can't visit children of addressing expression, which is address dot and arrow and process above
                }
            }

            true
        };
        let visit = &mut |n:&mut ASTItem, context:&mut ASTItemIterationContext| {
            if n.is_atom_item_id() {
                let atom_item = n.to_atom_item_mut().unwrap();
                let type_def_clang_item = atom_item.get_declaration_type().cloned();
                if let Some(type_def_clang_item) = type_def_clang_item {
                    n.add_semantic_linkage(SemanticLinkage::Definition(type_def_clang_item.clone()));
                    type_def_clang_item.get_ast_item_mut().add_semantic_linkage(SemanticLinkage::DefinitionUsage(n.get_current_node().unwrap().clone()));
                }
                else {
                    let name = atom_item.get_name();
                    context.add_error_node(n.get_current_node().unwrap().clone(), 
                        (format!("Failed to get declaration type for AtomItem in semantic linkage build: '{name}'"), n.get_source_file_location()).into());
                }
            }
        };
        ast_item.postorder_tree_mut(context, visit, child_can_visit);

        context_obj
    }

    /// enrich symbol table, like address_taken
    pub fn enrich_symbol_table(&mut self, _code_gen_context:&mut CodeGenContext) {
        let ast = self.get_ast_item_mut();
        let citem = ast.get_current_node().unwrap().clone();

        let mut context = ASTItemIterationContext::new(citem);
        ast.preorder_tree_mut(&mut context, &mut |x, context| {
            let item = x.get_current_node().unwrap().clone();
            if item.is_expression() {
                let mut all_children = item.get_expression().unwrap().get_all_clangitems();
                context.visited_nodes.append(&mut all_children);
            }
        }, |_| { true } );

        // process expression to set the address taken flag
        let results = execute_find_address_taken_item_logic(context.visited_nodes);

        for item in results {
            if let Some(atom_item) = item.get_atom_item() {
                let symbol_name = atom_item.get_id_value().unwrap().clone();
                let symbol_table = atom_item.get_ast_item_mut().get_symbol_table_mut().unwrap();
                if let Some(symbol_entry) = symbol_table.lookup_mut(&symbol_name) {
                    symbol_entry.set_is_address_taken(true);
                }
            }
        }
    }

    /// assign registers
    pub fn assign_registers(&mut self, context:&mut CodeGenContext) {
        let ast_item = self.get_ast_item_mut();

        let mut iteration_context = ast_item.into();

        let child_can_visit = | _n : &mut ASTItem | {
           true
        };

        let visit = &mut |n:&mut ASTItem, _context0:&mut ASTItemIterationContext| {
            if n.is_scope_element() {
                let scope_item = n.get_scope_item().unwrap();
                scope_item.assgin_var_register(context);
                scope_item.assign_parameter_register(context);
            }
        };
        
        // assign register is from bottom to top, so post order is required.
        ast_item.postorder_tree_mut(&mut iteration_context, visit, child_can_visit);
    }

    /// get and set types for each variables in the program
    pub fn setup_type_system(&mut self, context:&mut CodeGenContext) -> ASTItemIterationContext {
        let ast_item = self.get_ast_item_mut();
        let type_config = context.get_type_config();

        let mut iteration_context = ast_item.into();

        let child_can_visit = | _n : &mut ASTItem | {
           true
        };

        let visit = &mut |n:&mut ASTItem, context0:&mut ASTItemIterationContext| {
            let need_check = type_config.is_ast_item_has_type_node(n);
            if need_check {
                type_config.infer_and_check_type(n, context0);
            }
        };
        
        ast_item.postorder_tree_mut(&mut iteration_context, visit, child_can_visit);
        iteration_context
    }

    pub fn access_children_deep_first(&mut self, context:&mut CodeGenContext) {
        for x in self.items.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(self.into());
    }
    
    /// merge another CLangProgram into this CLangProgram
    pub fn add_clang_program(&mut self, other:CLangProgram) {
        self.items.push(ExternalDeclaration::Program(
            ASTItem::default(),
            Box::new(other),
        ));
    }

    /// get referenced external file name list
    pub fn get_referenced_external_file_list(&self) -> Vec<String> {
        let mut r = vec![self.get_source_file_path().unwrap_or(&String::from("")).to_string()];
        for x in self.get_child_clang_progs() {
            if let Some(file_name) = x.get_source_file_path() {
                r.push(file_name.to_string());
            }
        }

        r
    }

    /// generate meta info string
    /// * returns String: meta info string
    /// * it includes all referenced external file names
    pub fn generate_meta_info(&self) -> String {
        let external_files = self.get_referenced_external_file_list();
        let meta_info = external_files.iter()
            .map(|x| format!("# External file: {}", x))
            .collect::<Vec<_>>()
            .join("\r\n");

        meta_info
    }

    /// perform sematic checks
    pub fn sematic_checks(&mut self) -> Result<Vec<CLangError>, CLangError> {
        let symbol_tables = self.get_all_symbol_tables()?;
        
        let mut errors = vec![];
        for symbol_table in symbol_tables {
            errors.append(&mut symbol_table.sematic_checks()?);
        }

        Ok(errors)
    }
}

impl Default for CLangProgram {
    fn default() -> Self {
        Self { items: vec![], ast_item: ASTItem::default(), symbol_table: SymbolTable::default(), source_file_path:None }
    }
}

impl ASTItemTrait for CLangProgram {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("CProg({})", self.get_ast_item().get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl ShlAssign for CLangProgram {
    fn shl_assign(&mut self, other: Self) {
        self.add_clang_program(other);
    }
} 
