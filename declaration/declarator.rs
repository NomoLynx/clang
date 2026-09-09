use std::fmt::Display;

use pest::iterators::Pair;

use crate::languages::clang::clang_pest::{ParameterType, Pointer};
use crate::languages::clang::{ASTItem, CLangItem, clang_pest::Rule};
use crate::languages::clang::CLangError;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use core_utils::debug::*;
use crate::languages::clang::traits::*;
use crate::languages::clang::declaration::*;

/// pest Rule::declarator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declarator {
    // poniter: Pointer,
    poniter: Option<Pointer>, //it is optional rule
    declarator: Box<DirectDeclarator>,
    ast_item : ASTItem,
}

impl Declarator {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::declarator); //ensure it is rule:declarator

        let ast_item = ASTItem::new(pair.line_col().into());
        let pairs = pair.to_owned().into_inner().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        match pairs.as_slice() {
            [(Rule::pointer, p), (Rule::direct_declarator, p1)] => 
                Ok(Declarator { poniter: Some(Pointer::from_pair(&p, context)?), declarator : Box::new(DirectDeclarator::from_pair(p1, context)?), ast_item, }),
            [(Rule::direct_declarator, p1)] => 
                Ok(Declarator { poniter: None, declarator : Box::new(DirectDeclarator::from_pair(p1, context)?), ast_item, }),
            _ => {
                let str = format!("cannot find {pairs:?} in Declarator processing logic");
                error_string(str.to_string());
                Err(CLangError::NoFound((file!().to_string(), line!()), str))
            }
        }
    }
    
    pub fn new_with_id(ast_item:ASTItem, id:&str) -> Self {
        let direct_declarator_id = DirectDeclarator_Id::new_from_id_ast_item(id.to_string(), ast_item.clone());
        let direct_declarator = DirectDeclarator::Id(ast_item.clone(), direct_declarator_id);
        Self { poniter: None, declarator : Box::new(direct_declarator), ast_item }
    }

    pub fn direct_decl_has_values(&self) -> bool {
        self.declarator.has_values()
    }

    pub fn get_id_name(&self) -> String {
        self.declarator.get_id_name()
    }

    pub fn get_direct_declarator(&self) -> &DirectDeclarator {
        &*self.declarator
    }

    pub fn get_pointer_level(&self) -> u8 {
        self.poniter
            .as_ref()
            .map(|p| p.get_level())
            .unwrap_or(0)
    }

    pub fn get_parameters(&self) -> Vec<&ParameterType> {
        self.get_direct_declarator()
            .get_direct_declarator_parts().iter()
            .filter_map(|x| x.get_parameters())
            .collect::<Vec<_>>()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        r.push((&mut *self.declarator).into());

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.declarator.access_children(context);

        context.access_clang_item(CLangItem::Declarator(CLangItem::new(self)));
    }
}

impl ASTItemTrait for Declarator {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("Declarator({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for Declarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        if let Some(p) = &self.poniter {
            r.push_str(&format!("{} ", p));
        }
        r.push_str(&format!("{}", self.declarator));
        write!(f, "{}", r)
    }
}
