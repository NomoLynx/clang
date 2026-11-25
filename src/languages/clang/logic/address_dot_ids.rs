use ascent::ascent;

use crate::languages::clang::CLangItem;
use crate::languages::clang::logic::{from_logic_clangitem_vec, to_logic_clangitem_vec};
use crate::languages::clang::symbol_table::SymbolEntry;
use crate::languages::clang::tanry_expression::TanryExpression;
use crate::languages::clang::traits::ASTItemTrait;

ascent! {
    pub struct AddressDotItems;

    relation node(CLangItem);
    relation single_child_of(CLangItem, CLangItem);
    relation child_of(CLangItem, CLangItem);

    relation is_id(CLangItem);
    relation is_atom(CLangItem);
    relation is_expr(CLangItem);
    relation is_address_dot_expr(CLangItem);

    relation ids(CLangItem);

    // rules
    is_id(item) <-- node(item), if item.is_atom_item_id();
    is_atom(item) <-- node(item), if item.is_atom();
    is_expr(item) <-- node(item), if item.is_expression();
    is_address_dot_expr(item) <-- node(item), if item.is_expression_expression();

    single_child_of(child, parent) <-- 
        node(parent), node(child),
        if parent.is_parent_of(child) && parent.get_children().len() == 1;

    child_of(child, parent) <-- node(parent), node(child), if parent.is_parent_of(child);

    ids(item) <-- 
        is_address_dot_expr(root),
        is_expr(expr), child_of(expr, root),
        is_atom(atom), single_child_of(atom, expr),
        is_id(item), single_child_of(item, atom);
}

/// Find all the id items in the address dot expressions, and return them in the order they appear in the source code.
pub fn execute_find_address_dot_items(items: Vec<CLangItem>) -> Vec<CLangItem> {
    let mut logic = AddressDotItems::default();
    logic.node = to_logic_clangitem_vec(items);
    logic.run();
    let results = from_logic_clangitem_vec(logic.ids);

    let mut ids = results.into_iter().map(|clang_item| {
                    if clang_item.is_atom_item() {
                        (clang_item.get_sequence_id(),
                        clang_item)
                    }
                    else {
                        panic!("Cannot find id for clang item in Index expression: {:?}", clang_item);
                    }
                }).collect::<Vec<_>>();
    ids.sort_by_key(|&(x,_)| x);
    ids.into_iter().map(|(_, id)| id).collect()
}

pub fn get_symbol_entries_from_address_dot_expr(tanry:&TanryExpression) -> Vec<&SymbolEntry> {
    let items = tanry.get_all_clangitems();
    let ids = execute_find_address_dot_items(items);

    let ids = ids.iter()
                .map(|x| x.get_atom_item()
                                        .and_then(|x| x.get_id_value()))
                .map(|x| x.unwrap().to_string())
                .collect::<Vec<_>>();

    let r = tanry.get_ast_item().get_var_symbol_entries_recursive(ids);
    r
}