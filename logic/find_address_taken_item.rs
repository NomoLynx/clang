use ascent::ascent;

use crate::languages::clang::CLangItem;
use crate::languages::clang::logic::{base_logic, from_logic_clangitem_vec, to_logic_clangitem_vec};

include!(concat!(env!("CARGO_MANIFEST_DIR"), "/src/languages/clang/logic/find_address_taken_item.gen.rs"));

/// Execute the find address taken item logic to get all items whose address is taken
pub fn execute_find_address_taken_item_logic(items: Vec<CLangItem>) -> Vec<CLangItem> {
    let mut logic = FindAddressTakenItem::default();
    logic.node = to_logic_clangitem_vec(items);
    logic.run();
    let results = from_logic_clangitem_vec(logic.result);
    results
}