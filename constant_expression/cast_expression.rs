use std::fmt::{Display, Formatter};

use pest::iterators::Pair;

use crate::languages::clang::traits::*;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use crate::languages::clang::clang_error::CLangError;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::clang_pest::*;
use crate::languages::clang::{ASTItem, CLangItem, constant_expression::*};
use crate::languages::clang::clang_pest::TypeName;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CastExpression {
    type_name: TypeName,
    constant_expression : ConstantExpression,
    ast_item : ASTItem,
}

impl CastExpression {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> { 
        assert_eq!(pair.as_rule(), Rule::cast_expression); //ensure it is Rule::assignment_expression

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner_pairs = pair.to_owned().into_inner().collect::<Vec<Pair<Rule>>>();
        let pairs = inner_pairs.into_iter().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::type_name, p), (Rule::constant_expression, p2)] => {
                let type_name = TypeName::from_pair(p, context)?;
                let constant_expression = ConstantExpression::from_pair(p2, context)?;
                Ok(Self { type_name, constant_expression, ast_item, })
            }
            _ => Err(CLangError::MissingCase(file!().to_string(), line!() as usize, format!("Missed case in SizeOfExpression {pairs:?}"))),
        }
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        let type_name = self.get_type_name();
        type_name.into()
    }

    pub fn get_type_name(&self) -> &TypeName {
        &self.type_name
    }

    /// Get the constant expression being casted
    pub fn get_constant_expr(&self) -> &ConstantExpression {
        &self.constant_expression
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![(&mut self.type_name).into(), (&mut self.constant_expression).into()]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        self.type_name.access_children(context);
        self.constant_expression.access_children(context);
        context.access_clang_item(CLangItem::CastExpression(CLangItem::new(self)));
    }
}

impl Display for CastExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "({}){}", self.type_name, self.constant_expression)
    }
}

impl ASTItemTrait for CastExpression {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("CastExpr({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.ast_item.get_sequence_id()
    }
}