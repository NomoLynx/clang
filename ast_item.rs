use core::panic;
use std::cmp::min;
use std::cmp::max;

use super::{clang_item::{CLangItem, ParentChildrenLinker}, function_definition::FunctionDefinition, symbol_table::{SymbolEntry, SymbolTable}, traits::scope_trait::ScopeTrait};

use crate::languages::clang::ast_item_iteration_context::*;
use crate::languages::clang::clang_program::CLangProgram;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::struct_union::StructUnionSpecifier;
use crate::languages::clang::tanry_expression::AtomItem;
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::type_system::CLangRuntimeType;
use crate::languages::mermaid_lang::zenUml_pest::Rule::aliases;
use crate::languages::{SourceFileLocation, clang::declaration::Declaration};
use crate::languages::clang::clang_asm_code_gen::*;
use crate::languages::clang::sematic_linkage::*;
use crate::languages::clang::symbol_table::*;
use core_utils::range::Range;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ASTItem {
    sequence_id : usize,
    current_node : Option<CLangItem>,
    common_linker : ParentChildrenLinker,
    scope_linker : ParentChildrenLinker,
    code : CodeSnippet,
    source_file_location : SourceFileLocation,
    semantic_linkages : Vec<SemanticLinkage>,
    assigned_registers : RegisterSet,

    // meta information about this ast item
    value_changed: bool,   // whether the id value has been changed in the expression, only applies to the atom id items
    runtime_type : CLangRuntimeType,
}

impl ASTItem {
    /// new instance by giving source file location
    pub fn new(location:SourceFileLocation) -> Self {
        let mut r = Self::default();
        r.set_source_file_location(location);
        r
    }

    /// get type name of current node
    pub fn get_type_name(&self) -> String {
        if let Some(citem) = self.get_current_node() {
            citem.get_name()
        }
        else {
            panic!("ASTItem has no current node to get name");
        }
    }

    /// get runtime type
    pub fn get_runtime_type(&self) -> Option<&CLangTypes> {
        self.runtime_type.as_ref().as_ref()
    }

    /// set runtime type
    pub fn set_runtime_type(&mut self, t:CLangTypes) {
        *(self.runtime_type.as_mut()) = Some(t);
    }

    pub fn get_sequence_id(&self) -> usize {
        self.sequence_id
    }

    pub fn set_sequence_id(&mut self, v:usize) {
        self.sequence_id = v;
    }

    pub fn get_assigned_registers(&self) -> &RegisterSet {
        &self.assigned_registers
    }

    pub fn has_assigned_register(&self) -> bool {
        !self.get_assigned_registers().is_empty()
    }

    pub fn add_child(&mut self, child:CLangItem) {
        self.common_linker.add_child(child);
    }

    pub fn set_children(&mut self, children:Vec<CLangItem>) {
        let v = children.into_iter().map(|x| Box::new(x)).collect::<Vec<_>>();
        self.common_linker.set_children(Box::new(v));
    }

    pub fn set_value_changed(&mut self, changed:bool) {
        self.value_changed = changed;
    }

    pub fn get_value_changed(&self) -> bool {
        self.value_changed
    }

    pub fn set_parent(&mut self, parent:Option<CLangItem>) {
        self.common_linker.set_parent(parent);
    }

    /// set parent scope, this builds up the parent-child relationship between scopes, 
    /// which is used to find symbol table in parent scopes when lookup symbol in current scope
    pub fn set_scope_parent(&mut self, parent:Option<CLangItem>) {
        self.scope_linker.set_parent(parent);
    }

    pub fn get_scope_linker(&self) -> &ParentChildrenLinker {
        &self.scope_linker
    }

    /// get current scope's parent scople clang item if any
    pub fn get_scope_parent(&self) -> Option<&CLangItem> {
        if let Some(scope_item) = self.scope_linker.get_parent() {
            Some(scope_item)
        }
        else {
            // first try to find a scope item and get its parent
            if let Some(item) = self.get_scope_item() {
                let ast_item = item.get_scope_ast_item();
                ast_item.scope_linker.get_parent()
            }
            else {
                None
            }
        }
    }

    pub fn get_scope_children_mut(&mut self) -> &Vec<Box<CLangItem>> {
        self.scope_linker.get_children()
    }

    pub fn get_scope_children(&self) -> &Vec<Box<CLangItem>> {
        self.scope_linker.get_children()
    }

    pub fn add_scope_child(&mut self, child:CLangItem) {
        self.scope_linker.add_child(child);
    }

    pub fn get_parent(&self) -> Option<&CLangItem> {
        self.common_linker.get_parent()
    }

