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
pub enum SizeOfExpression {
    ConstantExpression(ASTItem, ConstantExpression),
    TypeName(ASTItem, TypeName),
    CastExpression(ASTItem, CastExpression),
}

impl SizeOfExpression {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::sizeof_expression); //ensure it is Rule::assignment_expression

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner_pairs = pair.to_owned().into_inner().collect::<Vec<Pair<Rule>>>();
        let pairs = inner_pairs.into_iter().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        
        match pairs.as_slice() {
            [(Rule::SIZEOF, _), (Rule::constant_expression, p)] => {
                let r = ConstantExpression::from_pair(p, context)?;
                Ok(Self::ConstantExpression(ast_item, r))
            }
            [(Rule::SIZEOF, _), (Rule::type_name, p)] => {
                let r = TypeName::from_pair(p, context)?;
                Ok(Self::TypeName(ast_item, r))
            }
            [(Rule::cast_expression, p)] => {
                let r = CastExpression::from_pair(p, context)?;
                Ok(Self::CastExpression(ast_item, r))
            }
            _ => Err(CLangError::MissingCase(file!().to_string(), line!() as usize, format!("Missed case in SizeOfExpression {pairs:?}"))),
        }
    }

    /// Get the type of the sizeof expression, which is always int
    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        CLangTypes::Int32
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::ConstantExpression(_, n) => vec![n.into()],
            Self::TypeName(_, n) => vec![n.into()],
            Self::CastExpression(_, n) => vec![n.into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        match self {
            Self::ConstantExpression(_, n) => n.access_children(context),
            Self::TypeName(_, n) => n.access_children(context),
            Self::CastExpression(_, n) => n.access_children(context),
        }
    }
}

impl Display for SizeOfExpression {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConstantExpression(_, n) => write!(f, "sizeof({})", n),
            Self::TypeName(_, n) => write!(f, "sizeof({})", n),
            Self::CastExpression(_, n) => write!(f, "sizeof({})", n),
        }
    }
}

impl ASTItemTrait for SizeOfExpression {
    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::ConstantExpression(n, _) => n,
            Self::TypeName(n, _) => n,
            Self::CastExpression(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::ConstantExpression(n, _) => n,
            Self::TypeName(n, _) => n,
            Self::CastExpression(n, _) => n,
        }
    }

    fn get_name(&self) -> String {
        format!("SizeOfExpr({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}