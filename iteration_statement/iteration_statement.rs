use pest::iterators::Pair;

use core_utils::debug::*;
use crate::languages::clang::iteration_statement::*;
use crate::languages::clang::assignment_expression::AssignmentExpression;
use crate::languages::clang::traits::*;

use crate::languages::clang::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule, statement::{ExpressionStatement, Statement}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IterationStatement {
    WHILE(ASTItem, IterationStatement_WHILE),
    DO(ASTItem, IterationStatement_DO),
    FOR(ASTItem, IterationStatement_FOR),
}

impl IterationStatement {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::iteration_statement); //Ensure it is Rule::iteration_statement

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut inner_pairs = pair.to_owned().into_inner();
        let first_inner_rule = inner_pairs.next().unwrap();

        match first_inner_rule.as_rule() {
            Rule::WHILE => {
                let r = AssignmentExpression::from_clang(&inner_pairs.next().unwrap(), context)?;
                let s = Box::new(Statement::from_pair(&inner_pairs.next().unwrap(), context)?);
                Ok(IterationStatement::WHILE(ast_item, IterationStatement_WHILE::new(r, s)))
            }

            Rule::DO => {
                let s = Box::new(Statement::from_pair(&inner_pairs.next().unwrap(), context)?);
                inner_pairs.next(); //consumed the rule:WHILE
                let r = AssignmentExpression::from_clang(&inner_pairs.next().unwrap(), context)?;
                Ok(IterationStatement::DO(ast_item, IterationStatement_DO::new(s, r)))
            }

            Rule::FOR => {
                //parse the Rule::expression_statement until its collections is done
                let mut expression_statements = vec![];
                let mut clang_expression = None;
                while let Some(_) = inner_pairs
                    .peek()
                    .filter(|pair| pair.as_rule() == Rule::expression_statement)
                {
                    let pair = inner_pairs.next().unwrap(); //consumed this rule
                    expression_statements.push(ExpressionStatement::from_pair(&pair, context)?);
                }

                if expression_statements.len() > 2 {
                    let str = format!(
                        "FOR loop can only have up to 2 expression statements in init and condition, found {}",
                        expression_statements.len()
                    );
                    error_string(str.to_string());
                    return Err(CLangError::GeneralError(
                        (file!().to_string(), line!()),
                        str,
                    ));
                }

                //if rule:clang_expression exists, parse it
                if let Some(_) = inner_pairs
                    .peek()
                    .filter(|pair| pair.as_rule() == Rule::clang_expression)
                {
                    let pair = inner_pairs.next().unwrap();
                    clang_expression = Some(AssignmentExpression::from_clang(&pair, context)?);
                }

                //parse the rule:statement
                let statment = Box::new(Statement::from_pair(&inner_pairs.next().unwrap(), context)?);

                Ok(IterationStatement::FOR(ast_item, IterationStatement_FOR::new(
                    expression_statements,
                    clang_expression,
                    statment,
                )))
            }

            _ => {
                let str = format!(
                    "cannot find {first_inner_rule:?} in iteration_statement processing logic"
                );
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![match self {
            Self::DO(_, n) => n.into(),
            Self::FOR(_, n) => n.into(),
            Self::WHILE(_, n) => n.into(),
        }]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::DO(_, n) => {
                n.access_children(context);
            }
            Self::FOR(_, n) => {
                n.access_children(context);
            }
            Self::WHILE(_, n) => {
                n.access_children(context);
            }
        }

        // Keep wrapper iteration nodes in the visited list so they receive
        // unique sequence IDs used by diagram traversal de-duplication.
        context.access_clang_item(CLangItem::IterationStatement(CLangItem::new(self)));
    }
}

impl ASTItemTrait for IterationStatement {
    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::DO(n, _) => n,
            Self::FOR(n, _) => n,
            Self::WHILE(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::DO(n, _) => n,
            Self::FOR(n, _) => n,
            Self::WHILE(n, _) => n,
        }
    }

    fn get_name(&self) -> String {
        format!("Iterate({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}
