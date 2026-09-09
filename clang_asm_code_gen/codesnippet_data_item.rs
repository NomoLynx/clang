use crate::languages::clang::clang_asm_code_gen::*;
use crate::languages::r5asm::r5asm_pest::{SectionItem, SectionItem2};
use core_utils::traits::generate_code::GenerateCode;
use rust_macro::*;

#[derive(Debug, Clone, PartialEq, GenIsEnumVariant)]
pub enum CodeSnippetDataItem {
    Inc(SectionItem),
    Snippet(CodeSnippet),
    Markdown(MarkdownData),
}

impl CodeSnippetDataItem {

    pub fn get_inc(&self) -> Option<&SectionItem> {
        match self {
            CodeSnippetDataItem::Inc(inc) => {
                Some(inc)
            }
            _ => {
                None
            }
        }
    }

    pub fn get_inc_mut(&mut self) -> Option<&mut SectionItem> {
        match self {
            CodeSnippetDataItem::Inc(inc) => {
                Some(inc)
            }
            _ => {
                None
            }
        }
    }

    pub fn get_snippet(&self) -> Option<&CodeSnippet> {
        match self {
            CodeSnippetDataItem::Snippet(snippet) => {
                Some(snippet)
            }
            _ => {
                None
            }
        }
    }

    pub fn get_snippet_mut(&mut self) -> Option<&mut CodeSnippet> {
        match self {
            CodeSnippetDataItem::Snippet(snippet) => {
                Some(snippet)
            }
            _ => {
                None
            }
        }
    }

    pub fn get_md_data(&self) -> Option<&MarkdownData> {
        match self {
            CodeSnippetDataItem::Markdown(data) => {
                Some(data)
            }
            _ => {
                None
            }
        }
    }

    /// Generate the data string for this data item
    pub fn generate_data_string(&self) -> String {
        match self {
            CodeSnippetDataItem::Inc(inc) => {
                // if the label starts with "___" / three underscores, then it is a special label, which is used to indicate the label is not used
                // and won't be generated in the code
                if let Some(label_text) = inc.get_label() {
                    if label_text.starts_with("___") {
                        return String::new();
                    }
                }
                
                inc.generate_code_string()
            }
            CodeSnippetDataItem::Snippet(snippet) => {
                snippet.generate_data_string()
            }
            CodeSnippetDataItem::Markdown(_data) => {
                String::new()   // markdown data won't be generated in the code
            }
        }
    }
}

impl GenerateCode for CodeSnippetDataItem {
    fn generate_code_string(&self) -> String {
        match self {
            CodeSnippetDataItem::Inc(inc) => {
                // if the label starts with "___" / three underscores, then it is a special label, which is used to indicate the label is not used
                // and won't be generated in the code
                if let Some(label_text) = inc.get_label() {
                    if label_text.starts_with("___") {
                        return String::new();
                    }
                }
                
                inc.generate_code_string()
            }
            CodeSnippetDataItem::Snippet(snippet) => {
                snippet.generate_code_string()
            }
            CodeSnippetDataItem::Markdown(data) => {
                data.get_data().to_string()
            }
        }
    }
}

impl Eq for CodeSnippetDataItem {

}

impl From<SectionItem> for CodeSnippetDataItem {
    fn from(inc:SectionItem) -> CodeSnippetDataItem {
        CodeSnippetDataItem::Inc(inc)
    }
}

impl From<&SectionItem> for CodeSnippetDataItem {
    fn from(inc:&SectionItem) -> CodeSnippetDataItem {
        CodeSnippetDataItem::Inc(inc.clone())
    }
}

impl From<&SectionItem2> for CodeSnippetDataItem {
    fn from(inc:&SectionItem2) -> CodeSnippetDataItem {
        CodeSnippetDataItem::Inc(inc.get_item().clone())
    }
}

impl From<CodeSnippet> for CodeSnippetDataItem {
    fn from(snippet:CodeSnippet) -> CodeSnippetDataItem {
        CodeSnippetDataItem::Snippet(snippet)
    }
}

impl From<MarkdownData> for CodeSnippetDataItem {
    fn from(data:MarkdownData) -> CodeSnippetDataItem {
        CodeSnippetDataItem::Markdown(data)
    }
}