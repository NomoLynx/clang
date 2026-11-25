use pest::iterators::Pair;

use crate::{languages::clang::{iteration_statement::IterationStatement, jump_statement::JumpStatement, label_statement::LabelStatement, selection_statement::SelectionStatement}};
use super::assignment_expression::AssignmentExpression;
use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::{CLangExpression, Rule}, compound_statement::CompoundStatement, type_specifier::TypeSpecifier};
use crate::languages::clang::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Statement {
    Jump(ASTItem, JumpStatement),
    Iteration(ASTItem, IterationStatement),
    Selection(ASTItem, SelectionStatement),
    Expression(ASTItem, ExpressionStatement),
    Compound(ASTItem, Box<CompoundStatement>),
    Label(ASTItem, LabelStatement),
    AsmStatement(ASTItem, AsmStatement), //Inline assembly statement
}

impl Statement {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::statement); //Ensure it is Rule::statement

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner_pair: Vec<Pair<Rule>> = pair.to_owned().into_inner().collect();
        
        match inner_pair[0].as_rule() {
            Rule::jump_statement => Ok(Statement::Jump(ast_item, JumpStatement::from_pair(&inner_pair[0], context)?)),

            Rule::iteration_statement => Ok(Statement::Iteration(ast_item, IterationStatement::from_pair(
                &inner_pair[0],
                context
            )?)),

            Rule::selection_statement => Ok(Statement::Selection(ast_item, SelectionStatement::from_pair(
                &inner_pair[0],
                context
            )?)),

            Rule::expression_statement => Ok(Statement::Expression(ast_item, 
                ExpressionStatement::from_pair(&inner_pair[0], context)?,
            )),

            Rule::compound_statement => Ok(Statement::Compound(ast_item, Box::new(
                CompoundStatement::from_pair(&inner_pair[0], context)?,
            ))),

            Rule::labeled_statement => {
                Ok(Statement::Label(ast_item, LabelStatement::from_pair(&inner_pair[0], context)?))
            }
            Rule::asm_statement => {
                Ok(Statement::AsmStatement(ast_item, AsmStatement::from_pair(&inner_pair[0], context)?))
            }
            _ => {
                let str = format!("cannot find {inner_pair:?} in Statement processing logic");
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn get_jump(&self) -> Option<&JumpStatement> {
        match self {
            Self::Jump(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_iteration(&self) -> Option<&IterationStatement> {
        match self {
            Self::Iteration(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_selection(&self) -> Option<&SelectionStatement> {
        match self {
            Self::Selection(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_expression(&self) -> Option<&ExpressionStatement> {
        match self {
            Self::Expression(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_compound_statement(&self) -> Option<&CompoundStatement> {
        match self {
            Self::Compound(_, n) => Some(&**n),
            _ => None,
        }
    }

    pub fn get_compound_statement_mut(&mut self) -> Option<&mut CompoundStatement> {
        match self {
            Self::Compound(_, n) => Some(&mut **n),
            _ => None,
        }
    }

    pub fn get_label(&self) -> Option<&LabelStatement> {
        match self {
            Self::Label(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Jump(ast_item, _) => ast_item,
            Self::Iteration(ast_item, _) => ast_item,
            Self::Selection(ast_item, _) => ast_item,
            Self::Expression(ast_item, _) => ast_item,
            Self::Compound(ast_item, _) => ast_item,
            Self::Label(ast_item, _) => ast_item,
            Self::AsmStatement(ast_item, _) => ast_item,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Jump(ast_item, _) => ast_item,
            Self::Iteration(ast_item, _) => ast_item,
            Self::Selection(ast_item, _) => ast_item,
            Self::Expression(ast_item, _) => ast_item,
            Self::Compound(ast_item, _) => ast_item,
            Self::Label(ast_item, _) => ast_item,
            Self::AsmStatement(ast_item, _) => ast_item,
        }
    }

    pub fn get_name(&self) -> String {
        format!("Stat({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Compound(_, n) => vec![n.as_mut().into()],
            Self::Expression(_, n) => vec![n.into()],
            Self::Iteration(_, n) => vec![n.into()],
            Self::Jump(_, n) => vec![n.into()],
            Self::Label(_, n) => vec![n.into()],
            Self::Selection(_, n) => vec![n.into()],
            Self::AsmStatement(_, _) => vec![],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Compound(_, n) => n.access_children(context),
            Self::Expression(_, n) => n.access_children(context),
            Self::Iteration(_, n) => n.access_children(context),
            Self::Jump(_, n) => n.access_children(context),
            Self::Label(_, n) => n.access_children(context),
            Self::Selection(_, n) => n.access_children(context),
            Self::AsmStatement(_, _) => (),
        }

        context.access_clang_item(CLangItem::Statement(CLangItem::new(self)));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpressionStatement {
    declaration_specifiers: Vec<TypeSpecifier>,
    expr: CLangExpression,
    ast_item: ASTItem,
}
impl ExpressionStatement {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::expression_statement); //Ensure it is Rule::expression_statement

        let mut expression_statment = ExpressionStatement {
            declaration_specifiers: vec![],
            expr: vec![],
            ast_item : ASTItem::default(),
        };

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::expr_declaration_specifiers => {
                    expression_statment.declaration_specifiers = inner_pair
                        .into_inner()
                        .map(|pair| TypeSpecifier::from_pair(&pair, context).unwrap())
                        .collect()
                }

                Rule::clang_expression => {
                    expression_statment.expr = AssignmentExpression::from_clang(&inner_pair, context)?;
                }

                _ => {
                    let str = format!(
                        "cannot find {inner_pair:?} in expression_statement processing logic"
                    );
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }
        Ok(expression_statment)
    }

    pub fn has_type_specifier(&self) -> bool {
        !self.declaration_specifiers.is_empty()
    }

    pub fn get_assignments(&self) -> &Vec<AssignmentExpression> {
        &self.expr
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];
        self.declaration_specifiers.iter_mut().for_each(|x| r.push(x.into()));
        self.expr.iter_mut().for_each(|x| r.push(x.into()));   
        r     
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        for x in self.declaration_specifiers.iter_mut() {
            x.access_children(context);
        }

        for x in self.expr.iter_mut() {
            x.access_children(context);
        }
        
        context.access_clang_item(CLangItem::ExpressionStatement(self));
    }
}

impl ASTItemTrait for ExpressionStatement {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("ExprStat({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}