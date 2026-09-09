
use core_utils::debug::*;
use crate::languages::clang::ASTItem;
use crate::languages::clang::CLangError;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::function_definition::FunctionDefinition;
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::clang_pest::*;
use crate::languages::clang::postfix_expression_part::*;
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::{assignment_expression::AssignmentExpression, constant_expression::ConstantExpression_Tanry};

/// type used to represent different kinds of constant expressions tanry expressions 
#[allow(non_camel_case_types)]
#[derive(Debug, Clone)]
pub enum CLangConstExpression_Tanry {
    FunctionInvoke(TanryExpression, Vec<AssignmentExpression>),
    DotReference(TanryExpression, Vec<Id>),
    Index(TanryExpression, CLangExpression),
    TanaryExpressionOnly(TanryExpression),
}

impl CLangConstExpression_Tanry {
    
    /// get the clang type of the constant expression tanry
    pub fn get_clang_type(&self) -> CLangTypes {
        match self {
            CLangConstExpression_Tanry::FunctionInvoke(tanry_expr, _) => {
                tanry_expr.get_clang_type()
            }
            CLangConstExpression_Tanry::DotReference(tanry_expr, l) => {
                let ty = tanry_expr.get_composite_definition().unwrap();
                let mut symbol_table = ty.get_composite_type_symbol_table().unwrap();
                let type_definition = symbol_table.get_container().unwrap(); // this is the struct or union definition in AST

                for (index, member_name) in l.iter().enumerate() {
                    let ty = symbol_table.get_member_type(&member_name);
                    if ty.is_composite() {
                        let composite_type_name = ty.get_composite_type_name().unwrap();
                        if let Some(sub_type_symbol_entry) = type_definition.get_ast_item().get_symbol_recursive_type_definition(&composite_type_name) {
                            
                            if let Some (new_symbol_table) = sub_type_symbol_entry.get_composite_type_symbol_table() {
                                symbol_table = new_symbol_table;
                            }
                            else {
                                panic!("Cannot find symbol table for composite type '{}'", composite_type_name);
                            }
                        }
                        else {
                            panic!("Cannot find composite type definition for '{}'", composite_type_name);
                        }
                    }
                    else {
                        if index == l.len() - 1 {
                            return ty;
                        }
                        else {
                            panic!("Trying to access member '{}' of non-composite type '{:?}'", member_name, ty);
                        }
                    }
                }

                panic!("DotReference did not return type for Id list: {:?}", l);
            }
            CLangConstExpression_Tanry::TanaryExpressionOnly(tanry_expr) => {
                tanry_expr.get_clang_type()
            }
            CLangConstExpression_Tanry::Index(v, indexes) => {
                let array_owner_type = if let Some(var_name) = v.get_bool_expr_id() {
                    v.get_ast_item()
                        .get_symbol_recursive_all_types(&var_name)
                        .and_then(|symbol_entry| symbol_entry.get_data_type().cloned())
                        .unwrap_or_else(|| v.get_clang_type())
                }
                else {
                    v.get_clang_type()
                };

                if let Some((array_type, _name, dimension)) = array_owner_type.get_array_type() {
                    if indexes.len() == dimension.get_dimension_count() {
                        return array_type.clone();
                    }
                    else {
                        let err_msg = format!("Array indexing dimension mismatch for expression '{v}' with dimensions {dimension}");
                        error_string(err_msg.to_string());
                        panic!("{}", err_msg);
                    }
                }
                else {
                    panic!("Index case for ConstantExpression_Tanry expects an array expression, but got '{:?}' from '{v}'", array_owner_type);
                }
            }
        }
    }

    pub fn get_function_name(&self) -> Option<&String> {
        match self {
            CLangConstExpression_Tanry::FunctionInvoke(tanry_expr, _) => {
                tanry_expr.get_bool_expr_id()
            }
            _ => None,
        }
    }

    /// self check to make sure the function invoke expression is valid, such as the function definition exist and the number of arguments match
    pub fn self_check(&self) -> Result<(), CLangError> {
        match self {
            Self::FunctionInvoke(m, n) => {
                if let Some(function_name) = self.get_function_name() {
                    let function_entry = m.get_ast_item().get_symbol(function_name, &FunctionDefinition::get_symbol_table_type());
                    if let Some(function) = function_entry {
                        let function_args= function.get_attributes().len() - 1;
                        if  function_args != n.len() {
                            let err_msg = format!("function {function_name} has {} arguments, but {} arguments are passed", function_args,  n.len());
                            error_string(err_msg.to_string());
                            Err(CLangError::cannot_emit(& err_msg))
                        }
                        else {
                            Ok(())
                        }
                    }
                    else {
                        Err(CLangError::cannot_emit(& format!("cannot find function {function_name}")))
                    }
                }
                else {
                    Err(CLangError::GeneralError((file!().to_string(), line!()), format!("FunctionName in FunctionInvoke of CLangConstExpression_Tanry is NULL when searching for expr id. the value is = {m:?}")))
                }
            }
            _ => {
                Ok(())
            }
        }
    }

    /// get the underlying ASTItem of the constant expression tanry, 
    /// it is the ASTItem of the tanry expression part of the constant expression tanry
    pub fn get_underlying_ast_item(&self) -> &ASTItem {
        match self {
            Self::FunctionInvoke(tanry_expr, _) |
            Self::DotReference(tanry_expr, _) |
            Self::TanaryExpressionOnly(tanry_expr) => tanry_expr.get_ast_item(),
            Self::Index(tanry_expr, _) => tanry_expr.get_ast_item(),
        }
    }
}

impl From<&ConstantExpression_Tanry> for CLangConstExpression_Tanry {
    fn from(value: &ConstantExpression_Tanry) -> Self {
        let tanry_expression = value.get_tanry_expression().clone();

        if value.get_postfix_expression_parts().len() <= 1 {
            let postfix_parts = value.get_postfix_expression_parts();
            let postfix_part0 = postfix_parts.first();
            match postfix_part0 {
                Some(PostfixExpressionPart::Arguments(_, _) ) => {
                    CLangConstExpression_Tanry::FunctionInvoke(tanry_expression, value.get_arguments())
                }
                Some(PostfixExpressionPart::DotId(_, _)) => {
                    CLangConstExpression_Tanry::DotReference(tanry_expression, value.get_id_list())
                }
                Some(PostfixExpressionPart::CLangExpression(_, clang_expression)) => {
                    CLangConstExpression_Tanry::Index(tanry_expression, clang_expression.clone())
                }
                None => {
                    CLangConstExpression_Tanry::TanaryExpressionOnly(tanry_expression)
                }
                _ => {
                    error_string(format!("need process {postfix_part0:?}"));
                    panic!("Unsupported postfix expression part in ConstantExpression_Tanry : {:?}", value);
                }
            }
        }
        else {
            let mut expressions = vec![];
            for postfix_part in value.get_postfix_expression_parts().into_iter() {
                expressions.append(&mut postfix_part.get_clang_expression().cloned().unwrap());
            }

            CLangConstExpression_Tanry::Index(tanry_expression, expressions)
        }
    }
}

impl From<ConstantExpression_Tanry> for CLangConstExpression_Tanry {
    fn from(value: ConstantExpression_Tanry) -> Self {
        (&value).into()
    }
}