use std::collections::HashMap;

use crate::languages::clang::clang_asm_code_gen::{CodeGenContext, RegisterSet, RegisterUsage};
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::{clang_error::CLangError, clang_item::CLangItem, declaration::Declaration};
use crate::languages::clang::traits::to_markdown_table_row_trait::ToMarkdownTableRow;
use crate::languages::clang::logic::*;

use super::*;

/// The attribute key for variable type in symbol table entries, it mainly used for parameters
pub const ATTRIBUTE_KEY_VARTYPE: &str = "VARTYPE";

/// Represents a CLang symbol table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolTable {
    container : Option<CLangItem>,  // the CLangItem that contains this symbol table, like function or compound statement
    table: Vec<SymbolEntry>,
    complex_type_table: HashMap<TypeName, ComplexType>,
}

impl SymbolTable {
    /// Insert a new symbol into the table
    pub fn insert(&mut self, entry: SymbolEntry) {
        self.table.push(entry);
    }

    pub fn add_var_definition(&mut self, name:&str, data_type:CLangTypes, id:usize, scope:&str, clang_item:CLangItem) {
        let mut entry = SymbolEntry::new_var(id, name, scope);
        entry.set_data_type(Some(data_type));
        entry.set_clang_item_link(Some(clang_item));
        self.insert(entry);
    }

    /// get all symbol entries in the table
    pub fn get_all_entries(&self) -> &Vec<SymbolEntry> {
        &self.table
    }

    /// Lookup the first symbol by name
    pub fn lookup(&self, name: &str) -> Option<&SymbolEntry> {
        if let Some(n) = self.table.iter().find(|x| x.get_name() == name) {
            Some(n)
        }
        else {
            let sub_table = self.get_sub_tables();
            for table in sub_table {
                if let Some(n) = table.lookup(name) {
                    return Some(n);
                }
            }

            None
        }
    }

    pub fn lookup_mut(&mut self, name: &str) -> Option<&mut SymbolEntry> {
        let no_sub_table = self.get_sub_tables_mut().is_empty();
        if no_sub_table {
            self.table.iter_mut().find(|x| x.get_name() == name)
        }
        else {
            let sub_tables = self.get_sub_tables_mut();
            for table in sub_tables {
                if let Some(n) = table.lookup_mut(name) {
                    return Some(n);
                }
            }

            None
        }
    }

    /// get total size of all variable symbols in bytes, excluding register only variables
    pub fn get_var_symbol_total_size_in_bytes(&self, context:&CodeGenContext) -> usize {
        let mut total_size = 0;
        for entry in self.table.iter() {
            if entry.get_symbol_type() == &SymbolType::Var {
                if entry.is_register_only() {
                    continue;
                }

                if let Some(data_type) = entry.get_data_type() {
                    if let Some(size) = data_type.get_type_size_in_bytes(context) {
                        total_size += size;
                    }
                }
            }
        }

        total_size
    }

    /// lookup the first symbol by name and type
    pub fn lookup_by_type(&self, name:&str, symbol_type:&SymbolType) -> Option<&SymbolEntry> {
        if let Some(n) = self.table
                                        .iter()
                                        .find(|x| x.get_name() == name && x.get_symbol_type() == symbol_type) {
            Some(n)
        }
        else {
            let sub_table = self.get_sub_tables();
            for table in sub_table {
                if let Some(n) = table.lookup_by_type(name, symbol_type) {
                    return Some(n);
                }
            }

            None
        }
    }

    /// lookup mut the first symbol by name and type 
    pub fn lookup_by_type_mut(&mut self, name:&str, symbol_type:&SymbolType) -> Option<&mut SymbolEntry> {
        self.table.iter_mut().find(|x| x.get_name() == name && x.get_symbol_type() == symbol_type)
    }

    /// lookup the first symbol by name and type list
    pub fn lookup_by_type_list(&self, name:&str, symbol_types:&Vec<SymbolType>) -> Option<&SymbolEntry> {
        for symbol_type in symbol_types.iter() {
            if let Some(entry) = self.lookup_by_type(name, symbol_type) {
                return Some(entry);
            }
        }

        None
    }

