use core::panic;

use crate::languages::clang::constant_expression::SizeOfExpression;
use crate::languages::clang::constant_expression::clang_const_expression_tanry::CLangConstExpression_Tanry;
use crate::languages::clang::logic::*;
use crate::languages::clang::postfix_expression_part::PostfixExpressionPart;
use crate::languages::clang::struct_union::*;
use crate::languages::clang::compound_statement::*;
use core_utils::debug::*;
use core_utils::number::*;
use crate::languages::clang::post_operator::*;
use crate::languages::{clang::{clang_error::CLangError, clang_pest::UnaryOperator, clang_program::CLangProgram, clang_types::CLangTypes, constant_expression::ConstantExpression, declaration::Declaration, enum_type::EnumSpecifier, function_definition::FunctionDefinition, iteration_statement::IterationStatement, jump_statement::JumpStatement, label_statement::LabelStatement, parameter_declaration::ParameterDeclaration, selection_statement::SelectionStatement, statement::{ExpressionStatement, Statement}, tanry_expression::{Atom, AtomItem, Expression, TanryExpression}}, operators::LangOperators, r5asm::{directive::Directive, r5asm_pest::SectionItem}};
use crate::languages::clang::clang_asm_code_gen::*;
use crate::languages::clang::*;
use crate::languages::clang::symbol_table::*;
use crate::languages::clang::assignment_expression::*;
use crate::languages::clang::declaration::resolve_clang_type_for_usage;

/// convert simple value to code snippet (with help of type information)
pub (self) fn clang_type_and_value_to_codesnippet(ty:&CLangTypes, n:&str, context:&mut CodeGenContext, ast_item:&ASTItem) -> Result<(), CLangError> {
    let need_replace_label = ty.is_string() || ty.is_floating_point_type();

    if ty.can_default_to_int_for_rv5() && n=="0" {
        let r = context.get_code_with_debug2(& ty.into(), "load_zero_x0", "load 0 const", ast_item)?;
        context.set_code_snippet(r);
    }
    else {
        let debug_info = format!("load constant {n} of type '{ty}'");
        let mut r = context.get_code_with_debug2(& ty.into(), "load_const_to_a0", &debug_info, ast_item)?;
        r.replace_imm(&n);

        if need_replace_label {
            let id = context.attain_unique_id();
            r.replace_labelx(format!("const_float_or_string_value_{id}").as_str());
        }
        
        context.set_code_snippet(r);
    }

    Ok(())
}

/// convert simple variable name to code snippet (with help of symbol table entry)
pub (self) fn var_to_codesnippet(symbol_entry:&SymbolEntry) -> Result<CodeSnippet, CLangError> {
    assert!(symbol_entry.is_register_only());

    let v = symbol_entry.get_name();
    let reg_name = symbol_entry.get_register_name().to_string();
    let reg_usage : RegisterUsage = reg_name.clone().into();
    let name = format!("local var '{v}' in register '{}/{}' no load needed", 
                                reg_usage.get_reg_name(), reg_usage.get_reg_alias());
    let mut r = CodeSnippet::new(&name);
    r.set_debug_info(&name);
    r.set_output_register_names(vec![reg_name]);
    
    Ok(r)
}

/// convert simpel variable name to code snippet, this variable is not in register
pub (self) fn load_simple_var(symbol_entry:&SymbolEntry, context:&mut CodeGenContext, ast_item:&ASTItem ) -> Result<(), CLangError> {
    assert!(!symbol_entry.is_local_var());

    let ty = resolve_clang_type_for_usage(symbol_entry.get_data_type().unwrap(), ast_item)
        .get_type_or_embedded_type();
    let v = symbol_entry.get_name();
    let is_composite = ty.is_composite();
    let is_array = ty.is_array();
    let category = if ty.is_pointer() { 
                                    if context.get_code_gen_config().is_64bit_addressing() { SnippetKey::Long }
                                    else { SnippetKey::Int }
                                }
                                else { ty.into() } ;
    let mut code = if context.get_in_address_scope() {
                                        context.get_code_with_debug2(&category, "load_id", 
                                            & format!("load {v} address to register"), 
                                                        ast_item)?
                                }
                                else {
                                    if context.is_const_expresion_on_left() {
                                        context.get_code_with_debug2(&category, "load_id", 
                                            & format!("load {v} to register"), 
                                                        ast_item)?
                                    }
                                    else {
                                        if is_array || (is_composite && context.is_tanry_dots_following()) {
                                            context.get_code_with_debug2(&category, "load_id", 
                                            & format!("load {v} to register"), 
                                                        ast_item)?
                                        }
                                        else {
                                            context.get_code_with_debug2(&category, "load_id_to_t1", 
                                            & format!("load {v} to register"), 
                                                        ast_item)?
                                        }
                                    }
                                };
    code.replace_labelx_in_incs(&v);
    context.set_code_snippet(code);
    Ok(())
}

/// load simple local memory variable (not in register) to register
pub (self) fn load_simple_local_memory_var(symbol_entry:&SymbolEntry, context:&mut CodeGenContext, ast_item:&ASTItem) -> Result<(), CLangError> {
    assert!(!symbol_entry.is_register_only() && symbol_entry.is_local_var());

    let ty = resolve_clang_type_for_usage(symbol_entry.get_data_type().unwrap(), ast_item)
        .get_type_or_embedded_type();
    let is_composite = ty.is_composite();
    let is_array = ty.is_array();
    let category = ty.into();
    let v = symbol_entry.get_name();
    let stack_offset = symbol_entry.get_stack_offset().unwrap();
    let mut code = if context.is_const_expresion_on_left() {
        context.get_code_with_debug2(&category, "load_id_from_stack", 
            & format!("load {v} address from {stack_offset}(fp) to register"), 
            ast_item)?
    }
    else {
        // if it's composite type member access
        if is_array || context.get_in_address_scope() || (is_composite && context.is_tanry_dots_following()) {
            context.get_code_with_debug2(&category, "load_id_from_stack", 
                & format!("load {v} address from {stack_offset}(fp) to register"), 
                ast_item)?
        }
        else {
            context.get_code_with_debug2(&category, "load_value_from_memory_to_register", 
                & format!("load {v} from {stack_offset}(fp) to register"), 
                    ast_item)?
        }
    };
    code.replace_imm(& stack_offset.to_string());
    context.set_code_snippet(code);
    Ok(())
}

fn load_indirect_function_target_code(fn_name:&str, ast_item:&ASTItem, context:&mut CodeGenContext) -> Result<CodeSnippet, CLangError> {
    let (symbol_type, entry) = ast_item.get_symbol_type_recursive_up(fn_name);
    let symbol_entry = entry.ok_or(CLangError::NoFound(
        (file!().to_string(), line!()),
        format!("cannot resolve function pointer symbol '{fn_name}' for indirect call"),
    ))?;

    if symbol_entry.get_storage_location().is_empty() {
        return Err(CLangError::NoFound(
            (file!().to_string(), line!()),
            format!("symbol '{fn_name}' has no storage location for indirect call"),
        ));
    }

    match symbol_type {
        SymbolType::Var => {
            if symbol_entry.is_local_var() {
                if symbol_entry.is_register_only() {
                    var_to_codesnippet(symbol_entry)
                }
                else {
                    load_simple_local_memory_var(symbol_entry, context, ast_item)?;
                    Ok(context.get_code_snippet().clone())
                }
            }
            else {
                load_simple_var(symbol_entry, context, ast_item)?;
                Ok(context.get_code_snippet().clone())
            }
        }
        SymbolType::Declaration => {
            let storage_locations = symbol_entry.get_storage_location();
            let reg_name = storage_locations.iter()
                .find(|location| location.is_register_only())
                .and_then(|location| location.get_register_name());

            if let Some(reg_name) = reg_name {
                let mut code = CodeSnippet::new(&format!("load parameter function pointer {fn_name} from register {reg_name}"));
                code.set_output_register_names(vec![reg_name.to_string()]);
                code.set_debug_info(&format!("use parameter function pointer '{fn_name}' from register {reg_name}"));
                return Ok(code);
            }

            let stack_offset = storage_locations.iter()
                .find_map(|location| location.get_stack_offset())
                .ok_or(CLangError::NoFound(
                    (file!().to_string(), line!()),
                    format!("parameter '{fn_name}' has no stack storage for indirect call"),
                ))?;

            let category = if context.get_code_gen_config().is_64bit_addressing() {
                SnippetKey::Long
            }
            else {
                SnippetKey::Int
            };
            let mut code = context.get_code_with_debug2(
                &category,
                "load_value_from_memory_to_register",
                &format!("load parameter function pointer {fn_name} from {stack_offset}(fp)"),
                ast_item,
            )?;
            code.replace_imm(&stack_offset.to_string());
            Ok(code)
        }
        _ => Err(CLangError::GeneralError(
            (file!().to_string(), line!()),
            format!("symbol '{fn_name}' is '{symbol_type:?}', expected variable or parameter for indirect call"),
        )),
    }
}

fn generate_md_type_str(md_str:&str, type_name:&str, var_name:&str) -> String {
    md_str.replace(crate::languages::clang::struct_union::VAR_NAME_PLACEHOLDER, var_name)
          .replace(&format!("Type_{type_name}"), var_name)
}

/// code generation for each node
pub trait AsmCodeGenTrait {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError>;
    fn generate_code_string(&self, context:&mut CodeGenContext) -> Result<String, CLangError> {
        self.generate_code(context)?;
        let code = context.get_code_snippet();
        let r = code.generate_code_string();
        Ok(r)
    }
}

impl AsmCodeGenTrait for Declaration {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let symbol_table = self.get_symbol_table().unwrap();

