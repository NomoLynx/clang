use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeAlias(Vec<String>);

impl Default for TypeAlias { 
    fn default() -> Self {
        Self(vec![])
    }
}

impl Deref for TypeAlias {
    type Target = Vec<String>;

    fn deref(&self) -> &Self::Target {
        & self.0
    }
}

impl DerefMut for TypeAlias {
    //type Target = Vec<String>;

    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut  self.0
    }
}