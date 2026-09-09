pub(crate) mod tanry_expression;
pub(crate) mod atomitem;
pub(crate) mod atom;
pub(crate) mod expression;
pub mod expression_postfix;
pub mod expression_expression;
pub mod expression_prefix;

pub use tanry_expression::*;
pub use atomitem::*;
pub use atom::*;
pub use expression::*;
pub use expression_expression::*;
pub use expression_prefix::*;

use crate::languages::clang::CLangItem;

fn get_offset(patterns:&Vec<(CLangItem, CLangItem)>) -> i32 {
    if patterns.is_empty(){
        return 0;
    }
    
    let base_type = patterns.first().unwrap().0
                            .get_expression()
                            .unwrap();
    let dereferenced_type = if base_type.get_clang_type().is_pointer() { base_type.get_clang_type().dereference_pointer_type(1).unwrap() } 
                            else { base_type.get_clang_type() };
    if let Some(symbol_table) = dereferenced_type.get_composite_type_symbol_table() {
        let mut current_symbol_table = symbol_table;
        let mut storage_location = 0;
        for (_expr, part) in patterns {
            let member_name = part.get_postfix_expression_part().unwrap()
                                            .get_dot_or_ptr_member().unwrap();
            if let Some(member_entry) = current_symbol_table.lookup(member_name) {
                storage_location += member_entry.get_single_storage_location_member_offset().unwrap();
                if let Some(next_symbol_table) = member_entry.get_composite_type_symbol_table() {
                    current_symbol_table = next_symbol_table;
                }
                else {
                    break;
                }
            }
            else {
                break;
            }
        }

        storage_location
    }
    else {
        panic!("the base type of the postfix expression is not a composite type, cannot get storage location");
    }
}