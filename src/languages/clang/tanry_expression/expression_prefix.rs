use rust_macro::*;

use std::fmt::Display;

use crate::languages::clang::ASTItem;
use crate::languages::clang::CLangItem;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::clang_pest::UnaryOperator;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::traits::ASTItemTrait;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Accessors, GetMut)]
pub struct Expression_Prefix {
    op : UnaryOperator,
    expr : Box<Expression>,
    ast_item : ASTItem,
}

impl Expression_Prefix {
    pub fn new(op:UnaryOperator, expr:Box<Expression>) -> Self {
        Self { op, expr, ast_item : ASTItem::default() }
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

    /// get clang type of the expression
    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        let prefix = &self.op;
        let inner_type = self.get_expr().get_clang_type();

        let new_type = inner_type.apply_unary_operator(prefix);
        new_type        
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![self.expr.as_mut().into()]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.expr.access_children(context);

        context.access_clang_item(CLangItem::Expression_Prefix(CLangItem::new(self)));
    }

    /// check whether this is addressing operation
    pub fn is_addressing(&self) -> bool {
        self.get_op()
            .is_addressof()
    }
}

impl ASTItemTrait for Expression_Prefix {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("Expr_Pre_{}_({})", 
            self.op.get_mermaid_name(), 
            self.get_ast_item().get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for Expression_Prefix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.op, self.expr)
    }
}
