use std::fmt::{Display, Formatter};

use pest::error::LineColLocation;

use crate::languages::data_lang::data_lang_pest_error::DataLangError;

pub type ErrInfo = String;
pub type FileLocation = (String, u32);

#[derive(Debug, Clone)]
pub enum CLangError {
    ParsingError(String, usize),
    ParsingConversionError,
    ParameterError(ErrInfo),
    NoFound(FileLocation, ErrInfo),
    PestParsingError(String, usize, usize),
    CannotEmitType(String, u32, String),  //file name, line #, and error msg tuple
    LoadFromMDHashErr(String),
    IOError(String),
    DataLangErr(DataLangError),
    RegisterConflictResolveErr(String),
    MissingCase(String, usize, String), //file name, line #, and error msg tuple
    GeneralError(FileLocation, String), //file name, line #, and error msg tuple
}

impl CLangError {
    pub (crate) fn get_location_from_pest_input_location(str:&str, input_location:&LineColLocation) -> Self {
        let pos = match input_location {
            LineColLocation::Pos((a, b)) => (a, b),
            LineColLocation::Span((a, b), _) => (a, b),
        };

        Self::PestParsingError(str.to_string(), *pos.0, *pos.1)
    }

    /// Create a CannotEmitType error with the caller's file and line number
    #[track_caller]
    pub fn cannot_emit(msg: &str) -> Self {
        let location = std::panic::Location::caller();
        Self::CannotEmitType(
            location.file().to_string(),
            location.line(),
            msg.to_string(),
        )
    }
}

impl Display for CLangError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            CLangError::ParsingError(s, line) => write!(f, "Parsing Error at line {}: {}", line, s),
            CLangError::ParsingConversionError => write!(f, "Parsing Conversion Error"),
            CLangError::ParameterError(s) => write!(f, "Parameter Error: {}", s),
            CLangError::NoFound((file, line), s) => write!(f, "Not Found Error at {}:{}:0 {}", file, line, s),
            CLangError::PestParsingError(s, line, col) => write!(f, "Pest Parsing Error at {}:{}:0 {}", line, col, s),
            CLangError::CannotEmitType(file, line, s) => write!(f, "Cannot Emit Type Error at {}:{}:0 {}", file, line, s),
            CLangError::LoadFromMDHashErr(s) => write!(f, "Load From MD Hash Error: {}", s),
            CLangError::IOError(s) => write!(f, "IO Error: {}", s),
            CLangError::DataLangErr(e) => write!(f, "Data Language Error: {:?}", e),
            CLangError::RegisterConflictResolveErr(s) => write!(f, "Register Conflict Resolve Error: {}", s),
            CLangError::MissingCase(file, line, s) => write!(f, "Missing Case Error at {}:{}:0 {}", file, line, s),
            CLangError::GeneralError((file, line), s) => write!(f, "General Error at {}:{}:0 {}", file, line, s),
        }
    }
}