        let mut result = CodeSnippet::new("declaration code");
        if self.is_extern() {
            assert!(self.get_variable_init_values().len() == 1); // only handle one init value case for now
            let init0 = self.get_variable_init_values().first().unwrap();
            let var_name = init0.get_id_name();
            let code = context.get_code("extern", self.get_ast_item())?
                                            .with_labelx_repalced(&var_name);
            result.add_snippet(code);
        }
        else {
            if self.is_global() {
                let n = self.get_type();
                let init = self.get_variable_init_values();

                if n.is_built_in() || n.is_pointer() {
                    for init_first_value in init {
                        let mut r = Vec::default();
                        let id = init_first_value.get_id_name();
                        r.push(SectionItem::Lable(id));

                        let config = context.get_code_gen_config();
                        let directive_type_name = config.get_type_asm(n).unwrap();
                        if let Some(init_value) = init_first_value.get_init_constant_expr() {
                            init_value.generate_code(context)?;
                            result.add_snippet(context.get_code_snippet().clone());
                        }
                        else {
                            if let Some(default_value) = config.get_type_default_value(n) {
                                let directive = Directive::new(&directive_type_name, [default_value].to_vec()).unwrap();
                                let item = SectionItem::Directive(directive);
                                r.push(item);
                                result.add_data_snippet(CodeSnippet::new_with_data(r));
                            }
                            else {
                                return Err(CLangError::ParameterError(format!("cannot find default value for {n}")))
                            }
                        }
                    }
                }
                else {  // complex type like struct, union, enum generation
                    let struct_union_has_def = n.get_struct_union().map_or(false, |su| su.has_struct_def());
                    match n {
                        CLangTypes::Struct(_, _) |
                        CLangTypes::Union(_, _) if struct_union_has_def => {
                            let struct_type = n.get_struct_union().unwrap();
                            struct_type.generate_code(context)?;
                            let md = context.get_code_snippet().clone();
                            let type_name = struct_type.get_name_id().unwrap().to_owned();
                            if let Some(symbol_table) = self.get_ast_item_mut_from_ref().get_symbol_table_mut() {
                                let default_md_data = MarkdownData::default();
                                let md_str = md.get_data().first().unwrap().get_md_data().unwrap_or(&default_md_data).get_data();
                                symbol_table.get_complex_type_table_mut().insert(type_name.clone(), md_str.to_owned());
                            }
                            else {
                                panic!("cannot find symbol table doing declaration code generation")
                            }

                            // handle initialization code
                            let var_names = self.get_var_names();
                            for var_name in var_names {
                                if let Some(md_str) = symbol_table.lookup_complex_type(&type_name) {
                                    let var_md_str = generate_md_type_str(md_str, &type_name, &var_name);
                                    result.add_snippet(CodeSnippet::new_with_md(&var_name, &var_md_str)); // inserted as md or ini file, so no need to add_data_snippet here
                                }
                                else {
                                    return Err(CLangError::cannot_emit(&format!("cannot find struct type {type_name}")))
                                }
                            }
                        }
                        CLangTypes::Struct(_, _) |
                        CLangTypes::Union(_, _) if !struct_union_has_def => {
                            let struct_type = n.get_struct_union().unwrap();
                            let var_names = self.get_var_names();
                            let var_name = var_names.first().unwrap();
                            let struct_type_name = struct_type.get_name_id().unwrap();
                            if let Some(symbol_table) = self.get_symbol_table() {
                                if let Some(md_str) = symbol_table.lookup_complex_type(struct_type_name) {
                                    let var_md_str = generate_md_type_str(md_str, &struct_type_name, &var_name);
                                    result.add_snippet(CodeSnippet::new_with_md(&var_name, &var_md_str)); // inserted as md or ini file, so no need to add_data_snippet here
                                }
                                else {
                                    return Err(CLangError::cannot_emit(&format!("cannot find struct type {struct_type_name}")))
                                }
                            }
                            else {
                                panic!("cannot find symbol table doing declaration code generation")
                            }
                        }
                        CLangTypes::Enum(_, _) => {
                            let enum_type = n.get_enum().unwrap();
                            enum_type.generate_code(context)?;
                            result.add_data_snippet(context.get_code_snippet().clone()); // enum type definition is stored in data snippet, so add data snippet to result
                        }
                        CLangTypes::Array(_ty, name, _dimension) => {
                            let total_size = n.get_type_size_in_bytes(context).unwrap();
                            let mut code = context.get_code_with_debug("array_definition", "array definition", self.get_ast_item())?
                                                            .with_labelx_repalced(name);
                            code.replace_imm(& format!("{total_size}"));
                            result.add_snippet(code);
                        }
                        _ => {
                            todo!("{}", format!("need to process type '{n}' in declaration code generation"))
                        }
                    }
                }
            }
            else {
                for item in self.get_variable_init_values() {  //variable init has type definition already
                    let var_name = item.get_id_name();
                    let var_type = resolve_clang_type_for_usage(self.get_type(), self.get_ast_item());
                    let ast_item = item.get_ast_item();
                    let symbol_item_entry = symbol_table.get_var_storage_location(&var_name);
                    match symbol_item_entry {
                        SymbolLocation::Stack(stack_frame_pointer) => {  // local var is stored in stack
                            assert!(*stack_frame_pointer <= 0);  // the stack frame pointer should be negative offset to fp, since we grow stack to lower address, so the offset is negative, and the offset should not be positive, otherwise it means it's not in stack or there's a bug in stack offset calculation
                            match &var_type {
                                _ if var_type.is_built_in() || var_type.is_pointer() || var_type.is_enum() => {
                                    let mut code = CodeSnippet::new_with_debug_info("local declaration code", &format!("local declaration {var_name}"));
                                    if item.has_initializer() {
                                        assert!(item.get_init().len() == 1);
                                        let init_declarator = item.get_init().first().unwrap();
                                        init_declarator.generate_code(context)?;
                                        let mut push_code = context.get_code_with_debug2(&(&var_type).into(), "save_local_variable_value", 
                                                                                    &format!("save {var_type} '{var_name}' to {stack_frame_pointer}(fp) with value"),
                                                                                    ast_item)?;
                                        push_code.replace_imm(& stack_frame_pointer.to_string());
                                        let init_code = context.get_code_snippet().clone();
                                        push_code.get_input_registers_from(vec![&init_code], context)?;
                                        code.add_snippet(init_code);
                                        code.add_snippet(push_code);
                                    }
                                    else {
                                        let mut push_code = context.get_code_with_debug2(&(&var_type).into(), "set_local_variable_to_stack", 
                                                                        &format!("load local {var_type} '{var_name}' to {stack_frame_pointer}(fp)"),
                                                                        ast_item)?;
                                        push_code.replace_imm(& stack_frame_pointer.to_string());
                                        code.add_snippet(push_code);
                                    }

                                    result.add_snippet(code);
                                }
                                _ if var_type.is_array()=> {
                                    let (ty_base, _name, _dimension) = var_type.get_array_type().unwrap();
                                    let ty_base_size = ty_base.get_type_size_in_bytes(context).unwrap();
                                    if item.has_initializer() {
                                        let mut get_fp_code = context.get_code("move_register", ast_item)?;
                                        get_fp_code.set_input_register("fp"); // fp register
                                        get_fp_code.set_output_register("t0"); // use t0 as temp register to hold stack frame pointer

                                        for init in item.get_init().iter().rev() {
                                            init.generate_code(context)?;
                                            let mut init_code = context.get_code_snippet().clone();
                                            init_code.move_register_from(get_fp_code.get_contaminated_registers(), [].into(), context)?;
                                            match ty_base {
                                                _ if ty_base.is_built_in() => {
                                                    let mut store_code = context.get_code_with_debug2(& ty_base.into(), "push_local_varialbe_with_value_to_stack", 
                                                                                                    &format!("store long array element to stack for local variable '{var_name}'"), 
                                                                                                    ast_item)?;
                                                    store_code.move_register_from(init_code.get_output_register_as_registerset() + get_fp_code.get_contaminated_registers(), [].into(), context)?;
                                                    store_code.get_input_registers_from(vec![&init_code, &get_fp_code], context)?;
                                                    result.add_snippet(init_code);  // init code first
                                                    result.add_snippet(store_code); // store code next
                                                }
                                                _ => {
                                                    panic!("unsupported base type size {ty_base_size} in array initialization for local variable '{var_name}'")
                                                }
                                            }
                                        }

                                        result.insert_at_beginning(vec![get_fp_code]);
                                    }
                                }
                                _ if var_type.is_composite() => {
                                    // assert composite type does not have init data
                                    assert!(item.get_init().is_empty() && var_type.is_composite());
                                    
                                    // create space for composite type variable on stack
                                    let info = format!("declaration code for composite type local variable '{var_name}' of type '{var_type}' @ {stack_frame_pointer}(fp)");
                                    let code_snippet = CodeSnippet::new_with_debug_info(&info, &info);
                                    result.add_snippet(code_snippet);
                                }
                                _ => {
                                    let err_msg = format!("need to process local variable '{var_name}' of type '{var_type}' stored in stack '{stack_frame_pointer}'");
                                    todo!("{}", err_msg)
                                }
                            }
                        }
                        SymbolLocation::Register(reg_name) => {  // local var is stored in register
                            match var_type {
                                _ if var_type.is_built_in() || var_type.is_pointer() || 
                                    var_type.is_enum() || var_type.is_functionpointer() => {
                                    if item.has_initializer() {
                                        assert!(item.get_init().len() == 1);
                                        let init_declarator = item.get_init().first().unwrap();
                                        // generate code to load value to the register and assign output register to the local var register
                                        // assumption: init code won't use the local var register
                                        init_declarator.generate_code(context)?;
                                        let mut init_code = context.get_code_snippet().clone();
                                        if init_code.get_output_register_names().len() == 1 {
                                            let output_reg_name = init_code.get_output_register_names().first().unwrap().to_string();
                                            if !symbol_table.is_fixed_register(&output_reg_name) {
                                                // if output register is not fixed register, directly replace it with local var register
                                                init_code.replace_register(&output_reg_name, reg_name);
                                                // If the snippet emitted no actual instructions (e.g. load_zero_x0
                                                // only declares output=x0 with an empty .text block), replace_register
                                                // has nothing to rewrite.  Fall back to an explicit mv so the target
                                                // register actually receives the value.
                                                if init_code.get_code_len() == 0 {
                                                    let move_code = context.new_move_register_snippet(&var_type.into(), ast_item, &output_reg_name, reg_name)?;
                                                    init_code.add_snippet(move_code);
                                                }
                                            } else {
                                                // if output register is fixed register, need to move it to local var register
                                                let move_code = context.new_move_register_snippet(&var_type.into(), ast_item, &output_reg_name, reg_name)?;
                                                init_code.add_snippet(move_code);
                                            }
                                        } else {
                                            panic!("need to process more than 1 output register in local var initialization")
                                        }
                                        result.add_snippet(init_code);
                                    }
                                }
                                _ if var_type.is_array() || var_type.is_composite() => {
                                    let code = self.generate_big_data_type_init_code(context)?;
                                    result.add_snippet(code);
                                }
                                _ => {
                                    let err_msg = format!("need to process local variable '{var_name}' of type '{var_type}' stored in register '{reg_name}'");
                                    todo!("{}", err_msg)
                                }
                            }
                        }
                        _ => {
                            error_string(format!("unsupported storage location for local variable declaration of {var_name}"));
                            return Err(CLangError::cannot_emit(& format!("unsupported storage location for local variable declaration")))
                        }
                    }
                }
            }
        }     

        context.set_code_snippet(result);
        Ok(())
    }
}

impl AsmCodeGenTrait for FunctionDefinition {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let function_body_label = "__function__body";
        let parameter_label = "__parameters__to__stack";
        let function_end_label = "__function_body_end";
        let function_name = self.get_function_name();

        if self.is_main_function() {
            let ast_item = self.get_ast_item();
            let mut r = context.get_code_with_debug("function_template", &format!("function {function_name}"), ast_item)?
                                    .with_labelx_repalced(&function_name);

            let mut parameter_code = CodeSnippet::new("main function parameter");
            parameter_code.add_asm_inc("mv", vec![Some("fp"), Some("sp")])?;
            r.insert_after_label(parameter_label, parameter_code)?;

            self.get_compound_statement().generate_code(context)?;
            let compound_code = context.get_code_snippet().clone(); 
            r.insert_after_label(function_body_label, compound_code)?;

            // process the last statement
            let last_statement = self.get_compound_statement().get_last_statement()
                .ok_or(CLangError::cannot_emit(& format!("cannot find last statement")))?;

            // if last statement is return statement, no need to add exit
            if !last_statement.get_jump().map_or(false, |x| x.is_return()) {
                // add return 0 template code
                let code = context.get_code("main_asm_return0", ast_item)?;
                r.add_snippet(code);
            }

            r.replace_label(function_body_label, & format!("{function_body_label}_{function_name}"));
            r.replace_label(parameter_label, & format!("{parameter_label}_{function_name}"));
            
            context.set_code_snippet(r);
        }
        else {
            if context.get_is_build_to_lib() {
                let ast_item = self.get_ast_item();
                let mut r = context.get_code_with_debug("global_function_template", &format!("function {function_name}"), ast_item)?
                                        .with_labelx_repalced(&function_name);
                self.get_compound_statement().generate_code(context)?;
                let mut compound_code = context.get_code_snippet().clone();

                // For functions without an explicit trailing return, emit a fallback epilogue.
                let need_default_return = self.get_compound_statement().needs_fallback_return_epilogue();
                if need_default_return {
                    let code = context.get_code("function_return", ast_item)?;
                    compound_code.add_snippet(code);
                }

                r.insert_after_label(function_body_label, compound_code)?;

                r.replace_label(function_body_label, & format!("{function_body_label}_{function_name}"));
                r.replace_label(parameter_label, & format!("{parameter_label}_{function_name}"));
                r.replace_label(function_end_label, & format!("{function_end_label}_{function_name}"));

                context.set_code_snippet(r);
            }
            else {
                let ast_item = self.get_ast_item();
                let mut r = context.get_code_with_debug("function_template", &format!("function {function_name}"), ast_item)?
                                        .with_labelx_repalced(&function_name);
                self.get_compound_statement().generate_code(context)?;
                let mut compound_code = context.get_code_snippet().clone();

                // For functions without an explicit trailing return, emit a fallback epilogue.
                let need_default_return = self.get_compound_statement().needs_fallback_return_epilogue();
                if need_default_return {
                    let code = context.get_code("function_return", ast_item)?;
                    compound_code.add_snippet(code);
                }

                r.insert_after_label(function_body_label, compound_code)?;

                r.replace_label(function_body_label, & format!("{function_body_label}_{function_name}"));
                r.replace_label(parameter_label, & format!("{parameter_label}_{function_name}"));

                context.set_code_snippet(r);
            }
        }

        Ok(())
    }
}

impl AsmCodeGenTrait for CompoundStatement {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let ast_item = self.get_ast_item();
        let mut r = context.get_code("compound_template", ast_item)?;
        
        // set the local variable stack size
        let var_size = self.get_symbol_table().get_var_symbol_total_size_in_bytes(context);
        if var_size > 0 {
            // round up to 16 bytes alignment
            let aligned_size = round_up_to_16(var_size) as isize;
            let allocate_code = gen_add_offset("sp", "sp", -aligned_size);
            let mut stack_allocation_code = CodeSnippet::from_asm_code_str("allocate stack pointer", &allocate_code)?;
            stack_allocation_code.set_debug_info(&format!("allocate {aligned_size} bytes for local variables in compound statement({})", self.get_ast_item().get_sequence_id()));
            r.add_snippet(stack_allocation_code);
        }

        for item in self.get_compound_statement_items() {
            match item {
                CompoundStatementItem::Declaration(local_var) => {
                    local_var.generate_code(context)?;
                    r.add_snippet(context.get_code_snippet().clone());
                }
                CompoundStatementItem::Statement(statement) => {
                    statement.generate_code(context)?;
                    r.add_snippet(context.get_code_snippet().clone());
                }
            }
        }

        context.set_code_snippet(r);
        Ok(())
    }
}

