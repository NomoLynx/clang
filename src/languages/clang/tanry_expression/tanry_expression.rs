use std::fmt::Display;

use pest::iterators::Pair;

use crate::languages::clang::{clang_types::CLangTypes, traits::ASTItemTrait};
use crate::languages::clang::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule};
use crate::languages::clang::tanry_expression::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TanryExpression {
    bool_expression : Expression,
    true_expression : Option<Expression>,
    false_expression : Option<Expression>,
    ast_item : ASTItem,
}

impl TanryExpression {
    pub fn from_pair(pair:&Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        let location = pair.line_col();
        let ast_item = ASTItem::new(location.into());
        let inners = pair.to_owned()
                                                .into_inner()
                                                .map(|x| (x.as_rule(), x))
                                                .collect::<Vec<_>>();
        
        match inners.as_slice() {
            [(Rule::expression, p)] => {
                let expr = Expression::from_pair(p, context)?;
                Ok(Self { bool_expression: expr, true_expression: None, false_expression: None, ast_item, })
            }
            [(Rule::expression, p), (Rule::expression, p1), (Rule::expression, p2)] => {
                let bool_expression = Expression::from_pair(p, context)?;
                let true_expression = Some(Expression::from_pair(p1, context)?);
                let false_expression = Some(Expression::from_pair(p2, context)?);
                Ok(Self { bool_expression, true_expression, false_expression, ast_item, })
            }
            _ => todo!()
        }
    }

    /// get the id of the boolean expression if it is an atom
    /// this can be function name or variable name
    pub fn get_bool_expr_id(&self) -> Option<&String> {
        self.bool_expression.get_atom_id()
    }

    pub fn get_string_value(&self) -> Option<String> {
        self.bool_expression.get_string_value()
    }

    /// get the boolean expression, it is the bool part in bool ? true : false part
    pub fn get_expression(&self) -> &Expression {
        &self.bool_expression
    }

    /// get the true branch expression's composite definition if any
    /// the tanry expression cannot be tanry expression, it will return None
    pub fn get_composite_definition(&self) -> Option<CLangItem> {
        if self.is_tanry() {
            return None;
        }

        self.get_expression().get_composite_definition()
    }

    /// check whether this is a tanry expression which is of form bool ? true_expr : false_expr
    pub fn is_tanry(&self) -> bool {
        self.false_expression.is_some() || self.true_expression.is_some()
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        if self.is_tanry() {
            self.true_expression.as_ref().unwrap().get_clang_type()
        }
        else {
            self.bool_expression.get_clang_type()
        }
    }

    /// set whether the id value has been changed in the expression
    /// this function assumpt the bool_expression is always Atom
    pub fn set_id_value_changed(&mut self, changed:bool) {
        self.bool_expression.set_id_value_changed(changed);
    }

    pub fn get_branches(&self) -> Option<(&Expression, &Expression)> {
        if self.is_tanry() {
            let left = self.true_expression.as_ref().unwrap();
            let right = self.false_expression.as_ref().unwrap();
            Some((left, right))
        }
        else {
            None
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut v = vec![(&mut self.bool_expression).into()];
        if let Some(n) = self.true_expression.as_mut() {
            v.push(n.into());
        }
        if let Some(n) = self.false_expression.as_mut() {
            v.push(n.into());
        }

        v
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match (self.true_expression.as_mut(), self.false_expression.as_mut()) {
            (Some(n), Some(m)) => {
                n.access_children(context);
                m.access_children(context);
            }
            _ => (),
        }

        self.bool_expression.access_children(context);

        context.access_clang_item(CLangItem::TanryExpression(CLangItem::new(self)));
    }
}

impl ASTItemTrait for TanryExpression {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("TanryExpr({})", self.get_ast_item().get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Default for TanryExpression {
    fn default() -> Self {
        Self { bool_expression: Expression::Atom(ASTItem::default(), Atom::new_id("")), 
                true_expression: None, 
                false_expression: None, 
                ast_item : ASTItem::default() 
            }
    }
}

impl Display for TanryExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        
        let s = match (self.true_expression.as_ref(), self.false_expression.as_ref()) {
            (Some(a), Some(b)) => format!("{} ? {a} : {b}", self.bool_expression),
            (None, None) => format!("{}", self.bool_expression),
            _ => panic!("Invalid format, tanry expression must have both true and false branches")
        };

        write!(f, "{}", s)
    }
}
