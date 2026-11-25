use std::fmt::Display;

use crate::languages::SourceFileLocation;

#[derive(Debug, Clone)]
pub struct ASTItemIterationContextError {
    msg: String,
    location: SourceFileLocation,
}

impl ASTItemIterationContextError {
    pub fn new(msg:String, location:SourceFileLocation) -> Self {
        Self { msg, location }
    }

    pub fn get_message(&self) -> &String {
        &self.msg
    }

    pub fn get_location(&self) -> &SourceFileLocation {
        &self.location
    }
}

impl From<(String, SourceFileLocation)> for ASTItemIterationContextError {
    fn from(value: (String, SourceFileLocation)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl From<(String, &SourceFileLocation)> for ASTItemIterationContextError {
    fn from(value: (String, &SourceFileLocation)) -> Self {
        Self::new(value.0, value.1.clone())
    }
}

impl Display for ASTItemIterationContextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}", self.get_message(), self.get_location())
    }
}