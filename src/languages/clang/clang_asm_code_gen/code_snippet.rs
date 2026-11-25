use core::panic;
use std::ops::{AddAssign, BitAndAssign};
use std::fmt::Debug;

use parser_lib::markdown_lang::*;
use crate::languages::{clang::{ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError}, data_lang::data_lang_pest::{ListWithId, parse_list_with_id}, r5asm::{asm_solution::{ASMSolution, FileObject}, r5asm_pest::SectionItem, section::Section}};
use core_utils::debug::*;
use core_utils::traits::generate_code::GenerateCode;
use self::markdown_pest::File;

use super::*;

const IMM_TO_REPLACE : &str = "144";
const LABLE_REPLACEMENT : &str = "labelx";

static mut UNIQUE_ID : usize = 0;

#[derive(Clone, PartialEq, Eq)]
pub struct CodeSnippet {
    name : String,
    code : Vec<CodeSnippetDataItem>,
    data: Vec<CodeSnippetDataItem>,

    /// these register names are output of the code snippet, it's like function return value
    output_register_names : Vec<String>,

    /// these label names are output of the code snippet, it's like function return value
    output_labels : Vec<String>,

    /// these register names are required to be given by upstream code snippet, it's like function parameter
    required_input_register_names : Vec<String>,

    /// required register names used for temp calcuation, cannot be conflicted with input or output register names, and also should be resolved when resolving register conflict
    required_temporary_register_names : Vec<String>,

    /// these label names are required to be given by upstream code snippet, it's like function parameter
    required_output_labels : Vec<String>,

    /// tag to store debug info
    debug_info : String,

    /// is unique id has been added
    is_unique_id_added : bool,

    /// register names which are impacted by the code snippet, this is used to resolve register conflict
    /// impacted registers are register value is unknown and should not be used in other snippet as input register
    impacted_regs : Vec::<String>,

    /// register names which are fixed by the code snippet, and it should not be used in resolving register conflict
    fixed_regs : Vec::<String>,

    /// tags to provide additional information for code snippet
    tags : Vec<String>,
}

impl CodeSnippet {
    
    /// new instance with name
    pub fn new(name:&str) -> CodeSnippet {
        CodeSnippet {
            name: name.to_string(),
            code: Vec::new(),
            data: Vec::new(),
            output_register_names: Vec::new(),
            output_labels: Vec::new(),
            required_input_register_names: Vec::new(),
            required_temporary_register_names: Vec::new(),
            required_output_labels: Vec::new(),
            debug_info: String::new(),
            is_unique_id_added: false,
            impacted_regs : Vec::new(),
            fixed_regs : Vec::new(),
            tags: Vec::new(),
        }
    }

    pub fn new_with_debug_info(name:&str, debug_info:&str) -> CodeSnippet {
        let mut r = CodeSnippet::new(name);
        r.set_debug_info(debug_info);
        r
    }

    /// convert to code snippet and data snippet tuple
    pub fn to_code_data_snippet_tuple(&self) -> (CodeSnippet, CodeSnippet) {
        let mut code_snippet = CodeSnippet::new(&self.get_name());
        for item in self.get_code_items() {
            code_snippet.add_code(item.clone());
        }

        let mut data_snippet = CodeSnippet::new(&format!("{}_data", self.get_name()));
        for item in self.get_data().iter() {
            data_snippet.add_data(item.clone());
        }

        (code_snippet, data_snippet)
    }

    pub fn get_unique_id() -> usize {
        let r;
        unsafe {
            r = UNIQUE_ID;
            UNIQUE_ID = UNIQUE_ID + 1;
        }
        r 
    }

    pub fn set_unique_id(id:usize) {
        unsafe {
            UNIQUE_ID = id;
        }
    }

    pub fn reset_unique_id() {
        Self::set_unique_id(0);
    }

    /// new instsance data with list of SectionItem
    pub fn new_with_data(data:Vec<SectionItem>) -> CodeSnippet {
        let mut r = CodeSnippet::new("data");
        r.data = data.into_iter().map(|x| x.into()).collect();
        r
    }

    /// new istance with MD data
    pub fn new_with_md(var_name:&str, md:&str) -> CodeSnippet {
        let mut r = CodeSnippet::new("md_data");
        r.data.push(CodeSnippetDataItem::Markdown(MarkdownData::new(md.to_owned(), var_name.to_string())));
        r
    }

    /// add a new code snippet data item in the list
    pub fn add_data(&mut self, data:CodeSnippetDataItem) {
        self.data.push(data);
    }

    /// get the data list from this object only
    pub fn get_data(&self) -> &Vec<CodeSnippetDataItem> {
        &self.data
    }

    /// get markdown data recursively from self and inside snippets
    pub fn get_markdown_data_recursive(&self) -> Vec<MarkdownData> {
        let mut r = Vec::new();

        for item in self.get_data().iter() {
            match item {
                CodeSnippetDataItem::Markdown(md) => {
                    r.push(md.clone());
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    r.extend(snippet.get_markdown_data_recursive());
                }
                _ => {}
            }
        }

        for item in self.get_code_items().iter() {
            match item {
                CodeSnippetDataItem::Markdown(md) => {
                    r.push(md.clone());
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    r.extend(snippet.get_markdown_data_recursive());
                }
                _ => {}
            }
        }

        r
    }

    /// get data list mut
    pub fn get_data_mut(&mut self) -> &mut Vec<CodeSnippetDataItem> {
        &mut self.data
    }

    /// find a code item which is ASMCode and append the code to it
    pub fn add_code(&mut self, code: CodeSnippetDataItem) {
        match code {
            CodeSnippetDataItem::Inc(_) => {
                self.code.push(code);
            }
            CodeSnippetDataItem::Snippet(snippet) => {
                self.add_snippet(snippet);
            }
            CodeSnippetDataItem::Markdown(_) => {
                panic!("Cannot add markdown as code")
            }
        }
    }

    /// add inc item in the current (self) code snippet
    pub fn add_asm_inc(&mut self, inc_name:&str, args:Vec<Option<&str>>) -> Result<(), CLangError> {
        // join args with comma if it is not None
        let args = args.iter().map(|x| x.map(|y| y.to_string()).unwrap_or(String::new())).collect::<Vec<_>>().join(", ");
        let inc_str = format!("{} {}", inc_name, args);

        let mut asm_config = crate::languages::r5asm::code_gen_config::CodeGenConfiguration::default();
        asm_config.reset_generate_bin_and_code();
        asm_config.set_replace_pseudo_code(false);

        if let Ok(inc) = crate::languages::r5asm::instruction::Instruction::from_string(&inc_str, &mut asm_config) {
            for item in inc {
                self.code.push(SectionItem::Instruction(item).into());
            }
    
            Ok(())
        }
        else {
            let err = format!("cannot parse inc: '{}'", inc_str);
            error_string(err);
            Err(CLangError::ParsingError(file!().to_string(), line!() as usize))
        }
    }

    /// create code snippet from asm code string
    /// please note: this does not set the register info such as input/output registers
    pub fn from_asm_code_str(name:&str, code:&str) -> Result<CodeSnippet, CLangError> {
        let (section_text, section_data) = Self::get_section_from_str(name, code)?;
        let mut snippet = CodeSnippet::new(name);
        for item in section_text.get_section_items() {
            snippet.code.push(item.into());
        }

        for item in section_data.get_section_items() {
            snippet.data.push(item.into());
        }

        Ok(snippet)
    }

