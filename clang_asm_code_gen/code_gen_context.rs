use std::collections::HashMap;

use crate::languages::clang::traits::*;
use crate::{clr_lib::unique_id::UniqueId, languages::clang::{ASTItem, CLangError, clang_item::CLangItem, clang_program::CLangProgram, clang_types::CLangTypes, symbol_table::SymbolTable, traits::display_diagram_trait::StateDiagramDisplay}};
use crate::languages::markdown_lang::to_clr_type::*;
use super::code_gen_configuration::CodeGenConfiguration;
use crate::languages::clang::clang_asm_code_gen::*;
use crate::languages::clang::type_system::*;
use core_utils::debug::*;
use parser_lib::markdown_lang::*;
use core_utils::filesystem::*;

use rust_macro::*;
use rust_macro_internal::state_type_mapping;
use crate::languages::clang::CLangBuildParameters;

pub type ScopeId = u32;

state_type_mapping!("src/languages/clang/type_conversions.mmd", CodeGenContext);

#[derive(Debug, Clone, Accessors)]
pub struct CodeGenContext {
    config : CodeGenConfiguration,
    /// used to append the labels
    id : UniqueId,
    /// should the boolean comparison expression be flip to the opposite operation
    flip_boolean_expr : bool,
    ///available registres
    available_reg_names : RegisterSet,
    /// instruction mapping
    hash: HashMap<String, String>,
    
    /// is the constant expression on left side
    const_expresion_on_left : bool,

    /// true if a nested function call has overwritten a0 (caller return register),
    /// so the current function's first parameter in a0 must be reloaded from stack.
    a0_parameter_clobbered_by_call_return : bool,

    /// tanry has dots following
    tanry_dots_following : bool,

    /// clang_program from #include directive
    included_programs : Vec<CLangProgram>,

    /// base folder for solution
    base_folder : String,

    /// if the current processing is in switch...case statement
    in_switch : bool,

    /// if the current processing is in address scope
    in_address_scope : bool,

    //symbol table section
    symbol_table :SymbolTable,
    unique_id : UniqueId,
    scope_list : Vec<ScopeId>,

    //2nd round data
    iter_result : IterationResult,
    parent_item : Option<CLangItem>,
    
    is_generate_diagram : bool,
    diagram_file_name : String,
    diagram_scope_file_name : String,

    show_semantic_links : bool,   // new field to control semantic links display to mermaid diagram

    symbol_table_md_file_name: Option<String>,

    /// type conversion mapping
    type_conversion_mapping: StringValueTable,

    /// type system generate configuration
    type_config : TypeConfiguration,

    /// generate lib (not exe)
    is_build_to_lib : bool,

    /// code snippet cache for code generation
    code_snippet : CodeSnippet,
}

impl CodeGenContext {
    /// add clang program to the included programs list if not already included
    /// this is to avoid multiple inclusion of the same file
    pub fn add_unique_included_clang_program(&mut self, program:CLangProgram) {
        let path = program.get_source_file_path();
        if let Some(p) = path {
            for existing in &self.included_programs {
                if let Some(existing_path) = existing.get_source_file_path() {
                    if existing_path == p {
                        // already included
                        return;
                    }
                }
            }
        }

        self.included_programs.push(program);
    }

    pub fn get_included_clang_programs_mut(&mut self) -> &mut Vec<CLangProgram> {
        &mut self.included_programs
    }

    pub fn get_code_gen_config(&self) -> &CodeGenConfiguration {
        self.get_config()
    }

    pub fn get_code_gen_config_mut(&mut self) -> &mut CodeGenConfiguration {
        &mut self.config
    }

    pub fn is_flip_boolean_expr(&self) -> bool {
        self.flip_boolean_expr
    }

    /// get the instruction mapping, 
    /// the mapping is from instruction name to code template name, 
    /// which is used to get code template for the instruction
    /// the file is instruction_mapping.md
    pub fn get_instruction_mapping(&self) -> &HashMap<String, String> {
        &self.hash
    }

    pub fn reset_flip_bool(&mut self) {
        self.flip_boolean_expr = false
    }

    pub fn set_const_expression_on_left(&mut self, v:bool) {
        self.const_expresion_on_left = v;
    }

    pub fn reset_const_expression_on_left(&mut self) {
        self.set_const_expression_on_left(false);
    }

    pub fn mark_a0_parameter_clobbered_by_call_return(&mut self) {
        self.a0_parameter_clobbered_by_call_return = true;
    }

    pub fn clear_a0_parameter_clobbered_by_call_return(&mut self) {
        self.a0_parameter_clobbered_by_call_return = false;
    }

