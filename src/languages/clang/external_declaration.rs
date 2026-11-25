use pest::iterators::Pair;
use core_utils::debug::*;
use crate::{languages::clang::{clang_program::CLangProgram, function_definition::FunctionDefinition}};

use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule, declaration::Declaration, preprocessor_directive::PreProcessorDirective};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalDeclaration {
    PreProcessor(ASTItem, PreProcessorDirective),
    Function(ASTItem, FunctionDefinition),
    Declaration(ASTItem, Declaration),
    Program(ASTItem, Box<CLangProgram>),    // this is used for included files added by preprocessor and later parsed as CLangProgram
}
impl ExternalDeclaration {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::external_declaration); //Ensure the parsed rule is 'Rule::external_declaration'

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner = pair.to_owned().into_inner().next().unwrap();

        match inner.as_rule() {
            Rule::preprocessor_directives => Ok(ExternalDeclaration::PreProcessor(ast_item, PreProcessorDirective::from_pair(&inner, context)?)),
            Rule::function_definition => Ok(ExternalDeclaration::Function(ast_item, FunctionDefinition::from_pair(&inner, context)?)),
            Rule::declaration => Ok(ExternalDeclaration::Declaration(ast_item, Declaration::from_pair(&inner, context)?)),
            _ => {
                // no need to parse Program here, it will be handled in CLangSolution when handling included files
                let str = format!("cannot find {inner:?} in ExternalDeclaration processing logic");
                error_string(str.to_string());
                Err(CLangError::NoFound((file!().to_string(), line!()), str))
            }
        }
    }

    pub fn get_function(&self) -> Option<&FunctionDefinition> {
        match self {
            Self::Function(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_function_mut(&mut self) -> Option<&mut FunctionDefinition> {
        match self {
            Self::Function(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_declaration(&self) -> Option<&Declaration> {
        match self {
            Self::Declaration(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_declaration_mut(&mut self) -> Option<&mut Declaration> {
        match self {
            Self::Declaration(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_clang_program(&self) -> Option<&CLangProgram> {
        match self {
            Self::Program(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::PreProcessor(_, n) => vec![n.into()],
            Self::Function(_, n) => vec![n.into()],
            Self::Declaration(_, n) => vec![n.into()],
            Self::Program(_, n) => vec![(&mut **n).into()],
        }
    }

    pub fn get_name(&self) -> String {
        format!("ExternalDecl({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::PreProcessor(n, _) => n,
            Self::Function(n, _) => n,
            Self::Declaration(n, _) => n,
            Self::Program(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::PreProcessor(n, _) => n,
            Self::Function(n, _) => n,
            Self::Declaration(n, _) => n,
            Self::Program(n, _) => n,
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        match self {
            Self::Declaration(_, n) => n.access_children(context),
            Self::Function(_, n) => n.access_children(context),
            Self::PreProcessor(_, n) => n.access_children(context),
            Self::Program(_, n) => n.access_children_deep_first(context),
        }

        //context.access_clang_item(self.into());
    } 
}
