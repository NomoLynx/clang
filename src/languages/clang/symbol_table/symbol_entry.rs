use crate::languages::clang::struct_union::StructUnionSpecifier;
use crate::languages::clang::traits::*;
use crate::languages::clang::function_definition::*;
use crate::languages::clang::{ASTItem, CLangError, CLangItem};
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::clang_asm_code_gen::CodeGenConfiguration;
use crate::languages::clang::symbol_table::type_alias::*;
use core_utils::debug::*;
use core_utils::range::Range;

use super::*;

pub type StorageLocations = Vec<SymbolLocation>;
pub type Attribute = (String, String);
pub const ATTRIBUTE_KEY_DATATYPE_SIZE: &str = "DATATYPE_SIZE";

/// Represents an entry in the symbol table
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SymbolEntry {
    id : usize,          // Unique identifier for the symbol entry, which is from the ast item sequence id
    name: String,        // The identifier name
    symbol_type: SymbolType, // The type (e.g., var, function, etc. it does not show the actual data type)
    scope: String,       // The scope (e.g., global, local, etc.)
    attributes: Vec<Attribute>, // Additional attributes (e.g., data type, memory location, value, etc.)
    clang_item_link : Option<CLangItem>,   //link to the clang item, this item can be the parent element
    
    /// symbol storage location, this is mainly for variable
    storage_location : StorageLocations,

    /// data type info
    data_type : Option<CLangTypes>,

    /// whether the variable is address taken
    is_address_taken : bool,

    /// is external item
    is_extern : bool,

    /// type alias, like typedef in c
    alias : TypeAlias,
}

impl SymbolEntry {
    /// Create a new symbol entry with extern set to false
    pub fn new(id : usize, name: &str, symbol_type: SymbolType, scope: &str) -> Self {
        Self {
            id,
            name: name.to_string(),
            symbol_type,
            scope: scope.to_string(),
            attributes: Vec::new(),
            clang_item_link : None,
            storage_location : StorageLocations::default(),
            data_type : None,
            is_extern : false,
            is_address_taken : false,
            alias : TypeAlias::default(),
        }
    }

    /// create new symbol entry for external item
    pub fn new_external_item(id:usize, name: &str, symbol_type: SymbolType, scope: &str) -> Self {
        let mut r = Self::new(id, name, symbol_type, scope);
        r.is_extern = true;
        r
    }

    /// create new symbol entry from ToSymbolTableElement trait object
    pub fn from_to_symbol_table_element<T: ToSymbolTableElement>(item:&T) -> Self {
        let id = item.get_symbol_entry_id();
        let name = item.get_symbol_entry_name();
        let symbol_type = item.get_sybmol_entry_type();
        let scope = item.get_sybmol_entry_scope();
        let attributes_map = item.get_symbol_entry_attributes();

        let mut r = Self::new(id, &name, symbol_type, &scope);
        r.set_attributes(attributes_map.into_iter().map(|(k, v)| (k, v)).collect());

        r
    }

