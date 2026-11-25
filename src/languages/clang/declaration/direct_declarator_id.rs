use std::fmt::Display;

use pest::iterators::Pair;

use ascent::ascent;

use crate::languages::clang::clang_types::ArrayDimension;
use crate::languages::clang::{ASTItem, CLangItem, clang_pest::Rule};

use crate::languages::clang::clang_pest::Id;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use crate::languages::clang::traits::*;
use crate::languages::clang::declaration::*;

ascent! {
    struct FindConstantValue;

    // facts
    relation node(CLangItem);

    relation child_of(CLangItem, CLangItem);
    relation single_child_of(CLangItem, CLangItem);
    relation single_object(CLangItem);
    relation unilinage_of(CLangItem, CLangItem);
    relation is_leaf_node(CLangItem);
    relation id_node(CLangItem);

    // rules
    child_of(child, parent) <--
        node(parent), node(child),
        if parent.is_parent_of(child);

    single_child_of(child, parent) <-- 
        node(parent), node(child),
        if parent.is_parent_of(child) && parent.get_children().len() == 1;
    
    unilinage_of(parent,child) <-- 
        single_child_of(child, parent);
    unilinage_of(parent,grandchild) <--
        single_child_of(child, parent),
        unilinage_of(child, grandchild);

    is_leaf_node(node) <--
        node(node),
        if node.get_children().is_empty();

    id_node(x) <--   //follow the single line from direct_decl_id/root -> direct_decl_part -> constant_expr -> ...-> x
        node(root), if root.is_direct_declarator_id(),
        node(direct_decl_part), child_of(direct_decl_part, root), if direct_decl_part.is_direct_declarator_part(),
        node(constant_expr), child_of(constant_expr, direct_decl_part), if constant_expr.is_constant_expression(),
        unilinage_of(constant_expr, x),
        is_leaf_node(x);
}


#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectDeclarator_Id {
    id : Id,
    values : Vec<DirectDelaratorPart>,
    ast_item : ASTItem,
}

impl DirectDeclarator_Id {
    pub fn new(id:Id, values:Vec<DirectDelaratorPart>, p:&Pair<Rule>) -> Self {
        Self { id, values, ast_item : ASTItem::new(p.line_col().into()) }
    }

    pub fn new_with_ast_item(id:Id, values:Vec<DirectDelaratorPart>, ast_item:ASTItem) -> Self {
        Self { id, values, ast_item }
    }

    pub fn new_from_id_ast_item(id:Id, ast_item:ASTItem) -> Self {
        Self::new_with_ast_item(id, vec![], ast_item)
    }

    pub fn get_id_name(&self) -> String {
        self.id.to_string()
    }

    pub fn get_array_info(&self) -> Option<(Id, ArrayDimension)> {
        if self.has_values() == false {
            return None;
        }

        let id = self.id.clone();

        let has_clang_item = self.get_ast_item().get_current_node().is_some();
        if has_clang_item {
            let mut prog = FindConstantValue::default();
            prog.node = self.get_all_clangitems_for_logic();
            prog.run();

            let r = prog.id_node.into();
            Some((id, r))
        }
        else {
            let mut demision_values = vec![];
            for n in self.get_values() {
                if let Some(v) = n.get_constant_expression()
                    .and_then(|n| n.get_const_value()) {
                    demision_values.push(v);
                }
            }

            Some((id, demision_values.into()))
        }
    }

    pub fn get_values(&self) -> &Vec<DirectDelaratorPart> {
        &self.values
    }

    pub fn has_values(&self) -> bool {
        !self.get_values().is_empty()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        for x in self.values.iter_mut() {
            r.push(x.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        for x in self.values.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::DirectDeclarator_Id(CLangItem::new(self)));
    }
}

impl ASTItemTrait for DirectDeclarator_Id {
    fn get_name(&self) -> String {
        format!("DirectDeclId_{}({})", self.get_id_name(), self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }

    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }
}

impl Display for DirectDeclarator_Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        r.push_str(&self.id);
        for x in self.values.iter() {
            r.push_str(&format!("{}", x));
        }
        write!(f, "{}", r)
    }
}
