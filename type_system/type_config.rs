use crate::languages::clang::traits::*;
use crate::languages::clang::{ASTItem, CLangItem};
use crate::languages::clang::ast_item_iteration_context::*;
use core_utils::debug::debug_string;
use parser_lib::json::*;

#[derive(Debug, Clone)]
pub struct TypeConfiguration {
    clang_type_node_names: Vec<String>,
}

impl TypeConfiguration {
    /// get clang type node names
    pub fn get_clang_type_node_names(&self) -> &Vec<String> {
        &self.clang_type_node_names
    }

    pub fn is_ast_item_has_type_node(&self, ast_item:&ASTItem) -> bool {
        ast_item.get_current_node()
            .unwrap()
            .is_type_node()
    }

    /// infer types and perform type checking
    /// the type is inferred roughly based on the pest data structure
    /// this function will set the runtime type in ASTItem if inferred successfully
    /// and it focus on type checking
    pub fn infer_and_check_type(&self, ast_item:&mut ASTItem, context:&mut ASTItemIterationContext) {
        let clang_item = ast_item.get_current_node()
                                        .unwrap().clone();
        match clang_item {
            CLangItem::Atom(n) => {
                let atom = CLangItem::from_raw(n);
                let runtime_type = atom.get_clang_type().clone();
                if !(runtime_type.is_none_type() || runtime_type.is_invalid_type()) {
                    atom.set_clang_type(runtime_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for Atom: '{}'", atom.get_name()), 
                                atom.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::AtomItem(n) => {
                let atom_item = CLangItem::from_raw(n);
                let runtime_type = atom_item.get_clang_type().clone();
                if !(runtime_type.is_none_type() || runtime_type.is_invalid_type()) {
                    atom_item.set_clang_type(runtime_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for AtomItem: '{}'", atom_item.get_name()), 
                                atom_item.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::Expression(n) => {
                let expression = CLangItem::from_raw(n);
                let runtime_type = expression.get_clang_type().clone();
                if !(runtime_type.is_none_type() || runtime_type.is_invalid_type()) {
                    expression.set_clang_type(runtime_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for Expression: '{}'", 
                                            expression.get_name()), 
                                expression.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::TanryExpression(n) => {
                let tanry_expression = CLangItem::from_raw(n);
                if tanry_expression.is_tanry() {
                    let (true_branch, false_branch) = tanry_expression.get_branches().unwrap();
                    let true_branch_type = true_branch.get_clang_type();
                    let false_branch_type = false_branch.get_clang_type();
                    if true_branch_type == false_branch_type && !true_branch_type.is_invalid_type() && !true_branch_type.is_none_type() {
                        tanry_expression.set_clang_type(true_branch_type);
                    } else {
                        context.add_error_node(clang_item, 
                            (format!("Type mismatch in TanryExpression branches or has invalid type: '{}' vs '{}'",
                                    true_branch_type.to_string(),
                                    false_branch_type.to_string()), 
                                    tanry_expression.get_ast_item().get_source_file_location()).into());
                    }
                }
                else {
                    let clang_type = tanry_expression.get_expression().get_clang_type();
                    if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                        tanry_expression.set_clang_type(clang_type);
                    } else {
                        context.add_error_node(clang_item, 
                            (format!("Failed to infer type for TanryExpression: '{}'", 
                                    tanry_expression.get_name()), 
                                    tanry_expression.get_ast_item().get_source_file_location()).into());
                    }
                }
            }
            CLangItem::ConstantExpression_Tanry(n) => {
                let const_expr_tanry = CLangItem::from_raw(n);
                let clang_type = const_expr_tanry.get_clang_type();
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                    const_expr_tanry.set_clang_type(clang_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for ConstantExpression_Tanry: '{}'", 
                                const_expr_tanry.get_name()), 
                                const_expr_tanry.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::SizeOfExpression(n) => {
                let sizeof_expr = CLangItem::from_raw(n);
                let clang_type = sizeof_expr.get_clang_type();
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                    sizeof_expr.set_clang_type(clang_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for SizeOfExpression: '{}'", 
                                sizeof_expr.get_name()), 
                                sizeof_expr.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::CastExpression(n) => {
                let cast_expr = CLangItem::from_raw(n);
                let clang_type = cast_expr.get_clang_type();
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                    cast_expr.set_clang_type(clang_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for CastExpression: '{}'", 
                                cast_expr.get_name()), 
                                cast_expr.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::ConstantExpression(n) => {
                let const_expr = CLangItem::from_raw(n);
                let clang_type = const_expr.get_clang_type();
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                    const_expr.set_clang_type(clang_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for ConstantExpression: '{}'", 
                                const_expr.get_name()), 
                                const_expr.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::AssignmentExpressionLeftPart(n) => {
                let assign_expr_left = CLangItem::from_raw(n);
                let clang_type = assign_expr_left.get_clang_type();
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                    assign_expr_left.set_clang_type(clang_type);
                } else {
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for AssignmentExpressionLeftPart: '{}'", 
                                assign_expr_left.get_name()), 
                                assign_expr_left.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::Expression_Expression(n) => {
                let expr_expr = CLangItem::from_raw(n);
                let clang_type = expr_expr.get_clang_type();
                let ty0 = expr_expr.get_left().get_clang_type();
                let ty1 = expr_expr.get_right().get_clang_type();
                
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) && 
                    expr_expr.get_op().is_valid_operation(ty0, ty1) {
                    expr_expr.set_clang_type(clang_type);
                } else {
                    let expr_str = format!("{} -- '{}' -- {}", 
                                        expr_expr.get_left().get_clang_type(),
                                        expr_expr.get_op(),
                                        expr_expr.get_right().get_clang_type());
                    context.add_error_node(clang_item, 
                        (format!("Failed to infer type for Expression_Expression '{expr_str}': '{}'", 
                                expr_expr.get_name()), 
                                expr_expr.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::Expression_Postfix(n) => {
                let expr_postfix = CLangItem::from_raw(n);
                let clang_type = expr_postfix.get_clang_type();
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                    debug_string(format!("Expression_Postfix: '{}', inferred type: '{}'", 
                                expr_postfix.get_name(), 
                                clang_type.to_string()));
                    expr_postfix.set_clang_type(clang_type);
                } else {
                    context.add_error_node(clang_item,
                        (format!("Failed to infer type for Expression_Postfix: '{}'",
                                expr_postfix.get_name()),
                                expr_postfix.get_ast_item().get_source_file_location()).into());
                }
            }
            CLangItem::Expression_Prefix(n) => {
                let expr_prefix = CLangItem::from_raw(n);
                let clang_type = expr_prefix.get_clang_type();
                if !(clang_type.is_none_type() || clang_type.is_invalid_type()) {
                    expr_prefix.set_clang_type(clang_type);
                } else {
                    context.add_error_node(clang_item,
                        (format!("Failed to infer type for Expression_Prefix: '{}'",
                                expr_prefix.get_name()),
                                expr_prefix.get_ast_item().get_source_file_location()).into());
                }
            }
            _ => {
                context.add_error_node(clang_item, 
                    (format!("Type inference not implemented for ASTItem with type node: '{}'",
                            ast_item.get_type_name()), 
                            ast_item.get_source_file_location()).into());
            }
        }
    }
}

impl Default for TypeConfiguration {
    fn default() -> Self {
        let mut r = Self {
            clang_type_node_names: Vec::default(),
        };

        let type_config_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                                        .join("src/languages/clang/type_system/type_config.json");
        let type_config_data = parse_json_from_file(type_config_path.to_str().unwrap())
                                        .expect("cannot load type_config.json");

        // load clang_type_node_names
        type_config_data.get_value_from_key("type_node_names").and_then(|v| {
            if let Value::Array(arr) = v {
                for item in arr {
                    if let Value::String(s) = item {
                        r.clang_type_node_names.push(s.clone());
                    }
                }
                Some(())
            } else {
                None
            }
        });

        r
    }
}