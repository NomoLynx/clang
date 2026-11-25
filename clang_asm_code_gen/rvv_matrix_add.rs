use crate::languages::clang::assignment_expression::AssignmentExpression;
use crate::languages::clang::clang_pest::AssignmentOp;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::compound_statement::CompoundStatementItem;
use crate::languages::clang::constant_expression::ConstantExpression;
use crate::languages::clang::iteration_statement::{IterationStatement, IterationStatement_FOR};
use crate::languages::clang::post_operator::PostOperator;
use crate::languages::clang::postfix_expression_part::PostfixExpressionPart;
use crate::languages::clang::statement::Statement;
use crate::languages::clang::symbol_table::{SymbolEntry, SymbolLocation, SymbolType};
use crate::languages::clang::tanry_expression::Expression;
use crate::languages::clang::traits::{gen_add_offset, ASTItemTrait};
use crate::languages::clang::CLangError;
use crate::languages::operators::LangOperators;

use super::{CodeGenContext, CodeSnippet};

struct LoopCounter<'a> {
    symbol: &'a SymbolEntry,
    bound: i32,
}

fn same_symbol(left: &SymbolEntry, right: &SymbolEntry) -> bool {
    left.get_id() == right.get_id() && left.get_name() == right.get_name()
}

fn unqualified_variable(symbol: &SymbolEntry) -> bool {
    if symbol.get_symbol_type() != &SymbolType::Var || symbol.is_extern() {
        return false;
    }

    let mut node = symbol.get_clang_item_link();
    while let Some(item) = node {
        if let Some(declaration) = item.get_declaration() {
            return !declaration.has_type_qualifiers() && !declaration.is_typedef();
        }
        node = item.get_ast_item().get_parent();
    }
    false
}

fn plain_expression(value: &ConstantExpression) -> Option<&Expression> {
    let ConstantExpression::Tanry(_, value) = value else {
        return None;
    };
    let expression = value.get_tanry_expression();
    if expression.is_tanry() || !value.get_postfix_expression_parts().is_empty() {
        return None;
    }
    Some(expression.get_expression())
}

fn variable(expression: &Expression) -> Option<&SymbolEntry> {
    let Expression::Atom(_, atom) = expression else {
        return None;
    };
    if atom.has_unary_op() || atom.has_post_op() {
        return None;
    }
    let name = expression.get_atom_id()?;
    let symbol = expression.get_ast_item().get_symbol_recursive_all_types(name)?;
    unqualified_variable(symbol).then_some(symbol)
}

fn integer_literal(expression: &Expression) -> Option<i32> {
    let Expression::Atom(_, atom) = expression else {
        return None;
    };
    if atom.has_unary_op() || atom.has_post_op() || atom.get_clang_type() != CLangTypes::Int32 {
        return None;
    }
    atom.get_string_value()?.parse().ok()
}

fn single_assignment(assignments: &[AssignmentExpression]) -> Option<&AssignmentExpression> {
    let [assignment] = assignments else {
        return None;
    };
    Some(assignment)
}

fn assignment_target(assignment: &AssignmentExpression) -> Option<&ConstantExpression> {
    let [left] = assignment.get_left_parts().as_slice() else {
        return None;
    };
    if left.get_operator() != &AssignmentOp::SimpleAssign {
        return None;
    }
    Some(left.get_constant_expr())
}

fn value_expression(assignment: &AssignmentExpression) -> Option<&Expression> {
    if !assignment.is_constant_expr() {
        return None;
    }
    plain_expression(assignment.get_constant_expr())
}

fn storage(symbol: &SymbolEntry) -> Option<&SymbolLocation> {
    let [location] = symbol.get_storage_location().as_slice() else {
        return None;
    };
    match location {
        SymbolLocation::Stack(_) | SymbolLocation::Memory(_) | SymbolLocation::Register(_) => Some(location),
        _ => None,
    }
}

