use std::fmt::Display;

use pest::iterators::Pair;

use crate::languages::clang::{ASTItem, CLangItem, clang_pest::Rule};
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use crate::languages::clang::traits::*;
use crate::languages::clang::declaration::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectDeclarator_Declarator {
    declarator: Declarator,
    values: Vec<DirectDelaratorPart>,
    ast_item : ASTItem,
}

impl DirectDeclarator_Declarator {
    pub fn new(declarator:Declarator, values:Vec<DirectDelaratorPart>, p:&Pair<Rule>) -> Self {
        Self { declarator, values, ast_item : ASTItem::new(p.line_col().into()) }
    }

    pub fn get_values(&self) -> &Vec<DirectDelaratorPart> {
        &self.values
    }

    pub fn get_declarator(&self) -> &Declarator {
        &self.declarator
    }

    pub fn has_values(&self) -> bool {
        !self.get_values().is_empty()
    }

    /// check if this direct_declarator has parameters, 
    /// it is used to determine if this direct_declarator is function definition or not
    pub fn has_parameters(&self) -> bool {
        self.get_values().iter()
            .any(|x| x.is_parameter())
    }

    pub fn get_id_name(&self) -> String {
        self.declarator.get_id_name()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![(&mut self.declarator).into()];

        for x in self.values.iter_mut() {
            r.push(x.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.declarator.access_children(context);
        for x in self.values.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::DirectDeclarator_Declarator(CLangItem::new(self)));
    }
}

impl ASTItemTrait for DirectDeclarator_Declarator {
    fn get_name(&self) -> String {
        format!("DirectDecl_Decl({})", self.get_id())
    }

    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for DirectDeclarator_Declarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        r.push_str(&format!("{}", self.declarator));
        for x in self.values.iter() {
            r.push_str(&format!("{}", x));
        }
        write!(f, "{}", r)
    }
}