impl AsmCodeGenTrait for Statement {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let ast_item = self.get_ast_item();
        match self {
            Statement::Label(_, n) => {
                match n {
                    LabelStatement::Default(_, expr) => {
                        let mut template = context.get_code("default_template", ast_item)?;

                        //generate default statement section
                        expr.generate_code(context)?;
                        let mut statements = context.get_code_snippet().clone();
                        statements.set_name("case_statements");
                        template.insert_after_label_bulk(vec![("case_statements".to_string(), statements)], context)?;

                        template.append_unique_id_to_label();
                        context.set_code_snippet(template);
                    }
                    LabelStatement::Id(_, n) => {
                        let expr = &n.statement;
                        expr.generate_code(context)?;
                        let incs_code = context.get_code_snippet().clone();
                        let mut r = CodeSnippet::new("label_statement_id");
                        r.add_code(SectionItem::new_label(&n.id).into()); 
                        r.add_snippet(incs_code);
                        context.set_code_snippet(r);                      
                    }
                    LabelStatement::Case(_, case_value) => {
                        let n = &case_value.constant_expression;
                        let expr = &case_value.statement;
                        let mut template = context.get_code("case_template", ast_item)?;
                        n.generate_code(context)?;
                        let mut const_code = context.get_code_snippet().clone();
                        const_code.set_name("case_const_code");
                        const_code.move_register_from(template.get_all_registers(), [].into(), context)?;
                        template.get_input_register_from(0, &const_code, 0, context)?;
                        expr.generate_code(context)?;
                        let mut statements = context.get_code_snippet().clone();
                        statements.set_name("case_statements");
                        statements.move_register_from(template.get_all_registers(), [].into(), context)?;
                        template.insert_after_label("load_case_const", const_code)?;
                        template.insert_after_label("case_statements", statements)?;
                        template.append_unique_id_to_label();
                        context.set_code_snippet(template);
                    }
                };
    
                Ok(())
            }
            Statement::Compound(_, n) => {
                let statements = n.get_statements();
                let mut r = CodeSnippet::new("compound statement");
                for n in statements.iter() {
                    n.generate_code(context)?;
                    r.add_snippet(context.get_code_snippet().clone());
                }
    
                context.set_code_snippet(r);
                Ok(())
            }
            Statement::Expression(_, n) => {
                n.generate_code(context)           
            }
            Statement::Iteration(_, n) => {
                match n {
                    IterationStatement::FOR(_, n) => {
                        let start_stop_expr = &n.expression_statements;
                        let inc_expr = &n.clang_expression;
                        let statements = &n.statement;
                        let mut template = context.get_code("for_statement_template", ast_item)?;
                       
                        statements.generate_code(context)?;
                        let mut statement_code = context.get_code_snippet().clone();
                        match start_stop_expr.as_slice() {
                            [a, b] => {
                                a.generate_code(context)?;
                                let start_expr = context.get_code_snippet().clone();
                                b.generate_code(context)?;
                                let condition_expr = context.get_code_snippet().clone();
                                template.get_input_register_from(0, &condition_expr, 0, context)?;
    
                                //handle continue and break
                                statement_code.apply_to_code_snippet("continue", |x| x.replace_label("CONTINUE", "status_change"));
                                statement_code.apply_to_code_snippet("break", |x| x.replace_label("BREAK_TARGET", "end_if"));
    
                                if let Some(n) = inc_expr {
                                    n.first().unwrap().generate_code(context)?;
                                    let status_change_expr = context.get_code_snippet().clone(); 
                                    template.insert_after_label_bulk(vec![("for_statement_start".to_string(), start_expr), 
                                                                                ("for_evaluation".to_string(), condition_expr), 
                                                                                ("statement".to_string(), statement_code), 
                                                                                ("status_change".to_string(), status_change_expr)], context)?;
                                }
                                else {
                                    template.insert_after_label_bulk(vec![("for_statement_start".to_string(), start_expr), 
                                                                                ("for_evaluation".to_string(), condition_expr), 
                                                                                ("statement".to_string(), statement_code)], context)?;
                                }                            
                            }
                            _ => {
                                todo!()
                            }
                        }

                        template.append_unique_id_to_label();
                        context.set_code_snippet(template);
                        Ok(())
                    }
                    IterationStatement::DO(_, n) => {
                        let statements = &n.statement;
                        let bool_expr = &n.expression;
                        let mut template = context.get_code("do_while_template", ast_item)?;
                        context.set_flip_boolean_expr(true);
                        bool_expr.first().unwrap().generate_code(context)?;
                        let status_change_code = context.get_code_snippet().clone(); 
                        context.set_flip_boolean_expr(false);
                        statements.generate_code(context)?;
                        let mut statement_code = context.get_code_snippet().clone();

                        //handle continue and break
                        statement_code.apply_to_code_snippet("continue", |x| x.replace_label("CONTINUE", "do_while_start"));
                        statement_code.apply_to_code_snippet("break", |x| x.replace_label("BREAK_TARGET", "do_while_end"));
                        template.insert_after_label("end_if", statement_code)?;
                        
                        template.get_input_register_from(0, &status_change_code, 0, context)?;
                        template.insert_after_label("do_condition", status_change_code)?;

                        template.append_unique_id_to_label();
                        context.set_code_snippet(template);
                        Ok(())
                    }
                    IterationStatement::WHILE(_, n) => {
                        let bool_expr = &n.expression;
                        let statements = &n.statement;
                        let mut template = context.get_code("while_template", ast_item)?;
                        bool_expr.first().unwrap().generate_code(context)?;
                        let status_change_code = context.get_code_snippet().clone();
                        statements.generate_code(context)?;
                        let mut statement_code = context.get_code_snippet().clone();

                        //handle continue and break
                        statement_code.apply_to_code_snippet("continue", |x| x.replace_label("CONTINUE", "while_start"));
                        statement_code.apply_to_code_snippet("break", |x| x.replace_label("BREAK_TARGET", "while_end"));

                        template.get_input_register_from(0, &status_change_code, 0, context)?;
                        template.insert_after_label("while_start", status_change_code)?;
                        template.insert_after_label("while_statements", statement_code)?;

                        template.append_unique_id_to_label();
                        context.set_code_snippet(template);
                        Ok(())
                    }
                }
            }
            Statement::Jump(_, n) => {
                match n {
                    JumpStatement::Break(_) => {
                        let mut template = context.get_code("break_template", ast_item)?;
                        template.set_debug_info("break statement");
                        context.set_code_snippet(template);
                        Ok(())
                    }
                    JumpStatement::Continue(_) => {
                        let mut template = context.get_code("continue_template", ast_item)?;
                        template.set_debug_info("continue statement");
                        context.set_code_snippet(template);
                        Ok(())
                    }
                    JumpStatement::Go(_, m) => {
                        let mut template = context.get_code("goto_template", ast_item)?;
                        template.replace_label("GOTO", &m);
                        template.set_debug_info(&format!("goto {m}"));
                        context.set_code_snippet(template);
                        Ok(())
                    }
                    JumpStatement::Return(_, m) => {
                        let current_function = n.get_ast_item().get_current_function().unwrap();  //return must have a containing function

                        let mut template = if current_function.is_main_function() { context.get_code("exit", ast_item)? } 
                                                        else { context.get_code("function_return", ast_item)? };
                        match m.as_slice() {
                            [a] => {
                                a.generate_code(context)?;
                                let expr = context.get_code_snippet().clone();
                                let register_names = expr.get_output_register_names().clone();
                                match register_names.as_slice() {
                                    [n] => {
                                        // insert_after_label uses LIFO/prepend semantics: last inserted
                                        // goes immediately after the label.  To get the correct order:
                                        //   [__label__exit, expr, mv a0 t2, ret]
                                        // insert move_code FIRST (it ends up last) then insert expr.
                                        if n != "a0" {
                                            // Move the final result register to a0 at return boundary.
                                            // Insert first so it lands AFTER the expression code.
                                            let move_code = context.new_move_register_snippet(&SnippetKey::default(), ast_item, n, "a0")?;
                                            template.insert_after_label("__label__exit", move_code)?;
                                        }

                                        if expr.get_code_len() > 0 {
                                            // Insert expr second so it lands BEFORE move_code.
                                            // For function calls that already return in a0, move_code is
                                            // skipped above and only the call code is emitted.
                                            template.insert_after_label("__label__exit", expr.clone())?;
                                        }

                                        template.append_unique_id_to_label();
                                    }
                                    _ => {
                                        todo!("Need process more than 1 register in return statement")
                                    }
                                }

                                context.set_code_snippet(template);
                                Ok(())
                            }
                            _ => {
                                todo!("Need process more than 1 assginment expression in return statement")
                            }
                        }
                    }
                }
            }
            Statement::Selection(_, n) => {
                match n {
                    SelectionStatement::IF(_, n) => {
                        let bool_expr = n.get_boolean_expression();
                        let if_statement = n.get_if_branch_statement();
                        let else_statement_option= n.get_else_branch_statement();
                        
                        assert!(bool_expr.len() == 1);
                        let const_expr = bool_expr.last().unwrap();
                        const_expr.generate_code(context)?;
                        let bool_expr_code = context.get_code_snippet().clone();
                        if_statement.generate_code(context)?;
                        let if_branch_code = context.get_code_snippet().clone();
                        
                        if let Some(else_statement) = else_statement_option {  //if..else
                            let mut code = context.get_code("if_else", ast_item)?;
                            else_statement.generate_code(context)?;
                            let else_branch_code = context.get_code_snippet().clone();
                            code.get_input_register_from(0, &bool_expr_code, 0, context)?;
                            code.insert_after_label("if_only", bool_expr_code)?;   //no need to resolve conflict with bool_expr_code, that value will not be used and no conflict
                            code.insert_after_label("if_branch", if_branch_code)?;//no need to resolve conflict with bool_expr_code, that value will not be used and no conflict
                            code.insert_after_label("end_if", else_branch_code)?;//no need to resolve conflict with bool_expr_code, that value will not be used and no conflict
                            code.append_unique_id_to_label();
                            context.set_code_snippet(code);
                            Ok(())                
                        }
                        else {  //if only 
                            let mut code = context.get_code("if_only", ast_item)?;
                            code.get_input_register_from(0, &bool_expr_code, 0, context)?;
                            code.insert_after_label("if_only", bool_expr_code)?;  //no need to resolve conflict with bool_expr_code, that value will not be used and no conflict
                            code.insert_after_label("if_branch", if_branch_code)?; //no need to resolve conflict with bool_expr_code, that value will not be used and no conflict
                            code.append_unique_id_to_label();
                            context.set_code_snippet(code);
                            Ok(()) 
                        }
                    }
                    SelectionStatement::SWITCH(_, n) => {
                        let const_expr = n.get_boolean_expression();
                        let case_statements = n.get_switch_statement();
                        let const_expr = const_expr.last()
                                                                    .ok_or(CLangError::cannot_emit(& format!("need a Boolean const expression")))?
                                                                    .get_constant_expr();
                        const_expr.generate_code(context)?;
                        let bool_expr_code = context.get_code_snippet().clone();
                        case_statements.generate_code(context)?;
                        let mut case_statements_code = context.get_code_snippet().clone();

                        // build up the jumping table 
                        let mut jumping_table_code = CodeSnippet::new("jump_table");
                        jumping_table_code.set_debug_info("switch jumping table code");
                        for item in case_statements_code.get_code_mut() {                    
                            match item {
                                CodeSnippetDataItem::Snippet(n) if n.get_name() == "case_template" || 
                                                                                     n.get_name() == "default_template"=> {
                                    let mut item_to_remove = Vec::new();

                                    for case_item in n.get_code_mut() {
                                        match case_item {
                                            CodeSnippetDataItem::Inc(n) if n.get_inc_name() == Some("BEQ".to_string()) || 
                                                                                             n.get_inc_name() == Some("JAL".to_string()) => {
                                                item_to_remove.push(case_item);
                                            }
                                            CodeSnippetDataItem::Snippet(n) if n.get_name() == "case_const_code" => {
                                                item_to_remove.push(case_item);
                                            }
                                            _ => {}
                                        }
                                    }

                                    let items_to_remove = item_to_remove.into_iter().map(|x| x.clone()).collect::<Vec<_>>();
                                    for x in items_to_remove {
                                        n.remove_code_item(&x);
                                        jumping_table_code.add_code(x);
                                    }
                                }
                                _ => {}
                            }
                        }

                        // build up the statements
                        let mut template = context.get_code("switch_template", ast_item)?;
                        template.insert_after_label_bulk(vec![("switch_start".to_string(), bool_expr_code), 
                                                                        ("case_section".to_string(), jumping_table_code), 
                                                                        ("case_statements".to_string(), case_statements_code)], context)?;

                        template.append_unique_id_to_label();
                        context.set_code_snippet(template);
                        Ok(())
                    }
                }
            }
            Statement::AsmStatement(_, n) => {
                n.generate_code(context)
            }
        }
    }
}

impl AsmCodeGenTrait for &Vec<AssignmentExpression> {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        if self.len() == 1 {
            let assignment = self.first().unwrap();
            assignment.generate_code(context)
        }
        else {
            let mut rr = CodeSnippet::new("expression statement");

            for assignment in self.iter() {
                assignment.generate_code(context)?;
                let r = context.get_code_snippet().clone();
                rr.add_snippet(r);
            }

            context.set_code_snippet(rr);
            Ok(()) 
        }
    }
}

impl AsmCodeGenTrait for ExpressionStatement {
    
    /// generate code for expression statement
    /// if there is only one item in the assignment list, code from that list will be returned so save one layer
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        self.get_assignments().generate_code(context)      
    }
}