    /// get mutable reference to the symbol table entries whose type is given by parameter
    pub fn get_entries_by_type_mut(&mut self, symbol_type:&SymbolType) -> Vec<&mut SymbolEntry> {
        self.table.iter_mut()
            .filter(|x| x.get_symbol_type() == symbol_type)
            .collect::<Vec<_>>()
    }

    pub fn set_container(&mut self, container:Option<CLangItem>) {
        self.container = container;
    }

    /// get the container CLangItem of the symbol table, like function or compound statement
    pub fn get_container(&self) -> Option<&CLangItem> {
        self.container.as_ref()
    }

    pub fn get_complex_type_table(&self) -> &HashMap<TypeName, ComplexType> {
        &self.complex_type_table
    }

    pub fn get_complex_type_table_mut(&mut self) -> &mut HashMap<TypeName, ComplexType> {
        &mut self.complex_type_table
    }

    /// get the sub symbol tables of the current symbol table
    /// sub symbol tables are from ENUM, STRUCT, or UNION
    pub fn get_sub_tables(&self) -> Vec<&SymbolTable> {
        let mut r = Vec::new();
        for entry in self.table.iter() {
            if let Some(citem) = entry.get_clang_item_link() {
                if let Some(decl) = citem.get_declaration() {
                    if entry.is_enum_type() || entry.is_struct_type() || entry.is_union_type() {  // some variable has struct/union/enum type, so we have to check the var type
                        if let Some(enum_item) =decl.get_enum() {
                            r.push(enum_item.get_symbol_table());
                        }
                        else if let Some(struct_item) = decl.get_struct() {
                            r.push(struct_item.get_symbol_table());
                        }
                        else if let Some(union_item) = decl.get_union() {
                            r.push(union_item.get_symbol_table());
                        }
                    }
                }
            }
        }

        r
    }

    pub fn get_sub_tables_mut(&self) -> Vec<&mut SymbolTable> {
        let mut r = Vec::new();
        for entry in self.table.iter() {
            if let Some(citem) = entry.get_clang_item_link() {
                if let Some(enum_item) = citem.get_enum() {
                    r.push(enum_item.get_symbol_table_mut());
                }
                else if let Some(struct_item) = citem.get_struct_or_union() {
                    r.push(struct_item.get_symbol_table_mut());
                }
            }
        }

        r
    }

    pub fn lookup_complex_type(&self, name:&str) -> Option<&ComplexType> {
        let struct_name = format!("struct {}", name);
        let union_name = format!("union {}", name);
        let keys = vec![name, &struct_name, &union_name];
        for key in keys {
            if let Some(value) = self.complex_type_table.get(key) {
                return Some(value);
            }
        }

        None
    }

    pub fn set_parameter_storage_location(&mut self, name:&str, location:SymbolLocation) -> Result<(), CLangError> {
        if let Some(entry) = self.lookup_by_type_mut(name, &Declaration::get_symbol_table_type_declaration()) {
            entry.add_storage_location(location);
            Ok(())
        }
        else {
            Err(CLangError::NoFound((file!().to_string(), line!()), format!("Symbol {} not found", name)))
        }
    }

    pub fn get_parameter_storage_location(&self, name: &str) -> &SymbolLocation {
        self.lookup_by_type(name, &Declaration::get_symbol_table_type_declaration())
            .and_then(|x| x.get_single_storage_location())
            .unwrap_or(&SymbolLocation::NotSet)
    }

    pub fn set_var_storage_location(&mut self, name:&str, location:SymbolLocation) -> Result<(), CLangError>{
        if let Some(entry) = self.lookup_by_type_mut(name, &Declaration::get_symbol_table_type_var()) {
            entry.add_storage_location(location);
            Ok(())
        }
        else {
            Err(CLangError::NoFound((file!().to_string(), line!()), format!("Symbol {} not found", name)))
        }
    }

    pub fn get_var_storage_location(&self, name: &str) -> &SymbolLocation {
        self.lookup_by_type(name, &Declaration::get_symbol_table_type_var())
            .and_then(|x| x.get_single_storage_location())
            .unwrap_or(&SymbolLocation::NotSet)
    }

