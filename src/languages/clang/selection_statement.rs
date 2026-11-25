use pest::iterators::Pair;

use super::assignment_expression::AssignmentExpression;

use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::{CLangExpression, Rule}, statement::Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectionStatement {
    IF(ASTItem, SelectionStatement_IF),
    SWITCH(ASTItem, SelectionStatement_SWITCH),
}

impl SelectionStatement {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::selection_statement); //Ensure it is Rule::selection_statement

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner: Vec<Pair<Rule>> = pair.to_owned().into_inner().collect();
        
        //if it begins with rule:if
        if inner[0].as_rule() == Rule::IF {
            let r = AssignmentExpression::from_clang(&inner[1], context)?;
            let if_statement = Box::new(Statement::from_pair(&inner[2], context)?);
            let mut else_statment = None;
            // if len!=3, means it contains the else statement
            if inner.len() != 3 {
                else_statment = Some(Box::new(Statement::from_pair(&inner[4], context)?));
            }
            Ok(SelectionStatement::IF(ast_item, SelectionStatement_IF::new(r, if_statement, else_statment)))
        } else {
            let r = AssignmentExpression::from_clang(&inner[1], context)?;
            let s = Box::new(Statement::from_pair(&inner[2], context)?);
            Ok(SelectionStatement::SWITCH(ast_item, SelectionStatement_SWITCH::new(r, s)))
        }
    }

    pub fn get_boolean_expression(&self) -> &CLangExpression {
        match self {
            Self::IF(_, n) => n.get_boolean_expression(),
            Self::SWITCH(_, n) => n.get_boolean_expression(),
        }
    }

    pub fn get_if_branch_statement(&self) -> &Box<Statement> {
        match self {
            Self::IF(_, n) => n.get_if_branch_statement(),
            Self::SWITCH(_, n) => n.get_switch_statement(),
        }
    }

    pub fn get_else_branch_statement(&self) -> Option<&Box<Statement>> {
        match self {
            Self::IF(_, n) => n.get_else_branch_statement(),
            _ => None
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::IF(n, _) => n,
            Self::SWITCH(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::IF(n, _) => n,
            Self::SWITCH(n, _) => n,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![match self {
            Self::IF(_, n) => n.into(),
            Self::SWITCH(_, n) => n.into(),
        }]
    }

    pub fn get_name(&self) -> String {
        format!("SelectionStat({})", self.get_ast_item().get_sequence_id())
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::IF(_, n) => {
                n.access_children(context);
            }
            Self::SWITCH(_, n) => {
                n.access_children(context);
            }
        }

        context.access_clang_item(CLangItem::SelectionStatement(CLangItem::new(self)));
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionStatement_IF {
    pub expression: CLangExpression,
    pub statement: Box<Statement>,
    pub else_statement: Option<Box<Statement>>,
    pub ast_item: ASTItem,
}

impl SelectionStatement_IF {
    pub fn new(expression: CLangExpression, statement: Box<Statement>, else_statement: Option<Box<Statement>>) -> Self {
        Self { expression, statement, else_statement, ast_item: ASTItem::default() }
    }

    pub fn get_name(&self) -> String {
        format!("IF({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_boolean_expression(&self) -> &CLangExpression {
        &self.expression
    }

    pub fn get_if_branch_statement(&self) -> &Box<Statement> {
        &self.statement
    }

    pub fn get_else_branch_statement(&self) -> Option<&Box<Statement>> {
        self.else_statement.as_ref()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut v = vec![self.statement.as_mut().into()];
        for x in self.expression.iter_mut() {
            v.push(x.into());
        }

        if let Some(n) = self.else_statement.as_mut() {
            v.push(n.as_mut().into());
        }

        v
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.statement.access_children(context);
        for x in self.expression.iter_mut() {
            x.access_children(context);
        }

        if let Some(n) = self.else_statement.as_mut() {
            n.access_children(context);
        }

        context.access_clang_item(CLangItem::SelectionStatement_IF(CLangItem::new(self)));
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionStatement_SWITCH {
    pub expression: CLangExpression,
    pub statement: Box<Statement>,
    pub ast_item: ASTItem,
}

impl SelectionStatement_SWITCH {
    pub fn new(expression: CLangExpression, statement: Box<Statement>) -> Self {
        Self { expression, statement, ast_item: ASTItem::default() }
    }

    pub fn get_name(&self) -> String {
        format!("SWITCH({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_boolean_expression(&self) -> &CLangExpression {
        &self.expression
    }

    pub fn get_switch_statement(&self) -> &Box<Statement> {
        &self.statement
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut v = vec![self.statement.as_mut().into()];
        for x in self.expression.iter_mut() {
            v.push(x.into());
        }
        v
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.statement.access_children(context);
        for x in self.expression.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::SelectionStatement_SWITCH(CLangItem::new(self)));
    }
}