    pub fn is_a0_parameter_clobbered_by_call_return(&self) -> bool {
        self.a0_parameter_clobbered_by_call_return
    }

    pub fn set_is_in_switch(&mut self, v:bool) {
        self.in_switch = v
    }

    pub fn reset_is_in_switch(&mut self) {
        self.set_is_in_switch(false);
    }

    //symbol table section
    pub fn attain_unique_id(&mut self) -> u32 {
        self.unique_id.next_id()
    }

    pub fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    pub fn get_is_generarate_diagram(&self) -> bool {
        self.is_generate_diagram
    }

    /// get default asm code generation configuration, it will set replace pseudo code to true
    pub fn get_default_asm_config(&self) -> crate::languages::r5asm::code_gen_config::CodeGenConfiguration {
        let is_build_lib = self.get_is_build_to_lib();
        let mut r = self.get_code_gen_config()
                                                .get_asm_codegen_config()
                                                .clone();
        r.set_replace_pseudo_code(true);
        r.get_linker_config_mut().set_is_build_lib(is_build_lib);
        
        r
    }

    /// get current parameter register number, by default it is 8 (a0-a7)
    pub fn get_parameter_register_number(&self) -> u32 {
        self.get_code_gen_config().get_parameter_register_number()
    }

    /// get code from code generation configuration and incoming code name and debug info, it return Result instead of Option
    pub fn get_code_with_debug(&self, code_name: &str, debug_info:&str, ast_item:&ASTItem) -> Result<CodeSnippet, CLangError> {
        self.get_code_with_debug2(& SnippetKey::default(), code_name, debug_info, ast_item)
    }

    /// get code from code archive with category and code name, debug info as well.
    pub fn get_code_with_debug2(&self, category: &SnippetKey, code_name: &str, debug_info:&str, ast_item:&ASTItem) -> Result<CodeSnippet, CLangError> {
        let key = (category, ast_item).into();
        if ast_item.has_assigned_register() {
            todo!("Need work get_code_with_debug2 for ast_item with assigned register for '{code_name}' + '{category:?}'");
        }
        else {
            let mut r = self.get_code_gen_config()
                .get_code(&key, code_name)
                .ok_or(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find code template '{code_name}' with '{category:?}'")))?;
            
            // set code snippet to the AST item
            ast_item.get_current_node().unwrap()
                    .get_ast_item_mut()
                    .set_code(r.clone());
            
            let debug_info_str = format!("[{}]: {debug_info}", ast_item.get_sequence_id());
            r.set_debug_info(&debug_info_str);
            Ok(r)
        }
    }

    /// get code from code generation configuration and incoming code name, debug info set to empty string
    pub fn get_code(&self, code_name: &str, ast_item:&ASTItem) -> Result<CodeSnippet, CLangError> {
        self.get_code_with_debug(code_name, "", ast_item)
    }

    pub fn get_code2(&self, category: &SnippetKey, code_name: &str, ast_item:&ASTItem) -> Result<CodeSnippet, CLangError> {
        self.get_code_with_debug2(category, code_name, "", ast_item)
    }

    fn get_available_registers(&self) -> Vec<&RegisterUsage> {
        self.available_reg_names.get_registers()
    }

    pub fn get_available_register_as_registerset(&self) -> RegisterSet {
        self.get_available_registers().into_iter()
            .map(|x| x.clone())
            .collect::<Vec<_>>()
            .into()
    }

    pub fn get_available_temp_float_registerset(&self) -> RegisterSet {
        self.get_available_register_as_registerset()
            .get_temp_registers()
            .get_float_registers()
    }

    pub fn get_available_temp_integer_registerset(&self) -> RegisterSet {
        self.get_available_register_as_registerset()
            .get_temp_registers()
            .get_int_registers()
    }

    /// get available registers as register set by type, this happens on symbol type
    /// register usage is only for s registers
    /// the register set is s1 to s11 for integer type. s0 is reserved for frame pointer an won't be used
    pub fn get_available_register_as_registerset_by_type(&self, ty:&CLangTypes) -> RegisterSet {
        let mut regs = self.get_available_register_as_registerset()
                                .get_s_registers();
        regs.remove_regs(&vec!["s0".to_string()]); // remove s0 which is frame pointer
        regs.get_registerset_by_type(ty)
    }

    pub fn is_const_expresion_on_left(&self) -> bool {
        self.const_expresion_on_left
    }

    /// set const expression on left value to new_value, execute the closure f, and then restore the old value
    pub fn with_const_expresion_on_left_value<F, T>(&mut self, new_value: bool, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let old = self.const_expresion_on_left;
        self.const_expresion_on_left = new_value;

        struct Restore<'a> {
            target: &'a mut CodeGenContext,
            old: bool,
        }

        impl<'a> Drop for Restore<'a> {
            fn drop(&mut self) {
                self.target.set_const_expresion_on_left(self.old);
            }
        }

        let restore = Restore { target: self, old };

        f(restore.target)
    }