    pub fn get_child_scope_elements(&self) -> Vec<&CLangItem> {
        self.table.iter()
            .filter_map(|x| x.get_clang_item_link() )
            .filter( |x| x.is_scope())
            .collect::<Vec<_>>()
    }

    pub fn get_markdown_header() -> String {
        // get markdown table header from string vec
        let header_names = vec![
            "Id",
            "Name",
            "Type",
            "Scope",
            "DataType_CLangType",
            "Storage_Location",
            "Attributes",
            "CLangItem",
            "Global",
            "ValueChanged",
            "ImpactedRange",
            "IsExtern",
            "IsAddressTaken",
            "TypeAlias",
        ];
        let line0_content = header_names.join(" | ");
        let line0 = format!("| {} |", line0_content);

        // line1 will be generated based on line0, how many columns in line0, there will be that many '------' in line1
        let bar_count = line0.matches('|').count() - 1;
        let pattern = "|----";
        let line1 = pattern.repeat(bar_count) + "|";
        format!("{}\n{}\n", line0, line1)
    }

    pub fn get_table_markdown_name(&self) -> String {
        let table_name = self.get_container()
                                    .as_ref()
                                    .map(|x| x.get_name())
                                    .unwrap_or("<NONAME>".to_string());
        table_name
    }

    /// get used registers in the symbol table
    pub fn get_used_registers(&self) -> Vec<RegisterName> {
        let mut r = Vec::new();
        for entry in self.table.iter() {
            match entry.get_symbol_type() {
                SymbolType::Var => {
                    r.extend(entry.get_storage_location().iter()
                        .filter_map(|loc| loc.get_register_name())
                        .map(|name| name.to_string())
                        .collect::<Vec<_>>()
                    );
                }
                SymbolType::CompoundStatement |
                SymbolType::Function => {
                    if let Some(citem) = entry.get_clang_item_link() {
                        if let Some(scope_trait) = citem.get_scope_trait() {  
                            let child_symbols = scope_trait.get_symbol_table().get_used_registers();
                            for reg in child_symbols {
                                if !r.contains(&reg) {
                                    r.push(reg);
                                }
                            }
                        }
                        else {
                            // the function can be extern function without body
                        }
                    }
                }
                _ => {}
            }
        }

        r.sort();
        r.dedup();
        r
    }

    pub fn to_register_uses(&self) -> RegisterUseTypes {
        let mut r = RegisterUseTypes::default();
        for entry in self.table.iter() {
            r.push(entry.to_register_use());
        }

        r
    }

    /// check if a register is fixed register in the symbol table
    /// fixed register means it is used by function or prefixed to a specific register and cannot be changed
    pub fn is_fixed_register(&self, reg_name:&str) -> bool {
        let reg = RegisterUsage::new(reg_name);
        let regs = self.to_register_uses();

        let fixed= regs.contains_register(&reg);
        let is_parameter_reg = reg.is_parameter_register();

        fixed || is_parameter_reg
    }

    /// assign registers to parameters in the symbol table, according to RISC-V calling convention
    /// a0 - a7 are used for parameter passing
    /// parameters beyond 8 are passed on stack
    pub fn assign_parameter_register(&mut self, context:&mut CodeGenContext) {
        let mut parameter_entries = self.get_entries_by_type_mut(&SymbolType::Declaration);
        let parameter_register_names = RegisterSet::get_riscv_parameter_register_names();

        for (i, parameter_entry) in parameter_entries.iter_mut().enumerate() {
            if i < parameter_register_names.len() {
                let reg_name = parameter_register_names[i].clone();
                parameter_entry.set_storage_location(SymbolLocation::Register(reg_name));
            }
            else {
                // parameters beyond 8 are passed on stack
                if let Some(param_type) = parameter_entry.get_data_type() {
                    let data_size = param_type.get_type_size_as_parameter_in_bytes(context).unwrap();
                    // stack offset is calculated from function stack frame pointer
                    // here we use negative offset to indicate stack location
                    let stack_offset = - ( (i - parameter_register_names.len() + 1) as i32 * data_size as i32);
                    parameter_entry.set_storage_location(SymbolLocation::Stack(stack_offset));
                }
                else {
                    panic!("cannot get var type for parameter {}", parameter_entry.get_name());
                }
            }
        }
    }