    /// get symbol entry name, like variable name, function name, etc.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn new_var(id:usize, name:&str, scope: &str) -> Self {
        Self::new(id, name, SymbolType::Var, scope)
    }

    /// get symbol data type
    pub fn get_data_type(&self) -> Option<&CLangTypes> {
        self.data_type.as_ref()
    }

    /// get symbol entry id
    pub fn get_id(&self) -> usize {
        self.id
    }

    pub fn get_is_address_taken(&self) -> bool {
        self.is_address_taken
    }

    pub fn set_is_address_taken(&mut self, is_address_taken:bool) {
        self.is_address_taken = is_address_taken;
    }

    /// set symbol data type
    pub fn set_data_type(&mut self, data_type:Option<CLangTypes>) {
        self.data_type = data_type;
    }

    /// get symbol table entry type (not data type)
    pub fn get_symbol_type(&self) -> &SymbolType {
        &self.symbol_type
    }

    /// get symbol storage location
    pub fn get_storage_location(&self) -> &StorageLocations {
        self.storage_location.as_ref()
    }

    /// get single symbol storage location, only when there is one location
    pub fn get_single_storage_location(&self) -> Option<&SymbolLocation> {
        if self.storage_location.len() == 1 {
            Some(&self.storage_location[0])
        }
        else {
            error_string(format!("There are '{}' storage locations for symbol entry '{}' @ '{}', cannot get single storage location", 
                                self.storage_location.len(), 
                                self.get_name(),
                                self.get_id()));
            panic!("There are '{}' storage locations for symbol entry, cannot get single storage location", self.storage_location.len());
        }
    }

    /// get single symbol storage location's member offset
    pub fn get_single_storage_location_member_offset(&self) -> Option<MemberOffset> {
        self.get_single_storage_location()
            .and_then(|x| x.get_struct_union_member_offset().cloned())
    }

     /// get symbol clang type, it will first look at var type attribute, then look at data type, if both are not set, it will look at semantic linkage for type, if still not found, return None
    pub fn get_register_name(&self) -> &str {
        self.get_single_storage_location()
            .and_then(|x| x.get_register_name())
            .expect("can't get register name for location")
    }

    /// check whether any storage location uses the RISC-V a0 register
    pub fn is_a0_register(&self) -> bool {
        self.get_storage_location()
            .iter()
            .filter_map(|x| x.get_register_name())
            .any(|name| name == "a0")
    }

    /// set symbol storage location by adding one location
    pub fn add_storage_location(&mut self, storage_location:SymbolLocation) {
        self.storage_location.push(storage_location);
    }

    /// set symbol storage locations
    pub fn set_storage_locations(&mut self, storage_locations:StorageLocations) {
        self.storage_location = storage_locations;
    }

    /// set symbol storage location, only one location
    pub fn set_storage_location(&mut self, storage_location:SymbolLocation) {
        self.set_storage_locations(vec![storage_location]);
    }

    /// check if the symbol is stored in register only
    pub fn is_register_only(&self) -> bool {
        let r = self.get_storage_location()
            .iter()
            .all(|x| x.is_register_only());
        
        r
    }

    /// get composite type name if any
    /// for exmaple, struct A => "A"
    pub fn get_composite_type_name(&self) -> Option<String> {
        self.get_data_type()
            .and_then(|x| x.get_composite_type_name())
    }

    /// get composite type symbol table if the current entry is composite type
    pub fn get_composite_type_symbol_table(&self) -> Option<&SymbolTable> {
        let clang_item = self.get_clang_item_link()?;
        if clang_item.is_declaration() {
            let decl = clang_item.get_declaration()?;
            decl.get_struct_union()
                .and_then(|x| Some(x.get_definition_symbol_table()))
        }
        else if clang_item.is_structunionspecifier_declarator() {   // this is case like Struct A aa;
            let decl = clang_item.get_struct_union_declarator()?;
            decl.get_item_type()
                .get_struct_union()
                .and_then(|x| Some(x.get_definition_symbol_table()))
        }
        else {
            debug_string(format!("id = {}", clang_item.get_sequence_id()));
            debug_string(format!("data type = {:?}", self.get_data_type()));
            let coomposite_type = clang_item.get_struct_or_union()?;
            let r = coomposite_type.get_definition_symbol_table();
            Some(r)
        }
    }

    /// get stack offset option
    pub fn get_stack_offset(&self) -> Option<StackOffset> {
        let r = self.get_storage_location()
            .iter()
            .filter(|x| x.is_stack());
        for loc in r {
            if let Some(offset) = loc.get_stack_offset() {
                return Some(offset);
            }
        }
        None
    }

    /// check if the symbol entry is enum constant type
    pub fn is_enum_type(&self) -> bool { 
        self.get_symbol_type() == &SymbolType::Enum
    }

    /// check if the symbol entry is struct type
    pub fn is_struct_type(&self) -> bool {
        self.get_symbol_type() == &SymbolType::Struct
    }

    /// check if the symbol entry is union type
    pub fn is_union_type(&self) -> bool {
        self.get_symbol_type() == &SymbolType::Union
    }

    /// check if the symbol entry is struct or union item type
    pub fn is_struct_or_union_item_type(&self) -> bool {
        self.get_symbol_type() == &SymbolType::StructUnionItem
    }

    pub fn get_struct_union_type(&self) -> Result<CLangTypes, CLangError> {
        if self.is_struct_type() || self.is_union_type() {
            if let Some(clang_item) = self.get_clang_item_link() {
                if let Some(decl) = clang_item.get_declaration() {
                    if self.is_struct_type() {
                        let struct_ty = decl.get_struct().unwrap();
                        return Ok(CLangTypes::Struct(struct_ty.clone(), Some(struct_ty.get_clang_item())));
                    }
                    else if self.is_union_type() {
                        let union_ty = decl.get_union().unwrap();
                        return Ok(CLangTypes::Union(union_ty.clone(), Some(union_ty.get_clang_item())));
                    }
                }
            }

            Err(CLangError::NoFound((file!().to_string(), line!()), format!("{} is not a struct or union type", self.get_name())))
        }
        else {
            Err(CLangError::NoFound((file!().to_string(), line!()), format!("{} is not a struct or union type", self.get_name())))
        }
    }

    /// get enum const type
    pub fn get_enum_type(&self) -> Result<CLangTypes, CLangError> {
        if self.is_enum_type() {
            Ok(self.get_data_type().unwrap().clone())
        }
        else {
            Err(CLangError::NoFound((file!().to_string(), line!()), format!("{} is not an enum constant", self.get_name())))
        }
    }

    /// get attribute value by key
    pub fn get_attribute(&self, key: &str) -> Option<String> {
        self.attributes.iter()
            .find_map(|(k, v)| if k == key { Some(v.clone()) } else { None })
    }

    /// get attributes reference
    pub fn get_attributes(&self) -> &Vec<Attribute> {
        &self.attributes
    }

    /// Add an attribute to the entry
    pub fn add_attribute(&mut self, key: &str, value: &str) {
        self.attributes.push((key.to_string(), value.to_string()));
    }

    /// set hashmap attributes to the entry
    pub fn set_attributes(&mut self, attributes:Vec<Attribute>) {
        if attributes.is_empty() {
            return;
        }
        
        self.attributes = attributes;
    }

    /// add a typedef alias to the entry
    pub fn add_type_alias(&mut self, alias_name:&str) {
        if self.alias.iter().any(|x| x == alias_name) {
            return;
        }

        self.alias.push(alias_name.to_string());
    }

    pub fn get_type_aliases(&self) -> &TypeAlias {
        &self.alias
    }

    /// set the clang item link
    pub fn set_clang_item_link(&mut self, clang_item_link:Option<CLangItem>) {
        self.clang_item_link = clang_item_link;
    }

    /// get clang item link
    pub fn get_clang_item_link(&self) -> Option<&CLangItem> {
        self.clang_item_link.as_ref()
    }

    /// get ast_item mut
    pub fn get_ast_item_mut(&mut self) -> Option<&mut ASTItem> {
        self.get_clang_item_link()
            .and_then(|x| Some(x.get_ast_item_mut()))
    }

    /// get ast_item
    pub fn get_ast_item(&self) -> Option<&ASTItem> {
        self.get_clang_item_link()
            .and_then(|x| Some(x.get_ast_item()))
    }

    fn is_global_scope(&self) -> bool {
        self.scope.starts_with("CProg")
    }

    /// is var symbol entry global variable: only its scope is CLangProgram can be global. 
    /// function parameter's scope should be that function
    /// local variable's scope should be compound statement
    pub fn is_global_variable(&self) -> bool {
        self.symbol_type == SymbolType::Var &&
        self.is_global_scope()
    }

    /// check if symbol entry is local variable: 
    /// * symbol type is var and storage location has not been set
    pub fn is_local_var(&self) -> bool {
        let is_symbol_type_var = self.get_symbol_type().is_var();
        let r = is_symbol_type_var && 
                !self.is_global_scope();
        r
    }

    /// check if symbol entry is function definition
    pub fn is_function_def(&self) -> bool {
        let r = self.get_symbol_type() == &SymbolType::Function;
        r
    }

    pub fn is_extern(&self) -> bool {
        self.is_extern
    }

    pub fn get_value_changed(&self) -> bool {
        if let Some(clang_item) = self.get_clang_item_link() {
            let ast_item = clang_item.get_ast_item();
            let r = ast_item.get_semantic_linkages().iter().any(|x| {
                x.get_target().get_ast_item().get_value_changed()
            });
            r
        }
        else {
            false
        }
    }

    /// get function argument length
    pub fn get_function_return_type(&self) -> Result<&CLangTypes, CLangError> {
        if self.is_function_def() {
            self.get_data_type()
                .and_then(|x| x.get_function_return_type())
                .ok_or(CLangError::NoFound((file!().to_string(), line!()), "Return type not found".to_string()))
        }
        else {
            Err(CLangError::NoFound((file!().to_string(), line!()), format!("{} is not a function", self.get_name())))
        }
    }

    /// get range option
    /// None if no ast item link
    pub fn get_range(&self) -> Option<Range> {
        let range = self.get_ast_item().map_or(None, |ast_item| {
            let mut range = ast_item.get_decesant_range();
            if ast_item.is_scope_element() {
                Some(range)
            }
            else {
                let links = ast_item.get_semantic_linkages();
                for link in links {
                    let target_ast_item = link.get_target().get_ast_item();
                    let target_range = target_ast_item.get_decesant_range();
                    range += &target_range;
                }

                Some(range)
            }
        });

        range
    }

    /// get scope element's symbol table, if the entry is not scope element, return None
    pub fn get_scope_symbol_table(&self) -> Option<&SymbolTable> {
        self.get_clang_item_link()
            .and_then(|x| x.get_ast_item().get_scope_item())
            .and_then(|x| Some(x.get_symbol_table()))
    }

    /// convert to register use type, which is used in register allocation
    /// assume the symbol entry has clang item link and has register storage location
    pub fn to_register_use(&self) -> RegisterUseType {
        let clang_item = self.get_clang_item_link().unwrap().clone();
        let register_names = self.get_storage_location()
            .iter()
            .filter_map(|x| {
                match x {
                    SymbolLocation::Register(name) => Some(name.clone()),
                    SymbolLocation::RegisterAndStack(name, _) => Some(name.clone()),
                    _ => None,
                }
            })
            .collect::<Vec<_>>();
        
        let sorted_register_names = {
            let mut names = register_names;
            names.sort();
            names.dedup();
            names
        };

        (self.symbol_type.clone(), clang_item, sorted_register_names, self.get_range().unwrap_or_default()).into()
    }

    /// check whether the variable can use register for storage
    pub fn can_use_register(&self) -> bool {
        let no_used_in_address = !self.used_in_addressing();
        let is_large_data_type = self.is_large_data_type();
        let is_address_taken = self.get_is_address_taken();
        no_used_in_address && !is_large_data_type && !is_address_taken
    }

    /// check whether the variable is large data type (e.g., struct, array, etc.)
    fn is_large_data_type(&self) -> bool {
        if let Some(data_type) = &self.data_type {
            if data_type.is_composite() || data_type.is_array() {
                return true;
            }
        }

        false
    }

    /// check whether the variable is used in addressing mode (e.g., &var to get address of var)
    fn used_in_addressing(&self) -> bool {
        if self.is_global_variable() || self.is_local_var() {
            self.check_semantic_link_items(|x| x.used_in_addressing())
        }
        else {
            false
        }
    }

    fn check_semantic_link_items(&self, check_fn: fn(&ASTItem) -> bool) -> bool {
        if let Some(ast_item) = self.get_ast_item() {
            for link in ast_item.get_semantic_linkages() {
                let target = link.get_target();
                let ast_item = target.get_ast_item();
                if check_fn(ast_item) {
                    return true;
                }
            }

            false
        }
        else {
            false
        }
    }

    pub fn set_data_type_size(&mut self, size:usize) {
        self.add_attribute(ATTRIBUTE_KEY_DATATYPE_SIZE, &size.to_string());
    }

    pub fn get_data_type_size(&self) -> Option<usize> {
        if let Some(size_str) = self.get_attribute(ATTRIBUTE_KEY_DATATYPE_SIZE) {
            if let Ok(size) = size_str.parse::<usize>() {
                return Some(size);
            }
        }

        None
    }

    /// get struct or union specifier if the symbol entry is struct or union type, otherwise return None
    /// if the current symbol entry is like 
    /// struct A {
    ///     struct S s;
    /// }
    /// this function can find full S definition for member s
    pub fn get_struct_union_specifier(&self) -> Option<&StructUnionSpecifier> {
        let is_data_type_struct_or_union = self.data_type.as_ref().map_or(false, |dt| dt.is_composite());
        if is_data_type_struct_or_union {
            let struct_union = self.get_data_type()
                .and_then(|dt| dt.get_struct_union()).unwrap();
            let r = self.get_clang_item_link()
                                .and_then(|x| Some(x.get_ast_item()))
                                .and_then(|x| x.get_struct_or_union_clang_item(struct_union))
                                .and_then(|x| x.get_declaration())
                                .and_then(|x| Some(x.get_type()))
                                .and_then(|x| x.get_struct_union());
            r                     
        }
        else {
            if self.is_struct_type() || self.is_union_type() {
                if let Some(clang_item) = self.get_clang_item_link() {
                    if let Some(decl) = clang_item.get_declaration() {
                        if self.is_struct_type() {
                            let struct_ty = decl.get_struct().unwrap();
                            return Some(struct_ty);
                        }
                        else if self.is_union_type() {
                            let union_ty = decl.get_union().unwrap();
                            return Some(union_ty);
                        }
                    }
                }
            }

            None
        }
    }

    /// update the symbol entry's data type size attribute based on its data type, 
    /// this is used for struct/union member and enum constant to set the size of 
    /// the member/constant based on its data type
    /// also, it returns the type size in bytes if the data type is set, otherwise returns -1
    pub fn populate_type_size_from_data_type(&mut self, config:&CodeGenConfiguration) -> isize {
        if let Some(data_type) = &self.data_type {
            if data_type.is_composite() {
                let struct_union_type = self.get_struct_union_specifier()
                                    .expect("data type is composite but cannot get struct/union specifier");
                let size = struct_union_type.get_type_size(config);
                self.set_data_type_size(size);
                return size as isize;
            }
            else {
                if let Some(size) = data_type.get_type_size_in_bytes2(config) {
                    self.set_data_type_size(size);
                    return size as isize;
                }
            }
        }

        return -1;
    }
}

