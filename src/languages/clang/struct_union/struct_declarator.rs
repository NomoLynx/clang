use std::fmt::{Display, Formatter};

use pest::iterators::Pair;

use core_utils::debug::*;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::clang_pest::Rule;
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::{ASTItem, constant_expression::ConstantExpression};
use crate::languages::clang::{CLangError, CLangItem};
use crate::languages::clang::declaration::*;

pub const STRUCT_DECLARATOR_ANONYMOUS : &str = "anonymous";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructDeclarator {
    declarator : Option<Declarator>,
    expr : Option<ConstantExpression>,
    ast_item: ASTItem,
}

impl StructDeclarator {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::struct_declarator);

        let ast_item = ASTItem::default();
        let inner_pairs = pair.to_owned().into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match inner_pairs.as_slice() {
            [(Rule::declarator, p)] => Ok(Self { declarator : Some(Declarator::from_pair(p, context)?), expr : None, ast_item, }),
            _ => {
                let str = format!(
                    "cannot find {inner_pairs:?} in StructDeclarator processing logic"
                );
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn get_name_id(&self) -> Option<&Declarator> {
        self.declarator.as_ref()
    }

    pub fn get_id_name(&self) -> String {
        if let Some(declarator) = &self.declarator {
            declarator.get_id_name()
        } else {
            STRUCT_DECLARATOR_ANONYMOUS.to_string()
        }
    }

    pub fn get_init(&self) -> Option<&ConstantExpression> {
        self.expr.as_ref()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];
        if let Some(n) = self.declarator.as_mut() {
            r.push(n.into());
        }

        if let Some(n) = self.expr.as_mut() {
            r.push(n.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        if let Some(n) = self.declarator.as_mut() {
            n.access_children(context);
        }

        if let Some(n) = self.expr.as_mut() {
            n.access_children(context);
        }

        context.access_clang_item(CLangItem::StructDeclarator(CLangItem::new(self)));
    }
}

impl ASTItemTrait for StructDeclarator {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("StructDecl({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for StructDeclarator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name_id().unwrap())?;
        if let Some(expr) = self.get_init() {
            write!(f, " = {}", expr)?;
        }
        write!(f, ",\n")
    }
}
