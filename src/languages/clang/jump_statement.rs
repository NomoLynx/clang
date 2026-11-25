use pest::iterators::Pair;

use core_utils::debug::*;
use super::assignment_expression::AssignmentExpression;

use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::{CLangExpression, Id, Rule}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JumpStatement {
    Go(ASTItem, Id),
    Continue(ASTItem),
    Break(ASTItem),
    Return(ASTItem, CLangExpression),
}

impl JumpStatement {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::jump_statement); //Ensure the parsed rule is 'Rule::jump_statement'

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();
        match inner.as_slice() {
            [(Rule::GOTO, _), (Rule::identifier, p)] => {
                let r = Self::Go(ast_item, p.as_str().to_string());
                Ok(r)
            }
            [(Rule::CONTINUE, _)] => Ok(Self::Continue(ast_item)),
            [(Rule::BREAK, _)] => Ok(Self::Break(ast_item)),
            [(Rule::RETURN, _), (Rule::clang_expression, p)] => {
                let r = AssignmentExpression::from_clang(p, context)?;
                Ok(Self::Return(ast_item, r))
            }
            _ => {
                let str = format!("cannot find {inner:?} in jump statement processing logic");
                error_string(str.to_string());
                Err(CLangError::NoFound((file!().to_string(), line!()), str))
            }
        }
    }

    /// check if the jump statement is a return statement
    pub fn is_return(&self) -> bool {
        match self {
            Self::Return(_, _) => true,
            _ => false,
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Break(n) => n,
            Self::Continue(n) => n,
            Self::Go(n, _) => n,
            Self::Return(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Break(n) => n,
            Self::Continue(n) => n,
            Self::Go(n, _) => n,
            Self::Return(n, _) => n,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Break(_) => vec![],
            Self::Continue(_) => vec![],
            Self::Go(_, _) => vec![],
            Self::Return(_, n) => n.iter_mut().map(|x| x.into()).collect(),
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Self::Break(_) => format!("BREAK_({})", self.get_ast_item().get_sequence_id()),
            Self::Continue(_) => format!("CONTINUE_({})", self.get_ast_item().get_sequence_id()),
            Self::Go(_, n) => format!("GO_{n}_({})", self.get_ast_item().get_sequence_id()),
            Self::Return(_, _n) => format!("RETURN_({})", self.get_ast_item().get_sequence_id()),
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Break(_) |
            Self::Continue(_) => (),
            Self::Go(_, _) => (),
            Self::Return(_, n) => { 
                for x in n {
                    x.access_children(context);
                } 
            }
        }

        context.access_clang_item(CLangItem::JumpStatement(CLangItem::new(self)));
    }
}
