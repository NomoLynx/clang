use std::fmt::Display;

use pest::iterators::Pair;

use crate::languages::clang::clang_asm_code_gen::{CodeGenConfiguration, Footnote, MDTableCell};
use core_utils::debug::*;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::enum_type::*;
use crate::languages::clang::symbol_table::*;
use crate::languages::clang::traits::*;
use crate::languages::clang::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::{Id, Rule}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumSpecifier {
    name: Option<Id>,
    enumerators: Vec<EnumSpecifier_Constant>,
    symbol_table : SymbolTable,
    ast_item : ASTItem,
}

impl EnumSpecifier {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::enum_specifier); //ensure it is rule:enum_specifier

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut enum_specifier = EnumSpecifier::default();
        enum_specifier.ast_item = ast_item;

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::identifier => enum_specifier.name = Some(inner_pair.as_str().to_string()),
                Rule::enumerator_list => {
                    // enum item const items creation
                    let mut enum_starting_point = 0isize;
                    for enumerator in inner_pair.into_inner() {
                        let mut enum_constant = EnumSpecifier_Constant::from_pair(&enumerator, context)?;

                        // If explicit value is provided, use it; otherwise assign from running point.
                        if enum_constant.has_explicit_value() {
                            if let Some(explicit_value) = enum_constant.try_get_explicit_base_value() {
                                enum_constant.set_base_value(explicit_value);
                                enum_starting_point = explicit_value + 1;
                            }
                            else {
                                enum_constant.set_base_value(enum_starting_point);
                                enum_starting_point += 1;
                            }
                        }
                        else {
                            enum_constant.set_base_value(enum_starting_point);
                            enum_starting_point += 1;
                        }
                        
                        enum_specifier.enumerators.push(enum_constant);
                    }
                }
                Rule::ENUM => { /* do nothing, this is just the enum keyword */}
                _ => {
                    let str =
                        format!("cannot find {inner_pair:?} in enum_specifier processing logic");
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }

        // set the base type for each enumerator and base value
        for enumerator in enum_specifier.enumerators.iter_mut() {
            enumerator.set_base_type(context.get_config().get_enum_base_clang_type());
        }

        Ok(enum_specifier)
    }

    /// build up the symbol table for the enum specifier
    pub fn setup_symbol_table(&mut self, config:&CodeGenConfiguration) -> Result<(), CLangError> {
        let mut offset = 0;
        for mut item in self.get_symbol_table_elements() {
            let data_type_size = item.populate_type_size_from_data_type(config);
            if data_type_size < 0 {
                error_string(format!("data type size is not set for symbol entry {item:?} in enum specifier {self:?}"));
                return Err(CLangError::NoFound((file!().to_string(), line!()), format!("data type size is not set for symbol entry {item:?} in enum specifier {self:?}")));
            }
            else {
                item.set_storage_location(SymbolLocation::StructUnionMemberOffset(offset));
                offset += data_type_size as i32;
            }
            self.symbol_table.insert(item);
        }

        self.symbol_table.set_container(Some(self.get_ast_item().get_current_node().unwrap().clone()));

        // get scope element and add itself to symbol table
        if let Some(scope_item) = self.get_ast_item().get_scope_item() {
            let symbol_table = scope_item.get_symbol_table_mut();
            let mut entry = SymbolEntry::from_to_symbol_table_element(self);
            let enum_node = self.get_ast_item().get_current_node().unwrap().clone();
            let decl_node = self.get_ast_item().get_parent().cloned().unwrap_or(enum_node);
            
            assert!(decl_node.is_declaration());
            
            entry.set_clang_item_link(Some(decl_node));
            symbol_table.insert(entry);
        }

        Ok(())
    }

    /// get symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    /// get enum items
    pub fn get_enum_items(&self) -> &Vec<EnumSpecifier_Constant> {
        &self.enumerators
    }

    /// check if has enum items
    pub fn has_enum_items(&self) -> bool {
        self.enumerators.len() > 0
    }

    /// get the storage size of enum type in bytes
    /// enum object size is one base enum unit (e.g. int32), independent of enumerator count
    pub fn get_type_size(&self, config:&CodeGenConfiguration) -> usize {
        config.get_enum_base_type_size_in_bytes() as usize
    }

    /// get assembly markdown table code representation
    pub fn get_asm_md_table_code(&self, config:&CodeGenConfiguration) -> (Vec<MDTableCell>, Vec<Footnote>) {
        let mut table = vec![];
        let mut footnotes = vec![];
        for enumerator in self.enumerators.iter() {
            let mut cell = enumerator.to_md_table_cell(config);
            cell.set_value(format!("{}", enumerator.get_base_value()));
            table.push(cell);
        }

        for cell in table.iter() {
            for tag in cell.get_tags().iter(){
                let footnote = Footnote::new2(tag);
                footnotes.push(footnote);
            }
        }

        (table, footnotes)
    }

    /// get the name identifier if exists
    pub fn get_name_id(&self) -> Option<&String> {
        self.name.as_ref()
    }

    /// get the enum type name
    pub fn get_type_name(&self) -> String {
        if let Some(name) = self.get_name_id() {
            format!("{}", name)
        }
        else {
            format!("anonymous_type_{}", self.get_id())
        }
    }

    /// get base type for the enum specifier and will need verify all enumerators have same base type
    pub fn get_base_type(&self) -> CLangTypes {
        if self.enumerators.len() > 0 {
            let ty = self.enumerators[0].get_base_type();
            assert!(self.enumerators.iter().all(|x| x.get_base_type() == ty));
            ty.clone()
        }
        else {
            CLangTypes::Int32
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        self.enumerators.iter_mut()
            .map(|x| x.into())
            .collect()
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        for n in self.enumerators.iter_mut() {
            n.access_children(context);
        }

        context.access_clang_item(CLangItem::EnumSpecifier(CLangItem::new(self)));
    }
}

impl ASTItemTrait for EnumSpecifier {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("EnumSpecifier({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Default for EnumSpecifier {
    fn default() -> Self {
        Self {
            name: None,
            enumerators: vec![],
            symbol_table : SymbolTable::default(),
            ast_item : ASTItem::default(),
        }
    }
}

impl Display for EnumSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let binding = "".to_string();
        let name = self.name.as_ref().unwrap_or(&binding);
        write!(f, "enum {} {{\n", name)?;
        for e in self.enumerators.iter() {
            write!(f, "{}", e)?;
        }
        write!(f, "}}")
    }
}
