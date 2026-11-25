use std::collections::HashMap;

use core_utils::debug::*;
use crate::languages::clang::ASTItem;

use super::code_snippet::CodeSnippet;
use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeSnippetKey {
    cateory: SnippetKey,
    has_assigned_register: bool,
}

impl CodeSnippetKey {
    pub fn new(cateory: SnippetKey, has_assigned_register: bool) -> Self {
        Self {
            cateory,
            has_assigned_register,
        }
    }
    
    pub fn get_category(&self) -> &SnippetKey {
        &self.cateory
    }

    pub fn has_assigned_register(&self) -> bool {
        self.has_assigned_register
    }

    pub fn is_address(&self) -> bool {
        self.cateory.is_address()
    }
}

impl From<SnippetKey> for CodeSnippetKey {
    fn from(value: SnippetKey) -> Self {
        (&value).into()
    }
}

impl From<&SnippetKey> for CodeSnippetKey {
    fn from(value: &SnippetKey) -> Self {
        Self {
            cateory: value.clone(),
            has_assigned_register: false,
        }
    }
}

impl From<(&SnippetKey, &ASTItem)> for CodeSnippetKey {
    fn from(value: (&SnippetKey, &ASTItem)) -> Self {
        Self {
            cateory: value.0.clone(),
            has_assigned_register: value.1.has_assigned_register(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct CodeSnippetArchive {
    snippets: HashMap<CodeSnippetKey, Vec<CodeSnippet>>,
}

impl CodeSnippetArchive {
    pub fn new() -> Self {
        Self {
            snippets: HashMap::new(),
        }
    }

    pub fn add_snippet(&mut self, key: CodeSnippetKey, snippet: CodeSnippet) {
        if let Some(v) = self.snippets.get_mut(&key) {
            v.push(snippet);
        } else {
            self.snippets.insert(key, vec![snippet]);
        }
    }

    pub fn add_snippets(&mut self, key: CodeSnippetKey, snippets: Vec<CodeSnippet>) {
        self.snippets.insert(key, snippets);
    }

    pub fn get_snippets(&self, key: &CodeSnippetKey) -> Option<&Vec<CodeSnippet>> {
        self.snippets.get(key)
    }

    /// get codesnippet with key and codesnippet's name
    pub fn get_snippet(&self, key: &CodeSnippetKey, name: &str) -> Option<CodeSnippet> {
        if let Some(v) = self.get_snippets(key) {
            let r = v.iter().find(|x| x.get_name() == name);
            if r.is_none() { 
                warn_string(format!("cannot find snippets by using ({key:?}, {name}), will default to (int, {name}), or you can add by modifying the markdown files in code_snippets folder"));
            }

            r.cloned()
        } else {
            warn_string(format!("cannot find snippets from key '{key:?}' for '{name}', can add by modifying the markdown files in code_snippets folder"));
            None
        }
    }
}