    /// assign registers to variables in the symbol table
    pub fn assgin_var_register(&mut self, context:&mut CodeGenContext) {
        let used_registers = self.get_used_registers();
        let register_info = self.to_register_uses();

        let function_clang_item = self.get_container().unwrap().clone();
        let variable_items = self.get_entries_by_type_mut(&SymbolType::Var);

        let container = function_clang_item.get_ast_item();
        if container.is_clang_program_item() {
            return;
        }
        
        let container_function_option = container.get_current_function();
        if container_function_option.is_none() {
            panic!("Cannot get container function for symbol table");
        }

        let container_function = container_function_option.unwrap();
        let mut stack_frame_pointer = container_function.get_stack_frame_pointer();
        
        let mut assigned_reg_names = Vec::new();
        for variable_entry in variable_items {
            let variable_range = variable_entry.get_range();
            let var_type = variable_entry.get_data_type();
            if var_type.is_none() {
                panic!("Variable {} has no data type set", variable_entry.get_name());
            }

            if variable_entry.can_use_register() {
                // reuse register from functions and compound statements, the variable_range is used to match the variable scope
                // if the scope is conflicted, the register can be potentially reused
                let reg_name_option = register_info.get_reusable_register_name(SymbolType::Function, variable_range.as_ref(), &assigned_reg_names)
                                                            .or_else(|| register_info.get_reusable_register_name(SymbolType::CompoundStatement, variable_range.as_ref(), &assigned_reg_names));
                
                if let Some(reg_name) = reg_name_option {
                    variable_entry.set_storage_location(SymbolLocation::Register(reg_name.clone()));
                    assigned_reg_names.push(reg_name);
                }
                else {
                    // no reusable register found, need to allocate new register
                    let available_regs = context.get_available_register_as_registerset_by_type(var_type.unwrap());
                    let free_registers = available_regs - used_registers.clone().into() - assigned_reg_names.clone().into();
                    let reg_name = free_registers.get_sorted_reg_names().first().cloned();
                    if let Some(reg_name) = reg_name {
                        variable_entry.set_storage_location(SymbolLocation::Register(reg_name.clone()));
                        assigned_reg_names.push(reg_name);
                    }
                    else {
                        // assign the variable to stack if no register available
                        if let Some(var_type) = variable_entry.get_data_type() {
                            let data_size = var_type.get_type_size_in_bytes(context).unwrap();
                            stack_frame_pointer -= data_size as i32;
                            variable_entry.set_storage_location(SymbolLocation::Stack(stack_frame_pointer));
                        }
                        else {
                            panic!("cannot get var type for variable {}", variable_entry.get_name());
                        }
                    }
                }
            }
            else {
                // assign the variable to stack if no register available
                if let Some(var_type) = variable_entry.get_data_type() {
                    let data_size = var_type.get_type_size_in_bytes(context).unwrap();
                    stack_frame_pointer -= data_size as i32;
                    variable_entry.set_storage_location(SymbolLocation::Stack(stack_frame_pointer));
                }
                else {
                    panic!("cannot get var type for variable {}", variable_entry.get_name());
                }
            }
        }

        container_function.set_stack_frame_pointer(stack_frame_pointer);
    }

    /// check whether the container of the symbol table is struct or union
    pub fn is_container_struct_or_union(&self) -> bool {
        if self.get_container().is_none() {
            return false;
        }

        let citem = self.get_container().unwrap();
        citem.is_struct_or_union()
    }

    /// get member offset of a member in struct or union symbol table
    /// it will look at the stoage location and get member offset, if it's other type of location, it will return None
    pub fn get_offset(&self, name: &str) -> Option<isize> {
        if self.is_container_struct_or_union() {
            if let Some(entry) = self.lookup(name) {
                if let Some(offset) = entry.get_single_storage_location_member_offset() {
                    return Some(offset as isize);
                }
            }

            None
        }
        else {
            None
        }
    }

