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

use super::{CodeGenContext, CodeSnippet, ScratchRegisters, VectorGroupSize};

struct LoopCounter<'a> {
    symbol: &'a SymbolEntry,
    bound: i32,
}

#[derive(Clone, Copy)]
enum ElementwiseOperation {
    Add,
    Subtract,
}

impl ElementwiseOperation {
    fn instruction(self) -> &'static str {
        match self {
            Self::Add => "vadd.vv",
            Self::Subtract => "vsub.vv",
        }
    }
}

struct ElementwiseKernel<'a> {
    counters: Vec<LoopCounter<'a>>,
    left: &'a SymbolEntry,
    right: &'a SymbolEntry,
    destination: &'a SymbolEntry,
    elements: i32,
    operation: ElementwiseOperation,
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

fn array_access<'a>(
    expression: &'a Expression,
    suffixes: &'a [PostfixExpressionPart],
    counters: &[LoopCounter<'_>],
) -> Option<&'a SymbolEntry> {
    let mut indices = Vec::new();
    let symbol = array_expression(expression, &mut indices)?;
    for suffix in suffixes {
        indices.push(index_variable(suffix)?);
    }
    if indices.len() != counters.len()
        || indices.iter().zip(counters).any(|(index, counter)| !same_symbol(index, counter.symbol))
    {
        return None;
    }

    let CLangTypes::Array(element, _, dimensions) = symbol.get_data_type()? else {
        return None;
    };
    if element.as_ref() != &CLangTypes::Int32
        || dimensions.get_dimensions_values().as_slice() != counters.iter()
            .map(|counter| counter.bound as usize).collect::<Vec<_>>().as_slice()
    {
        return None;
    }

    // Only objects with checked FP-relative, word-aligned storage are eligible.
    match storage(symbol) {
        Some(SymbolLocation::Stack(offset)) if *offset % 4 == 0 => Some(symbol),
        _ => None,
    }
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
        _ => return Err(CLangError::cannot_emit("RVV elementwise lowering requires an addressable object")),
    }
    Ok(())
}

fn match_kernel(outer: &IterationStatement_FOR) -> Option<ElementwiseKernel<'_>> {
    let mut counters = Vec::new();
    let mut current = outer;
    let statement = loop {
        let counter = loop_counter(current)?;
        if counters.iter().any(|previous: &LoopCounter<'_>| same_symbol(previous.symbol, counter.symbol)) {
            return None;
        }
        counters.push(counter);
        if counters.len() > 3 {
            return None;
        }
        match single_statement(&current.statement)? {
            Statement::Iteration(_, IterationStatement::FOR(_, inner)) => current = inner,
            Statement::Expression(_, statement) if counters.len() >= 2 => break statement,
            _ => return None,
        }
    };
    if statement.has_type_specifier() {
        return None;
    }
    let assignment = single_assignment(statement.get_assignments())?;
    let ConstantExpression::Tanry(_, destination) = assignment_target(assignment)? else {
        return None;
    };
    if destination.get_tanry_expression().is_tanry() {
        return None;
    }
    let destination = array_access(
        destination.get_tanry_expression().get_expression(),
        destination.get_postfix_expression_parts(),
        &counters,
    )?;
    let Expression::Expression(_, operation) = plain_expression(assignment.get_constant_expr())? else {
        return None;
    };
    let operation_kind = match operation.get_op() {
        LangOperators::Add => ElementwiseOperation::Add,
        LangOperators::Sub => ElementwiseOperation::Subtract,
        _ => return None,
    };
    let left = array_access(operation.get_left(), &[], &counters)?;
    let right = array_access(operation.get_right(), &[], &counters)?;
    if same_symbol(destination, left) || same_symbol(destination, right) || same_symbol(left, right) {
        return None;
    }
    let elements = counters.iter().try_fold(1i32, |count, counter| count.checked_mul(counter.bound))?;
    if elements > i32::MAX / 4 {
        return None;
    }
    Some(ElementwiseKernel { counters, left, right, destination, elements, operation: operation_kind })
}