impl AsmCodeGenTrait for Atom {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let ast_item = self.get_ast_item();
        match &**self.get_item() {
            AtomItem::BasicTypeObject(_, v) => {
                
                let ty : CLangTypes = v.get_type_def()
                                        .map_err(|e| CLangError::NoFound((file!().to_string(), line!()), format!("{e:?}")))?
                                        .into();

                let n = if ty.is_char() { format!("'{v}'")} 
                                else { v.to_string() };

                clang_type_and_value_to_codesnippet(&ty, &n, context, ast_item)
            }
            AtomItem::Id(_, v) => {  //this section will process local var, global var, and function parameter cases
                let (symbol_type, entry) = self.get_ast_item().get_symbol_type_recursive_up(v);
                match symbol_type {
                    SymbolType::Var => {
                        if let Some(symbol_entry) = self.get_ast_item().get_symbol(v, &Declaration::get_symbol_table_type_var()) {
                            let ty = resolve_clang_type_for_usage(symbol_entry.get_data_type().unwrap(), ast_item)
                                .get_type_or_embedded_type();
                            let (is_local_var, is_register) = (symbol_entry.is_local_var(), symbol_entry.is_register_only());
                            match ty {
                                _ if ty.is_built_in() || 
                                    ty.is_pointer() || 
                                    ty.is_array() ||
                                    ty.is_composite() ||
                                    ty.is_functionpointer()                                    
                                => {
                                    match (is_local_var, is_register) {
                                        (true, true) => {
                                            context.set_code_snippet(var_to_codesnippet(symbol_entry)?);
                                        }
                                        (true, false) => {
                                            load_simple_local_memory_var(symbol_entry, context, ast_item)?;
                                        }
                                        (false, _) => {
                                            load_simple_var(symbol_entry, context, ast_item)?;
                                        }
                                    }

                                    Ok(())
                                }
                                _ => {
                                    error_string(format!("need to do '{ty}' for varialbe '{v}' and its ast_item id = {}", self.get_id()));
                                    todo!()
                                }
                            }
                        }
                        else {
                            panic!("cannot find variable symbol entry for {} and conflict with the type search", v);
                        }
                    }
                    SymbolType::Declaration => {
                        if let Some(symbol_entry) = self.get_ast_item().get_symbol(v, &&ParameterDeclaration::get_symbol_table_type_parameter_declaration()) {
                            let ty = symbol_entry.get_data_type()
                                                            .ok_or(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find parameter declaration type for {v} and its ast_item id = {}", self.get_id())))?;
                            let storage_location = symbol_entry.get_single_storage_location().unwrap().to_owned();
                            let is_pointer_type = ty.is_pointer();
                            let is_composite_type = ty.is_composite();
                            let is_array_type = ty.is_array();

                            let category = if is_pointer_type {
                                if context.get_code_gen_config().is_64bit_addressing() { SnippetKey::Long }
                                else { SnippetKey::Int }
                            }
                            else { ty.into() };

                            // composite/array parameters are treated as address-like values in codegen.
                            let load_address_only = context.is_const_expresion_on_left() ||
                                                    context.get_in_address_scope() ||
                                                    is_composite_type ||
                                                    is_array_type;

                            if storage_location.is_register_only() ||
                                (storage_location.is_register_and_stack() &&
                                 storage_location.get_register_name() == Some("a0")) {
                                let reg_name = storage_location.get_register_name().unwrap();
                                let mut code = CodeSnippet::new(&format!("load parameter {v} from register {reg_name}"));
                                code.set_output_register_names(vec![reg_name.to_string()]);
                                context.set_code_snippet(code);
                                Ok(())
                            }
                            else {
                                // RegisterAndStack parameters other than a0 are recovered from fp-relative stack slots.
                                let stack_offset = storage_location.get_stack_offset().unwrap();
                                let mut code = if load_address_only {
                                    context.get_code_with_debug2(&category, "load_id_from_stack", 
                                        &format!("load parameter {v} address from {stack_offset}(fp)"),
                                                    ast_item)?
                                }
                                else {
                                    context.get_code_with_debug2(&category, "load_value_from_memory_to_register", 
                                        &format!("load parameter {v} from {stack_offset}(fp)"),
                                                    ast_item)?
                                };
                                code.replace_imm(& stack_offset.to_string());

                                context.set_code_snippet(code);                                  
                                Ok(())
                            }
                        }
                        else {
                            panic!("cannot find parameter declaration symbol entry for {} and conflict with the type search", v);
                        }
                    }
                    SymbolType::EnumConstant => {
                        let symbol_entry = entry.unwrap();
                        let clang_item = symbol_entry.get_clang_item_link().unwrap();
                        let enum_item = clang_item.get_enum_item().unwrap();
                        
                        clang_type_and_value_to_codesnippet(enum_item.get_base_type(), 
                                        &enum_item.get_base_value().to_string(), 
                                        context, 
                                        ast_item)
                    }
                    SymbolType::StructUnionItem => {
                        let system_entry = entry.unwrap();
                        let offset = system_entry.get_storage_location().first().unwrap()
                                                        .get_struct_union_member_offset().unwrap();
                        let mut code = context.get_code_with_debug("load_const_to_a0", &format!("load offset {offset}"), ast_item)?;
                        code.replace_imm(&offset.to_string());
                        context.set_code_snippet(code);
                        Ok(())
                    }
                    SymbolType::Function => {
                        let mut code = context.get_code_with_debug("load_id", &format!("load function address for {v}"), ast_item)?;
                        code.replace_labelx(v);
                        context.set_code_snippet(code);
                        Ok(())
                    }
                    _ => {
                        error_string(format!("need handle symbol type = {symbol_type}"));
                        error_string(format!("need to do atomitem::id '{v}' with seqId = ({}) \n\n {self:?}", self.get_id()));
                        todo!()
                    }
                }
            }
            AtomItem::Expression(_, n) => {
                n.generate_code(context)
            }
            AtomItem::TanryExpression(_, n) => {
                n.generate_code(context)
            }
        }
    }
}