/// Provide a default implementation for `SymbolEntry`
impl Default for SymbolEntry {
    fn default() -> Self {
        Self {
            id : 0,
            name: String::from("<default>"),
            symbol_type: SymbolType::Unknown,
            scope: String::from("global"),
            attributes: Vec::new(),
            clang_item_link : None,
            storage_location : StorageLocations::default(),
            data_type : None,
            is_extern : false,
            is_address_taken : false,
            alias : TypeAlias::default(),
        }
    }
}

/// Implement ToMarkdownTableRow trait for SymbolEntry
/// This will allow us to convert the symbol entry into a markdown table format
impl ToMarkdownTableRow for SymbolEntry {
    fn get_markdown_header(&self) -> String {
        SymbolTable::get_markdown_header()
    }

    fn to_markdown(&self) -> String {
        let clang_item_name = if let Some(citem) = &self.clang_item_link {
            citem.get_name()
        } else {
            "<NoLink>".to_string()
        };

        let attributes = self
                .attributes
                .iter()
                .map(|(k, v)| format!("({}: {})", k, v))
                .collect::<Vec<_>>()
                .join(", ");
        let global = if self.is_global_variable() { "Yes" } else { "No" };
        let value_changed = if self.get_value_changed() { "Yes" } else { "No" };
        let range = self.get_range()
            .map_or("φ".to_string(), |r| format!("[{r}]"));

        let locations = self.get_storage_location();
        let location_str = if locations.is_empty() {
            "<NoLocation>".to_string()
        } else {
            locations.iter().map(|loc| format!("{loc}")).collect::<Vec<_>>().join(", ")
        };

        let is_extern = if self.is_extern { "Yes" } else { "No" };
        let is_address_taken = if self.is_address_taken { "Yes" } else { "No" };
        let data_type_str = if self.symbol_type == SymbolType::Declaration && self.scope == "FUNCTION" {
            self.get_attribute(ATTRIBUTE_KEY_VARTYPE)
                .unwrap_or_else(|| {
                    self.get_data_type()
                        .map(|dt| dt.to_string())
                        .unwrap_or_else(|| "<NoType>".to_string())
                })
        } else if let Some(dt) = self.get_data_type() {
            dt.to_string()
        } else {
            "<NoType>".to_string()
        };

        let type_alias_str = if (*self.alias).is_empty() { "φ".to_string() } 
                                    else { format!("[{}]", (*self.alias).join(",")) };

        format!("| {} | {} | {} | {} | {data_type_str} | {location_str} | {attributes} | {clang_item_name} | {global} | {value_changed} | {range} | {is_extern} | {is_address_taken} | {type_alias_str} |", 
            self.id,
            self.name, 
            self.symbol_type, 
            self.scope)
    }
}
