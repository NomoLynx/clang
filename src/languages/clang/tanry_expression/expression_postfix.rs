use rust_macro::*;

use std::fmt::Display;

use crate::languages::clang::ASTItem;
use crate::languages::clang::CLangItem;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::logic::get_postfix_ref;
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::post_operator::*;
use crate::languages::clang::traits::ASTItemTrait;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Accessors, GetMut)]
pub struct Expression_Postfix {
    expr : Box<Expression>,
    op : PostOperator,
    ast_item : ASTItem,
}

impl Expression_Postfix {
    pub fn new(expr:Box<Expression>, op:PostOperator) -> Self {
        Self { expr, op, ast_item : ASTItem::default() }
    }

    /// get single id if the expression is atom; otherwise return None
    pub fn get_single_id(&self) -> Option<&String> {
        self.get_expr()
            .get_atom_id()
    }

    pub fn get_reg_name(&self) -> Option<&str> {
        let reg_name_option = if let Some(var_name) = self.get_single_id() {
                    if let Some(symbol_table) = self.get_ast_item().get_symbol_table() {
                        let location = symbol_table.get_var_storage_location(&var_name);
                        location.get_register_name()
                    }
                    else {
                        None
                    }
                }
                else {
                    None
                };

        reg_name_option
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match &mut self.op {
            PostOperator::PostfixExpressionPart(part) => {
                vec![self.expr.as_mut().into(), part.into()]
            }
            _ => vec![self.expr.as_mut().into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.expr.access_children(context);

        if let PostOperator::PostfixExpressionPart(part) = &mut self.op {
            part.access_children(context);
        }

        context.access_clang_item(CLangItem::Expression_Postfix(CLangItem::new(self)));
    }

    /// get clang type of the expression
    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }

        let base_type = self.get_expr().get_clang_type();
        match self.get_op() {
            PostOperator::PostfixExpressionPart(part) => part.resolve_result_type(&base_type),
            PostOperator::Inc |
            PostOperator::Desc => base_type,
        }
    }
}

/// implement the logic to resolve the resulting type after applying this postfix part to `base_type`
impl Expression_Postfix {
    pub fn get_array_type(&self) -> CLangTypes {
        self.get_clang_type()
    }

    pub fn get_index_operand_type(&self) -> CLangTypes {
        self.get_expr()
            .get_clang_type()
    }

    /// get storage_location of the expression
    /// structA.structB.xx, the storage location is structA.structB.xx
    pub fn get_storage_location(&self) -> isize {
        let patterns = get_postfix_ref(self.get_all_clangitems());
        get_offset(&patterns) as isize
    }
}

impl ASTItemTrait for Expression_Postfix {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("Expr_Post_Op({})_({})", 
            self.op.get_operator_name(), 
            self.get_ast_item().get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for Expression_Postfix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.expr, self.op)
    }
}