impl AsmCodeGenTrait for Expression {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let ast_item = self.get_ast_item();
        match self {
            Expression::Atom(_, n) => {
                n.generate_code(context)
            }
            Expression::Expression(_, nn) => {
                let op = nn.get_op();
                let m = nn.get_left();
                let n = nn.get_right();
                
                let m_type = resolve_clang_type_for_usage(&m.get_clang_type(), ast_item);
                let n_type = resolve_clang_type_for_usage(&n.get_clang_type(), ast_item);
                let (snippet_key, m_conv, n_conv) = 
                    context.get_snippet_key_and_conversion_code_for_assignment(Some(&m_type), Some(&n_type), ast_item);
                
                let hash = context.get_instruction_mapping();
                match op {
                    LangOperators::BitLeftMove |
                    LangOperators::BitRightMove |
                    LangOperators::BitAnd |
                    LangOperators::BitOr |
                    LangOperators::BitXor |
                    LangOperators::LogicXor |
                    LangOperators::Rem |
                    LangOperators::Add |
                    LangOperators::Mul | 
                    LangOperators::Sub |
                    LangOperators::Div => {
                        let asm_op = hash[&op.to_string()].to_string();
                        m.generate_code(context)?;
                        let mut a = context.get_code_snippet().clone();
                        n.generate_code(context)?;
                        let mut b = context.get_code_snippet().clone();
                        context.append_conversion_code(&mut a, m_conv)?;
                        context.append_conversion_code(&mut b, n_conv)?;
                        b.move_register_from(a.get_output_register_as_registerset(), [].into(), context)?;

                        // use add template and then replace the instruction with other operator
                        let mut r = context.get_code_with_debug2(& snippet_key, &asm_op, & format!("{self}"), ast_item)?;
                        r.get_input_registers_from(vec![&a, &b], context)?;
                        let label_str = r.get_all_labels().last().unwrap().clone();
                        r.insert_after_label_bulk(vec![(label_str.clone(), a), (label_str.clone(), b)], context)?;
                        context.set_code_snippet(r.clone());
                        Ok(())
                    }
                    LangOperators::Less |
                    LangOperators::Greater |
                    LangOperators::LessEqual |
                    LangOperators::GreaterEuqal |
                    LangOperators::LogicEqual |
                    LangOperators::NotEqual => {
                        let asm_op = if context.is_flip_boolean_expr() { hash[&op.get_opposite().to_string()].to_string() } 
                                            else { hash[&op.to_string()].to_string() };

                        m.generate_code(context)?;
                        let mut a = context.get_code_snippet().clone();
                        n.generate_code(context)?;
                        let mut b = context.get_code_snippet().clone();
                        context.append_conversion_code(&mut b, m_conv)?;
                        context.append_conversion_code(&mut a, n_conv)?;
                        b.move_register_from(a.get_output_register_as_registerset(), [].into(), context)?;

                        let mut r = context.get_code_with_debug2(&snippet_key, &asm_op, &format!("{self}"), ast_item)?;
                        r.move_register_from(a.get_output_register_as_registerset() + b.get_output_register_as_registerset(), [].into(), context)?;
                        r.get_input_registers_from(vec![&a, &b], context)?;
                        r.insert_at_beginning(vec![a, b]);
                        r.append_unique_id_to_label();   //append unique id to label so it won't be messed up with other unit
                        context.set_code_snippet(r);
                        Ok(())  
                    }
                    LangOperators::LogicAnd |
                    LangOperators::LogicOr => {
                        let asm_op = hash[&op.to_string()].to_string();
                        m.generate_code(context)?;
                        let mut a = context.get_code_snippet().clone();
                        n.generate_code(context)?;
                        let mut b = context.get_code_snippet().clone();
                        context.append_conversion_code(&mut b, m_conv)?;
                        context.append_conversion_code(&mut a, n_conv)?;
                        b.move_register_from(a.get_output_register_as_registerset(), [].into(), context)?;

                        // use boolean_operation template and then replace the instruction with other operator
                        let mut r = context.get_code("boolean_operation", ast_item)?;
                        r.replace_asm_op_in_last_inc(&asm_op)?;
                        r.get_input_registers_from(vec![&a, &b], context)?;
                        let label_str = r.get_all_labels().last().unwrap().clone();
                        r.insert_after_label_bulk(vec![(label_str.clone(), a), (label_str.clone(), b)], context)?;
                        r.set_debug_info(& format!("{self}"));
                        context.set_code_snippet(r);
                        Ok(())
                    }
                    LangOperators::AddressArrow | 
                    LangOperators::AddressDot => {
                        if nn.get_is_highest_addressing_expression() {
                            context.set_in_address_scope(true);
                        }

                        m.generate_code(context)?;
                        let mut a = context.get_code_snippet().clone();  //this should hold the address of the struct/union variable, and the code should be generated based on this address in atom codegen, so no need to generate code for left and right expression separately here, just need to prepare the address in atom codegen
                        let offset = n.get_atom_item().and_then(|x| x.get_item().get_struct_union_member_offset())
                            .ok_or(CLangError::cannot_emit(& format!("cannot find struct/union member offset for expression {self:?}")))?; // check if the right expression is a valid struct/union member access, if not, emit error, if yes, the offset will be used in atom codegen to generate code for loading the member value
                        
                        let output_reg = a.get_output_register_names().first().unwrap().to_string();
                        if offset!=0 {
                            a.add_asm_inc("addi", vec![Some(&output_reg), Some(&output_reg), Some(&offset.to_string())])?; // add offset to the address in the register so it will point to the member variable, this is the preparation for codegen of struct/union member access in atom codegen
                        }
                        a.set_debug_info(& format!("address operator '{m}.{n}'"));
                        
                        if nn.get_is_highest_addressing_expression() && !context.is_const_expresion_on_left() {
                            // add load value from address
                            // Pick opcode by the whole address-expression result type (e.g. `ss1.sa` -> int),
                            // avoiding accidental address-sized loads on RV64 for scalar members.
                            let resolved_result_type = resolve_clang_type_for_usage(&nn.get_clang_type(), ast_item);
                            let snippet_key = &resolved_result_type.into();
                            let mut load_value_snippet = context.get_code_with_debug2(snippet_key, 
                                "load_address_to_reg", 
                                "load value after address reference", 
                                n.get_ast_item())?;
                            load_value_snippet.set_input_register(&output_reg);
                            load_value_snippet.set_output_register(&output_reg);
                            a.insert_at_end(vec![load_value_snippet]);
                            context.set_in_address_scope(false);
                        }
                        else if !nn.get_is_highest_addressing_expression() && op == &LangOperators::AddressArrow {
                            // Intermediate pointer member access (e.g. `canvas->point` in `canvas->point->x`).
                            // We've computed the address of the pointer field; now load the pointer value
                            // stored there so subsequent member accesses use the correct base address.
                            // Use ld (64-bit) or lw (32-bit) depending on the addressing mode.
                            let ptr_category = if context.get_code_gen_config().is_64bit_addressing() { SnippetKey::Long }
                                               else { SnippetKey::Int };
                            let mut load_ptr_snippet = context.get_code_with_debug2(&ptr_category, 
                                "load_address_to_reg", 
                                &format!("dereference intermediate pointer member '{n}'"), 
                                n.get_ast_item())?;
                            load_ptr_snippet.set_input_register(&output_reg);
                            load_ptr_snippet.set_output_register(&output_reg);
                            a.insert_at_end(vec![load_ptr_snippet]);
                        }

                        context.set_code_snippet(a);

                        Ok(())
                    }
                    _ => {
                        error_string(format!("operation {op:?} need logic to process"));
                        todo!()
                    }
                }
            }
            Expression::Postfix(_, n) => {
                let v = n.get_expr();
                let op = n.get_op();

                if let Some(postfix) = op.get_postfix_expression_part() {
                    if postfix.is_arguments() {
                        // generate fucntion call code
                        let (fn_name, l) = logic::execute_fn_invoke_logic_get_invoke(self.get_all_clangitems());
                        let assginments = l.iter()
                                                .map(|x| x.get_assignment_expression().unwrap())
                                                .map(|x| &*x)
                                                .collect::<Vec<_>>();
                        generate_function_invoke_code(&fn_name, 
                                self.get_ast_item(), 
                                assginments, 
                                context)?;
                        return Ok(());
                    }
                    else if postfix.is_clangexpression() {
                        // get the array type's code (the array is n.Expression)
                        let (expr_clang, part) = get_postfix_ref_expr_and_names(n);

                        let expr = expr_clang.get_expression().unwrap();

                        // get the asm code get the array address
                        let mut array_code = context.with_const_expresion_on_left_value(true,
                            |ctx| generate_dot_reference_code(expr, &part, ctx))?;

                        // get the array access code 
                        let array_type = n.get_index_operand_type();
                        let assignments = postfix.get_clang_expression().unwrap();
                        assert!(array_code.get_output_register_names().len() == 1, "arraycode output register should be 1, but got {}", array_code.get_output_register_names().len());
                        let contaminated_regs = array_code.get_contaminated_registers();
                        let mut code = generate_array_access_asm_code2(self.get_ast_item(), 
                                &array_type, 
                                array_code.get_output_register_names().first().unwrap(), 
                                assignments, 
                                contaminated_regs, 
                                context)?;
                        code.set_debug_info(& format!("array access for {v}"));
                        let code_output_registers = code.get_output_register_names().to_vec();
                        
                        array_code.insert_at_end(vec![code]);

                        // The appended array-access snippet computes the final RHS value/address.
                        // Keep its output register(s) as the expression output so downstream users
                        // (e.g., return codegen) consume the latest value producer.
                        array_code.set_output_register_names(code_output_registers);

                        context.set_code_snippet(array_code);
                        return Ok(());
                    }
                }

                let const_code = context.with_const_expresion_on_left_value(true, |ctx| {
                    v.generate_code(ctx)?;
                    Ok::<CodeSnippet, CLangError>(ctx.get_code_snippet().clone())
                })?;
                
                let with_reg = n.get_reg_name().is_some();

                match op {
                    PostOperator::Inc if with_reg => {
                        if n.get_reg_name().unwrap() != const_code.get_output_register_names().first().unwrap().to_string() {
                            return Err(CLangError::cannot_emit(& format!("the register name in post inc/dec operator must be the same as the expression's output register")));
                        }

                        let mut inc_code = context.get_code( "inc_no_save", ast_item)?;     // float cannot perform ++ or --, so no need to handle the float case here
                        inc_code.get_input_registers_from(vec![&const_code], context)?;
                        inc_code.insert_at_beginning(vec![const_code]);
                        inc_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(inc_code);
                        Ok(())
                    }
                    PostOperator::Inc => {
                        let mut load_code = context.get_code("load_address_to_reg", ast_item)?;
                        load_code.get_input_register_from(0, &const_code, 0, context)?;
                        let mut inc_code = context.get_code( "inc", ast_item)?;     // float cannot perform ++ or --, so no need to handle the float case here
                        inc_code.get_input_registers_from(vec![&load_code, &const_code], context)?;
                        inc_code.insert_at_beginning(vec![const_code, load_code]);
                        inc_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(inc_code);
                        Ok(())
                    }
                    PostOperator::Desc if with_reg => {
                        if n.get_reg_name().unwrap() != const_code.get_output_register_names().first().unwrap().to_string() {
                            return Err(CLangError::cannot_emit(& format!("the register name in post inc/dec operator must be the same as the expression's output register")));
                        }

                        let mut dec_code = context.get_code( "desc_no_save", ast_item)?;     // float cannot perform ++ or --, so no need to handle the float case here
                        dec_code.get_input_registers_from(vec![&const_code], context)?;
                        dec_code.insert_at_beginning(vec![const_code]);
                        dec_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(dec_code);
                        Ok(())
                    }
                    PostOperator::Desc => {
                        let mut load_code = context.get_code("load_address_to_reg", ast_item)?;
                        load_code.get_input_register_from(0, &const_code, 0, context)?;
                        let mut dec_code = context.get_code( "desc", ast_item)?;     // float cannot perform ++ or --, so no need to handle the float case here
                        dec_code.get_input_registers_from(vec![&load_code, &const_code], context)?;
                        dec_code.insert_at_beginning(vec![const_code, load_code]);
                        dec_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(dec_code);
                        Ok(())
                    }
                    PostOperator::PostfixExpressionPart(part) => {
                        part.generate_code(context)?;
                        let mut add_template = context.get_code_with_debug("add", &format!("postfix {part}"), ast_item)?;
                        let mut part_code = context.get_code_snippet().clone();
                        part_code.move_register_from(const_code.get_output_register_as_registerset(), [].into(), context)?;
                        add_template.get_input_registers_from(vec![&const_code, &part_code], context)?;
                        add_template.insert_at_beginning(vec![const_code, part_code]);

                        let current_on_left_value = context.is_const_expresion_on_left();
                        let resolved_self_type = resolve_clang_type_for_usage(&self.get_clang_type(), ast_item);
                        let current_member_is_pointer = resolved_self_type.is_pointer();
                        if current_on_left_value && matches!(part, PostfixExpressionPart::Ptr(_, _)) && current_member_is_pointer {
                            let ptr_category = if context.get_code_gen_config().is_64bit_addressing() { SnippetKey::Long }
                                               else { SnippetKey::Int };
                            let mut load_ptr_snippet = context.get_code_with_debug2(&ptr_category, "load_address_to_reg", "dereference intermediate pointer member", part.get_ast_item())?;
                            load_ptr_snippet.set_input_register(add_template.get_output_register_names().first().unwrap());
                            load_ptr_snippet.set_output_register(add_template.get_output_register_names().first().unwrap());
                            add_template.insert_at_end(vec![load_ptr_snippet]);
                        }
                        
                        // if expression is NOT on left, the expr's value is required, so need to add load value code,
                        // if expression is on left, it means it's in a constant expression and the value is not required, 
                        // so no need to add load value code, just return code
                        if !current_on_left_value {
                            // Use the postfix expression result type (e.g. struct member type),
                            // not the base expression type, for final address load opcode selection.
                            let snippet_key = &resolved_self_type.into();
                            let mut load_value_snippet = context.get_code_with_debug2(snippet_key, "load_address_to_reg", "load value after postfix expression", part.get_ast_item())?;
                            load_value_snippet.set_input_register(add_template.get_output_register_names().first().unwrap());
                            load_value_snippet.set_output_register(add_template.get_output_register_names().first().unwrap());
                            add_template.insert_at_end(vec![load_value_snippet]);
                        }

                        context.set_code_snippet(add_template);
                        Ok(())
                    }
                }
            }
            Expression::Prefix(_, n) => {
                let op = n.get_op(); 
                let v = n.get_expr();

                match op {
                    UnaryOperator::Inc if n.get_reg_name().is_some() => {
                        let const_code = context.with_const_expresion_on_left_value(true, |ctx| {
                            v.generate_code(ctx)?;
                            Ok::<CodeSnippet, CLangError>(ctx.get_code_snippet().clone())
                        })?;

                        if n.get_reg_name().unwrap() != const_code.get_output_register_names().first().unwrap().to_string() {
                            return Err(CLangError::cannot_emit(& format!("the register name in post inc/dec operator must be the same as the expression's output register")));
                        }

                        let mut inc_code = context.get_code( "inc_no_save", ast_item)?;  // float cannot perform ++ or --, so no need to handle the float case here
                        inc_code.get_input_registers_from(vec![&const_code], context)?;
                        inc_code.insert_at_beginning(vec![const_code]);
                        inc_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(inc_code);
                        Ok(())
                    }
                    UnaryOperator::Inc=> {
                        let const_code = context.with_const_expresion_on_left_value(true, |ctx| {
                            v.generate_code(ctx)?;
                            Ok::<CodeSnippet, CLangError>(ctx.get_code_snippet().clone())
                        })?;

                        let mut load_value_code = context.get_code("load_address_to_reg", ast_item)?;
                        load_value_code.get_input_register_from(0, &const_code, 0, context)?;
                        let mut inc_code = context.get_code( "inc", ast_item)?;  // float cannot perform ++ or --, so no need to handle the float case here
                        inc_code.get_input_registers_from(vec![&load_value_code, &const_code], context)?;
                        inc_code.insert_at_beginning(vec![const_code, load_value_code]);
                        inc_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(inc_code);
                        Ok(())
                    }
                    UnaryOperator::Dec if n.get_reg_name().is_some() => {
                        let const_code = context.with_const_expresion_on_left_value(true, |ctx| {
                            v.generate_code(ctx)?;
                            Ok::<CodeSnippet, CLangError>(ctx.get_code_snippet().clone())
                        })?;

                        if n.get_reg_name().unwrap() != const_code.get_output_register_names().first().unwrap().to_string() {
                            return Err(CLangError::cannot_emit(& format!("the register name in post inc/dec operator must be the same as the expression's output register")));
                        }

                        let mut dec_code = context.get_code( "desc_no_save", ast_item)?;  // float cannot perform ++ or --, so no need to handle the float case here
                        dec_code.get_input_registers_from(vec![&const_code], context)?;
                        dec_code.insert_at_beginning(vec![const_code]);
                        dec_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(dec_code);
                        Ok(())
                    }
                    UnaryOperator::Dec => {
                        let const_code = context.with_const_expresion_on_left_value(true, |ctx| {
                            v.generate_code(ctx)?;
                            Ok::<CodeSnippet, CLangError>(ctx.get_code_snippet().clone())
                        })?;

                        let mut load_value_code = context.get_code("load_address_to_reg", ast_item)?;
                        load_value_code.get_input_register_from(0, &const_code, 0, context)?;
                        let mut dec_code = context.get_code( "desc", ast_item)?;  // float cannot perform ++ or --, so no need to handle the float case here
                        dec_code.get_input_registers_from(vec![&load_value_code, &const_code], context)?;
                        dec_code.insert_at_beginning(vec![const_code, load_value_code]);
                        dec_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(dec_code);
                        Ok(())
                    }
                    UnaryOperator::BitwiseNot => {
                        v.generate_code(context)?;
                        let const_code = context.get_code_snippet().clone();
                        let mut r = context.get_code("bitwise_not", ast_item)?;  // float cannot perform bitwise not, so no need to handle the float case here
                        r.get_input_registers_from(vec![&const_code], context)?;
                        r.insert_at_beginning(vec![const_code]);
                        r.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(r.clone());
                        Ok(())
                    }
                    UnaryOperator::LogicalNot => {
                        v.generate_code(context)?;
                        let const_code = context.get_code_snippet().clone();
                        let mut r = context.get_code("logic_not", ast_item)?;    // float cannot perform logical not, so no need to handle the float case here
                        r.get_input_registers_from(vec![&const_code], context)?;
                        r.insert_at_beginning(vec![const_code]);
                        r.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(r.clone());
                        Ok(())
                    }
                    UnaryOperator::AddressOf => {
                        if let Some(name) = v.get_atom_id() {
                            if v.get_ast_item().is_reg_used(Some(name)) {
                                return Err(CLangError::cannot_emit(& format!("cannot get address of a register for variable '{name}'")));
                            }
                        }

                        context.with_const_expresion_on_left_value(true, |ctx| v.generate_code(ctx))?;

                        Ok(())
                    }
                    UnaryOperator::Dereference(n) => {
                        assert!(*n > 0);

                        v.generate_code(context)?;
                        let code = context.get_code_snippet().clone();
                        let ty = resolve_clang_type_for_usage(&v.get_clang_type(), ast_item);

                        assert!(ty.is_pointer(), "only pointer can be dereferenced, but got {ty} for {v}");
                        if let Some(dereference_type) = ty.dereference_pointer_type(*n) {
                            if dereference_type.is_pointer() {
                                let category = ty.into();
                                let mut template = context.get_code_with_debug2(&category, "dereference", 
                                    format!("dereference {} a pointer, {self}", *n).as_str(), 
                                                ast_item)?;
                                
                                template.get_input_register_from(0, &code, 0, context)?;

                                // dereference for left side expression in assignment will reduce one level of pointer
                                let level = if context.is_const_expresion_on_left() { *n - 1 } else { *n };

                                template.replicate_output_code(level, context, &category, ast_item);
                                template.insert_at_beginning(vec![code]);
                                context.set_code_snippet(template.clone());
                                Ok(())
                            }
                            else {  // dereference to non-pointer type 
                                match ((*n), context.is_const_expresion_on_left()) {
                                    (1, true) => {
                                        Ok(())
                                    }
                                    (1, false) => {
                                        let category = dereference_type.into();
                                        let mut template = context.get_code_with_debug2(&category, "dereference", 
                                            format!("dereference a pointer, {self}").as_str(), 
                                                        ast_item)?;
                                        template.get_input_register_from(0, &code, 0, context)?;
                                        template.insert_at_beginning(vec![code]);
                                        context.set_code_snippet(template.clone());
                                        Ok(())
                                    }
                                    (m, true) => {
                                        let category = ty.into();
                                        let mut template = context.get_code_with_debug2(&category, "dereference", 
                                            format!("dereference a pointer, {self}").as_str(), 
                                                        ast_item)?;
                                        template.get_input_register_from(0, &code, 0, context)?;

                                        template.replicate_output_code(m-2, context, &category, ast_item);  //on left means it requires address to host the value
                                        template.insert_at_beginning(vec![code.clone()]);
                                        context.set_code_snippet(template.clone());
                                        Ok(())
                                    }
                                    (m, false) => {
                                        let category = ty.into();
                                        let mut template = context.get_code_with_debug2(&category, "dereference", 
                                            format!("dereference a pointer, {self}").as_str(), 
                                                        ast_item)?;
                                        template.get_input_register_from(0, &code, 0, context)?;
                                        template.replicate_output_code(m-1, context, &category, ast_item);
                                        template.insert_at_beginning(vec![code.clone()]);
                                        let mut load_value_code = context.get_code_with_debug2(&dereference_type.into(), "load_address_to_reg", 
                                            format!("load value from dereferenced pointer, {self}").as_str(), 
                                                        ast_item)?;
                                        let reg_name = template.get_output_register_names().first().unwrap();
                                        load_value_code.set_input_register(reg_name);
                                        load_value_code.set_output_register(reg_name);
                                        template.insert_at_end(vec![load_value_code]);
                                        context.set_code_snippet(template.clone());
                                        Ok(())
                                    }
                                }
                            }
                        }
                        else {
                            Err(CLangError::cannot_emit(& format!("cannot dereference pointer type {ty} with level {n}")))
                        }
                    }
                    UnaryOperator::Minus => {
                        // Unary minus is a pure value operation. Generate operand in normal
                        // expression mode so l-values are loaded from memory before negation.
                        v.generate_code(context)?;
                        let const_code = context.get_code_snippet().clone();

                        let mut neg_code = context.get_code( "neg_no_save", ast_item)?;  // float cannot perform -, so no need to handle the float case here
                        neg_code.get_input_registers_from(vec![&const_code], context)?;
                        neg_code.insert_at_beginning(vec![const_code]);
                        neg_code.set_debug_info(&format!("{self}"));
                        context.set_code_snippet(neg_code);
                        Ok(())
                    }
                    UnaryOperator::Plus => {
                        // + to an expression does not do anything, so just return Ok(())
                        Ok(())
                    }
                }
            }
            Expression::Tanry(_, n) => {
                n.generate_code(context)
            }
        }
    }
}

impl AsmCodeGenTrait for TanryExpression {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let ast_item = self.get_ast_item();
        if self.is_tanry() {
            let bool_expr = self.get_expression();
            let (if_branch, else_branch) = self.get_branches().unwrap();

            bool_expr.generate_code(context)?;
            let mut bool_expr_code = context.get_code_snippet().clone();      
            if_branch.generate_code(context)?;
            let mut if_branch_code = context.get_code_snippet().clone();
            else_branch.generate_code(context)?;
            let mut else_branch_code = context.get_code_snippet().clone();

            // implicit convert type for if and else branch
            let if_type = resolve_clang_type_for_usage(&if_branch.get_clang_type(), ast_item);
            let else_type = resolve_clang_type_for_usage(&else_branch.get_clang_type(), ast_item);
            let (_snippet_key, if_conv, else_conv) = 
                context.get_snippet_key_and_conversion_code_for_assignment(
                    Some(&if_type), 
                    Some(&else_type),
                    ast_item
                );

            context.append_conversion_code(&mut if_branch_code, if_conv)?;
            context.append_conversion_code(&mut else_branch_code, else_conv)?;

            else_branch_code.make_output_same_as(&if_branch_code)?;

            let mut code = context.get_code("if_else", ast_item)?;
            let else_branch_code_output_regs = else_branch_code.get_output_register_names().clone();
            bool_expr_code.replace_labelx_in_incs("end_if");   //bool expression jump to end_if
            code.insert_after_label("if_only", bool_expr_code)?;
            code.insert_after_label("if_branch", if_branch_code)?;
            code.insert_after_label("end_if", else_branch_code)?;
            code.set_output_register_names(else_branch_code_output_regs);

            code.append_unique_id_to_label();
            code.set_debug_info(&format!("{self}"));

            context.set_code_snippet(code);
            Ok(())
        }
        else {
            self.get_expression().generate_code(context)
        }
    }
}