fn loop_counter(loop_statement: &IterationStatement_FOR) -> Option<LoopCounter<'_>> {
    let [initial, condition] = loop_statement.expression_statements.as_slice() else {
        return None;
    };
    if initial.has_type_specifier() || condition.has_type_specifier() {
        return None;
    }

    let initial = single_assignment(initial.get_assignments())?;
    let symbol = variable(plain_expression(assignment_target(initial)?)?)?;
    if symbol.get_data_type()? != &CLangTypes::Int32
        || (!symbol.is_global_variable() && storage(symbol).is_none())
    {
        return None;
    }
    if integer_literal(plain_expression(initial.get_constant_expr())?)? != 0 {
        return None;
    }

    let condition = value_expression(single_assignment(condition.get_assignments())?)?;
    let Expression::Expression(_, comparison) = condition else {
        return None;
    };
    if comparison.get_op() != &LangOperators::Less
        || !same_symbol(variable(comparison.get_left())?, symbol)
    {
        return None;
    }
    let bound = integer_literal(comparison.get_right())?;
    if bound <= 0 {
        return None;
    }

    let increment = single_assignment(loop_statement.clang_expression.as_ref()?)?;
    if !same_symbol(variable(plain_expression(assignment_target(increment)?)?)?, symbol) {
        return None;
    }
    let Expression::Expression(_, addition) = plain_expression(increment.get_constant_expr())? else {
        return None;
    };
    if addition.get_op() != &LangOperators::Add
        || !same_symbol(variable(addition.get_left())?, symbol)
        || integer_literal(addition.get_right())? != 1
    {
        return None;
    }
    Some(LoopCounter { symbol, bound })
}

fn single_statement(statement: &Statement) -> Option<&Statement> {
    if let Statement::Compound(_, compound) = statement {
        let [CompoundStatementItem::Statement(statement)] =
            compound.get_compound_statement_items().as_slice()
        else {
            return None;
        };
        single_statement(statement)
    } else {
        Some(statement)
    }
}

fn index_variable(part: &PostfixExpressionPart) -> Option<&SymbolEntry> {
    let PostfixExpressionPart::CLangExpression(_, assignments) = part else {
        return None;
    };
    variable(value_expression(single_assignment(assignments)?)?)
}

fn array_expression<'a>(
    expression: &'a Expression,
    indices: &mut Vec<&'a SymbolEntry>,
) -> Option<&'a SymbolEntry> {
    if let Expression::Postfix(_, postfix) = expression {
        let PostOperator::PostfixExpressionPart(part) = postfix.get_op() else {
            return None;
        };
        let symbol = array_expression(postfix.get_expr(), indices)?;
        indices.push(index_variable(part)?);
        Some(symbol)
    } else {
        variable(expression)
    }
}

fn matrix_access<'a>(
    expression: &'a Expression,
    suffixes: &'a [PostfixExpressionPart],
    row: &LoopCounter<'_>,
    column: &LoopCounter<'_>,
) -> Option<&'a SymbolEntry> {
    let mut indices = Vec::new();
    let symbol = array_expression(expression, &mut indices)?;
    for suffix in suffixes {
        indices.push(index_variable(suffix)?);
    }
    let [row_index, column_index] = indices.as_slice() else {
        return None;
    };
    if !same_symbol(row_index, row.symbol) || !same_symbol(column_index, column.symbol) {
        return None;
    }

    let CLangTypes::Array(element, _, dimensions) = symbol.get_data_type()? else {
        return None;
    };
    if element.as_ref() != &CLangTypes::Int32
        || dimensions.get_dimensions_values().as_slice() != [row.bound as usize, column.bound as usize]
        || (!symbol.is_global_variable()
            && !matches!(storage(symbol), Some(SymbolLocation::Stack(_) | SymbolLocation::Memory(_))))
    {
        return None;
    }
    Some(symbol)
}

fn emit_address(code: &mut String, symbol: &SymbolEntry, register: &str) -> Result<(), CLangError> {
    if symbol.is_global_variable() {
        code.push_str(&format!("la {register}, {}\n", symbol.get_name()));
        return Ok(());
    }

    match storage(symbol) {
        Some(SymbolLocation::Stack(offset)) => {
            code.push_str(&gen_add_offset(register, "fp", *offset as isize));
            code.push('\n');
        }
        Some(SymbolLocation::Memory(label)) => {
            code.push_str(&format!("la {register}, {label}\n"));
        }
        _ => return Err(CLangError::cannot_emit("RVV matrix add requires an addressable object")),
    }
    Ok(())
}

