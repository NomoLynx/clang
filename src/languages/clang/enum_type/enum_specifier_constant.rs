use std::fmt::Display;

use pest::iterators::Pair;
use core_utils::number::get_i64_from_str;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::{CLangError, CLangItem, clang_asm_code_gen::*};
use core_utils::debug::*;

use crate::languages::clang::{ASTItem, clang_pest::{Id, Rule}, constant_expression::ConstantExpression};

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnumSpecifier_Constant {
    id : Id,
    const_expr : Option<ConstantExpression>,
    base_type : CLangTypes,
    base_value : isize,
    ast_item : ASTItem,
}

impl EnumSpecifier_Constant {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::enumerator);
        let inners = pair.to_owned().into_inner().into_iter()
                        .map(|x| (x.as_rule(), x))
                        .collect::<Vec<_>>();
        let ast_item = ASTItem::default();
        let base_type = CLangTypes::Invalid;
        match inners.as_slice() {
            [(Rule::identifier, p)] => {
                Ok(Self { id: p.as_str().to_string(), 
                        const_expr: None, 
                        ast_item, 
                        base_type, 
                        base_value: 0, })
            }
            [(Rule::identifier, p), (Rule::constant_expression, p2)] => {
                Ok(Self { id: p.as_str().to_string(), 
                    const_expr: Some(ConstantExpression::from_pair(p2, context)?), 
                    ast_item, 
                    base_type, 
                    base_value: 0,
                })
            }
            _ => {
                let str =
                    format!("cannot find {inners:?} in EnumSpecifier_Constant processing logic");
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    /// get the base value of the enum constant, this is the integer value assigned to the enum constant
    pub fn get_base_value(&self) -> isize {
        self.base_value
    }

    /// set the base value of the enum constant, this is the integer value assigned to the enum constant
    pub fn set_base_value(&mut self, value:isize) {
        self.base_value = value;
    }

    pub fn set_base_type(&mut self, base_type:CLangTypes) {
        self.base_type = base_type;
    }

    pub fn get_base_type(&self) -> &CLangTypes {
        &self.base_type
    }

    /// get the enum item name
    pub fn get_enum_item(&self) -> String {
        self.id.clone()
    }

    pub fn has_explicit_value(&self) -> bool {
        self.const_expr.is_some()
    }

    pub fn try_get_explicit_base_value(&self) -> Option<isize> {
        self.const_expr
            .as_ref()
            .and_then(|expr| expr.get_const_value())
            .and_then(|v| get_i64_from_str(&v).ok())
            .map(|v| v as isize)
    }

    /// get markdown table cell representation
    pub fn to_md_table_cell(&self, config:&CodeGenConfiguration) -> MDTableCell {
        let type_size = self.get_type_size(config);
        let asm_type = match type_size { 1 => ".byte", 2 => ".2byte", 4 => ".word", 8 => ".dword", _ => panic!("unsupported enum base type size"), };
        let tag = format!("{}", self.get_enum_item());
        let value = "0".to_string();
        MDTableCell::new(asm_type.to_string(), value, Some(tag))
    }

    /// get the type size in bytes using the configuration's enum base type size
    pub fn get_type_size(&self, config:&CodeGenConfiguration) -> usize {
        config.get_enum_base_type_size_in_bytes() as usize
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        if let Some(expr) = self.const_expr.as_mut() {
            vec![expr.into()]
        }
        else {
            vec![]
        }
    }

    pub fn access_children(&mut self,  context:&mut CodeGenContext) {
        if let Some(expr) = self.const_expr.as_mut() {
            expr.access_children(context);
        }

        context.access_clang_item(CLangItem::EnumSpecifier_Constant(CLangItem::new(self)));
    }
}

impl ASTItemTrait for EnumSpecifier_Constant {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("EnumSpec_Const_{}({})", self.get_enum_item(), self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for EnumSpecifier_Constant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.id)?;
        if let Some(expr) = self.const_expr.as_ref() {
            write!(f, " = {}", expr)?;
        }
        write!(f, ",\n")
    }
}