use std::fmt::Display;

use crate::languages::clang::clang_types::CLangTypes;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LangOperators {
    Add,
    Sub,
    Mul,
    Div,
    BitLeftMove,
    BitRightMove,
    LessEqual,
    GreaterEuqal,
    Less,
    Greater,
    LogicEqual,
    NotEqual,
    LogicAnd,
    LogicOr,
    LogicXor,
    Rem,
    BitAnd,
    BitOr,
    BitXor,
    Pow,
    AddressDot,
    AddressArrow,
}

impl LangOperators {
    pub fn is_valid_operation(&self, ty0: CLangTypes, ty1: CLangTypes) -> bool {
        match self {
            Self::Mul | Self::Div => {
                (ty0.is_numeric_type() && ty1.is_numeric_type())
                    || (ty0.is_pointer() && ty1.is_numeric_type())
                    || (ty0.is_numeric_type() && ty1.is_enum())
                    || (ty0.is_enum() && ty1.is_numeric_type())
            }
            Self::Add => {
                (ty0.is_numeric_type() && ty1.is_numeric_type())
                    || (ty0.is_pointer() && ty1.is_numeric_type())
                    || (ty0.is_numeric_type() && ty1.is_pointer())
                    || (ty0.is_numeric_type() && ty1.is_enum())
                    || (ty0.is_enum() && ty1.is_numeric_type())
            }
            Self::Sub => {
                (ty0.is_numeric_type() && ty1.is_numeric_type())
                    || (ty0.is_pointer() && ty1.is_numeric_type())
                    || (ty0.is_numeric_type() && ty1.is_pointer())
                    || (ty0.is_pointer() && ty1.is_pointer())
                    || (ty0.is_numeric_type() && ty1.is_enum())
                    || (ty0.is_enum() && ty1.is_numeric_type())
            }
            Self::Rem => ty0.is_integer_type() && ty1.is_integer_type(),
            Self::BitAnd | Self::BitOr | Self::BitXor | Self::BitLeftMove | Self::BitRightMove => {
                ty0.is_integer_type() && ty1.is_integer_type()
            }
            Self::Pow => ty0.is_integer_type() && ty1.is_integer_type(),
            Self::Greater | Self::GreaterEuqal | Self::Less | Self::LessEqual => {
                ty0.is_numeric_type() && ty1.is_numeric_type()
            }
            Self::LogicAnd | Self::LogicOr | Self::LogicXor => {
                ty0.is_integer_type() && ty1.is_integer_type()
            }
            Self::LogicEqual | Self::NotEqual => {
                (ty0.is_numeric_type() && ty1.is_numeric_type())
                    || (ty0.is_pointer() && ty1.is_pointer() || (ty0 == ty1))
            }
            Self::AddressDot | Self::AddressArrow => {
                ty0.is_struct() || ty0.is_union() || ty0.is_enum() || ty0.is_pointer()
            }
        }
    }

    pub fn is_add_or_sub(&self) -> bool {
        matches!(self, Self::Add | Self::Sub)
    }

    pub fn to_string(&self) -> String {
        let s = match self {
            Self::Add => "Add",
            Self::BitLeftMove => "BitLeftMove",
            Self::BitRightMove => "BitRightMove",
            Self::Div => "Div",
            Self::Greater => "Greater",
            Self::GreaterEuqal => "GreaterEuqal",
            Self::Less => "Less",
            Self::LessEqual => "LessEqual",
            Self::LogicAnd => "LogicAnd",
            Self::LogicEqual => "LogicEqual",
            Self::LogicOr => "LogicOr",
            Self::LogicXor => "LogicXor",
            Self::Mul => "Mul",
            Self::NotEqual => "NotEqual",
            Self::Sub => "Sub",
            Self::Rem => "Rem",
            Self::BitAnd => "BitAnd",
            Self::BitOr => "BitOr",
            Self::BitXor => "BitXor",
            Self::Pow => "Pow",
            Self::AddressDot => "AddressDot",
            Self::AddressArrow => "AddressArrow",
        };

        s.to_string()
    }

    pub fn from_string(v: &str) -> Self {
        match v {
            "Add" => Self::Add,
            "BitLeftMove" => Self::BitLeftMove,
            "BitRightMove" => Self::BitRightMove,
            "Div" => Self::Div,
            "Greater" => Self::Greater,
            "GreaterEqual" => Self::GreaterEuqal,
            "Less" => Self::Less,
            "LessEqual" => Self::LessEqual,
            "LogicAnd" => Self::LogicAnd,
            "LogicOr" => Self::LogicOr,
            "LogicXor" => Self::LogicXor,
            "Mul" => Self::Mul,
            "NotEqual" => Self::NotEqual,
            "Sub" => Self::Sub,
            "Rem" => Self::Rem,
            "BitAnd" => Self::BitAnd,
            "BitOr" => Self::BitOr,
            "BitXor" => Self::BitXor,
            "Pow" => Self::Pow,
            _ => todo!("{}", format!("need to implement lang operator {v:?}")),
        }
    }

    pub fn get_opposite(&self) -> Self {
        match self {
            Self::Greater => Self::LessEqual,
            Self::GreaterEuqal => Self::Less,
            Self::NotEqual => Self::LogicEqual,
            Self::LogicEqual => Self::NotEqual,
            Self::Less => Self::GreaterEuqal,
            Self::LessEqual => Self::Greater,
            _ => self.clone(),
        }
    }
}

impl Display for LangOperators {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Add => "+",
            Self::BitLeftMove => "<<",
            Self::BitRightMove => ">>",
            Self::Div => "/",
            Self::Greater => ">",
            Self::GreaterEuqal => ">=",
            Self::Less => "<",
            Self::LessEqual => "<=",
            Self::LogicAnd => "&&",
            Self::LogicEqual => "==",
            Self::LogicOr => "||",
            Self::LogicXor => "^",
            Self::Mul => "*",
            Self::NotEqual => "!=",
            Self::Rem => "%",
            Self::Sub => "-",
            Self::BitAnd => "&",
            Self::BitOr => "|",
            Self::BitXor => "^",
            Self::Pow => "**",
            Self::AddressDot => ".",
            Self::AddressArrow => "->",
        };

        write!(f, "{}", s)
    }
}
