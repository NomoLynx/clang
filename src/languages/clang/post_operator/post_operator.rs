use std::fmt::Display;

use rust_macro::EnumAccessors;

use crate::languages::clang::postfix_expression_part::PostfixExpressionPart;

#[derive(Debug, Clone, PartialEq, Eq, EnumAccessors)]
pub enum PostOperator {
    Inc,
    Desc,
    PostfixExpressionPart(PostfixExpressionPart),
}

impl PostOperator {
    pub fn from_string(str: &str) -> Self {
        match str {
            "inc" => Self::Inc,
            "desc" => Self::Desc,
            "++" => Self::Inc,
            "--" => Self::Desc,
            _ => todo!("{}", format!("unprocessed post operator {str}")),
        }
    }

    /// get the operator name inc or desc
    pub fn get_operator_name(&self) -> String {
        match self {
            Self::Inc => "Inc".to_string(),
            Self::Desc => "Desc".to_string(),
            Self::PostfixExpressionPart(n) => n.to_mermaid_name(),
        }
    }
}

impl Display for PostOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Inc => "++".to_string(),
            Self::Desc => "--".to_string(),
            Self::PostfixExpressionPart(n) => n.to_string(),
        };

        write!(f, "{}", s)
    }
}