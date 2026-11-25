use std::fmt;

use pest::{iterators::Pair, Parser};
use pest_derive::Parser;

use core_utils::debug::*;

use super::data_lang_pest_error::DataLangError;

#[derive(Parser)]
#[grammar = "languages/data_lang/data_lang.pest"]
pub (crate) struct DataLangParser;

pub fn parse_list_with_id(input:&str) -> Result<ListWithId, DataLangError> {
    let result = DataLangParser::parse(Rule::list, input);
    match result {
        Ok(mut pairs) => {
            if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
                match ListWithId::from_pair(&pair) {
                    Ok(rr) => {
                        Ok(rr)
                    }
                    Err(err2) => {
                        error_string(format!("{:?}", err2));
                        Err(err2)
                    }
                }
            }
            else {
                let err = format!("cannot match all string in {input}");
                error_string(err.to_string());
                Err(DataLangError::ListWithIdErr(err.to_string()))
            }
        }
        Err(err) => {
            let err = format!("{}", err);
            error_string(err.to_string());
            Err(DataLangError::ListWithIdErr(err.to_string()))
        }
    }
}

#[derive(Clone)]
pub struct ListWithId {
    id: String,
    list : Vec<String>,
}

impl ListWithId {
    pub fn from_pair(pair:&Pair<Rule>) -> Result<Self, DataLangError> {
        if pair.as_rule() == Rule::list {
            let inner = pair.to_owned().into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
            match inner.as_slice() {
                [] => Ok(Self::default()),
                [(Rule::list_id, id)] => Ok(Self { id : id.as_str().to_string(), list : Vec::default() }),
                [(Rule::list_id, id), tail@..] => {
                    let l = tail.iter().map(|(_, n)| n.as_str().trim().to_string()).collect::<Vec<_>>();
                    Ok(Self { id : id.as_str().to_string(), list : l })
                }
                _ => {
                    Err(DataLangError::ListWithIdErr(format!("parsing case need to process {inner:?}")))
                }
            }
        }
        else {
            Err(DataLangError::WrongPair("expected rule list".to_owned(), format!("found {:?}", pair.as_rule())))
        }
    }

    pub fn get_list(&self) -> &Vec<String> {
        &self.list
    }

    pub fn get_id(&self) -> &String {
        &self.id
    }
}

impl Default for ListWithId {
    fn default() -> Self {
        Self { id: String::default(), list: Vec::default() }
    }
}

impl fmt::Debug for ListWithId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{} : {}", self.id, self.list.join(", "));
        write!(f, "{}", s)
    }
}
