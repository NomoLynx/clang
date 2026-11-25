use std::fmt::Display;

use pest::iterators::Pair;
use rust_macro::GenIsEnumVariant;

use crate::languages::clang::clang_pest::{Id, ParameterType};
use crate::languages::clang::constant_expression::*;
use crate::languages::clang::{ASTItem, CLangItem, clang_pest::Rule};
use crate::languages::clang::CLangError;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use core_utils::debug::*;
use crate::languages::clang::traits::*;

/// pest rule : direct_declarator_part
#[derive(Debug, Clone, PartialEq, Eq, GenIsEnumVariant)]
pub enum DirectDelaratorPart {
    Constant(ASTItem, ConstantExpression),
    // Parameter(Vec<ParameterType>),
    Parameter(ASTItem, ParameterType),
    Id(ASTItem, Vec<Id>),
}

impl DirectDelaratorPart {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Option<Self>, CLangError> {
        if pair.as_rule() != Rule::direct_declarator_part {
            error_string(format!("expected direct_declarator_part, but got {pair:?}"));
            return Err(CLangError::ParameterError(format!(
                "expected direct_declarator_part, but got {pair:?}"
            )));
        }

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner_pair = pair.to_owned().into_inner().nth(0);

        if inner_pair.is_none() {
            return Ok(None);
        }

        let inner_pair = inner_pair.unwrap();

        match inner_pair.as_rule() {
            Rule::constant_expression => Ok(Some(DirectDelaratorPart::Constant(
                ast_item,
                ConstantExpression::from_pair(&inner_pair, context)?,
            ))),

            Rule::parameter_type_list => Ok(Some(DirectDelaratorPart::Parameter(
                ast_item,
                ParameterType::from_pair(&inner_pair, context)?,
            ))),

            Rule::identifier_list => {
                let ids: Vec<Id> = inner_pair
                    .into_inner()
                    .map(|pair| pair.as_str().to_string())
                    .collect();
                Ok(Some(DirectDelaratorPart::Id(ast_item, ids)))
            }

            _ => {
                let str =
                    format!("cannot find {inner_pair:?} in primary_expression processing logic");
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn is_constant_expr(&self) -> bool {
        matches!(self, Self::Constant(_, _))
    }

    pub fn get_parameters(&self) -> Option<&ParameterType> {
        match self {
            Self::Parameter(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_constant_expression(&self) -> Option<&ConstantExpression> {
        match self {
            Self::Constant(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Constant(_, n) => vec![n.into()],
            Self::Id(_, _) => vec![],
            Self::Parameter(_, n) => vec![n.into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Constant(_, n) => n.access_children(context),
            Self::Id(_, _n) => (),
            Self::Parameter(_, n) => n.access_children(context)
        }

        context.access_clang_item(CLangItem::DirectDelaratorPart(CLangItem::new(self)));
    }
}

impl ASTItemTrait for DirectDelaratorPart {
    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Constant(n, _) => n,
            Self::Id(n, _) => n,
            Self::Parameter(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Constant(n, _) => n,
            Self::Id(n, _) => n,
            Self::Parameter(n, _) => n,
        }
    }

    fn get_name(&self) -> String {
        format!("DirectDeclPart({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for DirectDelaratorPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Constant(_, n) => write!(f, "[{}]", n),
            Self::Id(_, n) => write!(f, "({})", n.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(",")),
            Self::Parameter(_, n) => write!(f, "({})", n),
        }
    }
}