fn emit_kernel(
    kernel: &ElementwiseKernel<'_>,
    bindings: &ScratchRegisters,
    context: &mut CodeGenContext,
) -> Result<CodeSnippet, CLangError> {
    let [left_address, right_address, destination_address, remaining, vl_bytes] = bindings.integer.as_slice() else {
        return Err(CLangError::cannot_emit("RVV kernel requires five owned integer temporaries"));
    };
    let [left_vector, right_vector, result_vector] = bindings.vector.as_slice() else {
        return Err(CLangError::cannot_emit("RVV kernel requires three owned vector groups"));
    };
    debug_assert!(!bindings.registers().has_conflict(context.get_live_registers()));
    let (left_vector, right_vector, result_vector) = (left_vector.name(), right_vector.name(), result_vector.name());
    let instruction = kernel.operation.instruction();
    let elements = kernel.elements;
    let label = format!("__rvv_elementwise_{}", context.attain_unique_id());
    let mut code = String::from(".text\n.option arch, +v\n");
    emit_address(&mut code, kernel.left, left_address)?;
    emit_address(&mut code, kernel.right, right_address)?;
    emit_address(&mut code, kernel.destination, destination_address)?;
    code.push_str(&format!(
        "li {remaining}, {elements}\n{label}:\n\
         vsetvli {vl_bytes}, {remaining}, e32, m1, ta, ma\n\
         vle32.v {left_vector}, ({left_address})\n\
         vle32.v {right_vector}, ({right_address})\n\
         {instruction} {result_vector}, {left_vector}, {right_vector}\n\
         vse32.v {result_vector}, ({destination_address})\n\
         sub {remaining}, {remaining}, {vl_bytes}\n\
         slli {vl_bytes}, {vl_bytes}, 2\n\
         add {left_address}, {left_address}, {vl_bytes}\n\
         add {right_address}, {right_address}, {vl_bytes}\n\
         add {destination_address}, {destination_address}, {vl_bytes}\n\
         bne {remaining}, zero, {label}\n"
    ));

    let mut register_updates = String::new();
    let mut clobbers = Vec::new();
    for counter in &kernel.counters {
        if counter.symbol.is_global_variable() {
            emit_address(&mut code, counter.symbol, vl_bytes)?;
            code.push_str(&format!("li {remaining}, {}\nsw {remaining}, 0({vl_bytes})\n", counter.bound));
            continue;
        }

        match storage(counter.symbol) {
            Some(SymbolLocation::Register(register)) => {
                register_updates.push_str(&format!("li {register}, {}\n", counter.bound));
                clobbers.push(register.clone());
            }
            Some(SymbolLocation::Stack(_) | SymbolLocation::Memory(_)) => {
                emit_address(&mut code, counter.symbol, vl_bytes)?;
                code.push_str(&format!("li {remaining}, {}\nsw {remaining}, 0({vl_bytes})\n", counter.bound));
            }
            _ => return Err(CLangError::cannot_emit("RVV elementwise lowering lost its induction variable storage")),
        }
    }
    code.push_str(&register_updates);

    let mut snippet = CodeSnippet::from_asm_code_str("rvv_elementwise", &code)?;
    snippet.set_temp_register_names(bindings.registers().get_register_alias_name());
    snippet.set_impacted_regs(clobbers);
    snippet.set_input_register_names(vec!["fp".to_string()]);
    Ok(snippet)
}