    /// get data type of a member in struct or union symbol table
    pub fn get_member_type(&self, name:&str) -> CLangTypes {
        assert!(self.is_container_struct_or_union(), "get_member_type can only be called on struct or union symbol table");

        for entry in self.table.iter() {
            if entry.get_name() == name {
                if let Some(data_type) = entry.get_data_type() {
                    return data_type.clone();
                }
                else {
                    panic!("Cannot get data type for member '{}'", name);
                }
            }
        }

        panic!("Member '{}' not found in struct or union symbol table", name);
    }

    pub fn sematic_checks(&self) -> Result<Vec<CLangError>, CLangError> {
        let mut errors = Vec::new();
        let data : Vec<(i32, String, String)> = self.into();
        let duplicates = find_duplicates(data);
        for (name, scope) in duplicates {
            errors.push(CLangError::GeneralError((file!().to_string(), line!()), format!("Duplicate definition of symbol '{}' in scope '{}'", name, scope)));
        }
        
        Ok(errors)
    }
}

/// Provide a default implementation for `SymbolTable`
impl Default for SymbolTable {
    fn default() -> Self {
        Self {
            table: Vec::default(),
            container : None,
            complex_type_table: HashMap::default(),
        }
    }
}

/// Implement ToMarkdownTableRow trait for SymbolTable
/// This will allow us to convert the symbol table into a markdown table format
impl ToMarkdownTableRow for SymbolTable {
    fn get_markdown_header(&self) -> String {
        SymbolTable::get_markdown_header()
    }

    fn to_markdown(&self) -> String {
        let table_name = self.get_table_markdown_name();
        let mut markdown = format!("# Symbol Table: {table_name}\n\n");
        
        if self.table.len() == 0 {
            markdown.push_str("No entries\r\n\r\n");
        }
        else {
            markdown.push_str(self.get_markdown_header().as_str());
            for entry in self.table.iter() {
                markdown.push_str(& format!("{}\r\n", entry.to_markdown()));
            }
        }
        markdown.push_str("\r\n");

        // add parent item
        markdown.push_str(format!("## {table_name} parent\r\n\r\n").as_str());
        if let Some(current) = self.container.as_ref() {
            if let Some(parent) = current.get_ast_item_mut().get_scope_parent() {
                markdown.push_str(&format!("{}", parent.get_name()) );
            }
            else {
                markdown.push_str("None");
            }
        }
        else {
            markdown.push_str("None");
        }
        markdown.push_str("\r\n");

        // add child items list
        markdown.push_str(format!("\r\n## {table_name} children\r\n").as_str());
        if let Some(n) = self.container.as_ref() {
            let mut citem = n.clone();
            if let Some(container) = citem.get_symbol_entry_elements() {
                markdown.push_str(container.get_child_symbol_table_elements().as_str());
                markdown.push_str("\r\n");
            }
            else {
                markdown.push_str("None\r\n");
            }
        }
        else {
            markdown.push_str("None\r\n");
        }

        // add type table
        if !self.complex_type_table.is_empty() {
            markdown.push_str("\r\n## Complex Type Table\r\n");
            let mut keys = self.complex_type_table.keys().collect::<Vec<_>>();
            keys.sort();

            // output the item in list format, list name is key and list content is value
            for key in keys {
                markdown.push_str(&format!("### {}\r\n\r\n", key));
                markdown.push_str(&format!("```markdown\r\n{}\r\n```\r\n", self.complex_type_table.get(key).unwrap()));
            }
        }

        markdown
    }
}

impl From<&SymbolTable> for Vec<(i32, String, String)> {
    fn from(table: &SymbolTable) -> Self {
        let mut r = vec![];
        for entry in table.get_all_entries() {
            let id = entry.get_id() as i32;
            let name = entry.get_name().to_string();
            let scope = table.get_container()
                                    .map(|x| x.get_name())
                                    .unwrap_or("<NONAME>".to_string());
            r.push( (id, name, scope) );
        }

        r 
    }
}