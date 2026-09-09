use rust_macro::*;

use std::fmt::Display;

use crate::languages::clang::ASTItem;
use crate::languages::clang::CLangItem;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::tanry_expression::*;
use crate::languages::operators::LangOperators;
use crate::languages::clang::traits::ASTItemTrait;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Accessors, GetMut)]
pub struct Expression_Expression {
    left : Box<Expression>,
    right : Box<Expression>,
    op : LangOperators,
    ast_item : ASTItem,

    // meta data for code generation

    /// whether the expression is the highest addressing expression in the expression tree, 
    /// it's used for code generation to determine whether to generate code for loading the value of the expression;
    is_highest_addressing_expression : bool,
}

impl Expression_Expression {
    pub fn new(left:Box<Expression>, right:Box<Expression>, op:LangOperators) -> Self {
        Self { left, right, op, ast_item : ASTItem::default(), is_highest_addressing_expression : false }
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        match self.op {
            LangOperators::Add |
            LangOperators::Sub |
            LangOperators::Mul |
            LangOperators::Div |
            LangOperators::Rem |
            LangOperators::Pow => {
                self.left.get_clang_type()
            }
            LangOperators::LogicEqual |
            LangOperators::NotEqual |
            LangOperators::LogicAnd |
            LangOperators::LogicOr |
            LangOperators::LogicXor |
            LangOperators::GreaterEuqal |
            LangOperators::LessEqual |
            LangOperators::Greater |
            LangOperators::Less => {
                CLangTypes::Int32   //boolean type as int32
            }
            LangOperators::BitAnd |
            LangOperators::BitOr |
            LangOperators::BitXor |
            LangOperators::BitLeftMove |
            LangOperators::BitRightMove => {
                self.left.get_clang_type()
            }
            LangOperators::AddressArrow |
            LangOperators::AddressDot => {
                self.get_right().get_clang_type()
            }
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![self.left.as_mut().into(), self.right.as_mut().into()]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.left.access_children(context);
        self.right.access_children(context);

        context.access_clang_item(CLangItem::Expression_Expression(CLangItem::new(self)));
    }
}

impl Display for Expression_Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "({} {} {})", self.left, self.op, self.right)
    }
}

impl ASTItemTrait for Expression_Expression {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        let op = self.get_op().to_string().replace(" ", "_");
        let tag = if self.get_is_highest_addressing_expression() { "_H" } else { "" };
        format!("{op}_ExpExp({}){tag}", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}
