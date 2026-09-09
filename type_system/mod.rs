pub(crate) mod type_config;

pub use type_config::*;

use crate::languages::clang::clang_types::CLangTypes;

pub type CLangRuntimeType = Box<Option<CLangTypes>>;