pub(crate) fn generate_matrix_add(
    outer: &IterationStatement_FOR,
    context: &mut CodeGenContext,
) -> Result<Option<CodeSnippet>, CLangError> {
    let config = context.get_code_gen_config();
    if !config.get_enable_rvv() || !config.is_64bit_addressing() {
        return Ok(None);
    }
    let Some(Statement::Iteration(_, IterationStatement::FOR(_, inner))) =
        single_statement(&outer.statement)
    else {
        return Ok(None);
    };
    let Some(Statement::Expression(_, statement)) = single_statement(&inner.statement) else {
        return Ok(None);
    };
    let Some(row) = loop_counter(outer) else {
        return Ok(None);
    };
    let Some(column) = loop_counter(inner) else {
        return Ok(None);
    };
    if same_symbol(row.symbol, column.symbol) || statement.has_type_specifier() {
        return Ok(None);
    }
    let Some(assignment) = single_assignment(statement.get_assignments()) else {
        return Ok(None);
    };
    let Some(ConstantExpression::Tanry(_, destination)) = assignment_target(assignment) else {
        return Ok(None);
    };
    if destination.get_tanry_expression().is_tanry() {
        return Ok(None);
    }
    let Some(destination) = matrix_access(
        destination.get_tanry_expression().get_expression(),
        destination.get_postfix_expression_parts(),
        &row,
        &column,
    ) else {
        return Ok(None);
    };
    let Some(Expression::Expression(_, addition)) = plain_expression(assignment.get_constant_expr()) else {
        return Ok(None);
    };
    if addition.get_op() != &LangOperators::Add {
        return Ok(None);
    }
    let Some(left) = matrix_access(addition.get_left(), &[], &row, &column) else {
        return Ok(None);
    };
    let Some(right) = matrix_access(addition.get_right(), &[], &row, &column) else {
        return Ok(None);
    };
    if same_symbol(destination, left) || same_symbol(destination, right) || same_symbol(left, right) {
        return Ok(None);
    }
    let Some(elements) = row.bound.checked_mul(column.bound).filter(|count| *count <= i32::MAX / 4) else {
        return Ok(None);
    };

    // The full, side-effect-free nest spans three distinct row-major objects.
    let label = format!("__rvv_matrix_add_{}", context.attain_unique_id());
    let mut code = String::from(".text\n.option arch, +v\naddi sp, sp, -48\n");
    for index in 0..6 {
        code.push_str(&format!("sd t{index}, {}(sp)\n", index * 8));
    }
    emit_address(&mut code, left, "t0")?;
    emit_address(&mut code, right, "t1")?;
    emit_address(&mut code, destination, "t2")?;
    code.push_str(&format!(
        "li t3, {elements}\n{label}:\n\
         vsetvli t4, t3, e32, m1, ta, ma\n\
         vle32.v v8, (t0)\n\
         vle32.v v9, (t1)\n\
         vadd.vv v10, v8, v9\n\
         vse32.v v10, (t2)\n\
         slli t5, t4, 2\n\
         add t0, t0, t5\n\
         add t1, t1, t5\n\
         add t2, t2, t5\n\
         sub t3, t3, t4\n\
         bne t3, zero, {label}\n"
    ));

    let mut register_updates = String::new();
    let mut clobbers = vec!["v8".to_string(), "v9".to_string(), "v10".to_string()];
    for counter in [&row, &column] {
        if counter.symbol.is_global_variable() {
            emit_address(&mut code, counter.symbol, "t4")?;
            code.push_str(&format!("li t5, {}\nsw t5, 0(t4)\n", counter.bound));
            continue;
        }

        match storage(counter.symbol) {
            Some(SymbolLocation::Register(register)) => {
                register_updates.push_str(&format!("li {register}, {}\n", counter.bound));
                clobbers.push(register.clone());
            }
            Some(SymbolLocation::Stack(_) | SymbolLocation::Memory(_)) => {
                emit_address(&mut code, counter.symbol, "t4")?;
                code.push_str(&format!("li t5, {}\nsw t5, 0(t4)\n", counter.bound));
            }
            _ => return Err(CLangError::cannot_emit("RVV matrix add lost its induction variable storage")),
        }
    }
    for index in 0..6 {
        code.push_str(&format!("ld t{index}, {}(sp)\n", index * 8));
    }
    code.push_str("addi sp, sp, 48\n");
    code.push_str(&register_updates);

    let mut snippet = CodeSnippet::from_asm_code_str("rvv_matrix_add", &code)?;
    snippet.set_impacted_regs(clobbers);
    snippet.set_input_register_names(vec!["fp".to_string(), "sp".to_string()]);
    context.set_rvv_vectorized_loops(context.get_rvv_vectorized_loops() + 1);
    Ok(Some(snippet))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::clang::clang_pest::{parse_clang, parse_clang_2nd};
    use crate::languages::clang::traits::AsmCodeGenTrait;
    use super::super::{RegisterSet, RegisterUsage};

    fn compile(declarations: &str, body: &str, enable_rvv: bool) -> (String, u32) {
        let source = format!(
            "{declarations}\nint main() {{ int row; int col;\n\
             for (row = 0; row < 2; row = row + 1) {{\n\
                 for (col = 0; col < 5; col = col + 1) {{ {body} }}\n\
             }}\nreturn row + col; }}"
        );
        compile_source(&source, enable_rvv)
    }

    fn compile_source(source: &str, enable_rvv: bool) -> (String, u32) {
        let mut context = CodeGenContext::default();
        context.set_symbol_table_md_file_name(None);
        context.get_code_gen_config_mut().set_enable_rvv(enable_rvv);
        let mut program = parse_clang(source, &mut context).expect("parse matrix regression");
        parse_clang_2nd(&mut program, &mut context).expect("analyze matrix regression");
        program.generate_code(&mut context).expect("generate matrix regression");
        (
            context.get_code_snippet().generate_code_string_with_section(true),
            context.get_rvv_vectorized_loops(),
        )
    }

    #[test]
    fn rvv_matrix_add_is_opt_in_and_emits_vector_strip_mining() {
        let declarations = "int lhs[2][5]; int rhs[2][5]; int result[2][5];";
        let body = "result[row][col] = lhs[row][col] + rhs[row][col];";
        let (scalar, scalar_count) = compile(declarations, body, false);
        assert_eq!(scalar_count, 0);
        assert!(!scalar.contains("vadd.vv"));

        let (vector, vector_count) = compile(declarations, body, true);
        assert_eq!(vector_count, 1);
        for instruction in ["vsetvli", "vle32.v", "vadd.vv", "vse32.v", "slli", "sd", "ld"] {
            assert!(vector.contains(instruction), "missing {instruction}");
        }
        assert!(vector.contains("__rvv_matrix_add_"));
        assert!(vector.replace(' ', "").contains("e32,m1,ta,ma"));
    }

    #[test]
    fn rvv_matrix_add_refuses_aliases_qualifiers_and_side_effects() {
        let declarations = "int lhs[2][5]; int rhs[2][5]; int result[2][5];";
        for body in [
            "lhs[row][col] = lhs[row][col] + rhs[row][col];",
            "result[row][col] = lhs[row][col] - rhs[row][col];",
            "result[row][col] = lhs[row][col] + rhs[row][col]; row = row + 1;",
            "result[col][row] = lhs[row][col] + rhs[row][col];",
        ] {
            let (code, count) = compile(declarations, body, true);
            assert_eq!(count, 0, "unsafe matrix loop was vectorized: {body}");
            assert!(!code.contains("vadd.vv"));
        }
        let (_, count) = compile(
            "volatile int lhs[2][5]; int rhs[2][5]; int result[2][5];",
            "result[row][col] = lhs[row][col] + rhs[row][col];",
            true,
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn rvv_local_matrix_initialization_and_complete_checks_preserve_live_addresses() {
        let source = "
            int main() {
                int lhs[2][5];
                int rhs[2][5];
                int result[2][5];
                int row;
                int col;
                for (row = 0; row < 2; row = row + 1) {
                    for (col = 0; col < 5; col = col + 1) {
                        lhs[row][col] = 1;
                        rhs[row][col] = 2;
                    }
                }
                for (row = 0; row < 2; row = row + 1) {
                    for (col = 0; col < 5; col = col + 1) {
                        result[row][col] = lhs[row][col] + rhs[row][col];
                    }
                }
                for (row = 0; row < 2; row = row + 1) {
                    for (col = 0; col < 5; col = col + 1) {
                        if (result[row][col] != 3) { return 7; }
                    }
                }
                return 42;
            }";
        for enabled in [false, true] {
            let (code, count) = compile_source(source, enabled);
            assert_eq!(count, u32::from(enabled));
            assert_eq!(code.contains("vadd.vv"), enabled);
        }
    }

    #[test]
    fn rvv_register_tracking_keeps_vector_clobbers_out_of_scalar_allocation() {
        let vector = CodeSnippet::from_asm_code_str(
            "vector registers",
            ".text\nvle32.v v8, (t0)\nvle32.v v9, (t1)\nvadd.vv v10, v8, v9\nvse32.v v10, (t2)\n",
        ).expect("parse real RVV instructions");
        let mut parent = CodeSnippet::new("parent");
        parent.add_snippet(vector);
        let registers = parent.get_all_registers();
        for name in ["v8", "v9", "v10"] {
            assert!(RegisterUsage::new(name).is_vector_register());
            assert!(registers.get_sorted_reg_names().contains(&name.to_string()));
            assert!(!registers.get_int_registers().get_sorted_reg_names().contains(&name.to_string()));
            assert!(!registers.get_float_registers().get_sorted_reg_names().contains(&name.to_string()));
        }
        assert!(RegisterSet::get_riscv_available_register_names()
            .iter().all(|name| !name.starts_with('v')));
    }
}