pub(crate) fn generate_elementwise(
    outer: &IterationStatement_FOR,
    context: &mut CodeGenContext,
) -> Result<Option<CodeSnippet>, CLangError> {
    let config = context.get_code_gen_config();
    if !config.get_enable_rvv() || !config.is_64bit_addressing() || config.get_stack_space_size().is_some() {
        return Ok(None);
    }
    let Some(kernel) = match_kernel(outer) else {
        return Ok(None);
    };
    let function = outer.statement.get_ast_item().get_current_function()
        .ok_or_else(|| CLangError::cannot_emit("RVV lowering requires a containing function"))?;
    if function.get_all_clangitems().iter().any(|item| {
        item.is_asmstatement() || item.get_statement()
            .map_or(false, |statement| matches!(statement, Statement::AsmStatement(_, _)))
    }) {
        core_utils::debug::output_string("RVV scalar fallback: function has unmodelled inline-assembly effects".to_string());
        return Ok(None);
    }
    if context.get_verified_frame_bytes().is_none() {
        return Err(CLangError::cannot_emit("RVV lowering requires a verified function frame"));
    }
    let snippet = context.with_scratch_registers(5, 3, VectorGroupSize::M1,
        |context, bindings| emit_kernel(&kernel, bindings, context))?;
    if snippet.is_some() {
        context.set_rvv_vectorized_loops(context.get_rvv_vectorized_loops().checked_add(1)
            .ok_or_else(|| CLangError::cannot_emit("RVV lowering count overflow"))?);
    } else {
        core_utils::debug::output_string("RVV scalar fallback: insufficient non-live scratch registers".to_string());
    }
    Ok(snippet)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::languages::clang::clang_pest::{parse_clang, parse_clang_2nd};
    use crate::languages::clang::traits::AsmCodeGenTrait;
    use super::super::{CodeSnippetDataItem, RegisterSet, RegisterUsage};

    fn matrix_source(declarations: &str, body: &str) -> String {
        format!(
            "int main() {{ {declarations} int row; int col;\n\
             for (row = 0; row < 2; row = row + 1) {{\n\
                 for (col = 0; col < 5; col = col + 1) {{\n\
                     lhs[row][col] = 1; rhs[row][col] = 2;\n\
                 }}\n\
             }}\n\
             for (row = 0; row < 2; row = row + 1) {{\n\
                 for (col = 0; col < 5; col = col + 1) {{ {body} }}\n\
             }}\nreturn row + col; }}"
        )
    }

    fn compile(declarations: &str, body: &str, enable_rvv: bool) -> (String, u32) {
        compile_source(&matrix_source(declarations, body), enable_rvv)
    }

    fn compile_source(source: &str, enable_rvv: bool) -> (String, u32) {
        let (code, count) = compile_ir(source, enable_rvv, None);
        (code.generate_code_string_with_section(true), count)
    }

    fn compile_ir(source: &str, enable_rvv: bool, stack_space_size: Option<u32>) -> (CodeSnippet, u32) {
        let mut context = CodeGenContext::default();
        context.set_symbol_table_md_file_name(None);
        context.get_code_gen_config_mut().set_enable_rvv(enable_rvv);
        context.get_code_gen_config_mut().set_stack_space_size(stack_space_size);
        let mut program = parse_clang(source, &mut context).expect("parse matrix regression");
        parse_clang_2nd(&mut program, &mut context).expect("analyze matrix regression");
        program.generate_code(&mut context).expect("generate matrix regression");
        (
            context.get_code_snippet().clone(),
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
        for instruction in ["vsetvli", "vle32.v", "vadd.vv", "vse32.v", "slli"] {
            assert!(vector.contains(instruction), "missing {instruction}");
        }
        assert!(vector.contains("__rvv_elementwise_"));
        assert!(vector.replace(' ', "").contains("e32,m1,ta,ma"));
    }

    #[test]
    fn rvv_subtraction_uses_real_ordered_vector_subtract() {
        let declarations = "int lhs[2][5]; int rhs[2][5]; int result[2][5];";
        let body = "result[row][col] = lhs[row][col] - rhs[row][col];";
        let (code, count) = compile(declarations, body, true);
        assert_eq!(count, 1);
        assert!(code.contains("vsub.vv v10, v8, v9"), "{code}");
        assert!(!code.contains("vadd.vv"));
        let (scalar, count) = compile(declarations, body, false);
        assert_eq!(count, 0);
        assert!(!scalar.contains("vsub.vv"));
    }

    #[test]
    fn rvv_refuses_functions_with_unmodelled_inline_assembly() {
        let source = matrix_source(
            "int lhs[2][5]; int rhs[2][5]; int result[2][5];",
            "result[row][col] = lhs[row][col] + rhs[row][col];",
        ).replace("return row + col;", "__asm { .text\naddi t0, t0, 0\n }; return row + col;");
        let (code, count) = compile_source(&source, true);
        assert_eq!(count, 0);
        assert!(!code.contains("vadd.vv"));
    }

    #[test]
    fn rvv_three_dimensional_nests_flatten_all_elements_for_both_operations() {
        for operation in ["+", "-"] {
            let source = format!("
                int main() {{
                    int lhs[2][3][5]; int rhs[2][3][5]; int result[2][3][5];
                    int i; int j; int k;
                    for (i = 0; i < 2; i = i + 1) {{
                        for (j = 0; j < 3; j = j + 1) {{
                            for (k = 0; k < 5; k = k + 1) {{
                                lhs[i][j][k] = 7; rhs[i][j][k] = 2;
                            }}
                        }}
                    }}
                    for (i = 0; i < 2; i = i + 1) {{
                        for (j = 0; j < 3; j = j + 1) {{
                            for (k = 0; k < 5; k = k + 1) {{
                                result[i][j][k] = lhs[i][j][k] {operation} rhs[i][j][k];
                            }}
                        }}
                    }}
                    return 42;
                }}");
            let (ir, count) = compile_ir(&source, true, None);
            assert_eq!(count, 1);
            let kernel = matrix_snippet(&ir).expect("3D kernel");
            let code = kernel.generate_code_string();
            assert!(code.contains("li t3, 30"), "{code}");
            assert!(code.contains(if operation == "+" { "vadd.vv" } else { "vsub.vv" }));
            assert_eq!(kernel.get_impacted_regs().len(), 3, "all three index registers must be updated");
        }
    }

    fn lower_with_live(live: &RegisterSet) -> Option<CodeSnippet> {
        let source = matrix_source(
            "int lhs[2][5]; int rhs[2][5]; int result[2][5];",
            "result[row][col] = lhs[row][col] + rhs[row][col];",
        );
        let mut context = CodeGenContext::default();
        context.set_symbol_table_md_file_name(None);
        context.get_code_gen_config_mut().set_enable_rvv(true);
        let mut program = parse_clang(&source, &mut context).unwrap();
        parse_clang_2nd(&mut program, &mut context).unwrap();
        let items = program.get_all_clangitems();
        for item in &items {
            let Some(outer) = item.get_for_loop() else {
                continue;
            };
            if match_kernel(outer).is_none() {
                continue;
            }
            let frame = outer.statement.get_ast_item().get_current_function().unwrap()
                .checked_frame_size(&context).unwrap();
            let result = context.with_verified_frame(frame, |context| {
                context.with_live_registers(live, |context| generate_elementwise(outer, context))
            }).unwrap();
            assert!(context.get_reserved_registers().is_empty());
            return result;
        }
        panic!("regression did not contain a canonical matrix nest");
    }

    #[test]
    fn rvv_bindings_avoid_live_gprs_vectors_and_decline_on_exhaustion() {
        let live = RegisterSet::from("t0, t1, v8");
        let kernel = lower_with_live(&live).expect("five alternative GPRs are available");
        assert!(!kernel.get_temp_register_as_registerset().has_conflict(&live));
        let code = kernel.generate_code_string();
        assert!(code.contains("vle32.v v9, (t2)"), "{code}");
        assert!(!code.contains("vsub.vv"));
        assert!(code.contains("vadd.vv v11, v9, v10"), "{code}");
        assert!(lower_with_live(&RegisterSet::from("t0, t1, t2")).is_none());
    }

    #[test]
    fn complete_frame_covers_nested_storage_and_alignment_padding() {
        let source = "
            int main() {
                char pad[3]; int lhs[2][5]; int rhs[2][5]; int result[2][5];
                pad[0] = 1;
                { int nested[8]; nested[0] = 2; }
                return 42;
            }";
        let (code, _) = compile_source(source, false);
        assert!(code.contains("addi sp, sp, -160"), "nested slots below the old 128-byte reservation must be covered: {code}");
    }

    #[test]
    fn large_in_place_offsets_preserve_sp_and_unsupported_frames_are_rejected() {
        let source = "
            int f() { return 1; }
            int main() {
                int result;
                { int big[600]; big[0] = 41; result = big[0]; }
                return result + f();
            }";
        let mut context = CodeGenContext::default();
        context.set_symbol_table_md_file_name(None);
        let mut program = parse_clang(source, &mut context).unwrap();
        parse_clang_2nd(&mut program, &mut context).unwrap();
        let error = program.generate_code(&mut context).expect_err("wide scalar FP offsets are unsupported");
        assert!(error.to_string().contains("2048 bytes"));
        let allocate = gen_add_offset("sp", "x2", -2400);
        assert!(allocate.contains("addi sp, sp, -2048"));
        assert!(allocate.contains("addi sp, sp, -352"));
        assert!(!allocate.contains("lui"));
        let deallocate = gen_add_offset("x2", "sp", 4096);
        assert!(!deallocate.contains("lui"));
        for line in deallocate.lines() {
            let chunk = line.rsplit(',').next().unwrap().trim().parse::<i32>().unwrap();
            assert_eq!(chunk % 16, 0, "SP must remain aligned at each chunk");
            assert!((-2048..=2047).contains(&chunk));
        }
    }

    #[test]
    fn frame_verification_rejects_offsets_outside_the_recorded_frame() {
        let mut context = CodeGenContext::default();
        context.set_symbol_table_md_file_name(None);
        let mut program = parse_clang("int main() { int data[4]; data[0] = 1; return 42; }", &mut context).unwrap();
        parse_clang_2nd(&mut program, &mut context).unwrap();
        let items = program.get_all_clangitems();
        let function = items.iter().find_map(|item| item.get_function_definition()).unwrap();
        assert_eq!(function.checked_frame_size(&context).unwrap(), 16);
        function.set_stack_frame_pointer(0);
        assert!(function.checked_frame_size(&context).is_err());
    }

    #[test]
    fn rvv_matrix_add_refuses_aliases_qualifiers_and_side_effects() {
        let declarations = "int lhs[2][5]; int rhs[2][5]; int result[2][5];";
        for body in [
            "lhs[row][col] = lhs[row][col] + rhs[row][col];",
            "result[row][col] = lhs[row][col] * rhs[row][col];",
            "result[row][col] = lhs[row][col] + rhs[row][col]; rhs[row][col] = 3;",
        ] {
            let (code, count) = compile(declarations, body, true);
            assert_eq!(count, 0, "unsafe matrix loop was vectorized: {body}");
            assert!(!code.contains("vadd.vv"));
        }
        let (_, count) = compile(
            "int lhs[2][5]; int rhs[2][5]; int result[5][2];",
            "result[col][row] = lhs[row][col] + rhs[row][col];",
            true,
        );
        assert_eq!(count, 0);
        let (_, count) = compile(
            "volatile int lhs[2][5]; int rhs[2][5]; int result[2][5];",
            "result[row][col] = lhs[row][col] + rhs[row][col];",
            true,
        );
        assert_eq!(count, 0);
    }

    #[test]
    fn rvv_stack_layout_aligns_arrays_and_refuses_unproven_storage() {
        let body = "result[row][col] = lhs[row][col] + rhs[row][col];";
        let (code, count) = compile(
            "char pad[3]; int lhs[2][5]; int rhs[2][5]; int result[2][5];",
            body,
            true,
        );
        assert_eq!(count, 1);
        assert!(code.contains("vadd.vv"));

        let source = "
            int lhs[2][5]; int rhs[2][5]; int result[2][5];
            int main() {
                int row; int col;
                for (row = 0; row < 2; row = row + 1) {
                    for (col = 0; col < 5; col = col + 1) {
                        result[row][col] = lhs[row][col] + rhs[row][col];
                    }
                }
                return 42;
            }";
        let (code, count) = compile_source(source, true);
        assert_eq!(count, 0);
        assert!(!code.contains("vadd.vv"));

        let local = matrix_source(
            "int lhs[2][5]; int rhs[2][5]; int result[2][5];",
            body,
        );
        let (code, count) = compile_ir(&local, true, Some(4096));
        assert_eq!(count, 0);
        assert!(!code.generate_code_string_with_section(true).contains("vadd.vv"));
    }

    fn matrix_snippet(code: &CodeSnippet) -> Option<&CodeSnippet> {
        if code.get_name() == "rvv_elementwise" {
            return Some(code);
        }
        code.get_code_items().into_iter().find_map(|item| match item {
            CodeSnippetDataItem::Snippet(child) => matrix_snippet(child),
            _ => None,
        })
    }

    #[test]
    fn rvv_nested_matrix_statements_declare_clobbers_without_stack_writes() {
        let source = "
            int main() {
              {
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
              }
              return 42;
            }";
        for enabled in [false, true] {
            let (ir, count) = compile_ir(source, enabled, None);
            let code = ir.generate_code_string_with_section(true);
            assert_eq!(count, u32::from(enabled));
            assert_eq!(code.contains("vadd.vv"), enabled);
            if enabled {
                let matrix = matrix_snippet(&ir).expect("nested RVV matrix snippet");
                let clobbers = matrix.get_contaminated_registers().get_register_alias_name();
                for index in 0..5 {
                    assert!(clobbers.contains(&format!("t{index}")));
                }
                assert!(!matrix.get_all_registers().get_sorted_reg_names().contains(&"x2".to_string()));
            }
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
