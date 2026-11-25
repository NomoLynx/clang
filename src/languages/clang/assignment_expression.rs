use std::fmt::Display;

use pest::iterators::Pair;
use core_utils::debug::*;
use crate::{languages::clang::{clang_pest::AssignmentOp, clang_types::CLangTypes, traits::ASTItemTrait}};

use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule, constant_expression::ConstantExpression};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentExpression {
    // left_parts: Vec<(UnaryExpression, AssignmentOp)>, type error? should be ConstantExpression
    left_parts: Vec<AssignmentExpressionLeftPart>,
    right_part: ConstantExpression,
    ast_item : ASTItem,
}

impl AssignmentExpression {
    pub fn from_clang(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Vec<Self>, CLangError> {
        //clang is a list of assignment, so get its inner and convert to vec<assignment>
        if pair.as_rule() != Rule::clang_expression {
            return Err(CLangError::ParameterError(format!(
                "expected clang_expression, but got {pair:?}"
            ))); //check if pair is clang_expression
        }

        let inner = pair.to_owned().into_inner();
        let mut r = Vec::default();
        for pair in inner.into_iter() {
            let n = AssignmentExpression::from_pair(&pair, context)?; // iterate the inner pair to get assginment structure, use ? to get content of Result<M,N>
            r.push(n);
        }

        Ok(r)
    }

    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        if pair.as_rule() != Rule::assignment_expression {
            error_string(format!("expected assignment expression, but got match for {:?} at {:?}", pair.as_rule(), pair.line_col()));
        }
        
        assert_eq!(pair.as_rule(), Rule::assignment_expression); //ensure it is Rule::assignment_expression

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut inner_pairs = pair.to_owned().into_inner().collect::<Vec<_>>();

        //only contains the rule:ConstantExpression as last element
        if inner_pairs.len() == 1 {
            Ok(AssignmentExpression {
                left_parts: vec![],
                right_part: ConstantExpression::from_pair(&inner_pairs[0].to_owned(), context)?,
                ast_item,
            })
        } else {
            let right_part_rule = inner_pairs.pop().unwrap();

            let mut assignment_expression = AssignmentExpression {
                left_parts: vec![],
                right_part: ConstantExpression::from_pair(&right_part_rule, context)?,
                ast_item,
            };

            //Since last element of rule:constant has been consumed, then, the left ones in inner_pairs must be pari of assignment_expression_left_part
            assert_eq!(inner_pairs.len() % 2, 0);
            inner_pairs.chunks(2).for_each(|slice| {
                let assignment_operator = AssignmentOp::from_pair(&slice[1]).unwrap();
                let mut constant_expression = ConstantExpression::from_pair(&slice[0], context).unwrap();
                constant_expression.set_id_value_changed(true);
                let assignment_expression_left_part = AssignmentExpressionLeftPart {
                    constant_expression: constant_expression,
                    assignment_op: assignment_operator,
                    ast_item : ASTItem::default(),
                };

                assignment_expression
                    .left_parts
                    .push(assignment_expression_left_part);
            });

            Ok(assignment_expression)
        }
    }

    pub fn get_constant_expr(&self) -> &ConstantExpression {
        &self.right_part
    }

    pub fn get_constant_expr_mut(&mut self) -> &mut ConstantExpression {
        &mut self.right_part
    }

    /// left part is empty, so the value is not being assigned
    pub fn is_constant_expr(&self) -> bool {
        self.left_parts.is_empty()
    }

    pub fn get_left_parts(&self) -> &Vec<AssignmentExpressionLeftPart> {
        &self.left_parts
    }

    pub fn get_left_parts_mut(&mut self) -> &mut Vec<AssignmentExpressionLeftPart> {
        &mut self.left_parts
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        for x in self.left_parts.iter_mut() {
            r.push(x.into());
        }

        r.push((&mut self.right_part).into());

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        for x in self.left_parts.iter_mut() {
            x.access_children(context);
        }

        self.right_part.access_children(context);

        context.access_clang_item(CLangItem::AssignmentExpression(CLangItem::new(self)));
    }
}

impl ASTItemTrait for AssignmentExpression {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("AssignExpr_Assign({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.ast_item.get_sequence_id()
    }
}

impl Display for AssignmentExpression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        for x in self.left_parts.iter() {
            r.push_str(&format!("{} {} ", x.constant_expression, x.assignment_op));
        }
        r.push_str(&format!("{}", self.right_part));
        write!(f, "{}", r)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentExpressionLeftPart {
    constant_expression: ConstantExpression,
    assignment_op: AssignmentOp,
    ast_item : ASTItem,
}

impl AssignmentExpressionLeftPart {
    pub fn get_operator(&self) -> &AssignmentOp {
        &self.assignment_op
    }

    pub fn get_constant_expr(&self) -> &ConstantExpression {
        &self.constant_expression
    }

    pub fn get_constant_expr_mut(&mut self) -> &mut ConstantExpression {
        &mut self.constant_expression
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        self.constant_expression.get_clang_type()
    }

    /// Get the ID of the constant expression if exists, otherwise return None
    /// constant expression can be tanary expression
    /// tanry's boolean expression must be an atom
    pub fn get_constant_expr_id(&self) -> Option<String> {
        self.constant_expression.get_tanary_id()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![(&mut self.constant_expression).into()]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.constant_expression.access_children(context);

        context.access_clang_item(CLangItem::AssignmentExpressionLeftPart(CLangItem::new(self)));
    }
}

impl ASTItemTrait for AssignmentExpressionLeftPart {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("AssignExprLeft({})", self.get_ast_item().get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.ast_item.get_sequence_id()
    }
}
