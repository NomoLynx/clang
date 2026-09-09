use ascent::ascent;

use crate::languages::clang::CLangItem;
use crate::languages::clang::logic::*;

ascent! {
    pub struct DelStruct;

    relation node(CLangItem);
    relation is_declaration(CLangItem);
    relation is_struct_or_union(CLangItem);
    relation single_child_of(CLangItem, CLangItem);
    relation structs(CLangItem);

    // rules
    is_declaration(item) <-- node(item), if item.is_declaration();
    is_struct_or_union(item) <-- node(item), if item.is_struct_or_union();

    single_child_of(child, parent) <-- 
        node(parent), node(child),
        if parent.is_parent_of(child) && parent.get_children().len() == 1;

    structs(item) <-- 
        is_declaration(root),
        is_struct_or_union(item),
        single_child_of(item, root);

    structs(item) <-- is_struct_or_union(item);
}

/// Execute the del struct logic to get all struct or union definitions that are directly 
/// under a declaration, or itself is a struct or union. this can be duplicated, but i want to leave it 
/// here for future simplification
pub fn execute_del_struct_logic(items: Vec<CLangItem>) -> Vec<CLangItem> {
    let mut logic = DelStruct::default();
    logic.node = to_logic_clangitem_vec(items);
    logic.run();
    let results = from_logic_clangitem_vec(logic.structs);
    results
}