use ascent::ascent;
use core_utils::debug::error_string;

use crate::languages::clang::CLangItem;
use crate::languages::clang::logic::*;
use crate::languages::clang::traits::ASTItemTrait;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/languages/clang/logic/expr_post_ref.gen.rs"));

/// get the postfix reference patterns of the expression, 
/// which is used to get the storage location of the expression
/// return tuple types are (expression_postfix, postfix_expression_part) OR (atom_id, postfix_expression_part)
pub fn get_postfix_ref(items: Vec<CLangItem>) -> Vec<(CLangItem, CLangItem)> {
    let mut logic = ExprPostfixRefLogic::default();
    logic.node = to_logic_clangitem_vec(items);
    logic.run();
    let mut r = logic.result;
    r.sort_by_key(|(left, _)| left.get_ast_item().get_sequence_id());
    r
}

/// get the postfix reference patterns of the expression, and also get the names of the postfix expression parts
pub fn get_postfix_ref_expr_and_names(item:&dyn ASTItemTrait) -> (CLangItem, Vec<String>) {
    let patterns = get_postfix_ref(item.get_all_clangitems());

    if patterns.is_empty() {
        error_string(format!("cannot find postfix expression pattern for item whose id = {}", item.get_id()));
        panic!("cannot find postfix expression pattern for current expression");
    }

    let mut names = vec![];

    assert!(patterns.first().unwrap().0.is_expression());

    let expr = patterns.first().unwrap().0.clone();
    for (_, right) in patterns.iter() {
        let postfix_expr = right.get_postfix_expression_part().unwrap();
        if let Some(name) = postfix_expr.get_dot_or_ptr_member() {
            names.push(name.to_string());
        }
    }

    (expr, names)
}