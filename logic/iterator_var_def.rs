use ascent::ascent;

use crate::languages::clang::{CLangItem, logic::base_logic};
use crate::languages::clang::logic::*;

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/languages/clang/logic/iterator_var_def.gen.rs"));

pub fn invoke_iterator_var_def(items: Vec<CLangItem>) -> Vec<(CLangItem, CLangItem, CLangItem)>{
    let mut prog = IterationFor::default();
    prog.node = to_logic_clangitem_vec(items);
    prog.run();

    prog.result
}