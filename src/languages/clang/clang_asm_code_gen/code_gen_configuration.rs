use std::collections::HashMap;
use std::sync::LazyLock;

use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::markdown_lang::to_clr_type::*;
use crate::languages::clang::clang_asm_code_gen::*;

fn load_code_snippet_archive() -> CodeSnippetArchive {
    let mut snippets = CodeSnippetArchive::new();

    snippets.add_snippets(SnippetKey::Int.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping.md")).unwrap());
    snippets.add_snippets(SnippetKey::UInt.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_uint.md")).unwrap());
    snippets.add_snippets(SnippetKey::Byte.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_byte.md")).unwrap());
    snippets.add_snippets(SnippetKey::UByte.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_ubyte.md")).unwrap());
    snippets.add_snippets(SnippetKey::Double.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_double.md")).unwrap());
    snippets.add_snippets(SnippetKey::Float.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_float.md")).unwrap());
    snippets.add_snippets(SnippetKey::Short.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_short.md")).unwrap());
    snippets.add_snippets(SnippetKey::UShort.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_ushort.md")).unwrap());
    snippets.add_snippets(SnippetKey::Long.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_long.md")).unwrap());
    snippets.add_snippets(SnippetKey::String.into(), CodeSnippet::vec_from_markdown(include_str!("../code_snippets/clang_asm_mapping_string.md")).unwrap());

    snippets
}

fn load_clang_op_mapping() -> HashMap<String, String> {
    md_table_to_hash2(include_str!("instruction_mapping.md")).unwrap()
}

static CODE_SNIPPETS: LazyLock<CodeSnippetArchive> = LazyLock::new(load_code_snippet_archive);
static CLANG_OP_MAPPING: LazyLock<HashMap<String, String>> = LazyLock::new(load_clang_op_mapping);

#[derive(Debug, Clone)]
pub struct CodeGenConfiguration {
    /// 8bit, 16 bit, 32 bit addressing, etc.
    addressing_in_bits : u32,      

    /// temp file to store the asm file before being compiled
    asm_temp_file_name : Option<String>,

    /// md file to store assembly language generation
    asm_md_data_file_name: Option<String>,

    /// asm code gen configuration
    asm_code_gen_config : crate::languages::r5asm::code_gen_config::CodeGenConfiguration,

    /// generate asm as temp file for debugging
    generate_debug_asm : bool,

    /// code snippet vector
    code_snippets : &'static CodeSnippetArchive,

    /// clang operator mapping to asm code snippet name
    clang_op_mapping : &'static HashMap<String, String>,

    /// parameter register number
    param_reg_num : u32,

    /// enum default base type size in bytes
    enum_base_type_size_in_bytes : u32,

    /// optional stack size in bytes for generated `.data` stack section
    stack_space_size : Option<u32>,
}

impl CodeGenConfiguration {

    pub fn get_enum_base_type_size_in_bytes(&self) -> u32 {
        self.enum_base_type_size_in_bytes
    }

    /// get the enum base clang type
    pub fn get_enum_base_clang_type(&self) -> CLangTypes {
        match self.enum_base_type_size_in_bytes {
            1 => CLangTypes::Byte,
            2 => CLangTypes::Int16,
            4 => CLangTypes::Int32,
            8 => CLangTypes::Int64,
            _ => panic!("unsupported enum base type size {}", self.enum_base_type_size_in_bytes),
        }
    }

    pub fn get_pointer_type(&self) -> String {
        if self.addressing_in_bits == 8 {
            ".byte".to_string()
        }
        else if self.addressing_in_bits == 16 {
            ".2bytes".to_string()
        }
        else if self.addressing_in_bits == 32 {
            ".word".to_string()
        }
        else if self.addressing_in_bits == 64 {
            ".dword".to_string()
        }
        else {
            panic!("{}", format!("invalid addressing bits {}", self.addressing_in_bits))
        }
    }

    pub fn get_type_asm(&self, ty:&CLangTypes) -> Option<String> {
        match ty {
            CLangTypes::Invalid |
            CLangTypes::None |
            CLangTypes::Void => None,
            CLangTypes::Byte |
            CLangTypes::UByte |
            CLangTypes::UChar |
            CLangTypes::Char => Some(".byte".to_string()),
            CLangTypes::Float64 => Some(".double".to_string()),
            CLangTypes::Float32 => Some(".float".to_string()),
            CLangTypes::UInt32 |
            CLangTypes::Int32 => Some(".word".to_string()),
            CLangTypes::UInt64 |
            CLangTypes::Int64 => Some(".long".to_string()),
            CLangTypes::UInt16 |
            CLangTypes::Int16 => Some(".short".to_string()),
            CLangTypes::Enum(_,_) |
            CLangTypes::Union(_,_) |
            CLangTypes::Struct(_,_) => Some(self.get_pointer_type()),
            CLangTypes::Array(ty, _, _) => self.get_type_asm(ty),
            CLangTypes::FunctionPointer(_, _, _) |
            CLangTypes::Pointer(_, _) => Some(self.get_pointer_type()),
            CLangTypes::TypeAlias(_, n) => 
                n.as_ref()
                 .and_then(|x| self.get_type_asm(x)),
        }
    }

    pub fn get_type_default_value(&self, ty:&CLangTypes) -> Option<String> {
        let default_value = format!("0");
        match ty {
            CLangTypes::Void => None,
            CLangTypes::None => None,
            CLangTypes::Invalid => None,
            CLangTypes::Byte => Some(default_value),
            CLangTypes::UByte => Some(default_value),
            CLangTypes::UChar => Some(default_value),
            CLangTypes::Char => Some(default_value),
            CLangTypes::Float32 => Some(default_value),
            CLangTypes::Float64 => Some(default_value),
            CLangTypes::Int32 => Some(default_value),
            CLangTypes::Int64 => Some(default_value),
            CLangTypes::Int16 => Some(default_value),
            CLangTypes::UInt32 => Some(default_value),
            CLangTypes::UInt64 => Some(default_value),
            CLangTypes::UInt16 => Some(default_value),
            CLangTypes::Enum(_, _) |
            CLangTypes::Struct(_, _) |
            CLangTypes::Union(_, _) => Some(default_value),
            CLangTypes::Array(_, _, n) => {
                let total_size = n.get_total_elements();
                vec!["0"; total_size].join(",").into()
            }
            CLangTypes::FunctionPointer(_, _, _) |
            CLangTypes::Pointer(_, _) => Some(default_value),
            CLangTypes::TypeAlias(_, n) => 
                n.as_ref()
                 .and_then(|x| self.get_type_default_value(x)),
        }
    }

    pub fn get_asm_temp_file_name(&self) -> Option<&String> {
        self.asm_temp_file_name.as_ref()
    }

    pub fn get_asm_data_md_file_name(&self) -> Option<&String> {
        self.asm_md_data_file_name.as_ref()
    }

    pub fn get_asm_codegen_config(&self) -> &crate::languages::r5asm::code_gen_config::CodeGenConfiguration {
        &self.asm_code_gen_config
    }

    pub fn get_asm_codegen_config_mut(&mut self) -> &mut crate::languages::r5asm::code_gen_config::CodeGenConfiguration {
        &mut self.asm_code_gen_config
    }

    pub fn get_stack_space_size(&self) -> Option<u32> {
        self.stack_space_size
    }

    pub fn set_stack_space_size(&mut self, stack_space_size: Option<u32>) {
        self.stack_space_size = stack_space_size;
    }

    /// get the number of parameter registers, for risc-v, it is 8
    pub fn get_parameter_register_number(&self) -> u32 {
        self.param_reg_num
    }

    /// get the addressing in bits
    pub fn get_addressing_in_bits(&self) -> u32 {
        self.addressing_in_bits
    }

    /// get the addressing in bytes
    pub fn get_addressing_in_bytes(&self) -> u32 {
        self.get_addressing_in_bits() / 8
    }

    /// is 64-bit addressing
    pub fn is_64bit_addressing(&self) -> bool {
        self.get_addressing_in_bits() == 64
    }

    /// get code snippet from code category and code key
    /// if not found, the search fall back to the default category: SNIPPET_CATEGORY_INT
    pub fn get_code(&self, snippet_category_key:&CodeSnippetKey, code_key:&str) -> Option<CodeSnippet> {
        if let Some(r) = self.code_snippets.get_snippet(snippet_category_key, code_key) {
            return Some(r);
        }
        else {
            if snippet_category_key.is_address() {
                //try long or int
                let fallback_key = if self.is_64bit_addressing() { SnippetKey::Long } else { SnippetKey::Int };
                let key = CodeSnippetKey::new(fallback_key, snippet_category_key.has_assigned_register());
                if let Some(r) = self.code_snippets.get_snippet(&key, code_key) {
                    return Some(r);
                }
            }

            self.code_snippets.get_snippet(&SnippetKey::default().into(), code_key)
        }   
    }
}

impl Default for CodeGenConfiguration {
    fn default() -> Self {
        Self {
            addressing_in_bits: 64,
            asm_temp_file_name : Some("temp.s".to_string()),
            asm_md_data_file_name : Some("temp.md".to_string()),
            asm_code_gen_config : crate::languages::r5asm::code_gen_config::CodeGenConfiguration::new(false),
            generate_debug_asm : true,
            code_snippets : &CODE_SNIPPETS,
            clang_op_mapping : &CLANG_OP_MAPPING,
            param_reg_num : 8,
            enum_base_type_size_in_bytes : 4,
            stack_space_size : None,
        }
    }
}
