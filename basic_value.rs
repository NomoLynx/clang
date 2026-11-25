use std::fmt::{Display, Formatter};

use core_utils::expr_value::ExprValue;

use crate::languages::clang::clang_types::CLangTypes;

#[derive(Debug, Clone)]
pub enum BasicValue {
    Int8(i8),
    UInt8(u8),
    Int16(i16),
    UInt16(u16),
    Int32(i32),
    UInt32(u32),
    Int64(i64),
    UInt64(u64),
    Float32(f32),
    Float64(f64),
    Boolean(bool),
    Char(char),
    String(Option<String>),
}

impl BasicValue {
    pub fn from_bool(v: bool) -> Self {
        Self::Boolean(v)
    }

    pub fn from_char(v: char) -> Self {
        Self::Char(v)
    }

    pub fn from_string(v: String) -> Self {
        Self::String(Some(v))
    }

    pub fn conv_i32(&self) -> Result<Self, String> {
        let v = match self {
            Self::Int8(v) => *v as i32,
            Self::UInt8(v) => *v as i32,
            Self::Int16(v) => *v as i32,
            Self::UInt16(v) => *v as i32,
            Self::Int32(v) => *v,
            Self::UInt32(v) => *v as i32,
            Self::Int64(v) => *v as i32,
            Self::UInt64(v) => *v as i32,
            Self::Float32(v) => *v as i32,
            Self::Float64(v) => *v as i32,
            Self::Boolean(v) => i32::from(*v),
            Self::Char(v) => *v as i32,
            Self::String(_) => return Err("cannot convert string to i32".to_string()),
        };
        Ok(Self::Int32(v))
    }

    pub fn conv_f32(&self) -> Result<Self, String> {
        let v = match self {
            Self::Int8(v) => *v as f32,
            Self::UInt8(v) => *v as f32,
            Self::Int16(v) => *v as f32,
            Self::UInt16(v) => *v as f32,
            Self::Int32(v) => *v as f32,
            Self::UInt32(v) => *v as f32,
            Self::Int64(v) => *v as f32,
            Self::UInt64(v) => *v as f32,
            Self::Float32(v) => *v,
            Self::Float64(v) => *v as f32,
            Self::Boolean(v) => i32::from(*v) as f32,
            Self::Char(v) => *v as u32 as f32,
            Self::String(_) => return Err("cannot convert string to f32".to_string()),
        };
        Ok(Self::Float32(v))
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        match self {
            Self::Int8(_) => CLangTypes::Byte,
            Self::UInt8(_) => CLangTypes::UByte,
            Self::Int16(_) => CLangTypes::Int16,
            Self::UInt16(_) => CLangTypes::UInt16,
            Self::Int32(_) => CLangTypes::Int32,
            Self::UInt32(_) => CLangTypes::UInt32,
            Self::Int64(_) => CLangTypes::Int64,
            Self::UInt64(_) => CLangTypes::UInt64,
            Self::Float32(_) => CLangTypes::Float32,
            Self::Float64(_) => CLangTypes::Float64,
            Self::Boolean(_) => CLangTypes::Int32,
            Self::Char(_) => CLangTypes::Char,
            Self::String(_) => CLangTypes::Pointer(1, Box::new(CLangTypes::Char)),
        }
    }
}

impl From<ExprValue> for BasicValue {
    fn from(value: ExprValue) -> Self {
        match value {
            ExprValue::Int8(v) => Self::Int8(v),
            ExprValue::UInt8(v) => Self::UInt8(v),
            ExprValue::Int16(v) => Self::Int16(v),
            ExprValue::UInt16(v) => Self::UInt16(v),
            ExprValue::Int32(v) => Self::Int32(v),
            ExprValue::UInt32(v) => Self::UInt32(v),
            ExprValue::Int64(v) => Self::Int64(v),
            ExprValue::UInt64(v) => Self::UInt64(v),
            ExprValue::Float32(v) => Self::Float32(v),
            ExprValue::Float64(v) => Self::Float64(v),
            ExprValue::Boolean(v) => Self::Boolean(v),
            ExprValue::Char(v) => Self::Char(v),
            ExprValue::String(v) => Self::String(v),
            _ => Self::Int32(0),
        }
    }
}

impl PartialEq for BasicValue {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Float32(a), Self::Float32(b)) => a.to_bits() == b.to_bits(),
            (Self::Float64(a), Self::Float64(b)) => a.to_bits() == b.to_bits(),
            _ => format!("{self}") == format!("{other}"),
        }
    }
}

impl Eq for BasicValue {}

impl Display for BasicValue {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Int8(v) => write!(f, "{v}"),
            Self::UInt8(v) => write!(f, "{v}"),
            Self::Int16(v) => write!(f, "{v}"),
            Self::UInt16(v) => write!(f, "{v}"),
            Self::Int32(v) => write!(f, "{v}"),
            Self::UInt32(v) => write!(f, "{v}"),
            Self::Int64(v) => write!(f, "{v}"),
            Self::UInt64(v) => write!(f, "{v}"),
            Self::Float32(v) => write!(f, "{v}"),
            Self::Float64(v) => write!(f, "{v}"),
            Self::Boolean(v) => write!(f, "{v}"),
            Self::Char(v) => write!(f, "{v}"),
            Self::String(v) => write!(f, "{}", v.clone().unwrap_or_default()),
        }
    }
}
