use pest::iterators::Pair;

use crate::languages::clang::constant_expression::ConstantExpression;

use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::{Id, Rule}, statement::Statement};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LabelStatement {
    Case(ASTItem, LabelStatement_CASE),
    Default(ASTItem, Box<Statement>),
    Id(ASTItem, LabelStatement_Id),
}
impl LabelStatement {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::labeled_statement); //Ensure it is Rule::labeled_statement

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner: Vec<Pair<Rule>> = pair.to_owned().into_inner().collect();

        //If len=3, then it is begining with "CASE"
        if inner.len() == 3 {
            Ok(LabelStatement::Case(
                ast_item,
                LabelStatement_CASE::new(
                ConstantExpression::from_pair(&inner[1], context)?,
                Box::new(Statement::from_pair(&inner[2], context)?)),
            ))
        }
        // if len=2, and first rule is rule:default
        else if inner[0].as_rule() == Rule::DEFAULT {
            Ok(LabelStatement::Default(ast_item, Box::new(Statement::from_pair(
                &inner[1],
                context
            )?)))
        } else {
            Ok(LabelStatement::Id(
                ast_item,
                LabelStatement_Id::new(
                inner[0].as_str().to_string(),
                Box::new(Statement::from_pair(&inner[1], context)?))
            ))
        }
    }

    pub fn get_name(&self) -> String {
        match self {
            Self::Case(_, n) => {
                n.get_name()
            }
            Self::Default(_, n) => n.get_name(),
            Self::Id(_, n) => n.get_name(),
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Case(n, _) => n,
            Self::Default(n, _) => n,
            Self::Id(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Case(n, _) => n,
            Self::Default(n, _) => n,
            Self::Id(n, _) => n,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Case(_, n) => {
                vec![n.into()]
            }
            Self::Default(_, n) => vec![n.as_mut().into()],
            Self::Id(_, n) => vec![n.into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Case(_, n) => {
                n.access_children(context);
            }
            Self::Default(_, n) => n.access_children(context),
            Self::Id(_, n) => n.access_children(context),
        }
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelStatement_CASE {
    pub constant_expression: ConstantExpression,
    pub statement: Box<Statement>,
    pub ast_item: ASTItem,
}

impl LabelStatement_CASE {
    pub fn new(constant_expression: ConstantExpression, statement: Box<Statement>) -> Self {
        Self {
            constant_expression,
            statement,
            ast_item: ASTItem::default(),
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_name(&self) -> String {
        format!("Lable_CASE({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![(&mut self.constant_expression).into(), self.statement.as_mut().into()]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        self.constant_expression.access_children(context);
        self.statement.access_children(context);
        context.access_clang_item(CLangItem::LabelStatement_CASE(CLangItem::new(self)));
    }
}

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelStatement_Id {
    pub id: Id,
    pub statement: Box<Statement>,
    pub ast_item: ASTItem,
}

impl LabelStatement_Id {
    pub fn new(id: Id, statement: Box<Statement>) -> Self {
        Self {
            id,
            statement,
            ast_item: ASTItem::default(),
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![self.statement.as_mut().into()]
    }

    pub fn get_name(&self) -> String {
        format!("LabelStat_Id({})", self.get_ast_item().get_sequence_id())
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        self.statement.access_children(context);
        context.access_clang_item(CLangItem::LabelStatement_Id(CLangItem::new(self)));
    }
    
}