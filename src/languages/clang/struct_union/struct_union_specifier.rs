use std::fmt::{Display, Formatter};

use pest::iterators::Pair;

use core_utils::debug::*;
use crate::languages::clang::struct_union::*;
use crate::languages::clang::symbol_table::*;
use crate::languages::clang::declaration::resolve_clang_type_for_usage;

use crate::languages::clang::traits::*;
use crate::languages::clang::{ast_item::ASTItem, clang_asm_code_gen::{clang_asm_code_gen::{Footnote, MDTableCell}, code_gen_context::CodeGenContext}, clang_error::CLangError, clang_item::CLangItem, clang_pest::{Id, Rule}, clang_types::CLangTypes, type_specifier::TypeSpecifier};

use crate::languages::clang::clang_asm_code_gen::code_gen_configuration::*;

pub const VAR_NAME_PLACEHOLDER : &str = "<VarName>";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructUnionSpecifier {
    is_struct: bool,
    name: Option<Id>,

    /// items in struct/union, each item has its own type and name, the type can be basic type or composite type
    declares: Vec<StructUnionSpecifier_Declarator>,
    ast_item : ASTItem,

    /// struct/union symbol table
    symbol_table : SymbolTable,

    /// linkage to real defintion for struct/union declaration, it is used to link to the actual struct/union definition for struct/union declaration without definition, e.g. struct A a;
    /// it is set during declaration setup, and it is used to fix the struct/union
    declaration_linkage : Option<CLangItem>,
}