    /// get code list length
    pub fn get_code_len(&self) -> usize {
        self.code.len()
    }

    /// get code list which i section item (instruction or label)
    pub fn get_code(&self) -> Vec<&SectionItem> {
        let r = self.code.iter().map(|x| x.get_inc()).flatten().collect::<Vec<_>>();
        r
    }

    /// replicate the output instruction code n times, mainly used in dereference operation
    pub fn replicate_output_code(&mut self, n:u8, context:&mut CodeGenContext, snippet_key:&SnippetKey, ast_item:&ASTItem) {
        let output_regs = self.get_output_register_names().clone();
        assert!(output_regs.len() == 1, "only single output register is supported in replicate_output_code");

        let output_reg = &output_regs[0];
        for _ in 1..n {
            let mut output_code = context.get_code_with_debug2(snippet_key.into(), "load_address_to_reg", "deference", ast_item).unwrap();
            output_code.set_output_register(&output_reg);
            output_code.set_input_register(&output_reg);
            self.add_snippet(output_code);
        }
    }

    /// get code snippet data item list, the item can be inc or snippet or markdown
    pub fn get_code_items(&self) -> Vec<&CodeSnippetDataItem> {
        self.code.iter().collect()
    }

    pub fn get_code_mut(&mut self) -> &mut Vec<CodeSnippetDataItem> {
        self.code.as_mut()
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn set_name(&mut self, name:&str) {
        self.name = name.to_string();
    }

    pub fn get_input_register_names(&self) -> &Vec<String> {
        &self.required_input_register_names
    }

    pub fn get_input_register_as_registerset(&self) -> RegisterSet {
        self.get_input_register_names().into()
    }

    pub fn set_input_register_names(&mut self, input_register_names:Vec<String>) {
        if input_register_names.iter().all(|x| is_register_name(x)) {
            self.required_input_register_names = input_register_names;
        }
        else {
            panic!("input register names must be valid register names");
        }
    }

    /// set single input register name, if the code snippet is empty,
    /// then add mv instruction to move new input register to old input register
    pub fn set_input_register(&mut self, input_register_name:&str) {
        assert!(self.get_input_register_names().len() == 1, "only single input register can be set by this method");
        let old_name = self.get_input_register_names()[0].to_string();

        if self.is_empty() {
            let old_alias = RegisterUsage::new(&old_name).get_reg_alias().to_string();
            let input_alias = RegisterUsage::new(input_register_name).get_reg_alias().to_string();
            let _ = self.add_asm_inc("mv", vec![Some(&old_alias), Some(&input_alias)]);
            self.set_input_register_names(vec![input_alias]);
        }
        else {
            self.replace_register(&old_name, input_register_name);
        }
    }

    /// set single output register name, if the code snippet is empty, 
    /// then add mv instruction to move old output register to new output register
    pub fn set_output_register(&mut self, output_register_name:&str) {
        assert!(self.get_output_register_names().len() == 1, "only single output register can be set by this method");
        let old_name = self.get_output_register_names()[0].to_string();

        if self.is_empty() {
            let output_alias = RegisterUsage::new(output_register_name).get_reg_alias().to_string();
            let old_alias = RegisterUsage::new(&old_name).get_reg_alias().to_string();
            let _ = self.add_asm_inc("mv", vec![Some(&output_alias), Some(&old_alias)]);
            self.set_output_register_names(vec![output_alias]);
        }
        else {
            self.replace_register(&old_name, output_register_name);
        }
    }

    /// set output register names directly
    pub fn set_output_register_names(&mut self, output_register_names:Vec<String>) {
        if output_register_names.iter().all(|x| is_register_name(x)) {
            self.output_register_names = output_register_names;
        }
        else {
            panic!("output register names must be valid register names");
        }
    }

    pub fn get_output_register_names(&self) -> &Vec<String> {
        &self.output_register_names
    }

    pub fn get_output_labels(&self) -> &Vec<String> {
        &self.output_labels
    }

    pub fn set_output_labels(&mut self, output_labels:Vec<String>) {
        self.output_labels = output_labels;
    }

    pub fn get_debug_info(&self) -> &str {
        &self.debug_info
    }

    /// set debug info, this is a tag to provide additional information for the code snippet, and it does not affect the code generation result, but can be used for debugging and analysis
    pub fn set_debug_info(&mut self, debug_info:&str) {
        self.debug_info = debug_info.to_string();
    }

    pub fn get_temp_register_names(&self) -> &Vec<String> {
        &self.required_temporary_register_names
    }

    pub fn get_temp_register_as_registerset(&self) -> RegisterSet {
        self.get_temp_register_names().into()
    }

    pub fn set_temp_register_names(&mut self, temp_register_names:Vec<String>) {
        if temp_register_names.iter().all(|x| is_register_name(x)) {
            self.required_temporary_register_names = temp_register_names;
        }
        else {
            panic!("temporary register names must be valid register names");
        }
    }

    /// get output register as register set
    pub fn get_output_register_as_registerset(&self) -> RegisterSet {
        let r = self.output_register_names.to_owned().into();
        r 
    }

    /// get contaminated registers from the code snippet: the output register, temp registers, and impacted registers,
    /// and also the contaminated registers from inside snippets
    pub fn get_contaminated_registers(&self) -> RegisterSet {
        let mut r = self.get_output_register_as_registerset();
        r += RegisterSet::new_from_string_vec(self.get_impacted_regs());
        r += RegisterSet::new_from_string_vec(self.get_temp_register_names());

        // add contaminated registers from inside snippets
        for item in self.get_code_items().iter() {
            if let CodeSnippetDataItem::Snippet(snippet) = item {
                r += snippet.get_contaminated_registers();
            }
        }

        r
    }

    /// get all registers used in the code snippet, including input, output, temp, and impacted registers
    pub fn get_all_registers_defined(&self) -> RegisterSet {
        let mut r = self.get_contaminated_registers();
        r += self.get_input_register_as_registerset();
        r
    }

    /// get all label in the code snippet
    pub fn get_all_labels(&self) -> Vec<String> {
        let mut r = Vec::new();
        for item in self.get_code_items() {
            if let Some(lable) = item.get_inc().and_then(|x| x.get_label()) {
                r.push(lable.to_string());
            }
        }

        r
    }

    pub fn set_impacted_regs(&mut self, impacted_regs:Vec<String>) {
        self.impacted_regs = impacted_regs;
    }

    /// gets self.impacted_regs
    pub fn get_impacted_regs(&self) -> &Vec<String> {
        &self.impacted_regs
    }

    /// get impacted regs as register set, the impacted reg is defined in the code snippet directly
    pub fn get_impacted_regs_as_registerset(&self) -> RegisterSet {
        RegisterSet::new_from_string_vec(&self.impacted_regs)
    }

    /// add other to self.code and set impacted reg as self.impacted_reg + other.contaminated_reg.
    pub fn add_snippet(&mut self, other:CodeSnippet) {
        let impacted_regs = (self.get_impacted_regs_as_registerset() + other.get_contaminated_registers()).get_register_alias_name();  //impacted registers are the output registers and the impacted registers
        self.set_impacted_regs(impacted_regs);
        self.code.push(other.into());
    }

    /// resolve register conflict. other's input from self's output, and other's output is also self's output.
    /// then append other to self.code. Assuming input and output number are 1. 
    pub fn append_snippet(&mut self, mut other:CodeSnippet, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let fixed_regs = other.get_input_register_as_registerset(); 

        // other's input is from self, so fixed_regs is used to skip register move.
        // other's input will be overridden by self's output anyway, so no need to move away.
        other.move_register_from(self.get_contaminated_registers(), fixed_regs, context)?;
        other.get_input_register_from(0, self, 0, context)?;
        other.set_output_register(self.get_output_register_names().first().unwrap());
        self.add_snippet(other);

        Ok(())
    }

    /// resolve register conflict. other's input <- self's output, and keep other's output as final output.
    /// then append other to self.code. Assuming input and output number are 1.
    pub fn append_snippet_use_other_output(&mut self, mut other:CodeSnippet, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let fixed_regs = other.get_input_register_as_registerset(); 

        // other's input is from self, so fixed_regs is used to skip register move.
        // other's input will be overridden by self's output anyway, so no need to move away.
        other.move_register_from(self.get_contaminated_registers(), fixed_regs, context)?;
        other.get_input_register_from(0, self, 0, context)?;
        // Keep downstream output register(s) so callers can consume the latest value producer.
        self.set_output_register_names(other.get_output_register_names().to_owned());
        self.add_snippet(other);

        Ok(())
    }

    pub fn add_data_snippet(&mut self, other:CodeSnippet) {
        let impacted_regs = (self.get_impacted_regs_as_registerset() + other.get_contaminated_registers()).get_register_alias_name();  //impacted registers are the output registers and the impacted registers
        self.set_impacted_regs(impacted_regs);
        self.data.push(other.into());
    }

    /// merge the output labels from other code snippet
    pub fn merge_output_labels(&mut self, other:&CodeSnippet) {
        let mut labels = self.get_output_labels().to_vec();
        for label in other.get_output_labels() {
            if !labels.contains(label) {
                labels.push(label.to_string());
            }
        }
        self.set_output_labels(labels);
    }

    /// get fixed register as register set, 
    /// the fixed register is defined in the code snippet directly, 
    /// and it should not be used in resolving register conflict, 
    /// because the code snippet will not move it away when conflict happens, 
    /// and the register value is expected to be preserved across different code snippets
    pub fn get_fixed_reg_as_regset(&self) -> RegisterSet {
        // x0/zero is architectural constant and must never be renamed.
        let x0_regset = "x0".into();
        RegisterSet::new_from_string_vec(&self.fixed_regs) + x0_regset
    }

    pub fn move_temp_register(&mut self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let (int_regs, float_regs) = self.get_temp_register_as_registerset().get_int_float_registers();
        if int_regs.len() > 0 {
            self.move_int_temp_register_from(int_regs, context)?;
        }

        if float_regs.len() > 0 {
            self.move_float_temp_register_from(float_regs, context)?;
        }

        Ok(())
    }

    fn move_float_temp_register_from(&mut self, regs:RegisterSet, context:&mut CodeGenContext) -> Result<(), CLangError> {
        assert!(regs.is_float_register_set());

        let available_regset = context.get_available_temp_float_registerset();
        let input_regs = self.get_input_register_as_registerset().get_float_registers();
        let out_regs = self.get_output_register_as_registerset().get_float_registers();
        let fixed_regs = self.get_fixed_reg_as_regset().get_float_registers();

        let temp_regs = self.get_temp_register_as_registerset().get_float_registers();
        if temp_regs.has_conflict(&input_regs) || temp_regs.has_conflict(&out_regs) || temp_regs.has_conflict(&fixed_regs) {
            let available_temp = available_regset - input_regs - out_regs - fixed_regs;
            if available_temp.len() < temp_regs.len() {
                return Err(CLangError::RegisterConflictResolveErr("Not enough available temporary registers to move the conflicting temporary registers".to_string()));
            }

            let replacement = temp_regs.zip_with(&available_temp);
            self.replace_register_with(replacement);
        }

        Ok(())
    }

    fn move_int_temp_register_from(&mut self, regs:RegisterSet, context:&mut CodeGenContext) -> Result<(), CLangError> {
        assert!(regs.is_integer_register_set());

        let available_regset = context.get_available_temp_integer_registerset();
        let input_regs = self.get_input_register_as_registerset().get_int_registers();
        let out_regs = self.get_output_register_as_registerset().get_int_registers();
        let fixed_regs = self.get_fixed_reg_as_regset().get_int_registers();

        let temp_regs = self.get_temp_register_as_registerset().get_int_registers();
        if temp_regs.has_conflict(&input_regs) || temp_regs.has_conflict(&out_regs) || temp_regs.has_conflict(&fixed_regs) {
            let available_temp = available_regset - input_regs - out_regs - fixed_regs;
            if available_temp.len() < temp_regs.len() {
                return Err(CLangError::RegisterConflictResolveErr("Not enough available temporary registers to move the conflicting temporary registers".to_string()));
            }

            let replacement = temp_regs.zip_with(&available_temp);
            self.replace_register_with(replacement);
        }

        Ok(())
    }

    /// check if the code snippet is a single instruction, 
    /// which means it has only one instruction and no other code snippets or markdowns
    pub fn is_single_instruction_code_snippet(&self) -> bool {
        let len = self.get_code().iter()
            .filter(|x| x.is_inc() || x.is_pseodu_inc())
            .count();

        len == 1
    }

    fn respect_fixed_regs(&self, _other_regs:&RegisterSet, fixed_regs:&RegisterSet) -> bool {
        let is_single_instruction = self.is_single_instruction_code_snippet();
        let has_same_defined_regs = self.get_all_registers_defined() == *fixed_regs;
        is_single_instruction || has_same_defined_regs
    }

    /// move registers from regs (RegisterSet) to avoid conflict, 
    /// regs are the registers used by other code snippets and split into int and float registers
    /// fixed_regs are caller-selected registers that must not be moved during conflict resolution.
    pub fn move_register_from(&mut self, regs:RegisterSet, fixed_regs:RegisterSet, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let respect_fixed_regs = self.respect_fixed_regs(&regs, &fixed_regs);
        let fixed_regs = if respect_fixed_regs { fixed_regs } 
                                    else { [].into() };
        
        let (int_regs, float_regs) = regs.get_int_float_registers();
        if int_regs.len() > 0 {
            self.move_int_register_from(int_regs, fixed_regs.clone(), context)?;
        }

        if float_regs.len() > 0 {
            self.move_float_register_from(float_regs, fixed_regs.clone(), context)?;
        }

        Ok(())
    }

    /// move float registers from regs (RegisterSet) to avoid conflict
    /// logic is like the one in move_int_register_from
    fn move_float_register_from(&mut self, regs:RegisterSet, fixed_regs:RegisterSet, context:&mut CodeGenContext) -> Result<(), CLangError> {
        assert!(regs.is_float_register_set());

        let available_regset = context.get_available_temp_float_registerset();
        let self_regset = self.get_all_registers().get_float_registers();
        let fixed_regs = self.get_fixed_reg_as_regset().get_float_registers() + fixed_regs.get_float_registers();
        let protected_regs = self.get_output_register_as_registerset().get_float_registers();
        let (replacements, reg_to_push) = self_regset.move_away_from(&regs, &available_regset, &fixed_regs, &protected_regs);

        for n in replacements.iter() {
            self.perform_registerset_action(n);
        }

        let output_reg_aliases = self.get_output_register_as_registerset().get_register_alias_name();

        // if reg_to_push is not empty, then add push and pop instructions at beginning and end of the code snippet
        if !reg_to_push.is_empty() {
            let reg_names_str = reg_to_push.to_alias_vec().join(", ");
            let replacement_str = replacements.iter()
                                    .map(|n| format!("{}=>{}", n.old_reg, n.new_reg))
                                    .collect::<Vec<_>>()
                                    .join(", ");
            let mut push_code = CodeSnippet::new_with_debug_info("push float registers", format!("push float registers to avoid conflict: [{reg_names_str}] => [{replacement_str}]").as_str());
            let mut pop_code = CodeSnippet::new_with_debug_info("pop float registers", format!("pop float registers to avoid conflict: [{reg_names_str}] => [{replacement_str}]").as_str());
            let mut regs_to_restore = Vec::new();
            for reg in reg_to_push.get_register_alias_name() {
                let mapped_reg = replacements.iter()
                    .filter(|x| x.is_replace_action())
                    .find_map(|n| {
                        let (old, new) = n.to_tuple();
                        let old_usage = RegisterUsage::new(&old);
                        let reg_usage = reg.as_str().into();
                        if old_usage == reg_usage { Some(RegisterUsage::new(&new).get_reg_alias().to_string())}
                        else { None }
                    })
                    .unwrap_or_else(|| reg.clone());

                let reg_to_save = if output_reg_aliases.contains(&mapped_reg) {
                    reg.clone()
                }
                else {
                    mapped_reg
                };

                assert!(
                    !output_reg_aliases.contains(&reg_to_save),
                    "destination/output register must not be part of conflict save/restore set. output={output_reg_aliases:?}, spill={reg_to_save}, original={reg}, replacements={replacements:?}"
                );

                push_code.add_asm_inc("pushf64", vec![Some(&reg_to_save)])?;
                regs_to_restore.push(reg_to_save);
            }

            // Stack restore must be reverse of save order (LIFO).
            for reg in regs_to_restore.iter().rev() {
                pop_code.add_asm_inc("popf64", vec![Some(reg.as_str())])?;
            }

            self.insert_at_beginning(vec![push_code]);
            self.add_snippet(pop_code);
        }

        Ok(())
    }

    /// move self's registers from regs (RegisterSet) to avoid conflict including input and output names
    /// make sure self is going to be put after the other code snippets. The other code snippets generate the output registers
    /// the rule here is simple, first come first serve. The later comer will surrender the register to the previous one
    /// this first and previous is logic sequence, whichever take the register first, will be the one ask other to move_away
    fn move_int_register_from(&mut self, regs:RegisterSet, fixed_regs:RegisterSet, context:&mut CodeGenContext) -> Result<(), CLangError> {
        assert!(regs.is_integer_register_set());

        let available_regset = context.get_available_temp_integer_registerset();
        let self_regset = self.get_all_registers().get_int_registers();
        let fixed_regs = self.get_fixed_reg_as_regset().get_int_registers() + fixed_regs.get_int_registers();
        let protected_regs = self.get_output_register_as_registerset().get_int_registers();
        let (replacements, reg_to_push) = self_regset.move_away_from(&regs, &available_regset, &fixed_regs, &protected_regs);

        for n in replacements.iter() {
            self.perform_registerset_action(n);
        }

        let output_reg_aliases = self.get_output_register_as_registerset().get_register_alias_name();

        // if reg_to_push is not empty, then add push and pop instructions at beginning and end of the code snippet
        if !reg_to_push.is_empty() {
            let reg_names_str = reg_to_push.to_alias_vec().join(", ");
            let replacement_str = replacements.iter()
                                            .map(|n| format!("{}=>{}", n.old_reg, n.new_reg))
                                            .collect::<Vec<_>>()
                                            .join(", ");
            let mut push_code = CodeSnippet::new_with_debug_info("push int registers", format!("push int registers to avoid conflict: [{reg_names_str}] => [{replacement_str}]").as_str());
            let mut pop_code = CodeSnippet::new_with_debug_info("pop int registers", format!("pop int registers to avoid conflict: [{reg_names_str}] => [{replacement_str}]").as_str());
            let mut regs_to_restore = Vec::new();
            for reg in reg_to_push.get_register_alias_name() {
                let mapped_reg = replacements.iter()
                    .filter(|x| x.is_replace_action())
                    .find_map(|n| {
                        let (old, new) = n.to_tuple();
                        let old_usage = RegisterUsage::new(&old);
                        let reg_usage = reg.as_str().into();
                        if old_usage == reg_usage { Some(RegisterUsage::new(&new).get_reg_alias().to_string())}
                        else { None }
                    })
                    .unwrap_or_else(|| reg.clone());

                let reg_to_save = if output_reg_aliases.contains(&mapped_reg) {
                    reg.clone()
                }
                else {
                    mapped_reg
                };

                assert!(
                    !output_reg_aliases.contains(&reg_to_save),
                    "destination/output register must not be part of conflict save/restore set. output={output_reg_aliases:?}, spill={reg_to_save}, original={reg}, replacements={replacements:?}"
                );

                push_code.add_asm_inc("push64", vec![Some(&reg_to_save)])?;
                regs_to_restore.push(reg_to_save);
            }

            // Stack restore must be reverse of save order (LIFO).
            for reg in regs_to_restore.iter().rev() {
                pop_code.add_asm_inc("pop64", vec![Some(reg.as_str())])?;
            }

            self.insert_at_beginning(vec![push_code]);
            self.add_snippet(pop_code);
        }

        Ok(())
    }

    /// make the output same as the other snippet
    pub fn make_output_same_as(&mut self, other:&CodeSnippet) -> Result<(), CLangError> {
        let other_output_registers = other.get_output_register_as_registerset();
        let self_ouput_registers = self.get_output_register_as_registerset();

        assert!(self_ouput_registers.len() == other_output_registers.len());

        // replace reg name in self code snippet, the other_different is the target reigster name
        let self_difference = self_ouput_registers.clone() - other_output_registers.clone();
        let other_difference = other_output_registers.clone() - self_ouput_registers.clone();

        let replacements = self_difference.get_register_alias_name().into_iter()
                                                .zip(other_difference.get_register_alias_name().into_iter())
                                                .collect::<Vec<_>>();

        if other_difference.has_conflict(&self.get_all_registers()) { //the conflict means the target one or more target registers are currently being used in self snippet 
            // use mv instruction to move current output to new register
            let mut mv_code = CodeSnippet::new("mv registers");
            for (old, new) in replacements {
                mv_code.add_asm_inc("mv", vec![Some(&old), Some(&new)])?;
            }

            self.add_snippet(mv_code);
        }
        else {
            self.replace_register_with(replacements);
        }

        Ok(())
    }

    fn perform_registerset_action(&mut self, item:&RegisterSetReplacementItem) {
        match item.action {
            RegisterSetReplacementItemAction::Replace => {
                self.replace_register(&item.old_reg, &item.new_reg);
            }
            RegisterSetReplacementItemAction::Switch => {
                self.switch_register(&item.old_reg, &item.new_reg);
            }
        }
    }

    /// switch the register names in the code snippet, replace output register names, replace input register names
    pub fn switch_register(&mut self, reg0:&str, reg1:&str) {
        let temp_reg = TEMP_REGISTER_NAME;
        self.replace_register(reg0, temp_reg);
        self.replace_register(reg1, reg0);
        self.replace_register(temp_reg, reg1);
    }

    /// replace the register name in the code snippet, replace output register names, replace input register names
    /// see [`replace_register`] for more details
    pub fn replace_register_with(&mut self, regs:Vec<(String, String)>) {
        for (r0, r1) in regs.iter() {
            self.replace_register(r0, r1);
        }
    }

    /// Replace the register name in the code snippet, replace output register names, replace input register names
    /// for inside snippet reg replacement: 
    ///    1. old on inside snippet's input or output reg, then do old->new
    ///    2. old on inside snippet's internal/temp reg list (not on input or output reg), can do nothing, but still do old -> new, to save reg usage footprint
    ///    3. therefore, for inside snippet, always do old -> new
    ///    4. new on inside's all reg list, then panic!. this is not possible. Before this old -> new action, 'new' must be away from self's all reg list, and self's all reg list contains inside snippet's all reg list
    pub fn replace_register(&mut self, old_name:&str, new_name:&str) {
        let old : RegisterUsage = old_name.into();
        let new : RegisterUsage = new_name.into();

        for item in self.get_code_mut() {
            match item {
                CodeSnippetDataItem::Inc(item) => {
                    if let Some(inc) = item.get_inc_mut() {
                        let new_alias = new.get_reg_alias().to_string();
                        // Always rewrite to alias form to keep generated asm canonical and readable.
                        inc.replace_registers_from_tuple(& vec![(old.get_reg_name().to_string(), new_alias.clone()), 
                                                                        (old.get_reg_alias().to_string(), new_alias)]);
                    }
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    snippet.replace_register(old_name, new_name);
                }
                _ => {}
            }
        }

        //replace output register names
        for item in self.output_register_names.iter_mut() {
            if item == old.get_reg_name() || old.get_reg_alias() == item {
                *item = new.get_reg_alias().to_string();
            }
        }

        //replace required input register names
        for item in self.required_input_register_names.iter_mut() {
            if item == old.get_reg_name() || old.get_reg_alias() == item {
                *item = new.get_reg_alias().to_string();
            }
        }
    }

    /// Replace the immediate value in the code snippet if the immediate value is IMM_TO_REPLACE (144)
    pub fn replace_imm(&mut self, new_imm:&str) {
        for item in self.get_code_mut() {
            match item {
                CodeSnippetDataItem::Inc(item) => {
                    if let Some(inc) = item.get_inc_mut() {
                        inc.replace_imm(IMM_TO_REPLACE, new_imm);
                    }
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    snippet.replace_imm(new_imm);
                }
                _ => {}
            }
        }

        for item in self.get_data_mut() {
            match item {
                CodeSnippetDataItem::Inc(item) => {
                    if let Some(directive) = item.get_directive_mut() {
                        directive.replace_parameter(IMM_TO_REPLACE, new_imm);                       
                    }
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    snippet.replace_imm(new_imm);
                }
                _ => {}
            }
        }
    }

    /// replace label in the code snippet
    /// replace label section item, replace output labels, replace label in instruction which is IMM field
    /// also replace label in the inside snippets
    pub fn replace_label(&mut self, old_label:&str, new_label:&str) {
        for item in self.get_code_mut() {
            match item {
                CodeSnippetDataItem::Inc(item) => {
                    match item {
                        SectionItem::Instruction(inc) => {
                            inc.replace_imm(old_label, new_label);
                        }
                        SectionItem::Lable(lable) if lable == old_label => {
                            item.set_label(new_label);
                        }
                        SectionItem::Directive(n) => {
                            n.replace_parameter(old_label, new_label);
                        }
                        _ => { }
                    }
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    snippet.replace_label(old_label, new_label);
                }
                _ => {}
            }            
        }

        //replace output labels
        for item in self.output_labels.iter_mut() {
            if item == old_label {
                *item = new_label.to_string();
            }
        }

        for item in self.get_data_mut() {
            match item {
                CodeSnippetDataItem::Inc(n) => {
                    if let Some(label_text) = n.get_label() {
                        if label_text == old_label {
                            n.set_label(new_label);
                        }
                    }
                    else {
                        if n.is_directive() {
                            n.get_directive_mut().unwrap()
                                .replace_parameter(old_label, new_label);
                        }
                    }
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    snippet.replace_label(old_label, new_label);
                }
                _ => { }
            }
        }
    }

    /// replace label LABLE_REPLACEMENT in the code snippet
    pub fn replace_labelx(&mut self, new_label:&str) {
        self.replace_label(LABLE_REPLACEMENT, new_label);
    }

    /// return a new code snippet with labelx replaced
    pub fn with_labelx_repalced(mut self, name:&str) -> Self {
        self.replace_labelx(name);
        self
    }

    /// find the mut snippet with tag, the search is recursive and to deeper level of code snippet
    pub fn apply_to_code_snippet(&mut self, tag:&str, f:fn(&mut CodeSnippet) -> ()) {
        for item in self.code.iter_mut() {
            if let Some(snippet) = item.get_snippet_mut() {
                if snippet.tags.contains(& tag.to_string()) {                    
                    f(snippet);
                }
                snippet.apply_to_code_snippet(tag, f);
            }
        }
    }

    /// if snippet has label
    pub fn has_label(&self) -> bool {
        for item in self.get_code() {
            if item.is_label() {
                return true;
            }
        }
        false
    }

    /// append unique id to the label in the code snippet
    pub fn append_unique_id_to_label(&mut self) {
        if !self.has_label() {  // unique id added check cannot be added here, a label can be in sub snippet instruction and need to be replaced
            return;
        }

        let is_unique_id_added = self.is_unique_id_added;
        let id = Self::get_unique_id();

        // append label with unique id only the label is in the output labels or label is present individually in the code as label item
        // replace label in the inc first is must
        let all_labels = self.get_all_labels();
        let replacements = all_labels.iter().map(|x| (x.to_string(), format!("{}__{}", x, id))).collect::<Vec<_>>();
        for (old, new) in replacements.clone() {
            for item in self.get_code_mut() {
                match item {
                    CodeSnippetDataItem::Inc(item) => {
                        match item {
                            SectionItem::Instruction(inc) => {
                                inc.replace_imm(&old, &new);
                            }
                            SectionItem::Lable(lable) if lable == &old => {
                                if !is_unique_id_added {   // label can only be replace if unique id is not added for current snippet
                                    item.set_label(&new);
                                }
                            }
                            _ => {}
                        }
                    }
                    CodeSnippetDataItem::Snippet(snippet) => {
                        snippet.replace_label(&old, &new);
                    }
                    _ => {}
                }
            }
        }

        self.is_unique_id_added = true;
    }

    /// replace labelx with id in the instruction
    pub fn replace_labelx_in_incs(&mut self, new_id:&str) {
        self.replace_id_in_incs(LABLE_REPLACEMENT, new_id);
    }

    /// replace id in the instruction
    /// this operation only replace the immediate value in current snippet and won't replace in the inside snippets
    /// inside snippet might already have the value replaced, so this operation is not necessarily introduce error, 
    /// so the snippet replacement can only replace the LABLE_REPLACEMENT
    fn replace_id_in_incs(&mut self, old_label:&str, new_label:&str) {
        for item in self.get_code_mut() {
            match item {
                CodeSnippetDataItem::Inc(item) => {
                    match item {
                        SectionItem::Instruction(inc) => {
                            inc.replace_imm(old_label, new_label);
                        }
                        _ => {}
                    }
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    snippet.replace_id_in_incs(old_label, new_label);
                }
                _ => {}
            }
        }
    }

    /// insert list of section item(instruction or label) after a label in the code section
    pub fn insert_after_label(&mut self, label:&str, code:CodeSnippet) -> Result<(), CLangError> {
        if code.is_empty() {
            return Ok(());
        }
        
        let mut new_code = Vec::new();
        let mut is_inserted = false;
        for item in self.code.iter() {
            new_code.push(item.to_owned());
            if item.get_inc().and_then(|x| x.get_label().map(|l| l.as_str())) == Some(label) {
                new_code.push(CodeSnippetDataItem::Snippet(code.to_owned()));
                is_inserted = true;
            }
        }

        if !is_inserted {
            error_string(format!("cannot find label: {} in the code snippet", label));
            Err(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find label: {} in the code snippet", label)))
        }
        else {
            self.set_impacted_regs((self.get_impacted_regs_as_registerset() + code.get_contaminated_registers()).get_register_alias_name());
            self.code = new_code;
            Ok(())
        }
    }

    /// insert label and code snippets tuple in the code section
    pub fn insert_after_label_bulk(&mut self, codes:Vec<(String, CodeSnippet)>, _context:&mut CodeGenContext) -> Result<(), CLangError> {
        // reverse the list. if the code is after different label, the reverse does not impact
        // if the code is inserted after the same label, the one at first will be pushed back by the following ones
        // in order to keep the order when call this function, so make the code orde reversed 
        for (label, code) in codes.into_iter().rev() {
            self.insert_after_label(&label, code)?;
        }

        Ok(())
    }

    /// insert code snippet at the end of the code, please note that
    /// this function does not resolve register conflict.
    pub fn insert_at_end(&mut self, code:Vec<CodeSnippet>) {
        for snippet in code.into_iter() {
            self.add_snippet(snippet);
        }
    }

    /// insert at beginning of the code
    pub fn insert_at_beginning(&mut self, code:Vec<CodeSnippet>) {
        for (i, snippet) in code.into_iter().enumerate() {
            self.get_code_mut().insert(i, snippet.into());
        }
    }

    /// insert incs at beginning of the code
    pub fn insert_incs_at_beginning(&mut self, inc:Vec<&SectionItem>) {
        for (i, item) in inc.into_iter().enumerate() {
            self.get_code_mut().insert(i, CodeSnippetDataItem::Inc(item.to_owned()));
        }
    }

    /// generate code string from code snippet's section item list
    pub fn generate_code_string(&self) -> String {
        let debug_info = self.get_debug_info().trim();
        let mut r = if !debug_info.is_empty() {
            let debug_info_replaced = debug_info.replace("\r\n", "#").replace("\n", "#").replace("\r", "#");
            format!("# {}\r\n", debug_info_replaced)
        }
        else {
            String::new()
        };

        r.push_str(& self.generate_code_string_with_section(false));
        r
    }

    /// generate data string from data list
    pub fn generate_data_string(&self) -> String {
        let debug_info = self.get_debug_info().trim();
        let mut r = if !debug_info.is_empty() {
            let debug_info_replaced = debug_info.replace("\r\n", "#").replace("\n", "#").replace("\r", "#");
            format!("# {}\r\n", debug_info_replaced)
        }
        else {
            String::new()
        };

        for item in self.data.iter() {
            let mut code = item.generate_data_string();
            // insert new line if the data string is not empty and r is not empty and r does not end with new line
            if !code.is_empty() && !code.ends_with("\n") && !code.ends_with("\r\n") {
                code.push_str("\r\n");
            }

            r.push_str(& code);
        }

        // some code snipppet in code might have data
        for item in self.code.iter() {
            if item.is_snippet() {
                let mut code = item.generate_data_string();
                // insert new line if the data string is not empty and r is not empty and r does not end with new line
                if !code.is_empty() && !code.ends_with("\n") && !code.ends_with("\r\n") {
                    code.push_str("\r\n");
                }

                r.push_str(& code);
            }
        }

        r
    }

    /// remove item from code
    pub fn remove_code_item(&mut self, item:&CodeSnippetDataItem) {
        self.code.retain(|x| x != item);
    }

    /// generate code string from code. and there is a option to indicate to output .text seciton tag
    /// markdown data won't be generated in this function, please refer to generate_solution_data function
    pub fn generate_code_string_with_section(&self, with_section:bool) -> String {
        let mut r = String::new();

        if with_section {
            r.push_str(".text\n");
        }

        for item in self.code.iter() {
            let item_str = &item.generate_code_string();
            if item_str.is_empty() {
                continue;
            }
            
            r.push_str(&item_str);
            r.push_str("\r\n");
        }

        r
    }

    /// check if all output registers are parameter registers
    pub fn output_registers_all_parameter_registers(&self) -> bool {
        let output_registers = self.get_output_register_as_registerset();
        output_registers.get_registers()
            .iter()
            .all(|r| r.is_parameter_register())
    }

    /// generate data code string
    pub fn generate_data_string_with_section(&self, with_section:bool) -> String {
        let mut r = String::new();

        if with_section {
            r.push_str(".data\n");
        }

        for item in self.data.iter() {
            let item_str = &item.generate_data_string();
            if item_str.is_empty() {
                continue;
            }
            
            r.push_str(&item_str);
            r.push_str("\r\n");
        }

        for item in self.code.iter() {
            let item_str = &item.generate_data_string();
            if item_str.is_empty() {
                continue;
            }
            
            r.push_str(&item_str);
            r.push_str("\r\n");
        }

        r
    }

    /// self is the downstream system and other is the up stream system 
    /// other.output[j] --> self.input[i]
    pub fn get_input_register_from(&mut self, i:usize, other:&CodeSnippet, j:usize, context:&mut CodeGenContext) -> Result<(), CLangError>{
        let output_reg_name = other.get_output_register_names()
                                                            .iter()
                                                            .nth(j)
                                                            .ok_or(CLangError::NoFound((file!().to_string(), line!()), format!("cann't find '{j}' element in other's output register")))?;
        
        let r0 = self.get_input_register_names()
                            .iter()
                            .nth(i)
                            .ok_or(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find '{i}' element in self's input register")))?
                            .to_string();
        let fixed_regs = [r0.clone()].to_vec().into();
        self.move_register_from(RegisterSet::new_from_string_vec(&vec![output_reg_name.to_string()]), fixed_regs, context)?;
        
        self.replace_register(&r0, output_reg_name);
        Ok(())
    }

    /// replace the register names in the code snippet with the output register names from code snippet list
    /// also set the the input register names from the code snippet list
    /// this function check the register conflict and resolve the conflict by move_register_from before replace the register name, and the register conflict is defined as the intersection of self's all registers and other's output registers is not empty, 
    /// and the intersection is not empty after excluding the fixed registers
    pub fn get_input_registers_from(&mut self, snippets:Vec<&CodeSnippet>, context:&mut CodeGenContext) -> Result<(), CLangError> {
        // check if the input register number is equal to the sum of output register number from the snippets
        if self.required_input_register_names.len() != snippets.iter().map(|x| x.get_output_register_names().len()).sum::<usize>() {
            let mut msg = format!("input register number is not equal to the sum of output register number from the snippets");
            msg += &format!("\nself requires input_register number: {}", self.required_input_register_names.len());
            msg += &format!("\noutput register number from other snippets: {}", snippets.iter().map(|x| x.get_output_register_names().len()).sum::<usize>());
            msg += &format!("\nsnippets info: self = '{}' vs. others= '{}'", 
                format!("'{}'", self.get_name()), 
                snippets.iter().map(|x| format!("'{}'", x.get_name())).collect::<Vec<_>>().join(", "));
            error_string(msg.to_string());
            panic!("{}", msg);
        }

        let registers = snippets.into_iter()
            .flat_map(|x| x.get_output_register_names().iter())
            .cloned()
            .collect::<Vec<_>>();

        // do not need to check conflict if self.input_reg conflict with other's output reg, 
        // because self.input_reg will be replaced with other's output reg, 
        // so the conflict will be resolved by replacement. 
        // But if self's other registers conflict with other's output reg, 
        // then need to move away self's other registers to avoid conflict before calling this function
        // the self.input number same as the snippets' output number, so input reg no need to be checked for conflict
        let replacements = self.get_input_register_names().iter().zip(registers.iter())
                                                            .map(|(a, b)| (a.to_string(), b.to_string()))
                                                            .collect::<Vec<_>>(); 

        self.replace_register_with_temp_guard(replacements, context)?;

        self.set_input_register_names(registers);
        Ok(())
    }

    fn replace_register_with_temp_guard(&mut self, replacements:Vec<(String, String)>, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let replacements = replacements.into_iter()
            .filter(|(old, new)| old != new)
            .collect::<Vec<_>>();

        if replacements.is_empty() {
            return Ok(());
        }

        let mut unavailable = self.get_all_registers().get_register_alias_name();
        unavailable.extend(replacements.iter().map(|(old, _)| old.clone()));
        unavailable.extend(replacements.iter().map(|(_, new)| new.clone()));

        let temp_pool = context.get_available_register_as_registerset()
            .get_temp_registers()
            .get_register_alias_name();

        let mut staged = Vec::new();

        for (old, new) in replacements.iter() {
            let old_is_float = RegisterUsage::new(old).is_float_register();
            let candidate = temp_pool.iter()
                .find(|r| {
                    let is_float = RegisterUsage::new(r).is_float_register();
                    old_is_float == is_float && !unavailable.contains(r)
                })
                .cloned()
                .ok_or_else(|| CLangError::cannot_emit(
                    &format!("cannot find temp register for guarded replacement '{old}' -> '{new}'")
                ))?;

            unavailable.push(candidate.clone());
            staged.push((old.clone(), candidate, new.clone()));
        }

        // Stage 1: move all source registers to temporary placeholders.
        for (old, temp, _) in staged.iter() {
            self.replace_register(old, temp);
        }

        // Stage 2: move placeholders to their final target registers.
        for (_, temp, new) in staged.iter() {
            self.replace_register(temp, new);
        }

        Ok(())
    }

    /// get all registers used in the code snippet, including the registers used in the inside snippets, 
    /// and return as a register set. The function check the code level (NOT the snippet declaration level, like
    /// the input_reg or output_reg specified in the markdown file)
    pub fn get_all_registers(&self) -> RegisterSet {
        let mut r = RegisterSet::default();
        for item in self.get_code_items() {
            match item {
                CodeSnippetDataItem::Inc(item) => {
                    if let Some(inc) = item.get_inc() {
                        let inc_registers = inc.get_registers_used();
                        let register_usage = inc_registers.iter().map(|x| RegisterUsage::new(x)).collect::<Vec<_>>();
                        let register_set = RegisterSet::new_from_vec(&register_usage);
                        
                        r += register_set;
                    }
                }
                CodeSnippetDataItem::Snippet(snippet) => {
                    r += snippet.get_all_registers();
                }
                _ => {}
            }
        }

        r
    }

    /// check if the code snippet is empty
    pub fn is_empty(&self) -> bool {
        self.get_code_len() == 0
    }

    pub fn replace_asm_op_in_last_inc(&mut self, op:&str) -> Result<(), CLangError> {
        if let Some(n) = self.get_code_mut().last_mut() {
            if let Some(item) = n.get_inc_mut() {
                if let Some(inc) = item.get_inc_mut() {
                    inc.set_inc_name(op);
                    return Ok(())
                }
            }
        }

        Err(CLangError::ParameterError(format!("Must have last item in the incs, and the item must be an inc")))
    }

    /// generate asm solution from code snippet
    pub fn to_asm_solution(&self, main_file_name:&str, meta_info:Option<String>) -> ASMSolution {
        let meta_info_str = meta_info.unwrap_or(String::new());

        let code_str = self.generate_code_string_with_section(true);
        let data_str = self.generate_data_string_with_section(true);
        let code_data_str = format!("{meta_info_str}\n\n{}\n\n{}", data_str, code_str);

        let mut r = ASMSolution::new(FileObject::new(main_file_name, "s", &code_data_str));

        // get all markdown data recursively from self and inside snippets
        for item in self.get_markdown_data_recursive() {
            r.add_md_file(item.get_var_name(), item.get_data());
        }

        r
    }

    pub fn vec_from_markdown(file_content:&str) -> Result<Vec<Self>, CLangError> {

        let md = markdown_pest::parse(&file_content).map_err(|_| CLangError::ParsingError(file!().to_string(), line!() as usize))?;
        let tables = md.get_tables();
        if tables.is_empty() {
            return Ok(vec![]);
        }

        let hash = crate::languages::markdown_lang::to_clr_type::md_table_object_to_hash(tables.first().unwrap())
                                            .map_err(|_| CLangError::CannotEmitType(file!().to_string(), line!(), format!("cannot get hash from table")))?;
        
        let mut r = Vec::new();
        for ((key, header_text), _index) in hash.iter().zip(0..) {
            let header_level = 2;
            let headers = md.get_headers_with_level(header_level);
            if let Some(header) = headers.iter().find(|x| & x.get_text() == header_text) {
                if let Some(code) = md.find_code_after_header(header_text) {
                    let (section_text, section_data) = Self::get_section_from_str(&header.get_text(), code.get_code())?;
                    let id_list = Self::get_md_list(&md, header_text, header_level);

                    let id_list_to_string_vec = |name| { id_list.iter().find(|x| x.get_id().trim() == name).map(|x| x.get_list().to_owned()).unwrap_or(Vec::new()) };
                
                    r.push(Self { name : key.to_string(), 
                                    code: section_text.get_section_items().into_iter().map(|x| x.to_owned().into()).collect(), 
                                    data: section_data.get_section_items().into_iter().map(|x| x.to_owned().into()).collect(),
                                    debug_info: String::new(),

                                    output_register_names: id_list_to_string_vec("output"),
                                    output_labels: Vec::new(),
                                    required_input_register_names: id_list_to_string_vec("input"), 
                                    required_output_labels : id_list_to_string_vec("out_labels"), 
                                    required_temporary_register_names: id_list_to_string_vec("temp"),
                                    is_unique_id_added: false,
                                    impacted_regs: id_list_to_string_vec("impacted_regs"),
                                    fixed_regs: id_list_to_string_vec("fixed_regs"),
                                    tags: id_list_to_string_vec("tags"),
                                });
                }
                else {
                    // errr output cannot find code block after header
                    error_string(format!("cannot find code block after header: {}", header_text));
                    Err(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find code block after header: {}", header_text)))?;
                }
            }
            else {
                //err output cannot find header
                error_string(format!("cannot find header: {}", header_text));
                Err(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find header: {}", header_text)))?;
            }
            
        }

        Ok(r)
    }

    /// get list in the md file
    fn get_md_list(md:&File, header_text:&str, header_level: u8 ) -> Vec<ListWithId> {
        let md_list = md.get_items_in_header(header_text, header_level, |x| x.get_block().is_some() && x.get_block().unwrap().get_m_d_list().is_some());
        let md_list_v = md_list.iter().filter_map(|x| x.get_block())
                                        .filter_map(|x| x.get_m_d_list())
                                        .map(|x| x.get_value())
                                        .flatten()
                                        .map(|x| x.get_content())
                                        .map(|x| x.get_text())
                                        .collect::<Vec<_>>();
        let list_with_id = md_list_v.iter().map(|x| parse_list_with_id(x).map_err(|x| CLangError::DataLangErr(x)).unwrap())
                                .collect();
        list_with_id
    }

    /// parse code section from string, the return value is text section and data section tuple
    /// refer to clang_asm_mapping_float.md for float type, which requires .data section in addition to .text section
    fn get_section_from_str(key:&str, input:&str) -> Result<(Section, Section), CLangError> {
        let code = if input.trim().starts_with(".text") { input.to_string() } else { format!(".text\n{}", input) };

        let mut asm_config = crate::languages::r5asm::code_gen_config::CodeGenConfiguration::new(false);
        match crate::languages::r5asm::assembler::parse_asm(&code, &mut asm_config) {
            Ok(parsed_code) => {
                let sections = parsed_code.get_sections();
                let text_section = sections.iter().find(|x| x.is_text_section() ).cloned().unwrap_or(Section::default());
                let data_section = sections.iter().find(|x| x.is_data_section() ).cloned().unwrap_or(Section::new_empty_data_section());
                Ok((text_section, data_section))
            }
            Err(e) => {
                error_string(format!("parse code from MD file failed, please check the code section with key = '{key}' in the MD file."));
                error_string(format!("{:?}", e));
                Err(CLangError::ParsingError(file!().to_string(), line!() as usize))
            }
        }
    }

    pub fn set_code(&mut self, code:Vec<CodeSnippetDataItem>) {
        self.code = code;
    }

    pub fn set_data(&mut self, data:Vec<CodeSnippetDataItem>) {
        self.data = data;
    }
}

impl AddAssign for CodeSnippet {
    /// add code snippet to the current code snippet, it will append the code.
    /// for register: self's output register => other's input.
    /// the self's output register names = other's output register names.
    /// this function assume self's output register number is equal to other's input register number.
    /// input -> self -> other -> output.
    /// Note: this function will NOT resolve register conflict, the caller must ensure there is no conflict.
    fn add_assign(&mut self, other: Self) {
        if !self.is_empty() { 
            assert!(self.output_register_names.len() == other.output_register_names.len());
        }

        let mut snippet = other.clone();
        let other_output_registers = snippet.output_register_names.clone();
        let self_output_registers = self.output_register_names.clone();

        // get list of register (other, self) tuple to do replacement
        let zipped_registers = other_output_registers.into_iter().zip(self_output_registers.iter())
                                                                    .map(|(a, b)| (a.to_string(), b.to_string()))
                                                                    .collect::<Vec<_>>();

        for (r0, r1) in zipped_registers.iter() {
            snippet.replace_register(r0, r1);
        }

        self.set_output_register_names(snippet.get_output_register_names().to_owned());
        let impacted_regs = self.get_impacted_regs_as_registerset() + snippet.get_contaminated_registers();
        self.set_impacted_regs(impacted_regs.to_name_vec());
        
        self.merge_output_labels(&snippet);

        self.add_snippet(snippet);
    }
}

impl BitAndAssign for CodeSnippet {
    /// merge the code snippet with other code snippet.
    /// self.code append other.code.
    /// self.output_register_names union other.output_register_names.
    /// self.output_labels union other.output_labels.
    /// all conflict should be resolved before this operation.
    fn bitand_assign(&mut self, other: Self) {
        self.merge_output_labels(&other);
        self.add_snippet(other);
    }
}

impl Default for CodeSnippet {
    fn default() -> Self {
        Self::new("default_code_snippet")
    }
}

impl Debug for CodeSnippet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let code = &self.code;
        let data = &self.data;
        let code_str = if code.is_empty() { String::new() } else { format!("\n\n{:?}", code) };
        let data_str = if data.is_empty() { String::new() } else { format!("\n\n{:?}", data) };

        let tags_str = if self.tags.is_empty() { String::new() } else { format!(", tags: {:?}", self.tags) } ;
        let inputreg_str = if self.required_input_register_names.is_empty() { String::new() } else { format!(", input_registers: {:?}", self.required_input_register_names) } ;
        let fixed_reg_str = if self.fixed_regs.is_empty() { String::new() } else { format!(", fixed_regs: {:?}", self.fixed_regs) } ;
        let outputreg_str = if self.output_register_names.is_empty() { String::new() } else { format!(", output_registers: {:?}", self.output_register_names) } ;
        let output_label_str = if self.output_labels.is_empty() { String::new() } else { format!(", output_labels: {:?}", self.output_labels) } ;
        let impacted_regs_str = if self.impacted_regs.is_empty() { String::new() } else { format!(", impacted_regs: {:?}", self.impacted_regs) } ;
    
        let debug_info = &self.debug_info;
        write!(f, "CodeSnippet {{ name: {} - {debug_info}, (code/data) : ({}/{}){outputreg_str}{inputreg_str}{output_label_str}{impacted_regs_str}{fixed_reg_str}{tags_str}{code_str}{data_str} }}", 
                self.name, self.get_code_len(), self.get_data().len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_input_registers_from_does_not_cascade_replacements() {
        let mut context = CodeGenContext::default();

        let mut target = CodeSnippet::new("add operation");
        target.set_input_register_names(vec!["t0".to_string(), "t1".to_string()]);
        target.set_output_register_names(vec!["t2".to_string()]);
        target.add_asm_inc("add", vec![Some("t2"), Some("t0"), Some("t1")]).unwrap();

        let mut left = CodeSnippet::new("left");
        left.set_output_register_names(vec!["t1".to_string()]);

        let mut right = CodeSnippet::new("right");
        right.set_output_register_names(vec!["t6".to_string()]);

        target.get_input_registers_from(vec![&left, &right], &mut context).unwrap();

        let code = target.generate_code_string();
        assert!(code.contains("add t2, t1, t6"), "unexpected code: {code}");
        assert!(!code.contains("add t2, t6, t6"), "unexpected code: {code}");
    }

    #[test]
    fn get_input_register_from_keeps_bound_input_for_multi_instruction_snippet() {
        let mut context = CodeGenContext::default();

        let mut load_from_addr = CodeSnippet::new("load from bound address");
        load_from_addr.set_input_register_names(vec!["t0".to_string()]);
        load_from_addr.set_output_register_names(vec!["t1".to_string()]);
        load_from_addr.add_asm_inc("lw", vec![Some("t1"), Some("0(t0)")]).unwrap();
        load_from_addr.add_asm_inc("addi", vec![Some("t1"), Some("t1"), Some("1")]).unwrap();

        let mut upstream = CodeSnippet::new("upstream address producer");
        upstream.set_output_register_names(vec!["t0".to_string()]);

        load_from_addr.get_input_register_from(0, &upstream, 0, &mut context).unwrap();

        let code = load_from_addr.generate_code_string();
        assert!(code.contains("0(t0)"), "unexpected code: {code}");
        assert!(!code.contains("0(t5)"), "unexpected code: {code}");
    }
}