use std::fmt::Display;

use pest::iterators::Pair;

use crate::languages::clang::{ASTItem, CLangItem, clang_pest::Rule};
use crate::languages::clang::CLangError;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use core_utils::debug::*;
use crate::languages::clang::traits::*;
use crate::languages::clang::declaration::*;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectDeclarator {
    Id(ASTItem, DirectDeclarator_Id),                       // identifier + direct_declarator_part*
    Declarator(ASTItem, DirectDeclarator_Declarator),       // declarator + direct_declarator_part*
}

impl DirectDeclarator {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        let ast_item = ASTItem::new(pair.line_col().into());
        let pairs = pair.to_owned().into_inner().map(|x| { (x.as_rule(), x) }).collect::<Vec<_>>();
        
        match pairs.as_slice() {
            [(Rule::identifier, p), rest@..] => {
                let id = p.as_str().to_owned();
                let l = rest.iter().filter_map(|x| DirectDelaratorPart::from_pair(&x.1, context).unwrap())
                                                            .collect::<Vec<_>>();
                Ok(Self::Id(ast_item, DirectDeclarator_Id::new(id, l, p)))
            }
            [(Rule::declarator, p), rest@..] => {
                let l = rest.iter().filter_map(|x| DirectDelaratorPart::from_pair(&x.1, context).unwrap())
                                                            .collect::<Vec<_>>();
                let d = Declarator::from_pair(p, context)?;
                Ok(Self::Declarator(ast_item, DirectDeclarator_Declarator::new(d, l, p)))
            }
            _ => {
                let str = format!("cannot find {pair:?} in DirectDeclarator processing logic");
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn new_id(ast_item:ASTItem, id:&str) -> Self {
        let direct_declarator_id = DirectDeclarator_Id::new_from_id_ast_item(id.to_string(), ast_item.clone());
        Self::Id(ast_item, direct_declarator_id)
    }

    pub fn get_id_name(&self) -> String {
        match self {
            Self::Declarator(_, n) => n.get_id_name(),
            Self::Id(_, n) => n.get_id_name(),            
        }
    }

    pub fn has_values(&self) -> bool {
        match self {
            Self::Declarator(_, n) => n.has_values(),
            Self::Id(_, n) => n.has_values(),
        }
    }

    /// check if this direct_declarator has parameters, 
    /// it is used to determine if this direct_declarator is function definition or not
    pub fn has_parameter(&self) -> bool {
        self.get_direct_declarator_parts().iter()
            .any(|x| x.is_parameter())
    }

    pub fn get_direct_declarator_parts(&self) -> &Vec<DirectDelaratorPart> {
        match self {
            Self::Declarator(_, n) => n.get_values(),
            Self::Id(_, n) => n.get_values(),
        }
    }

    pub fn get_direct_decl_id(&self) -> Option<&DirectDeclarator_Id> {
        match self {
            Self::Id(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Declarator(_, n) => vec![n.into()],
            Self::Id(_, n) => vec![n.into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Declarator(_, n) => {
                n.access_children(context);
            }
            Self::Id(_, n) => {
                n.access_children(context);
            }
        }

        context.access_clang_item(CLangItem::DirectDeclarator(CLangItem::new(self)));
    }
}

impl ASTItemTrait for DirectDeclarator {
    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Declarator(n, _) => n,
            Self::Id(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Declarator(n, _) => n,
            Self::Id(n, _) => n,
        }
    }

    fn get_name(&self) -> String {
        match self {
            Self::Declarator(_, _) => format!("DirectDecl_Decl({})", self.get_id()),
            Self::Id(_, _) => format!("DirectDecl_Id({})", self.get_id())
        }
        
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for DirectDeclarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declarator(_, n) => write!(f, "{}", n),
            Self::Id(_, n) => write!(f, "{}", n),
        }
    }
}