    /// check if the ancestor is in the parent chain
    pub fn has_ancestor(&self, ancestor: &CLangItem) -> bool {
        let mut current = self.get_parent();
        while let Some(citem) = current {
            if citem == ancestor {
                return true;
            }
            current = citem.get_ast_item().get_parent();
        }
        false
    }

    /// get root ast item
    pub fn get_root_ast_item(&self) -> &ASTItem {
        if let Some(parent) = self.get_parent() {
            parent.get_ast_item().get_root_ast_item()
        }
        else {
            self
        }
    }

    pub fn get_current_node(&self) -> Option<&CLangItem> {
        self.current_node.as_ref()
    }

    pub fn set_current_node(&mut self, node:CLangItem) {
        self.current_node = Some(node);
    }

    pub fn get_children(&self) -> &Vec<Box<CLangItem>> {
        self.common_linker.get_children()
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Box<CLangItem>> {
        self.common_linker.get_children_mut()
    }

    /// set source file location
    pub fn set_source_file_location(&mut self, location:SourceFileLocation) {
        self.source_file_location = location;
    }

    /// get source file location
    pub fn get_source_file_location(&self) -> &SourceFileLocation {
        &self.source_file_location
    }

    /// get scope item above (close to the root) which contains symbol table
    /// FOR statement will be the compound statement inside
    pub fn get_scope_item(&self) -> Option<&mut dyn ScopeTrait> {
        if self.is_scope_element() {
            self.get_current_node().and_then(|x| x.get_scope_trait())
        }
        else {
            if let Some(node) = self.get_current_node() {
                if node.is_for_loop() {   //if FOR loop, scope is the compound statement inside FOR
                    let for_loop = self.get_current_node().unwrap().get_for_loop().unwrap();
                    for_loop.get_embedded_compound_statement_mut().map(|x| x as &mut dyn ScopeTrait)
                }
                else {
                    if let Some(citem) = self.get_parent() {
                        citem.get_ast_item().get_scope_item()
                    }
                    else {
                        None
                    }
                }
            }
            else {
                None
            }
        }
    }

    /// get struct or union clang item by struct or union specifier, it will search in current scope and parent scopes, 
    /// and then search in child clang programs of root node
    pub fn get_struct_or_union_clang_item(&self, struct_union:&StructUnionSpecifier) -> Option<&CLangItem> {
        let name = struct_union.get_type_name();
        if struct_union.is_struct() {
            self.get_struct_clang_item(&name)
        }
        else if struct_union.is_union() {
            self.get_union_clang_item(&name)
        }
        else {
            None
        }
    }

    /// get struct clang item by struct name, it will search in current scope and parent scopes, 
    /// and then search in child clang programs of root node
    pub fn get_struct_clang_item(&self, struct_name:&str) -> Option<&CLangItem> {
        if let Some(entry) = self.get_symbol(struct_name, &SymbolType::Struct) {
            let clang_item = entry.get_clang_item_link();
            clang_item
        }
        else {
            None
        }
    }

    /// get union clang item by union name, it will search in current scope and parent scopes, 
    /// and then search in child clang programs of root node
    pub fn get_union_clang_item(&self, union_name:&str) -> Option<&CLangItem> {
        if let Some(entry) = self.get_symbol(union_name, &SymbolType::Union) {
            let clang_item = entry.get_clang_item_link();
            clang_item
        }
        else {
            None
        }
    }

    /// get enum clang item by enum name, it will search in current scope and parent scopes, 
    /// and then search in child clang programs of root node
    pub fn get_enum_clang_item(&self, enum_name:&str) -> Option<&CLangItem> {
        if let Some(entry) = self.get_symbol(enum_name, &SymbolType::Enum) {
            let clang_item = entry.get_clang_item_link();
            clang_item
        }
        else {
            None
        }
    }

    /// get symbol from str 
    pub fn get_symbol(&self, symbol:&str, symbol_type:&SymbolType) -> Option<&SymbolEntry> {
        let r = self.get_scope_item()
                                            .and_then(|x| x.get_symbol_entry(symbol, symbol_type));
        if r.is_some() {
            return r;
        }
        
        // check extern item list
        let root_ast_item = self.get_root_ast_item();
        let clang_prog = root_ast_item.get_current_node().unwrap()
                                                            .to_clang_prog().unwrap();
        clang_prog.get_extern_items(symbol, symbol_type)
    }

    /// get symbol type from str recursively up the parent scopes
    pub fn get_symbol_type_recursive_up(&self, symbol:&str) -> (SymbolType, Option<&SymbolEntry>) {
        if let Some(scope_item) = self.get_scope_item() {
            let symbol_table = scope_item.get_symbol_table();
            if let Some(entry) = symbol_table.lookup(symbol) {
                let r = entry.get_symbol_type().clone();
                return (r, Some(entry));
            }
            else {
                // continue search in parent scope
                if let Some(parent_scope) = self.get_scope_parent() {
                    return parent_scope.get_ast_item().get_symbol_type_recursive_up(symbol);
                }
                else { 
                    // not found, then search for reference in root's child clang programs
                    // when reach root node, self does not have parent anymore, and will need to search child clang programs  
                }
            }
        }
        
        let root_ast_item = self.get_root_ast_item();
        let clang_prog = root_ast_item.get_current_node().unwrap()
                                                            .to_clang_prog().unwrap();
        for clang_item in clang_prog.get_child_clang_progs() {
            let symbol_table = clang_item.get_symbol_table();
            if let Some(entry) = symbol_table.lookup(symbol) {
                let r = entry.get_symbol_type().clone();
                return (r, Some(entry));
            }
        }

        (SymbolType::Unknown, None)
    }

    /// add linkage
    pub fn add_semantic_linkage(&mut self, link:SemanticLinkage) {
        self.semantic_linkages.push(link);
    }

    /// get semantic linkages
    pub fn get_semantic_linkages(&self) -> &Vec<SemanticLinkage> {
        &self.semantic_linkages
    }

    pub fn get_var_symbol_entries_recursive(&self, var_names:Vec<String>) -> Vec<&SymbolEntry> {
        match var_names.as_slice() {
            [var_name] => {
                self.get_var_symbol_entry_recursive(var_name)
                    .into_iter()
                    .collect()
            }
            [composite_name, rest@..] => {
                let mut result = vec![];
                let mut composite_var_entry = self.get_var_symbol_entry_recursive(composite_name).unwrap();
                result.push(composite_var_entry);

                for n in rest { 
                    let symbol_table = composite_var_entry.get_composite_type_symbol_table().unwrap();
                    composite_var_entry = symbol_table.lookup(n).unwrap();
                    result.push(composite_var_entry);
                }
                
                result
            }
            [] => {
                vec![]
            }
        }
    }

    /// get var declaration symbol recursively from parent scopes, it only returns variable declaration, it will not return type declaration or function declaration
    pub fn get_var_symbol_entry_recursive(&self, var_name:&str) -> Option<&SymbolEntry> {
        let r = self.get_scope_item()
                    .and_then(|x| x.get_symbol_entry(var_name, &SymbolType::Var));
        if r.is_some() {
            return r;
        }
        else {
            if let Some(parent_scope) = self.get_scope_parent() {
                return parent_scope.get_ast_item().get_var_symbol_entry_recursive(var_name);
            }
            else {
                // not found, then search for reference in root's child clang programs
                // when reach root node, self does not have parent anymore, and will need to search child clang programs  
                let root_ast_item = self.get_root_ast_item();
                let clang_prog = root_ast_item.get_current_node().unwrap()
                                                                    .to_clang_prog().unwrap();
                for clang_item in clang_prog.get_child_clang_progs() {
                    let symbol_table = clang_item.get_symbol_table();
                    if let Some(entry) = symbol_table.lookup(var_name) {
                        if entry.get_symbol_type().is_var() {
                            return Some(entry);
                        }
                    }
                }

                None
            }
        }
    }

    /// get var declaration symbol resursively from parent scopes and search all types if not found in current scope
    /// it returns var, type declarations, and function
    /// this function preserves the requested symbol name and does not rewrite it to a type name
    pub fn get_symbol_recursive_all_types(&self, symbol:&str) -> Option<&SymbolEntry> {
        let type_list = vec![
                            SymbolType::Struct,
                            SymbolType::Union,
                            SymbolType::EnumConstant,
                            SymbolType::TypeAlias,
                            Declaration::get_symbol_table_type_declaration(),
                            Declaration::get_symbol_table_type_var(),
                            FunctionDefinition::get_symbol_table_type(),
                        ];

        let r = self.get_scope_item()
                    .and_then(|x| { 
                        let symbol_table = x.get_symbol_table();
                        let r = symbol_table.lookup_by_type_list(symbol, &type_list);
                        if let Some(entry) = r {
                            let clang_item = entry.get_clang_item_link();
                            if let Some(clang_item) = clang_item {
                                // the same id can be used to initialize the current declaration but that name (e.g. val0) is from parent scope
                                // if that is the case, return None to continue searching in parent scope
                                let same_decleration = clang_item.is_declaration() && self.has_ancestor(clang_item);
                                if same_decleration {   
                                    return None;
                                }
                            }
                        }
                        
                        r 
                    })
                    .or_else( || {
                        if let Some(citem) = self.get_scope_parent() {
                            citem.get_ast_item().get_symbol_recursive_all_types(symbol)
                        }
                        else {
                            // need to search child clang program's global scope
                            for n in self.get_children() {
                                if n.is_clang_prog() {
                                    let ast_item = n.get_ast_item();
                                    if let Some(scope_item) = ast_item.get_scope_item() {
                                        if let Some(r) = scope_item.get_symbol_table().lookup_by_type_list(symbol, &type_list) {
                                            return Some(r);
                                        }
                                    }
                                    else {
                                        panic!("clang program must be a scope item");
                                    }
                                }
                            }

                            None
                        }
                    } );
        r
    }

    /// get symbol recursively and then resolve composite variables to their type definition entry
    /// this keeps get_symbol_recursive_all_types name-preserving while allowing explicit type-definition lookup
    pub fn get_symbol_recursive_type_definition(&self, symbol:&str) -> Option<&SymbolEntry> {
        let entry = self.get_symbol_recursive_all_types(symbol)?;

        if let Some(data_type) = entry.get_data_type() {
            match data_type {
                CLangTypes::Struct(n, _) |
                CLangTypes::Union(n, _) => {
                    if let Some(type_name) = data_type.get_struct_union()
                        .and_then(|x| x.get_name_id()) {
                        let ast_item = n.get_ast_item();
                        if let Some(type_entry) = ast_item.get_symbol_recursive_all_types(type_name) {
                            return Some(type_entry);
                        }
                    }
                }
                CLangTypes::TypeAlias(_type_alias, type_option) => {
                    let n = type_option.as_ref()
                                                                .and_then(|x| x.get_struct_union());
                    if let Some(type_name) = n.and_then(|x| x.get_name_id()) {
                        let ast_item = n.map(|x| x.get_ast_item());
                        if let Some(type_entry) = ast_item.and_then(|x| x.get_symbol_recursive_all_types(type_name)) {
                            return Some(type_entry);
                        }
                    }
                }
                _ => {}
            }
        }

        Some(entry)
    }

    /// get root clangproram item
    pub fn get_root_clang_program_item(&self) -> Option<&CLangProgram> {
        let root_ast_item = self.get_root_ast_item();
        if let Some(citem) = root_ast_item.get_current_node() {
            if citem.is_clang_prog() {
                return Some(citem.to_clang_prog().unwrap());
            }
        }

        None
    }

    /// check if the symbol is stored in register
    pub fn is_reg_used(&self, name:Option<&String>) -> bool {
        if name.is_none() {
            return false;
        }

        if let Some(symbol_entry) = self.get_symbol_recursive_all_types(name.unwrap()) {
            symbol_entry.get_storage_location()
                .iter()
                .any(|x| x.is_register())
        }
        else {
            false
        }
    }

    /// get current function which contains this ast item
    pub fn get_current_function(&self) -> Option<&mut FunctionDefinition> {
        if let Some(citem) = self.get_current_node() {
            if citem.is_function_definition() {
                let function_definition = citem.get_function_definition().unwrap();
                return Some(function_definition);
            }
        }

        if let Some(citem) = self.get_parent() {
            citem.get_ast_item().get_current_function()
        }
        else {
            None
        }
    }

    /// get parameter symbol entry by parameter name in current function
    pub fn get_parameter_symbol_entry(&self, param_name:&str) -> Option<&SymbolEntry> {
        self.get_current_function()
            .and_then(|function| {
                function
                    .get_symbol_table()
                    .lookup_by_type(param_name, &Declaration::get_symbol_table_type_declaration())
            })
    }

    /// get scope item's symbol table
    pub fn get_symbol_table(&self) -> Option<&SymbolTable> {
        self.get_scope_item()
            .and_then(|x| Some(x.get_symbol_table()))
    }

    /// get mutable scope item's symbol table
    pub fn get_symbol_table_mut(&mut self) -> Option<&mut SymbolTable> {
        self.get_scope_item()
            .and_then(|x| Some(x.get_symbol_table_mut()))
    }

    /// if the current node is a symbol entry element, which has symbol table or is a declaration
    pub fn is_symbol_entry_element(&self) -> bool {
        if let Some(citem) = self.get_current_node() {
            citem.is_symbol_entry_element()
        }
        else {
            false
        }
    }

    /// if the current node is scope element which has symbol table 
    /// currently it supports CLangProgram, FunctionDefinition, CompoundStatement as scope elements
    pub fn is_scope_element(&self) -> bool {
        if let Some(citem) = self.get_current_node() {
            citem.is_scope()
        }
        else {
            false
        }
    }

    /// is atom item id
    pub fn is_atom_item_id(&self) -> bool {
        if let Some(citem) = self.get_current_node() {
            citem.is_atom_item_id()
        }
        else {
            false
        }
    }

    /// is clang prog item
    pub fn is_clang_program_item(&self) -> bool {
        if let Some(citem) = self.get_current_node() {
            citem.is_clang_prog()
        }
        else {
            false
        }
    }

    /// to atom item mut
    pub fn to_atom_item_mut(&mut self) -> Option<&mut AtomItem> {
        self.get_current_node()
            .and_then(|x| x.get_atom_item())
    }

    pub fn get_code(&self) -> &CodeSnippet {
        &self.code
    }

    pub fn set_code(&mut self, code:CodeSnippet) {
        self.code = code;
    }

    pub fn get_code_mut(&mut self) -> &mut CodeSnippet {
        &mut self.code
    }

    /// get decesant range of sequence ids
    pub fn get_decesant_range(&self) -> Range {
        let current_id = self.get_sequence_id();
        let children = self.get_children();
        let min_id_child = children.iter()
            .min_by_key(|x| x.get_ast_item().get_sequence_id());
        let max_id_child = children.iter()
            .max_by_key(|x| x.get_ast_item().get_sequence_id());
        let r = match (min_id_child, max_id_child) {
            (Some(first), Some(last)) => {
                let first_ast_item = first.get_ast_item();
                let last_ast_item = last.get_ast_item();
                let min = min(current_id, first_ast_item.get_decesant_range().get_start());
                let max = max(current_id, last_ast_item.get_decesant_range().get_end());
                (min, max)
            },
            _ => (current_id, current_id),
        };
 
        r.into()
    }

    /// check if the ast_item is used in addressing mode, e.g., &var
    pub fn used_in_addressing(&self) -> bool {
        if !self.is_atom_item_id() {
            return false;
        }

        let parent = self.get_parent();
        if let Some(pitem) = parent {
            if let Some(expr) = pitem.get_expression() {
                return expr.is_addressing();
            }
        }

        false
    }

    pub fn preorder_tree_mut<'a, F>(&'a mut self, context:&'a mut ASTItemIterationContext, visit:&mut F, child_can_visit:fn(&mut ASTItem) -> bool)
        where F : FnMut(&mut ASTItem, &mut ASTItemIterationContext) -> () 
    {
        visit(self, context);

        let is_visited_root = context.current_scope_element.as_ref().unwrap().get_ast_item_mut() == self;
    
        if is_visited_root || child_can_visit(self) {
            for child in self.get_children_mut().iter_mut() {
                let ast_item = child.get_ast_item_mut();
                ast_item.preorder_tree_mut(context, visit, child_can_visit);
            }
        }
    }

    pub fn preorder_tree<'a, F>(&'a self, context:&'a mut ASTItemIterationContext, visit:&mut F, child_can_visit:fn(&ASTItem) -> bool)
        where F : FnMut(&ASTItem, &mut ASTItemIterationContext) -> () 
    {
        visit(self, context);

        let is_visited_root = context.current_scope_element.as_ref().unwrap().get_ast_item_mut() == self;
    
        if is_visited_root || child_can_visit(self) {
            for child in self.get_children().iter() {
                let ast_item = child.get_ast_item();
                ast_item.preorder_tree(context, visit, child_can_visit);
            }
        }
    }

    pub fn postorder_tree_mut<'a, F>(&'a mut self, context:&'a mut ASTItemIterationContext, visit:&mut F, child_can_visit:fn(&mut ASTItem) -> bool)
        where F : FnMut(&mut ASTItem, &mut ASTItemIterationContext) -> () 
    {
        let is_visited_root = context.current_scope_element.as_ref().unwrap().get_ast_item_mut() == self;
    
        if is_visited_root || child_can_visit(self) {
            for child in self.get_children_mut().iter_mut() {
                let ast_item = child.get_ast_item_mut();
                ast_item.postorder_tree_mut(context, visit, child_can_visit);
            }
        }

        visit(self, context);
    }
    
}

impl Default for ASTItem {
    fn default() -> Self {
        Self { sequence_id: 0, 
            current_node: None,
            common_linker : ParentChildrenLinker::default(), 
            scope_linker : ParentChildrenLinker::default(),
            code : CodeSnippet::new(""),
            source_file_location : SourceFileLocation::default(),
            semantic_linkages : Vec::new(),
            value_changed: false,
            assigned_registers : RegisterSet::default(),
            runtime_type : Box::new(None),
        }
    }
}
