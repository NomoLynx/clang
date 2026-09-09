use clang_asm_code_gen::code_gen_context::CodeGenContext;
use clang_pest::parse_clang;
use traits::display_diagram_trait::StateDiagramDisplay;

use core_utils::debug::*;
use crate::languages::{clang::{clang_solution::CLangSolution, sematic_linkage::SemanticLinkage, traits::{symbol_table::AccessSymbolTableTrait, to_markdown_table_row_trait::ToMarkdownTableRow}}, r5asm::assembler::build_asm_solution};
use crate::languages::clang::clang_asm_code_gen::*;
use crate::languages::clang::traits::*;
use core_utils::filesystem::*;

pub (crate) mod clang_pest;
pub (crate) mod clang_error;
pub (crate) mod type_specifier;
pub (crate) mod clang_program;
pub (crate) mod clang_asm_code_gen;
pub (crate) mod tanry_expression;
pub (crate) mod symbol_table;
pub mod ast_item;
pub mod statement;
pub mod clang_item;
pub mod iteration_statement;
pub mod selection_statement;
pub mod label_statement;
pub mod external_declaration;
pub mod constant_expression;
pub mod jump_statement;
pub mod compound_statement;
pub mod preprocessor_directive;
pub mod function_definition;
pub mod declaration;
pub mod traits;
pub mod parameter_declaration;
pub mod assignment_expression;
pub mod clang_types;
pub(crate) mod asm_statement;
pub(crate) mod sematic_linkage;
pub(crate) mod type_system;
pub(crate) mod ast_item_iteration_context;
pub(crate) mod clang_solution;
pub(crate) mod enum_type;
pub(crate) mod struct_union;
pub(crate) mod logic;
pub (crate) mod build_parameters;
pub (crate) mod post_operator;
pub(crate) mod postfix_expression_part;

pub use asm_statement::*;
pub use clang_error::*;
pub use ast_item::*;
pub use clang_item::*;
pub use build_parameters::*;

pub fn get_state_diagram_from_clang_program(full_path:&str) -> Result<String, String> {
    match read_file_content(full_path.to_string()) {
        Ok(input) => {
            let mut config = CodeGenContext::default();
            let folder = get_file_containing_folder(&full_path);
            config.set_base_folder(folder.unwrap());
            match parse_clang(&input, &mut config) {
                Ok(mut r) => {
                    r.set_source_file_path(full_path);
                    let solution = CLangSolution::from_config_and_main_program(&mut config, r);
                    let mut prog = solution.to_one_clang_program();
                    match crate::languages::clang::clang_pest::parse_clang_2nd(&mut prog, &mut config) {
                        Ok(_) => {
                            let md_str = prog.to_state_diagram(config.get_show_semantic_links());
                            output_string(md_str.to_string());
                            Ok(md_str)
                        }
                        Err(ex) => {
                            let err = format!("parse clang error: {ex:?}");
                            error_string(err.to_string());
                            Err(err)
                        }
                    }
                }
                Err(ex2) => {
                    let err = format!("parse clang error: {ex2:?}");
                    error_string(err.to_string());
                    Err(err)
                }
            }
        }
        Err(ex) => {
            let err = format!("symbol table test error: {ex:?}");
            error_string(err.to_string());
            Err(err)
        }
    }
}

/// entry point to build clang source code to asm and binary
pub fn build_clang(para:&CLangBuildParameters) -> Result<(), String> {
    let full_path = &para.full_path;
    let output_folder = para.output_folder.clone();
    let is_print_debug = para.is_print_debug;
    
    match read_file_content(full_path.to_string()) {
        Ok(input) => {
            let mut asm_context = para.into();
            match parse_clang(&input, &mut asm_context) {
                Ok(mut r) => {
                    r.set_source_file_path(full_path);
                    let solution = CLangSolution::from_config_and_main_program(&mut asm_context, r);
                    let mut r = solution.to_one_clang_program();
                    match crate::languages::clang::clang_pest::parse_clang_2nd(&mut r, &mut asm_context) {
                        Ok(_) => {
                            // Keep symbol table output behavior aligned with test_show_visualization.
                            asm_context.write_symbol_table_md_file(&mut r);

                            let code = r.generate_code(&mut asm_context);
                            match code {
                                Ok(_) => {
                                    let s = asm_context.get_code_snippet().clone();
                                    let file_name_without_extension = get_file_name_without_extension(&full_path).unwrap();
                                    let meta_info = r.generate_meta_info();
                                    let mut asm_solution = s.to_asm_solution(&file_name_without_extension, Some(meta_info));

                                    if is_print_debug { 
                                        debug_string(format!("{}", asm_solution)); 
                                        r.get_all_symbol_tables().unwrap()
                                            .iter()
                                            .for_each(|x| debug_string(x.to_markdown()));
                                    }

                                    if let Some(folder) = output_folder {
                                        asm_solution.set_container_path(&folder);
                                        asm_solution.save_to_disk(&folder);
                                    }
                                    else {
                                        asm_solution.save_to_disk("./");
                                    }
                                    
                                    let output_file_name = asm_solution.get_output_file_name();
                                    let mut asm_config = asm_context.get_default_asm_config();
                                    let build_result = build_asm_solution(&asm_solution, &mut asm_config);
                                    match build_result {
                                        Ok(_) => {
                                            if is_print_debug {
                                                output_string(format!("ASM Build {output_file_name} success"));
                                            }
                                        }
                                        Err(ex) => {
                                            error_string(format!("ASM Error - Build {output_file_name} failed: {ex:?}"));
                                            return Err(format!("ASM Build {output_file_name} failed"));
                                        }
                                    }
                                }
                                Err(ex) => {
                                    error_string(format!("generate code error: {ex}"));
                                    return Err(format!("generate code error"));
                                }
                            }
                        }
                        Err(ex) => {
                            // parse_clang_2nd can fail after symbol-table setup; still persist current table snapshot.
                            asm_context.write_symbol_table_md_file(&mut r);
                            error_string(format!("parse clang error: {ex}"));
                            return Err(format!("parse clang error"));
                        }
                    }
                }
                Err(ex2) => {
                    error_string(format!("parse clang error: {ex2}"));
                    return Err(format!("parse clang error"));
                }
            }
        }
        Err(ex) => {
            error_string(format!("symbol table test error: {ex} for file '{full_path}'"));
            return Err(format!("Failed to read file {full_path}"));
        }
    }

    CodeSnippet::reset_unique_id();
    Ok(())
}