    pub fn is_tanry_dots_following(&self) -> bool {
        self.tanry_dots_following
    }

    /// set tanry dots following value to new_value, execute the closure f, and then restore the old value
    pub fn with_tanry_dot_following_value<F, T>(&mut self, new_value: bool, f: F) -> T
    where
        F: FnOnce(&mut Self) -> T,
    {
        let old = self.tanry_dots_following;
        self.tanry_dots_following = new_value;

        struct Restore<'a> {
            target: &'a mut CodeGenContext,
            old: bool,
        }

        impl<'a> Drop for Restore<'a> {
            fn drop(&mut self) {
                self.target.set_tanry_dots_following(self.old);
            }
        }

        let restore = Restore { target: self, old };

        f(restore.target)
    }

    /// find value from mapping table with from_type and to_type, return Option<String>
    pub fn find_type_conversion_function_template_name(&self, from_type: &CLangTypes, to_type: &CLangTypes) -> Option<String> {
        let table = &self.type_conversion_mapping.get_table();
        for row in table.iter() {
            match (row.get(0), row.get(1), row.get(2)) {
                (Some(f), Some(t), Some(func_name)) if f == &from_type.to_string() && t == &to_type.to_string() => {
                    return Some(func_name.clone());
                }
                _ => {}
            }
        }

        None
    }

    /// get snippet key and conversion code snippets for assignment between two optional clang types
    pub fn get_snippet_key_and_conversion_code_for_assignment(&self, m:Option<&CLangTypes>, n:Option<&CLangTypes>, ast_item:&ASTItem) 
        -> (SnippetKey, Option<CodeSnippet>, Option<CodeSnippet>) 
    {
        match (m, n) {
            (Some(m_type), Some(n_type)) => {
                if m_type != n_type { 
                    if let Some(r) = self.find_type_conversion_function_template_name(m_type, n_type) {
                        let debug_info = format!("type conversion from {} to {} for assignment", m_type, n_type);
                        
                        match (m_type, n_type) {
                            (CLangTypes::Float64, CLangTypes::Float32) => {
                                let category: SnippetKey = n_type.clone().into();
                                let conv = self.get_code_with_debug2(&category, &r, &debug_info, ast_item).unwrap();
                                (m_type.clone().into(), None, Some(conv))
                            }
                            (CLangTypes::Float32, CLangTypes::Float64) => {
                                let category: SnippetKey = n_type.clone().into();
                                let conv = self.get_code_with_debug2(&category, &r, &debug_info, ast_item).unwrap();
                                (m_type.clone().into(), Some(conv), None)
                            }
                            _ => {
                                todo!("Need work {m_type:?} and {n_type:?} for assignment, add conversion function template in type_conversion_mapping.md")
                            }
                        }
                    }
                    else {
                        warn_string(format!("cannot find '{m_type}' => '{n_type}' conversion function template. So assume the conversion is implicit"));
                        warn_string(format!("the implicit conversion is deemed to be from m = {m_type:?} to n = {n_type:?}"));
                        warn_string(format!("the ast item passed in is {ast_item:?}"));
                        (m_type.clone().into(), None, None)
                    }
                }
                else {
                    (m_type.clone().into(), None, None)
                }                      
            }
            (Some(m_type), None) => {
                (m_type.clone().into(), None, None)
            }
            (None, Some(n_type)) => {
                (n_type.clone().into(), None, None)
            }
            (None, None) => {
                (SnippetKey::default(), None, None)
            }
        }
    }

    /// get snippet key and conversion code snippets for m and n type strings for operation like add, sub, etc.
    pub fn get_snippet_key_and_conversion_code(&self, m:Option<&String>, n:Option<&String>, ast_item:&ASTItem) 
        -> (SnippetKey, Option<CodeSnippet>, Option<CodeSnippet>) 
    {
        match (m, n) {
            (Some(m_str), Some(n_str)) => {
                let m_type : CLangTypes = m_str.into();
                let n_type : CLangTypes = n_str.into();
                if m_type != n_type { 
                    if m_type.can_default_to_int_for_rv5() && n_type.can_default_to_int_for_rv5() {
                        // both can default to int, so use int
                        (m_type.into(), None, None)
                    }
                    else if m_type.can_default_to_int64_for_rv5() && n_type.can_default_to_int64_for_rv5() {
                        // both can default to int64, so use int64
                        (m_type.into(), None, None)
                    }
                    else {
                        if let Some(r) = self.find_type_conversion_function_template_name(&m_type, &n_type) {
                            let debug_info = format!("type conversion from {} to {} for assignment", m_type, n_type);
                            match (&m_type, &n_type) {
                                (CLangTypes::Float64, CLangTypes::Float32) => {
                                    let conv = self.get_code_with_debug2(&n_type.into(), &r, &debug_info, ast_item).unwrap();
                                    (m_type.into(), None, Some(conv))
                                }
                                (CLangTypes::Float32, CLangTypes::Float64) => {
                                    let conv = self.get_code_with_debug2(&(&m_type).into(), &r, &debug_info, ast_item).unwrap();
                                    (m_type.into(), Some(conv), None)
                                }
                                _ => {
                                    todo!("Need work {m_type:?} and {n_type:?} for assignment, add conversion function template in type_conversion_mapping.md")
                                }
                            }
                        }
                        else {
                            warn_string(format!("no implicit conversion from m = {m_type:?} to n = {n_type:?}"));
                            (m_type.into(), None, None)
                        }
                    }
                }
                else {
                    (m_type.into(), None, None)
                }                      
            }
            (Some(m_str), None) => {
                let m_type : CLangTypes = m_str.into();
                (m_type.into(), None, None)
            }
            (None, Some(n_str)) => {
                let n_type : CLangTypes = n_str.into();
                (n_type.into(), None, None)
            }
            (None, None) => {
                (SnippetKey::default(), None, None)
            }
        }
    }

    pub fn append_conversion_code(
        &mut self, 
        original_code: &mut CodeSnippet, 
        conversion_code: Option<CodeSnippet>
    ) -> Result<(), CLangError> {
        if let Some(mut conv_code) = conversion_code {
            conv_code.get_input_register_from(0, original_code, 0, self)?;
            (*original_code) += conv_code;
        }

        Ok(())
    }

    /// add current item to the visited list and set the sequence id
    pub fn access_clang_item(&mut self, item: CLangItem) {
        let ast_item = item.get_ast_item_mut();
        ast_item.set_sequence_id(self.attain_unique_id() as usize);
        
        // keep the all the visited nodes
        self.iter_result.nodes_visited.push(item.clone());

        // keep the temp visited nodes
        self.iter_result.temp_visited_nodes.push(item.clone());
    }

    /// create a move register code snippet from 'from' register 'to' to register
    pub fn new_move_register_snippet(&self, category:&SnippetKey, ast_item:&ASTItem, from:&str, to:&str) -> Result<CodeSnippet, CLangError> {
        let from_reg : RegisterUsage = from.into();
        let to_reg : RegisterUsage = to.into();
        let mut move_code = self.get_code_with_debug2(category, "move_register", 
                                    &format!("move register '{}/{}' to '{}/{}'", 
                                                    from_reg.get_reg_alias(), from_reg.get_reg_name(),
                                                    to_reg.get_reg_alias(), to_reg.get_reg_name()),
                                    ast_item)?;
        let (input_reg, output_reg) = (move_code.get_input_register_names()[0].to_string(), move_code.get_output_register_names()[0].to_string());

        // Guard overlapping replacements, e.g. template `mv t1, t0` with from=a0/to=t0.
        // A naive sequential replace can collapse into `mv a0, a0`.
        let input_unique = RegisterUsage::new(&input_reg).get_reg_name().to_string();
        let output_unique = RegisterUsage::new(&output_reg).get_reg_name().to_string();
        let from_unique = RegisterUsage::new(from).get_reg_name().to_string();
        let to_unique = RegisterUsage::new(to).get_reg_name().to_string();
        let need_guard = input_unique == to_unique || output_unique == from_unique;

        if need_guard {
            let unavailable_unique = vec![input_unique, output_unique, from_unique, to_unique];
            let guard_reg = self.get_available_register_as_registerset()
                .get_temp_registers()
                .get_register_alias_name()
                .into_iter()
                .find(|r| {
                    let unique = RegisterUsage::new(r).get_reg_name().to_string();
                    !unavailable_unique.contains(&unique)
                })
                .ok_or_else(|| CLangError::cannot_emit("cannot find guard temp register for move register replacement"))?;

            move_code.replace_register(&input_reg, &guard_reg);
            move_code.replace_register(&output_reg, to);
            move_code.replace_register(&guard_reg, from);
        }
        else {
            move_code.replace_register(&output_reg, to);
            move_code.replace_register(&input_reg, from);
        }

        Ok(move_code)
    }

    /// write scope diagram file, it shows the scope linkages in AST
    fn write_scope_diagram_file(&self, clang_program:&CLangProgram) -> Result<(), CLangError> {
        if self.get_is_generarate_diagram() {
            let diagram = clang_program.to_scope_link_diagram();
            let diagram_file_name = self.get_diagram_scope_file_name();
            write_to_file_option((!diagram_file_name.is_empty()).then_some(&diagram_file_name), &diagram).unwrap();
        }

        Ok(())
    }

    /// write AST diagram file
    fn write_ast_diagram_file(&self, clang_program:&mut CLangProgram) -> Result<(), CLangError> {
        if self.get_is_generarate_diagram() {
            let diagram = clang_program.to_state_diagram(self.get_show_semantic_links());
            let diagram_file_name = self.get_diagram_file_name();
            write_to_file_option((!diagram_file_name.is_empty()).then_some(&diagram_file_name), &diagram).unwrap();
        }

        Ok(())
    }

    /// write diagram files
    pub fn write_diagram_file(&self, clang_program:&mut CLangProgram) -> Result<(), CLangError> {
        let _ = self.write_scope_diagram_file(clang_program);
        let _ = self.write_ast_diagram_file(clang_program);

        Ok(())
    }

    /// write symbol table markdown file
    pub fn write_symbol_table_md_file(&self, r:&mut CLangProgram) {
        // Always clear previous output first to avoid stale symbol tables.
        delete_file_option(self.get_symbol_table_md_file_name()).unwrap();

        if let Ok(symbol_tables) = r.get_all_symbol_tables() {
            for symbol_table in symbol_tables.iter() {
                let content = format!("{}\n\n", symbol_table.to_markdown());
                append_to_file_option(self.get_symbol_table_md_file_name(), &content).unwrap();
            }
        }
        else {
            error_str("Failed to get all symbol tables");
        }
    }

    fn from_clang_build_parameters(para:&CLangBuildParameters) -> Self {
        let mut r = Self::default();
        r.set_base_folder(get_file_containing_folder(&para.full_path).unwrap());
        r.set_is_build_to_lib(para.is_build_to_lib);
        let stack_space_size = para
            .stack_space_size
            .as_ref()
            .and_then(|x| x.parse::<u32>().ok());
        r.get_code_gen_config_mut().set_stack_space_size(stack_space_size);
        r
    }
}

