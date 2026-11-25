#[derive(Debug, Clone)]
pub enum DataLangError {
    ListWithIdErr(String),
    WrongPair(String, String),
}
