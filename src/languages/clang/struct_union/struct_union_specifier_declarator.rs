use std::fmt::{Display, Formatter};

use core_utils::debug::*;
use core_utils::number::get_i32_from_str;
use pest::iterators::Pair;

use crate::languages::clang::clang_asm_code_gen::CodeGenConfiguration;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::clang_pest::Rule;
use crate::languages::clang::ASTItem;
use crate::languages::clang::CLangItem;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::struct_union::STRUCT_DECLARATOR_ANONYMOUS;
use crate::languages::clang::struct_union::StructDeclarator;
use crate::languages::clang::traits::ASTItemTrait;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructUnionSpecifier_Declarator {
    /// the type of this struct/union item, it can be a basic type like int, or a composite type like structA, or even a pointer type like int*;
    declare_type : CLangTypes,
    declarators : Vec<StructDeclarator>,
    ast_item: ASTItem,
}

impl StructUnionSpecifier_Declarator {
    pub fn new(declare_type:CLangTypes, declarators : Vec<StructDeclarator>, p:&Pair<Rule>) -> Self {
        let declare_type = Self::process_declarator_type(declare_type, &declarators);
        Self { 
            declare_type, 
            declarators, 
            ast_item: ASTItem::new(p.line_col().into()), 
        }
    }

    fn process_declarator_type(declare_type:CLangTypes, declarators : &Vec<StructDeclarator>) -> CLangTypes {
        if declarators.len() != 1 {
            return declare_type;
        }

        let decl = &declarators[0];
        if let Some(decl) = decl.get_name_id() {
            let mut result_type = if decl.get_pointer_level() > 0 {
                CLangTypes::make_pointer_type(declare_type, decl.get_pointer_level() as usize)
            }
            else {
                declare_type
            };

            let mut dimension = vec![];
            let declarator_parts = decl.get_direct_declarator().get_direct_declarator_parts();
            for n in declarator_parts.iter() {
                if let Some(dimension_v) = n.get_constant_expression()
                                                 .and_then(|x| x.get_const_value())
                                                 .and_then(|x| get_i32_from_str(&x).ok()) {
                    dimension.push(dimension_v as usize);
                }
            }

            if dimension.is_empty() {
                result_type
            }
            else {
                assert_eq!(dimension.len(), declarator_parts.len(), "if the declarator has array dimension, then all the declarator parts must be array dimension");
                result_type = CLangTypes::create_array_type(result_type, String::new(), dimension.into());
                result_type
            }
        }
        else {
            declare_type
        }
    }

    pub fn get_declarators(&self) -> &Vec<StructDeclarator> {
        &self.declarators
    }

    /// get the name of this struct/union item
    pub fn get_struct_item_name(&self) -> String {
        let anonymous_name = STRUCT_DECLARATOR_ANONYMOUS.to_string();
        let declarators = self.get_declarators();
        let ids = declarators.iter()
                            .map(|x| x.get_id_name())
                            .collect::<Vec<_>>();
        let id_name = ids.first().unwrap_or(&anonymous_name);
        id_name.to_string()
    }

    /// get type size in bytes
    pub fn get_type_size(&self, config:&CodeGenConfiguration) -> usize {
        let r = self.get_item_type()
            .get_type_size_in_bytes2(config)
            .unwrap();

        if r == 0{ 
            if let Some(type_name) = self.get_item_type()
                                                .get_struct_union()
                                                .and_then(|x| x.get_name_id()) {
                if let Some(entry) = self.get_ast_item().get_symbol_recursive_type_definition(type_name) {
                    if let Some(struct_union) = entry.get_struct_union_specifier() {
                        return struct_union.get_type_size(config);
                    }
                }
            }
        }

        r 
    }

    /// get item type of this struct/union item, it can be a basic type like int, 
    /// or a composite type like structA, or even a pointer type like int*;
    pub fn get_item_type(&self) -> &CLangTypes {
        &self.declare_type
    }

    pub fn get_item_type_mut(&mut self) -> &mut CLangTypes {
        &mut self.declare_type
    }

    /// check if this struct/union item declaration is invalid, 
    /// e.g. it has a struct/union type but the actual struct/union definition is not found
    pub fn contains_invalid_struct_union_declaration(&self) -> bool {
        self.get_item_type().contains_invalid_clang_item_link()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        for n in self.declarators.iter_mut() {
            r.push(n.into())
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        for n in self.declarators.iter_mut() {
            n.access_children(context);
        }

        context.access_clang_item(CLangItem::StructUnionSpecifier_Declarator(CLangItem::new(self)));
    }
}

impl ASTItemTrait for StructUnionSpecifier_Declarator {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("StructUntionSpecifier_Decl({})_({})", self.get_id(), self.get_item_type())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for StructUnionSpecifier_Declarator {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let declarators = self.get_declarators();
        let ty = self.get_item_type();
        let declarators_str = declarators.iter()
                                        .map(|x| format!("{}", x))
                                        .collect::<Vec<_>>()
                                        .join(", \r\n");
        write!(f, "{} {}", ty, declarators_str)
    }
}