use core::panic;
use std::fmt::{Display, Formatter};

use pest::iterators::Pair;

use crate::languages::clang::clang_types::*;
use crate::languages::clang::constant_expression::*;
use crate::languages::clang::traits::*;
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::postfix_expression_part::*;
use crate::languages::clang::ast_item::ASTItem;
use core_utils::debug::*;
use crate::languages::clang::clang_pest::*;
use crate::languages::clang::{clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstantExpression {
    Tanry(ASTItem, ConstantExpression_Tanry),
    Sizeof(ASTItem, Box<SizeOfExpression>),
    Cast(ASTItem, Box<CastExpression>),
}

impl ConstantExpression {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::constant_expression); //ensure it is rule:constant_expression

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner = pair.to_owned().into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();

        match inner.as_slice() {
            [(Rule::tenary_expression, p), postfix_pairs@..] => {
                let r = TanryExpression::from_pair(p, context)?;
                let l = postfix_pairs.iter().map(|(r, p)| {
                    if r == &Rule::postfix_expression_part {
                        PostfixExpressionPart::from_pair(p, context).unwrap()
                    } else {
                        error_string(format!("need to process rule {r:?} in ConstantExpression case: {inner:?}"));
                        todo!()
                    }
                }).collect::<Vec<_>>();
                let ast_item2 = ASTItem::new(p.line_col().into());
                let rr = ConstantExpression_Tanry::new(r, l, ast_item2);
                Ok(Self::Tanry(ast_item, rr))
            }
            [(Rule::sizeof_expression, p)] => {
                let r = Box::new( SizeOfExpression::from_pair(p, context)? );
                Ok(Self::Sizeof(ast_item, r))
            }
            [(Rule::cast_expression, p)] => {
                let r = Box::new( CastExpression::from_pair(p, context)? );
                Ok(Self::Cast(ast_item, r))
            }
             _ => {
                error_string(format!("need to process rule in ConstantExpression case: {inner:?}"));
                todo!()
            }
        }
    }

    /// Get the string value of this constant expression if exists, otherwise return None
    pub fn get_const_value(&self) -> Option<String> {
        self.get_tanry()
            .and_then(|n| n.get_string_value())
    }

    /// Get the ID of the tanary expression if exists, otherwise return None
    /// the tanry expression is the only one that can produce a value, other expressions like sizeof and cast return None
    pub fn get_tanary_id(&self) -> Option<String> {
        match self {
            Self::Tanry(_, n) => n.get_tanry_expression()
                                                                .get_bool_expr_id()
                                                                .map(|x| x.clone()),
            _ => None,
        }
    }

    pub fn get_tanry(&self) -> Option<&TanryExpression> {
        match self {
            Self::Tanry(_, n) => Some(n.get_tanry_expression()),
            _ => None,
        }
    }

    pub fn set_id_value_changed(&mut self, changed:bool) {
        if let Self::Tanry(_, n) = self {
            n.get_tanry_expression_mut()
                .set_id_value_changed(changed);
        }
        else {
            panic!("set_id_value_changed only support Tanry ConstantExpression");
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Tanry(_, n) => {
                let r = vec![n.into()];
                r
            }
            Self::Sizeof(_, n) => vec![n.as_mut().into()],
            Self::Cast(_, n) => vec![n.as_mut().into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Tanry(_, n) => {
                n.access_children(context);
            }
            Self::Sizeof(_, n) => n.access_children(context),
            Self::Cast(_, n) => n.access_children(context),
        }

        context.access_clang_item(CLangItem::ConstantExpression(CLangItem::new(self)));
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        match self {
            ConstantExpression::Tanry(_, e) => e.get_clang_type(),
            ConstantExpression::Cast(_, e) => e.get_clang_type(),
            ConstantExpression::Sizeof(_, e) => e.get_clang_type(),
        }
    }
}

impl Display for ConstantExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Tanry(_, n) => write!(f, "{}", n.get_tanry_expression()),
            Self::Sizeof(_, n) => write!(f, "{}", n),
            Self::Cast(_, n) => write!(f, "({})", n),
        }
    }
}

impl ASTItemTrait for ConstantExpression {
    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Tanry(n, _) => n,
            Self::Sizeof(n, _) => n,
            Self::Cast(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Tanry(n, _) => n,
            Self::Sizeof(n, _) => n,
            Self::Cast(n, _) => n,
        }
    }

    fn get_name(&self) -> String {
        format!("ConstExpr({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