impl AsmCodeGenTrait for PostfixExpressionPart {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let is_left = context.is_const_expresion_on_left();

        let expr = self.get_parent_expression().unwrap();

        match self {
            Self::DotId(_, n) |
            Self::Ptr(_, n) => {
                let expr_type = resolve_clang_type_for_usage(&expr.get_clang_type(), self.get_ast_item());
                let clang_type = if self.is_dotid() { expr_type } 
                                else { expr_type.dereference_pointer_type(1)
                                            .ok_or_else(|| CLangError::cannot_emit(
                                    &format!("cannot dereference pointer type '{}' for '->' operator in postfix expression", expr_type)
                                ))? };
                                
                let composite_symbol = clang_type
                    .get_composite_type_symbol_table()
                    .ok_or_else(|| CLangError::cannot_emit(
                        &format!("the id '{n}' from '{self}' in postfix expression must be a struct/union member, but got symbol type '{clang_type}'")
                    ))?;

                let offset = composite_symbol
                    .get_offset(n)
                    .ok_or_else(|| CLangError::cannot_emit(
                        &format!("cannot find struct/union member '{n}' in symbol table for symbol type '{clang_type}'")
                    ))?;

                let mut code = context.get_code_with_debug(
                    "load_const_to_a0",
                    &format!("load offset {offset} for struct/union member '{n}' access"),
                    self.get_ast_item(),
                )?;

                code.replace_imm(&offset.to_string());
                context.set_code_snippet(code);
            }
            Self::Arguments(_, _) |
            Self::Inc(_) |
            Self::Dec(_) => {
                panic!("should be handled by Expression::Postfix level");
            }
            Self::CLangExpression(_, clang_expr) => {
                clang_expr.generate_code(context)?;
            }
        }

        context.set_const_expression_on_left(is_left);

        Ok(())
    }
}

impl AsmCodeGenTrait for AssignmentExpression {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let hash = context.get_instruction_mapping().clone();
        let ast_item = self.get_ast_item();
        match (self.get_left_parts().as_slice(), self.get_constant_expr()) {
            ([], b) => {
                b.generate_code(context)
            }
            ([a], b) => {
                let ty = resolve_clang_type_for_usage(&a.get_constant_expr().get_clang_type(), ast_item);
                let a_id = a.get_constant_expr_id();
                let is_reg_used = a.get_ast_item().is_reg_used(a_id.as_ref());
                let op_name = a.get_operator().get_name();
                let a_type = resolve_clang_type_for_usage(&a.get_constant_expr().get_clang_type(), ast_item);
                let b_type = resolve_clang_type_for_usage(&b.get_clang_type(), ast_item);
                b.generate_code(context)?;
                let mut right_code = context.get_code_snippet().clone();
                a.generate_code(context)?;
                let mut left_code = context.get_code_snippet().clone();

                let (snippet_key, a_conv, b_conv) = 
                    context.get_snippet_key_and_conversion_code_for_assignment(
                        Some(&a_type), 
                        Some(&b_type),
                            ast_item);

                context.append_conversion_code(&mut right_code, b_conv)?;
                context.append_conversion_code(&mut left_code, a_conv)?;

                match ty {
                    CLangTypes::Char |   //char is considred to be number
                    CLangTypes::Float32 |
                    CLangTypes::Float64 |
                    CLangTypes::Int64 |
                    CLangTypes::Int16 |
                    CLangTypes::Byte |
                    CLangTypes::UByte |
                    CLangTypes::UInt16 |
                    CLangTypes::UInt32 |
                    CLangTypes::Int32 | CLangTypes::Enum(_, _) => {
                        if is_reg_used {
                            let template_key = format!("{op_name}NoReg");
                            if hash.contains_key(&template_key) {
                                let template_name = hash.get(&template_key).unwrap();
                                let mut template = context.get_code_with_debug2(& snippet_key, &template_name, &format!("{self}"), ast_item)?;
                                let fixed_regs = left_code.get_output_register_as_registerset() + right_code.get_output_register_as_registerset();
                                template.move_register_from(left_code.get_contaminated_registers() + right_code.get_contaminated_registers(), fixed_regs, context)?;
                                template.get_input_registers_from(vec![&left_code, &right_code], context)?;
                                template.insert_at_beginning(vec![right_code, left_code]);

                                template.append_unique_id_to_label();
                                context.set_code_snippet(template);
                                Ok(())
                            }
                            else {  // this is for simple assign operation
                                let from = right_code.get_output_register_names().first().unwrap().to_string();
                                let to = left_code.get_output_register_names().first().unwrap().to_string();
                                let mut move_code = context.new_move_register_snippet(&ty.into(), ast_item, &from, &to)?;
                                move_code.insert_at_beginning(vec![right_code]);
                                context.set_code_snippet(move_code);
                                Ok(())
                            }
                        }
                        else {
                            // Keep RHS value register stable (it may be fixed by snippet templates),
                            // and move LHS address/temp code away from RHS live registers instead.
                            left_code.move_register_from(right_code.get_contaminated_registers(), [].into(), context)?;
                            let save_int_template_name = "save_value_to_address".to_string();
                            let template_name = hash.get(&op_name).unwrap_or(& save_int_template_name); //if op_name is AddAssign etc, it will return a template name. Otherwise, it is simple assignment.
                            let mut template = context.get_code_with_debug2(&snippet_key, &template_name, &format!("{self}"), ast_item)?;
                            let fixed_regs = left_code.get_output_register_as_registerset() + right_code.get_output_register_as_registerset();
                            template.move_register_from(left_code.get_contaminated_registers() + right_code.get_contaminated_registers(), fixed_regs, context)?;
                            template.get_input_registers_from(vec![&left_code, &right_code], context)?;
                            template.insert_at_beginning(vec![right_code, left_code]);

                            template.append_unique_id_to_label();
                            context.set_code_snippet(template);
                            Ok(())
                        }
                    }
                    CLangTypes::FunctionPointer(_, _, _) |
                    CLangTypes::Pointer(_, _)=> {
                        if is_reg_used {
                            let from = right_code.get_output_register_names().first().unwrap().to_string();
                            let to = left_code.get_output_register_names().first().unwrap().to_string();
                            let mut move_code = context.new_move_register_snippet(&ty.into(), ast_item, &from, &to)?;
                            move_code.set_debug_info(& format!("{} from {self}", move_code.get_debug_info()));
                            move_code.insert_at_beginning(vec![right_code]);
                            context.set_code_snippet(move_code);
                            Ok(())
                        }
                        else {
                            // Keep RHS value register stable (it may be fixed by snippet templates),
                            // and move LHS address/temp code away from RHS live registers instead.
                            left_code.move_register_from(right_code.get_contaminated_registers(), [].into(), context)?;
                            let save_int_template_name = "save_value_to_address".to_string();
                            let template_name = hash.get(&op_name).unwrap_or(& save_int_template_name); //if op_name is AddAssign etc, it will return a template name. Otherwise, it is simple assignment.
                            let category = if context.get_code_gen_config().is_64bit_addressing() { SnippetKey::Long }
                                        else { SnippetKey::Int };
                            let mut template = context.get_code_with_debug2(&category, &template_name, &format!("{self}"), ast_item)?;
                            let fixed_regs = left_code.get_output_register_as_registerset() + right_code.get_output_register_as_registerset();
                            template.move_register_from(left_code.get_contaminated_registers() + right_code.get_contaminated_registers(), fixed_regs, context)?;
                            template.get_input_registers_from(vec![&left_code, &right_code], context)?;  
                            template.insert_at_beginning(vec![right_code, left_code]);

                            template.append_unique_id_to_label();
                            context.set_code_snippet(template);
                            Ok(())
                        }
                    }
                    _ => {
                        todo!("need to process assignment for type '{ty}' for item id {}", self.get_id())
                    }
                }
            }
            ([list@..], b ) => {
                let b_type = resolve_clang_type_for_usage(&b.get_clang_type(), ast_item);
                b.generate_code(context)?;
                let mut right_code = context.get_code_snippet().clone();
                for a in list.iter().rev() {
                    let is_reg_used = a.get_ast_item().is_reg_used(a.get_constant_expr_id().as_ref());
                    let a_type = resolve_clang_type_for_usage(&a.get_constant_expr().get_clang_type(), ast_item);
                    a.generate_code(context)?;
                    let mut left_code = context.get_code_snippet().clone();
                    let op_name = a.get_operator().get_name();
                    right_code.move_register_from(left_code.get_contaminated_registers(), [].into(), context)?;

                    let (snippet_key, a_conv, b_conv) = 
                        context.get_snippet_key_and_conversion_code_for_assignment(
                            Some(&a_type), 
                            Some(&b_type),
                                ast_item);

                    context.append_conversion_code(&mut right_code, b_conv)?;
                    context.append_conversion_code(&mut left_code, a_conv)?;

                    match &a_type {
                            CLangTypes::Int64 |
                            CLangTypes::Int16 |
                            CLangTypes::Byte |
                            CLangTypes::UByte |
                            CLangTypes::UInt16 |
                            CLangTypes::UInt32 |
                            CLangTypes::Int32 => {
                                if is_reg_used {
                                    let template_key = format!("{op_name}NoReg");
                                    if hash.contains_key(&template_key) {  
                                        let template_name = hash.get(&template_key).unwrap();
                                        let mut template = context.get_code_with_debug2(& snippet_key, &template_name, &format!("{self}"), ast_item)?;
                                        let fixed_regs = left_code.get_output_register_as_registerset() + right_code.get_output_register_as_registerset();
                                        template.move_register_from(left_code.get_contaminated_registers() + right_code.get_contaminated_registers(), fixed_regs, context)?;
                                        template.get_input_registers_from(vec![&left_code, &right_code], context)?;
                                        template.insert_at_beginning(vec![right_code, left_code]);

                                        template.append_unique_id_to_label();
                                        right_code = template;
                                    }
                                    else { //this is for simple assign operation
                                        let from = right_code.get_output_register_names().first().unwrap().to_string();
                                        let to = left_code.get_output_register_names().first().unwrap().to_string();
                                        let mut move_code = context.new_move_register_snippet(&a_type.into(), ast_item, &from, &to)?;
                                        move_code.insert_at_beginning(vec![right_code]);
                                        right_code = move_code;
                                    }
                                }
                                else {
                                    let save_int_template_name = "save_value_to_address".to_string();
                                    let template_name = hash.get(&op_name).unwrap_or(& save_int_template_name); //if op_name is AddAssign etc, it will return a template name. Otherwise, it is simple assignment.
                                    let mut template = context.get_code_with_debug2(& snippet_key, &template_name, &format!("{self}"), ast_item)?;
                                    let fixed_regs = left_code.get_output_register_as_registerset() + right_code.get_output_register_as_registerset();
                                    template.move_register_from(left_code.get_contaminated_registers() + right_code.get_contaminated_registers(), fixed_regs, context)?;
                                    template.get_input_registers_from(vec![&left_code, &right_code], context)?;
                                    template.insert_at_beginning(vec![right_code, left_code]);

                                    template.append_unique_id_to_label();
                                    right_code = template;
                                }                             
                            }
                            CLangTypes::FunctionPointer(_, _, _) |
                            CLangTypes::Pointer(_, _)=> {
                                if is_reg_used {
                                    let template_key = format!("{op_name}NoReg");
                                    let save_int_template_name = "save_value_to_address".to_string();
                                    let template_name = hash.get(&template_key).unwrap_or(& save_int_template_name); //if op_name is AddAssign etc, it will return a template name. Otherwise, it is simple assignment.
                                    let category = if context.get_code_gen_config().is_64bit_addressing() { SnippetKey::Long }
                                                else { SnippetKey::Int };
                                    let mut template = context.get_code_with_debug2(&category, &template_name, &format!("{self}"), ast_item)?;
                                    let fixed_regs = right_code.get_output_register_as_registerset();
                                    template.move_register_from(right_code.get_contaminated_registers(), fixed_regs, context)?;
                                    template.get_input_registers_from(vec![&right_code], context)?;  
                                    template.insert_at_beginning(vec![right_code]);

                                    template.append_unique_id_to_label();
                                    right_code = template;
                                }
                                else {
                                    right_code.move_register_from(left_code.get_contaminated_registers(), [].into(), context)?;
                                    let save_int_template_name = "save_value_to_address".to_string();
                                    let template_name = hash.get(&op_name).unwrap_or(& save_int_template_name); //if op_name is AddAssign etc, it will return a template name. Otherwise, it is simple assignment.
                                    let category = if context.get_code_gen_config().is_64bit_addressing() { SnippetKey::Long }
                                                else { SnippetKey::Int };
                                    let mut template = context.get_code_with_debug2(&category, &template_name, &format!("{self}"), ast_item)?;
                                    let fixed_regs = left_code.get_output_register_as_registerset() + right_code.get_output_register_as_registerset();
                                    template.move_register_from(left_code.get_contaminated_registers() + right_code.get_contaminated_registers(), fixed_regs, context)?;
                                    template.get_input_registers_from(vec![&left_code, &right_code], context)?;  
                                    template.insert_at_beginning(vec![right_code, left_code]);

                                    template.append_unique_id_to_label();
                                    right_code = template;
                                }
                            }
                            _ => {
                                todo!()
                            }
                        }
                }

                context.set_code_snippet(right_code);
                Ok(())
            }
        }
    }
}

impl AsmCodeGenTrait for AssignmentExpressionLeftPart {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        context.with_const_expresion_on_left_value(true, |ctx| {
            self.get_constant_expr().generate_code(ctx)
        })?;
        Ok(())
    }
}

