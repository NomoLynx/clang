use core_utils::filesystem::read_file_content;
use pest::iterators::Pair;
use core_utils::debug::*;
use crate::languages::clang::clang_pest::ProcessorFile;

use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::{FileName, Rule}, tanry_expression::TanryExpression};
use crate::languages::clang::clang_pest::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PreProcessorDirective {
    Include(ASTItem, FileName),
    Progma(ASTItem, String),
    Define(ASTItem, PreProcessorDirective_Define),
}

impl PreProcessorDirective {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::preprocessor_directives); //Ensure the parsed rule is 'Rule::preprocessor_directives'

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner = pair.to_owned().into_inner().next().unwrap();
        
        match inner.as_rule() {
            Rule::include_directives => {
                let path = inner.into_inner().next().unwrap(); //Fetch the path rule
                let file_name = ProcessorFile::from_pair(&path, context)?.get_file_name();
                let file_path = format!("{}/{}", context.get_base_folder(), file_name);
                match read_file_content(&file_path) {
                    Ok(file_content) => {
                        match parse_clang(&file_content, context) {
                            Ok(mut clang_prog) => {
                                clang_prog.set_source_file_path(&file_name);
                                context.add_unique_included_clang_program(clang_prog);
                                Ok(PreProcessorDirective::Include(
                                    ast_item,
                                    file_name,
                                ))
                            }
                            Err(ex) => {
                                error_string(format!("include file '{file_name}' parse error: {ex:?}"));
                                Err(ex)
                            }
                        }
                    }
                    Err(ex) => {
                        let err = format!("include file '{file_name}' read error: {ex:?}");
                        error_string(err.to_string());
                        Err(CLangError::NoFound((file!().to_string(), line!()), err))
                    }
                }
            }

            Rule::pragma_directives => Ok(PreProcessorDirective::Progma(ast_item, inner.as_str().to_owned())),

            Rule::define_directives => {
                let mut inner_rules = inner.into_inner();

                //fetch the name
                let define_directive_name = inner_rules.next().unwrap().as_str();

                //fetch the value, it is a rule:profix_expression
                let postfix_expression_rule =
                    inner_rules.next().unwrap().into_inner().next().unwrap();

                Ok(PreProcessorDirective::Define(ast_item, 
                    PreProcessorDirective_Define { 
                        file_name: define_directive_name.to_owned(),
                        expression: TanryExpression::from_pair(&postfix_expression_rule, context)?,
                        ast_item : ASTItem::default(),
                    }
                ))
            }

            _ => {
                let str =
                    format!("cannot find {inner:?} in PreProcessor Directive processing logic");
                error_string(str.to_string());
                Err(CLangError::NoFound((file!().to_string(), line!()), str))
            }
        }
    }

    pub fn get_name(&self) -> String {
        format!("PreProcessorDirective({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Include(n, _) => n,
            Self::Progma(n, _) => n,
            Self::Define(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Include(n, _) => n,
            Self::Progma(n, _) => n,
            Self::Define(n, _) => n,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Define(_, n) => vec![(&mut n.expression).into()],
            Self::Include(_, _) => vec![],
            Self::Progma(_, _) => vec![],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Define(_, n) => n.access_children(context),
            Self::Include(_, _n) => (),
            Self::Progma(_, _n) => (),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreProcessorDirective_Define {
    pub file_name : String,
    pub expression : TanryExpression,
    pub ast_item : ASTItem,
}

impl PreProcessorDirective_Define {
    pub fn get_name(&self) -> String {
        format!("PreProcDirective_Def({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![(&mut self.expression).into()]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.expression.access_children(context);
        context.access_clang_item(CLangItem::PreProcessorDirective_Define(CLangItem::new(self)));
    }
}
