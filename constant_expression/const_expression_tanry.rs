use crate::languages::clang::assignment_expression::AssignmentExpression;
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::traits::*;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::postfix_expression_part::*;
use crate::languages::clang::constant_expression::*;
use crate::languages::clang::{ASTItem, CLangItem};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstantExpression_Tanry {
    tanry_expression: TanryExpression,
    postfix_expression_parts: Vec<PostfixExpressionPart>,
    ast_item : ASTItem,
}

impl ConstantExpression_Tanry {

    pub fn new(tanry_expression:TanryExpression, postfix_expression_parts:Vec<PostfixExpressionPart>, ast_item:ASTItem) -> Self {
        Self { tanry_expression, postfix_expression_parts, ast_item, }
    }

    pub fn get_tanry_expression(&self) -> &TanryExpression {
        &self.tanry_expression
    }

    pub fn get_tanry_expression_mut(&mut self) -> &mut TanryExpression {
        &mut self.tanry_expression
    }

    pub fn get_postfix_expression_parts(&self) -> &Vec<PostfixExpressionPart> {
        &self.postfix_expression_parts
    }

    pub fn get_arguments(&self) -> Vec<AssignmentExpression> {
        let mut r = Vec::new();
        for x in self.postfix_expression_parts.iter() {
            if let PostfixExpressionPart::Arguments(_, id) = x {
                r.extend(id.clone());
            }
        }
        r
    }

    /// get id list from dot reference in postfix expression parts
    pub fn get_id_list(&self) -> Vec<String> {
        let mut r = Vec::new();
        for x in self.postfix_expression_parts.iter() {
            if let PostfixExpressionPart::DotId(_, id) = x {
                r.push(id.clone());
            }
        }
        r
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }

        // Resolve postfix parts left-to-right so dot, ptr (->), index and call
        // consistently derive the final expression type.
        let mut result_type = self.get_tanry_expression().get_clang_type();
        for postfix_part in self.get_postfix_expression_parts().iter() {
            result_type = postfix_part.resolve_result_type(&result_type);
        }

        result_type
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![(&mut self.tanry_expression).into()];
        for x in self.postfix_expression_parts.iter_mut() {
            r.push(x.into());
        }
        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.tanry_expression.access_children(context);
        for x in self.postfix_expression_parts.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::ConstantExpression_Tanry(CLangItem::new(self)));
    }
}

impl ASTItemTrait for ConstantExpression_Tanry {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("ConstExpr_Tanry({})", self.ast_item.get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.ast_item.get_sequence_id()
    }
}