fn generate_function_invoke_code(fn_name:&str, ast_item:&ASTItem, args:Vec<&AssignmentExpression>, context:&mut CodeGenContext) -> Result<(), CLangError> { 
    let function_entry = ast_item.get_symbol(fn_name, &FunctionDefinition::get_symbol_table_type());
    let function_return_type = if let Some(entry) = function_entry {
        entry.get_function_return_type()?
    } else {
        // if cannnot find the function entry, try to get the runtime type from the AST item
        // function pointer is one use case
        if let Some(ty) = ast_item.get_runtime_type() {
            ty
        }
        else {
            return Err(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find function entry from name '{fn_name}'")));
        }
    };

    // check if the function is defined in the symbol table, 
    // if not, it may be a function pointer call
    let direct_function_call = function_entry.is_some();

    let mut function_call_code = if function_return_type.is_void() || function_return_type.returns_in_a0() {
        if direct_function_call {
            context.get_code("function_call", ast_item)?
        }
        else {
            context.get_code("indirect_function_call", ast_item)?
        }
    }
    else {
        todo!("cannot process function return type '{}'", function_return_type)
    };

    // prepare arguments for the function call
    let mut parameter_code = CodeSnippet::new("function_call_parameters");
    let argument_number = context.get_code_gen_config().get_parameter_register_number() as usize;
    let mut push64_actions = 0;
    for (i, assignment_expression) in args.into_iter().enumerate() {
        assignment_expression.generate_code(context)?;
        let mut code = context.get_code_snippet().clone();
        let output_reg_name = code.get_output_register_names().first().unwrap().clone();

        if i >= argument_number {
            // use memory to pass parameter
            code.add_asm_inc("push64", vec![Some(&output_reg_name)])?;
            code.set_debug_info(& format!("push parameter #{i} to stack"));
            parameter_code.add_snippet(code);
        }
        else {
            // use argument register a0 - a7 to pass parameters
            let a_reg = format!("a{}", i);
            if output_reg_name != a_reg {
                let move_to_arg_reg = context.new_move_register_snippet(&SnippetKey::default(), ast_item, &output_reg_name, &a_reg)?;
                code.add_snippet(move_to_arg_reg);
            }
            code.add_asm_inc("push64", vec![Some(&a_reg)])?;   // also push parameter onto stack, a0 - a7 can be used to call another function inside the function
            code.set_debug_info(& format!("push parameter #{i} to stack and {a_reg} register"));
            parameter_code.add_snippet(code);
        }
        push64_actions += 1;
    }

    if push64_actions % 2 != 0 {
        // if the number of push64 is odd, add one extra push64 to keep 16-byte stack alignment
        parameter_code.add_asm_inc("push64", vec![Some("x0")])?;
        push64_actions += 1;
    }

    if direct_function_call {
        parameter_code.set_debug_info(& format!("function '{fn_name}' call parameters"));

        function_call_code.replace_labelx_in_incs(fn_name);
        function_call_code.replace_imm(& (push64_actions * context.get_code_gen_config().get_addressing_in_bytes() as usize).to_string());
        function_call_code.insert_after_label("__label__parameters__onto__stack", parameter_code)?;
    }
    else {
        parameter_code.set_debug_info(& format!("function pointer {fn_name} indirect call parameters"));

        let function_target_code = load_indirect_function_target_code(fn_name, ast_item, context)?;
        function_call_code.get_input_register_from(0, &function_target_code, 0, context)?;
        function_call_code.replace_imm(& (push64_actions * context.get_code_gen_config().get_addressing_in_bytes() as usize).to_string());
        function_call_code.insert_after_label_bulk(vec![
            ("__label__parameters__onto__stack".to_string(), parameter_code),
            ("__label__parameters__onto__stack".to_string(), function_target_code),
        ], context)?;
    }

    // If current function's first parameter is in a0 and stack, nested function calls will
    // overwrite a0. Preserve return value first (for non-void), then restore caller's a0
    // from the fp-relative parameter stack slot.
    let first_parameter_name = ast_item.get_current_function()
        .and_then(|function| {
            function.get_parameters()
                .first()
                .and_then(|parameter| parameter.get_var_name())
                .map(|name| name.to_string())
        });

    if let Some(first_param) = first_parameter_name {
        if let Some(parameter_entry) = ast_item.get_parameter_symbol_entry(&first_param) {
            let has_a0_storage = parameter_entry.get_storage_location().iter()
                .any(|location| location.get_register_name() == Some("a0"));

            if has_a0_storage {
                if let Some(stack_offset) = parameter_entry.get_storage_location().iter()
                    .find_map(|location| location.get_stack_offset()) {

                    let mut restore_code = CodeSnippet::new("restore caller a0 parameter after nested call");

                    if function_return_type.returns_in_a0() {
                        let preserved_return_reg = "t2";
                        let move_return_code = context.new_move_register_snippet(&SnippetKey::default(), ast_item, "a0", preserved_return_reg)?;
                        restore_code.add_snippet(move_return_code);
                        function_call_code.set_output_register_names(vec![preserved_return_reg.to_string()]);
                    }

                    function_call_code.insert_after_label("__label__restore__after__call", restore_code)?;

                    let load_instr = if context.get_code_gen_config().is_64bit_addressing() { "ld" } else { "lw" };
                    let restore_asm = format!("{} a0, {}(fp)", load_instr, stack_offset);
                    let mut restore_a0_code = CodeSnippet::from_asm_code_str("restore caller a0", &restore_asm)?;
                    restore_a0_code.set_debug_info(&format!("restore caller first parameter '{first_param}' from {stack_offset}(fp)"));

                    function_call_code.add_snippet(restore_a0_code);
                }
            }
        }
    }

    function_call_code.set_debug_info(& format!("invoke function '{fn_name}'"));
    function_call_code.append_unique_id_to_label();
    context.set_code_snippet(function_call_code);
    Ok(())
}

fn generate_array_access_asm_code2(ast_item: &ASTItem,
    array_type: &CLangTypes,
    base_address_reg: &str,    // register stored the base address of the array, which is calculated by the left part of the array access expression, for example, for structA.arrayB[xx], the base_address_reg is the register stored the address of structA.arrayB, and the offset of the array element will be added to this base address to get the final address
    assignments: &Vec<AssignmentExpression>,
    contaminated_regs: RegisterSet,
    context: &mut CodeGenContext) -> Result<CodeSnippet, CLangError> 
{

    assert!(array_type.is_array());
    let (base_type, _name, dimensions) = array_type.get_array_type().unwrap();
    assert!(assignments.len() == dimensions.get_dimension_count());

    // offset = 0
    let mut code = context.get_code_with_debug("zero_register", "array index zero reg", ast_item)?;

    // get each assignment code and save to offset do
    // offset *= Dimensions[i]
    // offset += assignments[i] (assignment_code.output_reg)
    for (i, assignment) in assignments.iter().enumerate() {
        assignment.generate_code(context)?;
        let assignment_code = context.get_code_snippet().clone();  //output register contains the index value
        let mut index_code = context.get_code_with_debug("offset_from_array_index",
                                    &format!("array dimension[{i}] size = {} in array", dimensions[i]),
                                                ast_item)?;
        index_code.replace_imm(& format!("{}", dimensions[i]));
        let fixed_regs = code.get_output_register_as_registerset() + assignment_code.get_output_register_as_registerset();
        index_code.move_register_from(code.get_contaminated_registers() + assignment_code.get_contaminated_registers(), fixed_regs, context)?;
        index_code.get_input_registers_from(vec![&code, &assignment_code], context)?;
        code.add_snippet(assignment_code);
        code.add_snippet(index_code);
    }

    // offset *= base_type size
    let mut load_address_code = context.get_code_with_debug("mul_const", "multiply data size", ast_item)?;
    load_address_code.replace_imm(base_type.get_type_size_in_bytes(context).unwrap().to_string().as_str());
    load_address_code.move_register_from(code.get_impacted_regs_as_registerset()
                                                + code.get_output_register_as_registerset(),
                                                [].into(),
                                            context)?;
    // Bind mul_const input/output to current offset register explicitly.
    load_address_code.get_input_register_from(0, &code, 0, context)?;
    let mul_input_reg = load_address_code.get_input_register_names().first().unwrap().to_string();
    
    // move temp register used in load_address_code to offset register to avoid register conflict, 
    // because current load_address_code template uses t1 as temp register, 
    // but t1 may be used by the code before load_address_code, 
    // and the value in t1 may be needed after load_address_code, 
    // so we need to move the value in t1 to another register before executing load_address_code, 
    // and move it back after load_address_code. By doing this, 
    // we can ensure that the value in t1 is not overwritten by load_address_code, 
    // and we can also use t1 as temp register in load_address_code without worrying about register 
    // conflict.
    load_address_code.move_temp_register(context)?;
    load_address_code.set_output_register(&mul_input_reg);
    code.set_output_register_names(vec![mul_input_reg]);
    code.add_snippet(load_address_code);

    // final address = stack_offset + offset
    code.move_register_from(contaminated_regs, [].into(), context)?;
    let output_reg = code.get_output_register_names().first().unwrap().clone();

    // the base_address_reg is the register that already contains the base address of the array, so we can directly add the offset to this register 
    // to get the final address of the array element. This can save one register and one add instruction compared to loading the base address to a register and then adding the offset.
    code.add_asm_inc("add", vec![Some(base_address_reg), Some(base_address_reg), Some(&output_reg)])?;
    code.set_output_register_names(vec![base_address_reg.to_string()]);

    if !context.is_const_expresion_on_left() {
        // load value from the final address
        let mut load_value_code = context.get_code_with_debug2(& base_type.into(), "load_address_to_reg", "load array value", ast_item)?;
        load_value_code.move_register_from(code.get_contaminated_registers(), base_address_reg.into(),  context)?;
        load_value_code.set_input_register(base_address_reg);
        code.set_output_register_names(load_value_code.get_output_register_names().to_vec());
        code.add_snippet(load_value_code);
    }

    Ok(code)
}

fn generate_array_access_asm_code(
    ast_item: &ASTItem,
    array_type: &CLangTypes,
    storage_location: i32,
    assignments: &Vec<AssignmentExpression>,
    context: &mut CodeGenContext
) -> Result<CodeSnippet, CLangError> {

    assert!(array_type.is_array());

    let (base_type, _name, dimensions) = array_type.get_array_type().unwrap();
    assert!(assignments.len() == dimensions.get_dimension_count());

    // offset = 0
    let mut code = context.get_code_with_debug("zero_register", "array index zero reg", ast_item)?;

    // get each assignment code and save to offset do
    // offset *= Dimensions[i]
    // offset += assignments[i] (assignment_code.output_reg)
    for (i, assignment) in assignments.iter().enumerate() {
        assignment.generate_code(context)?;
        let assignment_code = context.get_code_snippet().clone();  //output register contains the index value
        let mut index_code = context.get_code_with_debug("offset_from_array_index",
                                    &format!("array dimension[{i}] size = {} in array", dimensions[i]),
                                                ast_item)?;
        index_code.replace_imm(& format!("{}", dimensions[i]));
        let fixed_regs = code.get_output_register_as_registerset() + assignment_code.get_output_register_as_registerset();
        index_code.move_register_from(code.get_contaminated_registers() + assignment_code.get_contaminated_registers(), fixed_regs, context)?;
        index_code.get_input_registers_from(vec![&code, &assignment_code], context)?;
        code.add_snippet(assignment_code);
        code.add_snippet(index_code);
    }

    // offset *= base_type size
    let mut load_address_code = context.get_code_with_debug("mul_const", "multiply data size", ast_item)?;
    load_address_code.replace_imm(base_type.get_type_size_in_bytes(context).unwrap().to_string().as_str());
    load_address_code.move_register_from(code.get_impacted_regs_as_registerset()
                                                + code.get_output_register_as_registerset(),
                                                [].into(),
                                            context)?;
    // Bind mul_const input/output to current offset register explicitly.
    load_address_code.get_input_register_from(0, &code, 0, context)?;
    let mul_input_reg = load_address_code.get_input_register_names().first().unwrap().to_string();
    // Current mul_const template uses t1 as a bridge source register.
    // Force it to the live offset register to avoid stale-register addressing.
    load_address_code.replace_register("t1", &mul_input_reg);
    load_address_code.set_output_register(&mul_input_reg);
    code.set_output_register_names(vec![mul_input_reg]);
    code.add_snippet(load_address_code);

    // final address = stack_offset + offset
    let mut add_stack_offset_code = context.get_code_with_debug("add_stack_fp_and_offset", 
        "add strut/union's offset", ast_item)?;
    add_stack_offset_code.replace_imm(& storage_location.to_string());
    code.append_snippet(add_stack_offset_code, context)?;

    if !context.is_const_expresion_on_left() {
        // load value from the final address
        let load_value_code = context.get_code_with_debug2(& base_type.into(), "load_address_to_reg", "load array value", ast_item)?;
        code.append_snippet_use_other_output(load_value_code, context)?;
    }

    Ok(code)
}

fn generate_dot_reference_code<T>(item:&T, names:&Vec<String>, context: &mut CodeGenContext) -> Result<CodeSnippet, CLangError> 
    where T: AsmCodeGenTrait + ASTItemTrait + GetCLangTypeTrait
{
    // this is for struct member access like structA.structB.xx, we need to calculate the offset of the member and add it to the base address of the struct
    // the base address of the struct is calculated by the left part of the dot reference, which can be another dot reference or a function invoke or a tanry expression. So we need to recursively calculate the code for the left part until we get the base address of the struct, then we can calculate the offset of the member and add it to the base address to get the final address of the member.
    context.with_tanry_dot_following_value(names.len() > 0, 
        |context| item.generate_code(context) )?;
    let mut composite_var_code = context.get_code_snippet().clone();
    composite_var_code.set_debug_info(& format!("calculate address for dot reference seq id = '{}'", item.get_ast_item().get_sequence_id()));
    if composite_var_code.output_registers_all_parameter_registers() {
        // move composite var output register (should be 1) to a temp register
        assert_eq!(composite_var_code.get_output_register_names().len(), 1);
        let from_reg = composite_var_code.get_output_register_names().first().unwrap().to_string();
        let blocked_regs = composite_var_code.get_contaminated_registers().get_register_alias_name();
        let to_temp_reg = context.get_available_register_as_registerset()
            .get_temp_registers()
            .get_register_alias_name()
            .into_iter()
            .find(|reg| !blocked_regs.contains(reg))
            .ok_or_else(|| CLangError::cannot_emit("cannot find free temp register to preserve parameter register output"))?;

        let move_reg_code = context.new_move_register_snippet(&SnippetKey::default(), item.get_ast_item(), &from_reg, &to_temp_reg)?;
        composite_var_code.set_output_register_names(vec![to_temp_reg]);
        composite_var_code.add_snippet(move_reg_code);
    }

    let on_left = context.is_const_expresion_on_left();

    let tanary_expr_type = resolve_clang_type_for_usage(&item.get_clang_type(), item.get_ast_item());

    if tanary_expr_type.is_composite() || tanary_expr_type.is_composite_pointer() || 
            tanary_expr_type.is_enum() || tanary_expr_type.is_enum_pointer() {
        
        let type_name = format!("{}", tanary_expr_type);
        let dereferenced_type = if tanary_expr_type.is_pointer() { tanary_expr_type.dereference_pointer_type(1).unwrap() } 
                                else { tanary_expr_type };
        let mut symbol_table = dereferenced_type.get_composite_type_symbol_table().unwrap();
        let ast_item = item.get_ast_item();

        for (i,member_name) in names.iter().enumerate(){
            let offset = symbol_table.get_offset(member_name)
                                    .ok_or_else(|| CLangError::cannot_emit(
                                        & format!("cannot find member '{member_name}' in type '{type_name}'")))?;
            let mut load_member_offset_code = context.get_code_with_debug("load_const_to_a0", &format!("get member '{member_name}' offset"), ast_item)?;
            load_member_offset_code.replace_imm(& offset.to_string());
            load_member_offset_code.move_register_from(composite_var_code.get_contaminated_registers(), [].into(), context)?;

            let mut inc_address_code = context.get_code_with_debug("add_assign_no_reg", &format!("increase offset {offset}"), ast_item)?;
            let fixed_regs = composite_var_code.get_output_register_as_registerset() + load_member_offset_code.get_output_register_as_registerset();
            inc_address_code.move_register_from(composite_var_code.get_contaminated_registers() + load_member_offset_code.get_contaminated_registers(), fixed_regs, context)?;
            inc_address_code.get_input_registers_from(vec![&composite_var_code, &load_member_offset_code], context)?;
            composite_var_code.add_snippet(load_member_offset_code);
            composite_var_code.add_snippet(inc_address_code);

            // last one does not need to find symbol table
            if i < names.len() - 1 {
                let symbol_table_name = symbol_table.get_table_markdown_name();
                symbol_table = symbol_table.lookup(member_name)
                                .ok_or(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find symbol entry for member '{member_name}' in type '{type_name}' in '{symbol_table_name}'")))?
                                .get_composite_type_symbol_table()
                                .ok_or(CLangError::NoFound((file!().to_string(), line!()), format!("cannot find composite symbol table for member '{member_name}' in type '{type_name}' in '{symbol_table_name}'")))?;            
            }
        }
    }

    // if on left, just return the address of the member
    if on_left {
        Ok(composite_var_code)
    }
    else {
        // load the member value from the address
        let member_type = item.get_ast_item().get_runtime_type().unwrap();
        let mut load_member_value_code = context.get_code_with_debug2(& member_type.into(), 
            "load_address_to_reg", 
            &format!("load member '{}' value", names.last().unwrap()), 
            item.get_ast_item())?;
        let fixed_regs = composite_var_code.get_output_register_as_registerset();
        load_member_value_code.move_register_from(composite_var_code.get_contaminated_registers(), fixed_regs, context)?;
        load_member_value_code.get_input_register_from(0, &composite_var_code, 0, context)?;
        composite_var_code.set_output_register_names(load_member_value_code.get_output_register_names().to_vec());
        composite_var_code.add_snippet(load_member_value_code);
        Ok(composite_var_code)
    }
}

/// the codegen for tanry expression is a bit special, because it can be used in different context, 
/// such as function invoke and struct member access. So we need to handle them separately in codegen.
impl AsmCodeGenTrait for CLangConstExpression_Tanry {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        self.self_check()?;

        let ast_item = self.get_underlying_ast_item();

        match self {
            CLangConstExpression_Tanry::FunctionInvoke(tanry_expr, n) => {
                let function_name = tanry_expr.get_bool_expr_id().unwrap();
                generate_function_invoke_code(function_name, ast_item, n.iter().collect(), context)
            }
            CLangConstExpression_Tanry::DotReference(tanry_expr, l) => {
                let code = generate_dot_reference_code(tanry_expr, l, context)?;
                context.set_code_snippet(code);
                Ok(())
            }
            CLangConstExpression_Tanry::TanaryExpressionOnly(tanry_expression) => {
                tanry_expression.generate_code(context)
            }
            CLangConstExpression_Tanry::Index(tanry_expression, assignments) => {
                // generate code like structA.structB.matrix[0][4]
                let symbol_entries = logic::get_symbol_entries_from_address_dot_expr(tanry_expression);
                assert!(symbol_entries.len() > 0);

                let array_type_entry = symbol_entries.last().unwrap();
                let array_type = array_type_entry.get_data_type().unwrap();
                assert!(array_type.is_array());

                let storage_location = match symbol_entries.as_slice() {
                    [single] => {
                        // this is case like matrix[0][4], the array is directly defined in the function, so it is on stack and we can get the stack offset directly
                        single.get_storage_location()[0].get_stack_offset().unwrap()
                    }
                    [parent, rest@..] => {
                        let base_offset =
                            * parent.get_single_storage_location()
                                .and_then(|x| x.get_stack())
                                .unwrap();
                        base_offset +
                            rest.iter().map(|x| x.get_single_storage_location()
                                                                .and_then(|x| x.get_offset_in_struct_union())
                                                                .unwrap()).sum::<i32>()
                    }
                    _ => unreachable!()
                };

                let code = generate_array_access_asm_code(
                    ast_item,
                    array_type,
                    storage_location,
                    assignments,
                    context,
                )?;
                context.set_code_snippet(code);
                Ok(())
            }
        }
    }
}

impl AsmCodeGenTrait for ConstantExpression {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let ast_item = self.get_ast_item();
        match self  {
            ConstantExpression::Tanry(_, n) => {
                let clang_const_expression_tanry: CLangConstExpression_Tanry = n.into();
                clang_const_expression_tanry.generate_code(context)
            }
            ConstantExpression::Cast(_, n) => {
                let new_type = resolve_clang_type_for_usage(&CLangTypes::from(n.get_type_name()), ast_item);
                let new_type_str = new_type.to_string();
                let expr = n.get_constant_expr();
                let source_type = resolve_clang_type_for_usage(&expr.get_clang_type(), ast_item);
                expr.generate_code(context)?;
                let expr_code = context.get_code_snippet().clone();
                let snippet_key = & source_type.into();
                let template_key = format!("cast_to_{new_type_str}");
                let mut template = context.get_code2(snippet_key, template_key.as_str(), ast_item)?;
                let fixed_regs = expr_code.get_output_register_as_registerset();
                template.move_register_from(expr_code.get_impacted_regs_as_registerset(), fixed_regs, context)?;
                template.get_input_register_from(0, &expr_code, 0, context)?;
                template.insert_at_beginning(vec![expr_code]);
                context.set_code_snippet(template);
                Ok(())
            }
            ConstantExpression::Sizeof(ast_item, n) => {
                match &**n {
                    SizeOfExpression::TypeName(_, type_name) => {
                        // Resolve aliases/composite links before sizeof calculation so codegen always
                        // uses the concrete type size even when source keeps typedef spelling.
                        let raw_type = CLangTypes::from(type_name);
                        let resolved_type = resolve_clang_type_for_usage(&raw_type, ast_item);
                        let type_size = resolved_type
                            .get_type_size_in_bytes(context)
                            .ok_or_else(|| CLangError::cannot_emit(&format!("cannot find clang type size for '{resolved_type}'")))?;
                        let mut code = context.get_code_with_debug("load_const_to_a0", &format!("sizeof {resolved_type}"), ast_item)?;
                        code.replace_imm(& type_size.to_string());
                        context.set_code_snippet(code);
                        Ok(())
                    }
                    SizeOfExpression::CastExpression(_, _n) => {
                        Err(CLangError::cannot_emit("sizeof CastExpression is invalid."))
                    }
                    SizeOfExpression::ConstantExpression(_, _n) => {
                        Err(CLangError::cannot_emit("sizeof ConstantExpression is invalid."))
                    }
                }                
            }
        }
    }
}