impl StructUnionSpecifier {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::struct_or_union_specifier); //ensure it is rule::struct_or_union_specifier

        let mut struct_union_specifier = StructUnionSpecifier::default();
        struct_union_specifier.ast_item = ASTItem::new(pair.line_col().into());

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::STRUCT => struct_union_specifier.is_struct = true,
                Rule::UNION => struct_union_specifier.is_struct = false,

                Rule::identifier => {
                    struct_union_specifier.name = Some(inner_pair.as_str().to_string())
                }

                //parse the rule::struct_declaration
                Rule::struct_declaration => {
                    let mut inners = inner_pair.clone().into_inner();

                    //parse the specifier_qualifier_list rule
                    let ty = inners
                        .next()
                        .unwrap()
                        .into_inner()
                        .map(|pair| TypeSpecifier::from_pair(&pair, context).unwrap())
                        .collect::<Vec<_>>()
                        .into();

                    //parse the struct_declarator_list rule
                    let declarator_list =
                        inners
                            .next()
                            .unwrap()
                            .into_inner() //fetch the struct_declarator rule
                            .map(|pair| {
                                StructDeclarator::from_pair(&pair, context).unwrap()
                            })
                            .collect();

                    struct_union_specifier
                        .declares
                        .push(StructUnionSpecifier_Declarator::new(ty, declarator_list, & inner_pair))
                }
                _ => {
                    let str = format!(
                        "cannot find {inner_pair:?} in struct_or_union_specifier processing logic"
                    );
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }

        Ok(struct_union_specifier)
    }

    /// get symbol table for struct/union, 
    /// the symbol table is used to store the member definition of struct/union, 
    /// and the symbol table is linked to the parent scope's symbol table, 
    /// which can be namespace or global scope
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// get mutable symbol table for struct/union
    /// struct/union has its own symbol table to store its member definition, and the symbol table is linked to the parent scope's symbol table, which can be namespace or global scope
    pub fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    /// set up symbol table and create linkage for struct/union
    pub fn setup_symbol_table(&mut self, config:&CodeGenConfiguration) -> Result<(), CLangError> {
        let current_node = self.get_clang_item();

        let mut offset = 0;
        for mut item in self.get_symbol_table_elements() {
            let data_type_size = item.populate_type_size_from_data_type(config);
            if data_type_size < 0 {
                error_string(format!("data type size is not set for symbol entry {item:?} in enum specifier {self:?}"));
                return Err(CLangError::NoFound((file!().to_string(), line!()), format!("data type size is not set for symbol entry {item:?} in enum specifier {self:?}")));
            }
            else {
                item.set_storage_location(SymbolLocation::StructUnionMemberOffset(offset));
                if self.is_struct() {
                    offset += data_type_size as i32;
                }
            }
            self.get_symbol_table_mut().insert(item);
        }

        self.symbol_table.set_container(Some(current_node.clone()));

        // Struct/union symbol table is created via declaration setup, not scope traversal.
        // Explicitly link this node to its containing scope so parent lookup/markdown works.
        let parent_scope = self.get_ast_item()
            .get_scope_item()
            .and_then(|x| x.get_scope_ast_item().get_current_node().cloned());
        self.get_ast_item_mut().set_scope_parent(parent_scope);

        // get scope element and add itself to symbol table
        if let Some(scope_item) = self.get_ast_item().get_scope_item() {
            // Add reverse scope linkage so parent symbol table markdown can list this child.
            scope_item.add_child_scope(&current_node);

            let symbol_table = scope_item.get_symbol_table_mut();
            let mut entry = SymbolEntry::from_to_symbol_table_element(self);
            let decl_node = self.get_ast_item().get_parent().cloned().unwrap_or(current_node);
            
            assert!(decl_node.is_declaration());
            
            entry.set_clang_item_link(Some(decl_node));
            symbol_table.insert(entry);
        }

        Ok(())
    }

    /// check if this struct/union declaration is invalid, 
    /// e.g. it has a struct/union type but the actual struct/union definition is not found
    pub fn contains_invalid_struct_union_declaration(&self) -> bool {
        self.get_declares().iter()
            .any(|x| x.contains_invalid_struct_union_declaration())
    }

    /// check if it is union, but it could be just declaration without definition
    pub fn is_union (&self) -> bool {
        !self.is_struct
    }

    /// check if it is struct, but it could be just declaration without definition
    pub fn is_struct(&self) -> bool {
        self.is_struct
    }

    /// get struct or union name, but it could be None for anonymous struct/union
    pub fn get_name_id(&self) -> Option<&String> {
        if let Some(name) = self.name.as_ref() {
            return Some(name);
        }

        // Anonymous struct/union may be named via typedef alias in parent scope symbol table.
        let expected_type = if self.is_struct() {
            SymbolType::Struct
        }
        else {
            SymbolType::Union
        };

        self.get_ast_item()
            .get_scope_item()
            .and_then(|scope| {
                scope
                    .get_symbol_table()
                    .get_all_entries()
                    .iter()
                    .find(|entry| entry.get_id() == self.get_id() && entry.get_symbol_type() == &expected_type)
                    .and_then(|entry| entry.get_type_aliases().first())
            })
    }

    /// get the effective type name; anonymous types use a stable synthetic name
    pub fn get_type_name(&self) -> String {
        self.get_name_id()
            .cloned()
            .unwrap_or_else(|| format!("anonymous_type_{}", self.get_id()))
    }

    pub fn get_declares(&self) -> &Vec<StructUnionSpecifier_Declarator> {
        &self.declares
    }

    pub fn get_declares_mut(&mut self) -> &mut Vec<StructUnionSpecifier_Declarator> {
        &mut self.declares
    }

    pub fn get_linkage(&self) -> Option<&CLangItem> {
        self.declaration_linkage.as_ref()
    }

    pub fn get_linkage_mut(&mut self) -> Option<&mut CLangItem> {
        self.declaration_linkage.as_mut()
    }

    pub fn set_linkage(&mut self, linkage:CLangItem) {
        assert!(linkage.is_struct_or_union());
        self.declaration_linkage = Some(linkage);
    }

    /// get member type for struct/union, the member is identified by id, and the struct/union definition is from this declaration's type
    pub fn get_member_type(&self, id:&str) -> Option<CLangTypes> {
        for declare in self.declares.iter() {
            if declare.get_struct_item_name() == id {
                return Some(declare.get_item_type().clone());
            }
        }

        None
    }

    /// check if this struct/union declaration has definition, it is considered to have definition if it has at least one member declaration with type information, e.g. struct A { int x; }; is a struct definition, while struct A; is a struct declaration without definition, and struct A { int x; } a; is a struct declaration with definition
    pub fn has_struct_def(&self) -> bool {
        let r = self.declares.iter()
            .any(|x| x.get_declarators().len() > 0);
        r
    }

    /// get the real struct/union definition for this struct/union declaration, 
    /// if this struct/union declaration has definition, then return itself, 
    /// otherwise return the linked struct/union definition
    pub fn get_real_definition(&self) -> &Self {
        assert!(self.self_check());

        if let Some(real_def_clang_item) = self.get_linkage() {
            real_def_clang_item.get_struct_or_union()
                .unwrap()
        }
        else {
            self
        }
    }

    pub fn get_real_definition_mut(&mut self) -> &mut Self {
        assert!(self.self_check());

        if self.get_linkage().is_some() {
            self.get_linkage_mut()
                .unwrap()
                .get_struct_or_union()
                .unwrap()
        }
        else {
            self
        }
    }

    /// get the symbol table for struct/union definition, 
    /// if this struct/union is a declaration without definition, 
    /// it will link to the actual struct/union definition and get the symbol table 
    /// from the linked struct/union definition
    pub fn get_definition_symbol_table(&self) -> &SymbolTable {
        assert!(self.self_check());

        self.get_real_definition()
            .get_symbol_table()
    }

    /// internal check for type integrity
    fn self_check(&self) -> bool {
        if self.contains_invalid_struct_union_declaration() {
            let r = !self.has_struct_def();
            return r;
        }
        
        true
    }

    /// get the size of struct/union in bytes
    pub fn get_type_size(&self, config:&CodeGenConfiguration) -> usize {
        let r = if self.is_struct() {
            if self.get_declares().is_empty() {
                if let Some(linked) = self.get_linkage() {
                    linked.get_struct_or_union()
                        .unwrap()
                        .get_type_size(config)
                }
                else {
                    0
                }
            }
            else {
                self.get_declares().iter()
                    .map(|x| x.get_type_size(config))
                    .sum()
            }
        }
        else { // the size of union is the max size of its member
            if self.get_declares().is_empty() {
                if let Some(linked) = self.get_linkage() {
                    linked.get_struct_or_union()
                        .unwrap()
                        .get_type_size(config)
                }
                else {
                    0
                }
            }
            else {
                self.get_declares().iter()
                .map(|x| x.get_type_size(config))
                .max()
                .unwrap()
            }
        };

        r as usize
    }

    /// generate struct to md supported for asm
    pub fn generate_struct_type_items(&self, type_name:&str, config:&CodeGenConfiguration) -> (Vec<MDTableCell>, Vec<Footnote>) {
        let mut current_offset = 0;
        let mut footnotes = vec![];
        let type_size = self.get_type_size(config);
        let mut memory_layout = vec![(".byte", "0").into(); type_size];
        let (_, notes) = self.generate_struct_type_items_internal(&mut memory_layout, type_name.to_string(), &mut current_offset, config);
        footnotes.extend(notes);
        (memory_layout, footnotes)
    }

    /// generate struct to md supported in asm
    /// this function is to set the lables to right location. The memroy layout has been set up at beginning
    fn generate_struct_type_items_internal(&self,  memory_layout:&mut Vec<MDTableCell>, name:String, current_offset:&mut usize, config:&CodeGenConfiguration) -> (Vec<MDTableCell>, Vec<Footnote>) {
        let mut footnotes = Vec::default();

        for nn in self.get_declares().iter() {
            let resolved_type = resolve_clang_type_for_usage(nn.get_item_type(), self.get_ast_item());
            let ty = &resolved_type;
            let id_name = nn.get_struct_item_name();

            match ty {
                CLangTypes::Struct(_, _) => {
                    let n = ty.get_struct().unwrap();
                    let (_, notes) = n.generate_struct_type_items_internal(memory_layout, id_name, current_offset, config);
                    footnotes.extend(notes);
                }
                CLangTypes::Union(_, _) => {
                    let n = ty.get_union().unwrap();
                    let (_, notes) = n.generate_union_type_items_internal(memory_layout, id_name, current_offset, config);
                    footnotes.extend(notes);
                }
                // Enum fields are stored as int32 in memory layout.
                CLangTypes::Enum(_, _) => {
                    let type_size = CLangTypes::Int32.get_type_size_in_bytes2(config).unwrap();
                    let cell = &mut memory_layout[*current_offset];
                    let footnote_name = format!("{}_{}", name, id_name);
                    cell.add_tag(footnote_name.to_string());
                    let mut note : Footnote = footnote_name.into();
                    note.set_comment(format!("offset: {}, type: {} (stored as Int32)", *current_offset, ty));
                    footnotes.push(note);
                    *current_offset += type_size;
                }
                CLangTypes::Array(element_type, name , diemension) => {
                    let type_size = element_type.get_type_size_in_bytes2(config).unwrap() as usize;
                    let indices = diemension.clang_indices_string();

                    for i in 0..diemension.get_total_elements() {
                        let index_string = &indices[i];
                        let cell = &mut memory_layout[*current_offset];
                        let footnote_name = format!("{name}_{id_name}_{}", index_string.join("_"));  //foodnote name
                        cell.add_tag(footnote_name.to_string());
                        let mut note : Footnote = footnote_name.into();
                        note.set_comment(format!("offset: {}, type: {ty}, name : '{name}', array index: {}", *current_offset, index_string.join(", ")));
                        footnotes.push(note);
                        *current_offset += type_size;
                    }
                }
                _ if ty.is_built_in() || ty.is_pointer() || ty.is_functionpointer() => {
                    let type_size = ty.get_type_size_in_bytes2(config).unwrap();
                    let cell = &mut memory_layout[*current_offset];
                    let footnote_name = format!("{}_{}", name, id_name);  //foodnote name
                    cell.add_tag(footnote_name.to_string());
                    let mut note : Footnote = footnote_name.into();
                    note.set_comment(format!("offset: {}, type: {}", current_offset, ty));
                    footnotes.push(note);
                    *current_offset += type_size;
                }            
                _ => {
                    panic!("unsupported type in struct type generation: '{ty}' for '{id_name}'");
                }
            }
        }

        (Vec::default(), footnotes)
    }

    /// generate md for union type for asm
    pub fn generate_union_type_items(&self, type_name:&str, config:&CodeGenConfiguration) -> (Vec<MDTableCell>, Vec<Footnote>) {
        let mut current_offset = 0;
        let type_size = self.get_type_size(config);
        let mut memory_layout = vec![(".byte", "0").into(); type_size];
        let (_, notes) = self.generate_union_type_items_internal(&mut memory_layout, type_name.to_string(), &mut current_offset, config);
        (memory_layout, notes)
    }

    /// generate union to md supported in asm
    /// this function is to set the lables to right location. The memroy layout has been set up at beginning
    fn generate_union_type_items_internal(&self, memory_layout:&mut Vec<MDTableCell>, name:String, current_offset:&mut usize, config:&CodeGenConfiguration) -> (Vec<MDTableCell>, Vec<Footnote>) {
        let mut footnotes = Vec::default();

        for nn in self.get_declares().iter() {
            let ty = nn.get_item_type();
            let id_name = nn.get_struct_item_name();

            match ty {
                CLangTypes::Struct(_,_) => {
                    let n = ty.get_struct().unwrap();
                    let mut new_offset = *current_offset;
                    let (_, notes) = n.generate_struct_type_items_internal(memory_layout, id_name, &mut new_offset, config);
                    footnotes.extend(notes);
                }
                CLangTypes::Union(_,_) => {
                    let n = ty.get_union().unwrap();
                    let mut new_offset = *current_offset;
                    let (_, notes) = n.generate_union_type_items_internal(memory_layout, id_name, &mut new_offset, config);
                    footnotes.extend(notes);
                }
                // Enum fields are stored as int32 in memory layout.
                CLangTypes::Enum(_, _) => {
                    let cell = &mut memory_layout[*current_offset];
                    let footnote_name = format!("{}_{}", name, id_name);
                    cell.add_tag(footnote_name.to_string());
                    let mut note : Footnote = footnote_name.into();
                    note.set_comment(format!("offset: {}, type: {} (stored as Int32)", *current_offset, ty));
                    footnotes.push(note);
                }
                CLangTypes::Array(element_type, name , diemension) => {
                    let memory_base = *current_offset;
                    let indices = diemension.clang_indices_string();

                    for i in 0..diemension.get_total_elements() {
                        let index_string = &indices[i];
                        let element_offset = memory_base + i * element_type.get_type_size_in_bytes2(config).unwrap() as usize;
                        let cell = &mut memory_layout[element_offset];
                        let footnote_name = format!("{}_{}[{}]", name, id_name, index_string.join(", "));  //foodnote name
                        cell.add_tag(footnote_name.to_string());
                        let mut note : Footnote = footnote_name.into();
                        note.set_comment(format!("offset: {}, type: {}, array index: {}", element_offset, ty, index_string.join(", ")));
                        footnotes.push(note);
                    }
                }
                _ if ty.is_built_in() || ty.is_pointer() => {
                    let cell = &mut memory_layout[*current_offset];
                    let footnote_name = format!("{}_{}", name, id_name);  //foodnote name
                    cell.add_tag(footnote_name.to_string());
                    footnotes.push(footnote_name.into());
                }
                _ => {
                    panic!("unsupported type in union type generation: {}", ty);
                }
            }
        }

        (Vec::default(), footnotes)
    }

    pub fn new_footnote(name:&str, comment:&str) -> Footnote {
        Footnote::new(name.to_string(), name.to_string(), format!("{comment}"))
    }

    /// if the type is struct (with fields definition)
    pub fn is_struct_def(&self) -> bool {
        self.is_struct() && self.has_struct_def()
    }

    /// if the type is union (with fields definition)
    pub fn is_union_def(&self) -> bool {
        self.is_union() && self.has_struct_def()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        self.declares.iter_mut()
            .map(|x| x.into())
            .collect()
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        for n in self.declares.iter_mut() {
            n.access_children(context)
        }

        context.access_clang_item(CLangItem::StructUnionSpecifier(CLangItem::new(self)));
    }
}

impl ASTItemTrait for StructUnionSpecifier {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        let linkage_id = if let Some(link) = self.get_linkage() { format!("_L{}", link.get_ast_item().get_sequence_id()) }
                                    else { if self.get_declares().is_empty() { format!("(?)") } 
                                            else { String::default() } 
                                    };
        format!("StructUnionSpecifier({}{linkage_id})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Default for StructUnionSpecifier {
    fn default() -> Self {
        StructUnionSpecifier {
            is_struct: true,
            name: None,
            declares: vec![],
            ast_item : ASTItem::default(),
            symbol_table : SymbolTable::default(),
            declaration_linkage : None,
        }
    }
}

impl Display for StructUnionSpecifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let binding = "<NoName>".to_string();
        let name = self.get_name_id().unwrap_or(&binding);
        let items = self.get_declares();
        // get items string list
        let items_str = items.iter().map(|x| format!("{}", x)).collect::<Vec<_>>().join(", \r\n");
        if self.is_struct() {
            write!(f, "struct {} {{\r\n{items_str}}}", name)
        } else {
            write!(f, "union {} {{\r\n{items_str}}}", name)
        }
    }
}

