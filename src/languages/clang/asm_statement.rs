use pest::iterators::Pair;
use crate::languages::clang::*;
use crate::languages::clang::clang_pest::Rule;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsmStatement {
    asm_code: String,
    ast_item: ASTItem,
}

impl Default for AsmStatement {
    fn default() -> Self {
        Self {
            asm_code: String::new(),
            ast_item: ASTItem::default(),
        }
    }
}

impl AsmStatement {
    pub fn new(asm_code: String) -> Self {
        Self { asm_code, ast_item: ASTItem::default() }
    }

    pub fn get_asm_code(&self) -> &String {
        &self.asm_code
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![]
    }

    pub fn access_children(&mut self, _context:&mut CodeGenContext) { 
        // No children to access
    }

    pub fn from_pair(pair: &Pair<Rule>, _context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::asm_statement); //Ensure the parsed rule is 'Rule::asm_statement'

        let ast_item = ASTItem::new(pair.line_col().into());
        let inners = pair.to_owned().into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match inners.as_slice() {
            [(Rule::asm_code, p)] => {
                let code = p.as_str().to_string();
                let mut r = Self::new(code);
                r.ast_item = ast_item;
                Ok(r)
            }
            _ => {
                let str = format!("cannot find {inners:?} in asm statement processing logic");
                error_string(str.to_string());
                Err(CLangError::NoFound((file!().to_string(), line!()), str))
            }
        }
    }
}

impl From<&str> for AsmStatement {
    fn from(s: &str) -> Self {
        Self {
            asm_code: s.to_string(), 
            ast_item: ASTItem::default(),
        }
    }
}