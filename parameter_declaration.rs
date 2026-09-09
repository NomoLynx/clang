use std::fmt::Display;

use pest::iterators::Pair;
use core_utils::debug::*;
use crate::languages::clang::{clang_pest::AbstractDeclarator, traits::ASTItemTrait};
use crate::languages::clang::symbol_table::*;
use crate::languages::clang::declaration::*;
use crate::languages::clang::clang_types::CLangTypes;
use super::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::{DeclatorOrAbstractDeclarator, Rule}, type_specifier::TypeSpecifier};

///since in the rule:ParameterDeclaration, (declartor|abstractDeclarator)? is optional
/// Change the structure from enum to struct
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterDeclaration {
    declarator_specifiers: Vec<TypeSpecifier>,
    parameter_declaration_part: Option<DeclatorOrAbstractDeclarator>,
    is_array_parameter: bool,
    ast_item : ASTItem,
}

impl ParameterDeclaration {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::parameter_declaration); //ensure it is rule:parameter_declaration

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner_pairs: Vec<Pair<Rule>> = pair.to_owned().into_inner().collect();

        let mut parameter_declaration = ParameterDeclaration {
            declarator_specifiers: vec![],
            parameter_declaration_part: None,
            is_array_parameter: false,
            ast_item,
        };

        parameter_declaration.declarator_specifiers = inner_pairs[0]
            .to_owned()
            .into_inner()
            .map(|pair| TypeSpecifier::from_pair(&pair, context).unwrap())
            .collect();

        //parse the rule:declarator or abstract_declarator if they exist
        if inner_pairs.len() == 2 {
            let inner_pair = inner_pairs[1].to_owned();
            let ast_item = ASTItem::new(inner_pair.line_col().into());
            match inner_pair.as_rule() {
                Rule::declarator => {
                    // Record whether this parameter declarator has array suffix(es),
                    // including unsized forms like `x[]`.
                    let declarator_source = inner_pair.as_str().to_string();
                    let declarator = Declarator::from_pair(&inner_pair, context)?;
                    let is_direct_id = declarator
                        .get_direct_declarator()
                        .get_direct_decl_id()
                        .is_some();
                    parameter_declaration.is_array_parameter =
                        is_direct_id && declarator_source.contains('[');

                    parameter_declaration.parameter_declaration_part =
                        Some(DeclatorOrAbstractDeclarator::Declarator(
                            ast_item,
                            declarator,
                        ))
                }
                Rule::abstract_declarator => {
                    parameter_declaration.parameter_declaration_part =
                        Some(DeclatorOrAbstractDeclarator::AbstractDeclarator(
                            ast_item,
                            AbstractDeclarator::from_pair(&inner_pair, context)?,
                        ))
                }
                Rule::identifier => {
                    let declarator = Declarator::new_with_id(ast_item.clone(), inner_pair.as_str());
                    parameter_declaration.parameter_declaration_part = 
                        Some(DeclatorOrAbstractDeclarator::Declarator(
                            ast_item,
                            declarator,
                        ));
                }
                _ => {
                    let str = format!("cannot process {inner_pair:?} in ParameterDeclaration");
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }
        Ok(parameter_declaration)
    }

    /// return const "SymbolType::Declaration"
    pub fn get_symbol_table_type_parameter_declaration() -> SymbolType {
        SymbolType::Declaration
    }

    /// get declaraton part, usually it is the 'i' in 'int i'
    pub fn get_declartion_part(&self) -> Option<&DeclatorOrAbstractDeclarator> {
        self.parameter_declaration_part.as_ref()
    }

    /// get parameter type as string
    pub fn get_type_name_string(&self) -> String {
        self.get_symbol_table_clang_type().to_string()
    }

    /// get parameter type for symbol table/signature output.
    /// C parameter arrays decay to pointers in function signatures.
    pub fn get_symbol_table_clang_type(&self) -> CLangTypes {
        let mut type_specifiers = self.declarator_specifiers.clone();
        let pointer_level = self.parameter_declaration_part
            .as_ref()
            .map(|x| x.get_pointer_level())
            .unwrap_or(0);

        if pointer_level > 0 {
            type_specifiers.push(TypeSpecifier::Pointer(ASTItem::default(), pointer_level));
        }

        let mut ty: CLangTypes = (&type_specifiers).into();
        if self.is_array_parameter {
            ty = ty.to_pointer_type();
        }

        if ty.contains_invalid_clang_item_link() {
            fix_struct_union_link_in_type(&mut ty, self.get_ast_item());
        }

        ty
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        let mut type_specifiers = self.declarator_specifiers.clone();
        let pointer_level = self.parameter_declaration_part
            .as_ref()
            .map(|x| x.get_pointer_level())
            .unwrap_or(0);

        if pointer_level > 0 {
            type_specifiers.push(TypeSpecifier::Pointer(ASTItem::default(), pointer_level));
        }

        let mut ty: CLangTypes = (&type_specifiers).into();
        if self.is_array_parameter {
            // Keep array shape for parameter array declarators so downstream
            // array-index codegen can still compute element addressing.
            let (array_name, array_dimension) = self
                .parameter_declaration_part
                .as_ref()
                .and_then(|part| part.get_declarator())
                .and_then(|decl| decl.get_direct_declarator().get_direct_decl_id())
                .and_then(|id_decl| id_decl.get_array_info())
                .map(|(name, dim)| (name, dim))
                // unsized `[]` has no constant-expression dimension node; use 1 as placeholder.
                .unwrap_or_else(|| {
                    (self.get_var_name().unwrap_or_default(), vec![1usize].into())
                });

            ty = CLangTypes::create_array_type(ty, array_name, array_dimension);
        }

        if ty.contains_invalid_clang_item_link() {
            // Parameters are materialized into symbol-table entries/attributes early,
            // so resolve struct/union links here to avoid `(?)` in markdown output.
            fix_struct_union_link_in_type(&mut ty, self.get_ast_item());
        }
        ty
    }

    /// get var name
    pub fn get_var_name(&self) -> Option<String> {
        let r = self.parameter_declaration_part.as_ref()
            .map(|x| x.get_declarator()
                                                    .and_then(|x| Some(x.get_id_name()))
                );

        r.flatten()
    }

    pub fn get_type(&self) -> &Vec<TypeSpecifier> {
        &self.declarator_specifiers
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        for x in self.declarator_specifiers.iter_mut() {
            r.push(x.into());
        }

        if let Some(n) = self.parameter_declaration_part.as_mut() {
            r.push(n.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        for x in self.declarator_specifiers.iter_mut() {
            x.access_children(context);
        }

        if let Some(n) = self.parameter_declaration_part.as_mut() {
            n.access_children(context);
        }

        context.access_clang_item(CLangItem::ParameterDeclaration(CLangItem::new(self)));
    }
}

impl Display for ParameterDeclaration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        for x in self.declarator_specifiers.iter() {
            r.push_str(&format!("{}, ", x));
        }
        if let Some(n) = &self.parameter_declaration_part {
            r.push_str(&format!("{}", n));
        }
        write!(f, "{}", r)
    }
}

impl ASTItemTrait for ParameterDeclaration {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("ParameterDeclaration({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}