use core_utils::traits::generate_code::GenerateCode;
use crate::languages::clang::clang_asm_code_gen::*;

pub trait ToMarkdownTableRow {
    fn get_markdown_header(&self) -> String;
    fn to_markdown(&self) -> String;
}

impl ToMarkdownTableRow for CodeSnippet {
    fn get_markdown_header(&self) -> String {
        let mut r = String::new();
        r.push_str("| Name |");
        r.push_str(& String::from(" Code |\r\n"));
        r.push_str(& format!("|------|------|\r\n"));
        r    
    }

    fn to_markdown(&self) -> String {
        let mut r = String::new();
        r.push_str(&format!("| {} |", self.get_name()));
        for item in self.get_code() {
            r.push_str(&format!(" {} |", item.generate_code_string()));
        }
        r
    }
}