fn insert_after_label_recursive(snippet: &mut CodeSnippet, label: &str, code: &CodeSnippet) -> bool {
    if snippet.get_all_labels().iter().any(|x| x == label) {
        return snippet.insert_after_label(label, code.clone()).is_ok();
    }

    for item in snippet.get_code_mut().iter_mut() {
        if let Some(child) = item.get_snippet_mut() {
            if insert_after_label_recursive(child, label, code) {
                return true;
            }
        }
    }

    false
}

impl AsmCodeGenTrait for CLangProgram {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let mut r = CodeSnippet::new("clang program");

        // add data item code
        for item in self.get_declarations().iter() {
            item.generate_code(context)?;
            let code = context.get_code_snippet().clone();
            r.add_data(code.into());
        }

        // add asm code
        for item in self.get_clang_functions().iter() {
            item.generate_code(context)?;
            let code = context.get_code_snippet().clone();
            r.add_code(code.into());
        }

        // add other clang prog items
        for item in self.get_child_clang_progs() {
            item.generate_code(context)?;
            let code = context.get_code_snippet().clone();
            r.add_code(code.into());
        }

        if self.is_root_prog() {
            if let Some(stack_space_size) = context.get_code_gen_config().get_stack_space_size() {
                let stack_code = format!(
                    ".text\n    la sp, stackend\n.data\n    stack: .space {stack_space_size}\n    stackend:\n"
                );
                let stack_def = CodeSnippet::from_asm_code_str("stack_definition", &stack_code)?;
                let (stack_init_code, stack_data) = stack_def.to_code_data_snippet_tuple();
                r.add_data_snippet(stack_data);

                if !insert_after_label_recursive(&mut r, "main", &stack_init_code) {
                    r.insert_at_beginning(vec![stack_init_code]);
                }
            }
        }

        context.set_code_snippet(r);
        Ok(())
    }
    
    fn generate_code_string(&self, context:&mut CodeGenContext) -> Result<String, CLangError> {
        let reference_info = self.get_referenced_external_file_list().iter()
                                                .map(|x| format!("# External file: {}", x))
                                                .collect::<Vec<_>>()
                                                .join("\r\n");
        self.generate_code(context)?;
        let code = context.get_code_snippet().clone();
        let code_str = code.generate_code_string_with_section(true);
        let data_str = code.generate_data_string_with_section(true);
        let r = format!("{reference_info}\r\n\r\n{data_str}\r\n{code_str}");
        Ok(r)
    }
}

impl AsmCodeGenTrait for EnumSpecifier {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let type_name = self.get_type_name();
        let header = format!("# {type_name} Enum\r\n\r\n");
        let md_header = format!("|  {type_name} |\r\n|----|\r\n");

        let (items, footnotes) = self.get_asm_md_table_code(context.get_code_gen_config());
        let table_rows = items.iter()
                                    .map(|x| x.to_asm_code())
                                    .collect::<Vec<_>>()
                                    .join("\r\n");
        let footnote = footnotes.iter()
                                    .map(|x| x.to_asm_code())
                                    .collect::<Vec<_>>()
                                    .join("\r\n");

        let md_data = format!("{header}\r\n{md_header}{table_rows}\r\n\r\n{footnote}\r\n\r\n");
        let type_name = self.get_type_name();
        let r = CodeSnippet::new_with_md(&type_name, &md_data);
        context.set_code_snippet(r);
        Ok(())
    }
}

impl AsmCodeGenTrait for StructUnionSpecifier {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let type_name = format!("Type_{}", self.get_name_id().unwrap());
        let struct_or_union = if self.is_struct() { "Struct" } else { "Union" };

        let header = format!("# {struct_or_union} {type_name}\r\n\r\n");
        let config = context.get_code_gen_config();

        let (mut items, mut footnotes) = if self.is_struct() { 
                self.generate_struct_type_items(&type_name, config)
            }
            else {
                self.generate_union_type_items(&type_name, config)
            };

        // add place holder cell to its first cell and append the footnote to the end
        if !items.is_empty() {
            let first_item = items.first_mut().unwrap();
            first_item.add_tag(crate::languages::clang::struct_union::VAR_NAME_PLACEHOLDER.to_owned());
            footnotes.push(Self::new_footnote(crate::languages::clang::struct_union::VAR_NAME_PLACEHOLDER, "var placeholder"));
        }

        let header_txt = type_name;
        let mut table_header = format!("| {header_txt} |\r\n");
        table_header = format!("{table_header}|----|\r\n");

        let table_rows = items.iter().map(|x| x.to_asm_code()).collect::<Vec<_>>().join("\r\n");
        let footnote = footnotes.iter().map(|x| x.to_asm_code()).collect::<Vec<_>>().join("\r\n");

        let md_data = format!("{header}\r\n{table_header}{table_rows}\r\n\r\n{footnote}\r\n\r\n");
        let r = CodeSnippet::new_with_md("", &md_data);
        context.set_code_snippet(r);
        Ok(())
    }
}

impl AsmCodeGenTrait for AsmStatement {
    fn generate_code(&self, context:&mut CodeGenContext) -> Result<(), CLangError> {
        let asm_code_str = self.get_asm_code();
        let config = context.get_code_gen_config_mut().get_asm_codegen_config_mut();
        if let Ok(asm) = crate::languages::r5asm::assembler::parse_asm(&asm_code_str, config) {
            let non_text_items = asm.get_non_text_section_items().into_iter().map(|x| x.into()).collect::<Vec<CodeSnippetDataItem>>();
            let text_items = asm.get_text_section_items().into_iter().map(|x| x.into()).collect::<Vec<CodeSnippetDataItem>>();
            let mut snippet = CodeSnippet::new_with_debug_info("asm", "asm code");
            snippet.set_data(non_text_items);
            snippet.set_code(text_items);
            context.set_code_snippet(snippet);
            Ok(())
        } else {
            let str = format!("Cannot parse asm code: {}", asm_code_str);
            error_string(str.to_string());
            Err(CLangError::cannot_emit(&str))
        }
    }
}
