use core::panic;

use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::*;

pub trait ASTItemTrait {
    fn get_ast_item(&self) -> &ASTItem;
    fn get_ast_item_mut(&mut self) -> &mut ASTItem;
    fn get_name(&self) -> String;
    fn get_id(&self) -> usize;

    /// get all clang items in the subtree of this item, including itself
    fn get_all_clangitems(&self) -> Vec<CLangItem> {
        let mut r = vec![self.get_clang_item()];
        let ast_item = self.get_ast_item();
        let mut context = ast_item.into();
        ast_item.preorder_tree(&mut context, 
                &mut |m, _| { 
                    let item = m.get_current_node().unwrap().clone();
                    r.push(item);
                }, 
                |_| { true });
        r
    }

    fn get_all_clangitems_for_logic(&self) -> Vec<(CLangItem,)> {
        self.get_all_clangitems().into_iter()
            .map(|x| (x, ))
            .collect::<Vec<_>>()
    }

    /// get clang item. If called before 2nd parse, it will return None
    fn get_clang_item(&self) -> CLangItem {
        if let Some(n) = self.get_ast_item().get_current_node() {
                n.clone()
        }
        else {
            error_string(format!("ASTItemTrait::get_clang_item: current node is None, this should never happen, please check the code logic"));
            panic!("ASTItemTrait::get_clang_item: current node is None, this should never happen, please check the code logic");
        }
    }

    fn add_semantic_linkage(&mut self, type_def_clang_item:CLangItem) {
        let n = self.get_ast_item_mut();
        n.add_semantic_linkage(SemanticLinkage::Definition(type_def_clang_item.clone()));
        type_def_clang_item.get_ast_item_mut().add_semantic_linkage(SemanticLinkage::DefinitionUsage(n.get_current_node().unwrap().clone()));
    }

    /// get full name, which is the name of the item plus the names of its parent scopes, separated by "::"
    fn get_full_name(&self) -> String {
        let mut names = vec![self.get_name()];
        let mut current_item = self.get_ast_item();
        while let Some(parent) = current_item.get_scope_item() {
            let parent_ast_item = parent.get_scope_ast_item();
            names.push(parent_ast_item.get_type_name());
            current_item = parent_ast_item;
        }

        names.reverse();
        names.join("::")
    }

    fn get_runtime_type(&self) -> Option<&CLangTypes> {
        self.get_ast_item()
            .get_runtime_type()
    }
}

impl ASTItemTrait for AsmStatement {
    fn get_ast_item(&self) -> &ASTItem {
        self.get_ast_item()
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        self.get_ast_item_mut()
    }

    fn get_name(&self) -> String {
        format!("AsmStat({})", self.get_ast_item().get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}