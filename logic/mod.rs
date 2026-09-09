pub mod duplication_check;
pub mod find_address_taken_item;
pub mod address_dot_ids;
pub mod del_struct;
pub mod iterator_var_def;
pub mod decl_facts;
pub mod fn_invoke;
pub mod expr_post_ref;

pub use duplication_check::*;
pub use find_address_taken_item::*;
pub use address_dot_ids::*;
pub use iterator_var_def::*;
pub use decl_facts::*;
pub use fn_invoke::*;
pub use expr_post_ref::*;

use crate::languages::clang::CLangItem;

pub (self) fn to_logic_clangitem_vec(items:Vec<CLangItem>) -> Vec<(CLangItem,)> {
    items.into_iter()
        .map(|x| (x, ))
        .collect()
}

pub (self) fn from_logic_clangitem_vec(items:Vec<(CLangItem, )>) -> Vec<CLangItem> {
    items.iter()
        .map(|x| x.0.clone())
        .collect::<Vec<_>>()
}

mod base_logic {
    ascent::ascent_source! {
        general:
        relation node(CLangItem);
        
        // general connection rules
        relation single_child_of(CLangItem, CLangItem);
        relation child_of(CLangItem, CLangItem);
        relation parent_of(CLangItem, CLangItem);
        relation ancestor_of(CLangItem, CLangItem);
        relation unilinage_of(CLangItem, CLangItem);

        single_child_of(child, parent) <-- 
            node(parent), node(child),
            if parent.is_parent_of(child) && parent.get_children().len() == 1;

        child_of(child, parent) <-- node(parent), node(child), if parent.is_parent_of(child);
        parent_of(parent, child) <-- node(parent), node(child), if parent.is_parent_of(&child);

        ancestor_of(ancestor, descendant) <-- parent_of(ancestor, descendant);
        ancestor_of(ancestor, descendant) <-- parent_of(parent, descendant), ancestor_of(ancestor, parent);

        unilinage_of(parent,child) <-- 
            single_child_of(child, parent);
        unilinage_of(parent,grandchild) <--
            single_child_of(child, parent),
            unilinage_of(child, grandchild);
    }
}