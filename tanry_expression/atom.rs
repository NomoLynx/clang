use std::fmt::Display;

use pest::iterators::Pair;
use rust_macro::*;

use crate::ast::clr_basic_type_object::CLRBasicTypeObject;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::clang_pest::{Rule, UnaryOperator};
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::{ASTItem, CLangError, CLangItem};
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::post_operator::*;
use core_utils::debug::*;
use core_utils::string::*;

#[derive(Debug, Clone, PartialEq, Eq, Accessors, GetMut)]
pub struct Atom {
    unary_op : Option<UnaryOperator>,
    post_op : Option<PostOperator>,
    item : Box<AtomItem>,
    ast_item : ASTItem,
}

impl Atom {
    pub fn from_pair(pair:&Pair<Rule>, _context:&mut CodeGenContext) -> Result<Self, CLangError> {
        let convert_fn = |s:&str| -> Result<CLRBasicTypeObject, CLangError> {
            match parser_lib::expr_lang::expr_to_clrobj(s, None) {
                Ok(v) => Ok(v.into()),
                Err(_) => Err(CLangError::ParsingError(file!().to_string(), line!() as usize)),
            }
        };
        
        let ast_item = ASTItem::new(pair.line_col().into());
        match pair.as_rule() {
            Rule::integer => {
                let inners = pair.to_owned().into_inner().into_iter().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                match inners.as_slice() {
                    [(Rule::unary_op, p), (Rule::bool_value, p1), (Rule::post_op, p2)] => {
                        let unary_op = UnaryOperator::from_pair2(p)?;
                        let post_op = PostOperator::from_string(p2.as_str());
                        let v = CLRBasicTypeObject::from_bool(string_to_bool(p1.as_str()));
                        let item = Box::new(AtomItem::BasicTypeObject(ASTItem::new(p1.line_col().into()), v));
                        Ok(Self { unary_op : Some(unary_op), post_op : Some(post_op), item, ast_item, })
                    }
                    [(Rule::unary_op, p), (Rule::identifier, p1), (Rule::post_op, p2)] => {
                        let unary_op = UnaryOperator::from_pair2(p)?;
                        let post_op = PostOperator::from_string(p2.as_str());
                        let item = Box::new(AtomItem::Id(ASTItem::new(p1.line_col().into()), p1.as_str().to_string()));
                        Ok(Self { unary_op : Some(unary_op), post_op : Some(post_op), item, ast_item, })
                    }
                    [(Rule::signed_integer, p)] |
                    [(Rule::bin_number, p)] |
                    [(Rule::oct_number, p)] |
                    [(Rule::hex_number, p)] => {
                        let v = convert_fn(p.as_str())?;
                        let int_value = v.conv_i32().map_err(|_| CLangError::ParsingError(file!().to_string(), line!() as usize))?;
                        let item = AtomItem::BasicTypeObject(ASTItem::new(p.line_col().into()), int_value);
                        Ok(Self { unary_op: None, post_op: None, item: Box::new(item), ast_item })
                    }
                    _ => {
                        error_string(format!("inners = {inners:?} and pair = {pair:?}"));
                        todo!()
                    }
                }
            }
            Rule::float => {
                let v = convert_fn(pair.as_str())?;
                let float_value = v.conv_f32().map_err(|_| CLangError::ParsingError(file!().to_string(), line!() as usize))?;
                let item = AtomItem::BasicTypeObject(ASTItem::new(pair.line_col().into()), float_value);
                Ok(Self { unary_op: None, post_op: None, item: Box::new(item), ast_item, })
            }
            Rule::single_character => {
                let str = pair.as_str();
                let ch = str.chars()
                                    .nth(1)
                                    .ok_or(CLangError::ParsingError(file!().to_string(), line!() as usize))?; //remove quotes
                let obj = CLRBasicTypeObject::from_char(ch);
                let item = AtomItem::BasicTypeObject(ASTItem::new(pair.line_col().into()), obj);
                Ok(Self { unary_op : None, post_op : None, item: Box::new(item), ast_item })
            }
            Rule::string => {
                let str = pair.as_str();
                let obj = CLRBasicTypeObject::from_string(str[1..str.len()-1].to_string()); //remove quotes
                let item = AtomItem::BasicTypeObject(ASTItem::new(pair.line_col().into()), obj);
                Ok(Self { unary_op : None, post_op : None, item: Box::new(item), ast_item })
            }
            _ => {
                error_string(format!("pair = {pair:?}"));
                todo!()
            }
        }
    }

    /// get the id value of the item if any
    pub fn get_item_id(&self) -> Option<&String> {
        self.item.get_id_value()
    }

    pub fn has_unary_op(&self) -> bool {
        self.unary_op.is_some()
    }

    pub fn has_post_op(&self) -> bool {
        self.post_op.is_some()
    }

    /// check whether this atom is an addressing operation
    pub fn is_addressing(&self) -> bool {
        if self.get_item_id().is_some() && self.has_unary_op() {
            if let Some(op) = &self.unary_op {
                if *op == UnaryOperator::AddressOf {
                    return true;
                }
            }
        }

        false
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        let ty = self.item.get_clang_type();
        if self.has_unary_op()
        {
            ty.apply_unary_operator(self.get_unary_op().as_ref().unwrap())
        }
        else {
            return ty;
        }
    }

    pub fn get_composite_definition(&self) -> Option<CLangItem> {
        if self.has_unary_op() || self.has_post_op() {
            return None;
        }

        self.item.get_composite_definition_from_id()
    }

    /// get the string value if any
    /// string value from self's AtomItem (item)
    pub fn get_string_value(&self) -> Option<String> {
        self.item.get_string_value()
    }

    pub fn set_id_value_changed(&mut self, changed:bool) {
        self.item.set_id_value_changed(changed);
    }

    pub fn new_id(id:&str) -> Self {
        Self::new_id_with_ast_items(id, ASTItem::default(), ASTItem::default())
    }

    pub fn new_id_with_ast_items(id:&str, atom_ast_item:ASTItem, atom_item_ast_item:ASTItem) -> Self {
        Self {
            unary_op: None,
            post_op: None,
            item: Box::new(AtomItem::Id(atom_item_ast_item, id.to_string())),
            ast_item: atom_ast_item,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        vec![self.item.as_mut().into()]
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.item.access_children(context);

        context.access_clang_item(CLangItem::Atom(CLangItem::new(self)));
    }
}

impl Display for Atom {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match (self.unary_op.as_ref(), self.post_op.as_ref()) {
            (Some(pre), Some(post)) => format!("{pre} {} {post}", self.item),
            (Some(pre), None) => format!("{pre} {}", self.item),
            (None, Some(post)) => format!("{} {post}", self.item),
            (None, None) => format!("{}", self.item)
        };

        write!(f, "{}", s)
    }
}

impl ASTItemTrait for Atom {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("Atom({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}