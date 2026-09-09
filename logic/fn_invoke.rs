/// this section is to identify the function invoke

use ascent::ascent;
use core_utils::debug::debug_string;

use crate::languages::clang::CLangItem;
use crate::languages::clang::logic::*;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/languages/clang/logic/fn_invoke.gen.rs"));

pub fn execute_fn_invoke_logic(items: Vec<CLangItem>) -> Vec<(CLangItem, CLangItem)> {
    let mut logic = FunctionInvokeLogic::default();
    logic.node = to_logic_clangitem_vec(items);
    logic.run();
    let results = logic.result;
    results
}

pub fn execute_fn_invoke_logic_get_invoke(items: Vec<CLangItem>) -> (String, Vec<CLangItem>) {
    let results = execute_fn_invoke_logic(items);
    if results.len() == 0 {
        panic!("no function invoke found");
    }
    else if results.len() > 1 {
        let mut names = vec![];
        for r in results.iter() {
            let id = r.0.get_atom_item().unwrap()
                            .get_id_value().unwrap();
            names.push(id.to_string());
        }
        debug_string(format!("multiple function invokes found: {names:?}"));
        panic!("multiple function invokes found, only support one function invoke in the expression");
    }
    else {
        let (expr, postfix_expr_part) = results.into_iter().next().unwrap();
        let id = expr.get_atom_item().unwrap()
                            .get_id_value().unwrap();
        let arguments = postfix_expr_part.get_postfix_expression_part().unwrap()
                            .get_children();
        (id.to_string(), arguments)
    }
}
    