impl Default for CodeGenContext {
    fn default() -> Self {
        let regs = RegisterSet::get_riscv_available_register_names();
        let mut r = Self { 
            config: CodeGenConfiguration::default(),
            in_address_scope : false, 
            id : UniqueId::default(),
            base_folder : format!("."),
            symbol_table_md_file_name: Some("clang_symbol_table.md".to_string()),
            flip_boolean_expr : false,
            available_reg_names : RegisterSet::new_from_string_vec(& regs),
            const_expresion_on_left : false,
            a0_parameter_clobbered_by_call_return : false,
            tanry_dots_following : false,
            included_programs : Vec::new(),
            hash : HashMap::default(),
            in_switch : false,
            symbol_table : SymbolTable::default(), 
            unique_id : UniqueId::default(),
            scope_list : Vec::default(),
            iter_result : IterationResult::default(),
            is_generate_diagram : false,
            diagram_file_name : "diagram.mermaid".to_string(),
            diagram_scope_file_name : "scope_diagram.mermaid".to_string(),
            show_semantic_links : false,
            parent_item : None,
            type_conversion_mapping: StringValueTable::default(),
            type_config : TypeConfiguration::default(),
            is_build_to_lib : false,
            code_snippet : CodeSnippet::default(),
        };

        let type_mapping_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src/languages/clang/clang_asm_code_gen/type_conversion_mapping.md");
        let string_table = md_table_to_string_row_table(type_mapping_path.to_str().unwrap()).unwrap();
        r.type_conversion_mapping = string_table;

        let instruction_mapping_content = include_str!("instruction_mapping.md");
        r.hash = md_table_to_hash2(instruction_mapping_content).unwrap();
        r.reset_flip_bool();
        r.reset_const_expression_on_left();
        r.reset_is_in_switch();
        r
    }
}

#[derive(Debug, Clone)]
pub (crate) struct IterationResult {
    pub nodes_visited : Vec<CLangItem>,

    pub temp_visited_nodes : Vec<CLangItem>,
}

impl Default for IterationResult {
    fn default() -> Self {
        Self {
            nodes_visited : Vec::default(),
            temp_visited_nodes : Vec::default(),
        }
    }
}