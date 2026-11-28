use pest::iterators::Pair;

use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::symbol_table::*;
use core_utils::debug::*;
use crate::languages::clang::type_specifier::TypeSpecifier;
use crate::languages::clang::traits::*;
use crate::languages::clang::declaration::*;
use super::{ast_item::{ASTItem}, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule, compound_statement::CompoundStatement, declaration::Declaration, parameter_declaration::ParameterDeclaration, symbol_table::{SymbolLocation, SymbolTable}};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionDefinition {
    ///function return type
    return_type: CLangTypes,

    ///contains function name and parameters
    declarator: Declarator,

    declarations: Vec<Declaration>,

    /// function body
    compound_statement: CompoundStatement, //from the rule definition, it contains the single rule:compound_statement

    ast_item : ASTItem,

    /// sybmol table for the function
    symbol_table : SymbolTable,

    /// stack frame pointer, change this variable when create local variables or parameters
    stack_frame_pointer : i32,
}

impl FunctionDefinition {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::function_definition); //Ensure it is Rule::function_definition

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner = pair
            .to_owned()
            .into_inner()
            .map(|x| (x.as_rule(), x))
            .collect::<Vec<_>>();

        let symbol_table = SymbolTable::default();

        match inner.as_slice() {
            //all  inner rules do exist
            [(Rule::declaration_specifiers, declare_specifiers), (Rule::declarator, declare), (Rule::declaration_list, delcarations_rules), (Rule::compound_statement, compound_statement)] =>
            {
                let return_type = declare_specifiers
                    .to_owned()
                    .into_inner()
                    .map(|pair| TypeSpecifier::from_pair(&pair, context).unwrap())
                    .collect::<Vec<_>>()
                    .into();

                let declarator = Declarator::from_pair(&declare, context)?;

                let declarations = delcarations_rules
                    .to_owned()
                    .into_inner()
                    .map(|pair| Declaration::from_pair(&pair, context).unwrap())
                    .collect();

                let compound_statement = CompoundStatement::from_pair(&compound_statement, context)?;

                Ok(FunctionDefinition {
                    return_type,
                    declarator,
                    declarations,
                    compound_statement,
                    ast_item,
                    symbol_table,
                    stack_frame_pointer : 0,
                })
            }

            //all inner rules except for rule:declaration_list exist
            [(Rule::declaration_specifiers, declare_specifiers), (Rule::declarator, declare), (Rule::compound_statement, compound_statement)] =>
            {
                let return_type = declare_specifiers
                    .to_owned()
                    .into_inner()
                    .map(|pair| TypeSpecifier::from_pair(&pair, context).unwrap())
                    .collect::<Vec<_>>()
                    .into();

                let declarator = Declarator::from_pair(&declare, context)?;
                let declarations = vec![];
                let compound_statement = CompoundStatement::from_pair(&compound_statement, context)?;

                Ok(FunctionDefinition {
                    return_type,
                    declarator,
                    declarations,
                    compound_statement,
                    ast_item,
                    symbol_table,
                    stack_frame_pointer : 0,
                })
            }

            //all  inner rules except for rule:declaration_specifiers exist
            [(Rule::declarator, declare), (Rule::declaration_list, delcarations_rules), (Rule::compound_statement, compound_statement)] =>
            {
                let return_type = vec![].into();

                let declarator = Declarator::from_pair(&declare, context)?;

                let declarations = delcarations_rules
                    .to_owned()
                    .into_inner()
                    .map(|pair| Declaration::from_pair(&pair, context).unwrap())
                    .collect::<Vec<_>>();

                let compound_statement = CompoundStatement::from_pair(&compound_statement, context)?;

                Ok(FunctionDefinition {
                    return_type,
                    declarator,
                    declarations,
                    compound_statement,
                    ast_item,
                    symbol_table,
                    stack_frame_pointer : 0,
                })
            }

            //only rule:declarator & rule:compound_statement exist
            [(Rule::declarator, declare), (Rule::compound_statement, compound_statement)] => {
                let return_type = vec![].into();

                let declarator = Declarator::from_pair(&declare, context)?;

                let declarations = vec![];

                let compound_statement = CompoundStatement::from_pair(&compound_statement, context)?;

                Ok(FunctionDefinition {
                    return_type,
                    declarator,
                    declarations,
                    compound_statement,
                    ast_item,
                    symbol_table,
                    stack_frame_pointer : 0,
                })
            }
            [(Rule::type_alias, p0), (Rule::declarator, p1), (Rule::compound_statement, p2)] => {
                let return_type = CLangTypes::TypeAlias(p0.as_str().to_string(), None);
                let declarator = Declarator::from_pair(&p1, context)?;
                let declarations = vec![];
                let compound_statement = CompoundStatement::from_pair(&p2, context)?;

                Ok(FunctionDefinition {
                    return_type,
                    declarator,
                    declarations,
                    compound_statement,
                    ast_item,
                    symbol_table,
                    stack_frame_pointer : 0,
                })
            }
            _ => {
                for (i, (inner_rule, inner_pair)) in inner.iter().enumerate() {
                    error_string(format!(
                        "inner rule {i} is '{inner_rule:?}', pair is '{inner_pair:?}'"
                    ));
                }
                let str = format!("cannot find {inner:?} in jump statement processing logic");
                Err(CLangError::NoFound((file!().to_string(), line!()), str))
            }
        }
    }

    /// return const "SymbolType::Function"
    pub fn get_symbol_table_type() -> SymbolType {
        SymbolType::Function
    }

    /// return const "SymbolType::ReturnType"
    pub fn get_symbol_table_attribute_key_return_type() -> SymbolType {
        SymbolType::ReturnType
    }

    pub fn get_symbol_table_attribute_key_parameters() -> SymbolType {
        SymbolType::Parameters
    }

    pub fn get_function_name(&self) -> String {
        let r = self.declarator.get_id_name();
        r
    }

    pub fn is_main_function(&self) -> bool {
        let name = self.get_function_name();
        &name == "main"
    }

    pub fn get_compound_statement(&self) -> &CompoundStatement {
        &self.compound_statement
    }
    
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    pub fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    pub fn get_return_type_name(&self) -> String {
        let r = self.return_type.to_string();
        r
    }

    pub fn get_return_type(&self) -> &CLangTypes {
        &self.return_type
    }

    pub fn get_return_type_mut(&mut self) -> &mut CLangTypes {
        &mut self.return_type
    }

    pub fn get_parameters(&self) -> Vec<&ParameterDeclaration> {
        self.declarator.get_direct_declarator()
            .get_direct_declarator_parts().iter()
            .map(|x| x.get_parameters().unwrap().get_parameters())
            .flatten()
            .collect::<Vec<_>>()
    }

    /// set the storage location of the parameters in the symbol table
    pub fn update_symbol_offset(&mut self, reg_number:u32, addressing_bytes:u32) -> Result<(), CLangError> {
        let mut stack_frame_pointer = self.get_stack_frame_pointer();
        let parameter = self.get_parameters().iter()
                                        .filter_map(|x| x.get_var_name())
                                        .collect::<Vec<_>>();
        let symbol_table = self.get_symbol_table_mut();        

        // add parameters to symbol table
        for (i, var_name) in parameter.iter().enumerate() {
            let parameter_reg_nubmer = reg_number as usize;
            stack_frame_pointer = aligned_stack_offset(stack_frame_pointer, addressing_bytes as usize, addressing_bytes as usize)?;
            if i >= parameter_reg_nubmer {
                symbol_table.set_parameter_storage_location(&var_name, SymbolLocation::Stack(stack_frame_pointer))?;
            }
            else {
                symbol_table.set_parameter_storage_location(&var_name, SymbolLocation::RegisterAndStack(format!("a{i}"), stack_frame_pointer))?;
            }
        }

        self.set_stack_frame_pointer(stack_frame_pointer);
        Ok(())
    }

    pub fn get_stack_frame_pointer(&self) -> i32 {
        self.stack_frame_pointer
    }

    pub fn set_stack_frame_pointer(&mut self, value:i32) {
        self.stack_frame_pointer = value;
    }

    pub fn checked_frame_size(&self, context: &CodeGenContext) -> Result<usize, CLangError> {
        fn collect_table(
            table: &SymbolTable,
            context: &CodeGenContext,
            bottom: i32,
            slots: &mut Vec<(i64, i64, String)>,
        ) -> Result<(), CLangError> {
            for entry in table.get_all_entries() {
                if !matches!(entry.get_symbol_type(), SymbolType::Var | SymbolType::Declaration) {
                    continue;
                }
                for location in entry.get_storage_location() {
                    let offset = match location {
                        SymbolLocation::Stack(offset) | SymbolLocation::RegisterAndStack(_, offset) => *offset,
                        _ => continue,
                    };
                    let (size, alignment) = if entry.get_symbol_type() == &SymbolType::Declaration {
                        let bytes = context.get_code_gen_config().get_addressing_in_bytes() as usize;
                        (bytes, bytes)
                    } else {
                        let ty = entry.get_data_type()
                            .ok_or_else(|| CLangError::cannot_emit("Stack slot has no type"))?;
                        (
                            ty.get_type_size_in_bytes(context)
                                .ok_or_else(|| CLangError::cannot_emit("Stack slot size is unknown"))?,
                            ty.get_stack_alignment_in_bytes(context)
                                .ok_or_else(|| CLangError::cannot_emit("Stack slot alignment is unknown"))?,
                        )
                    };
                    let start = i64::from(offset);
                    let size = i64::try_from(size)
                        .map_err(|_| CLangError::cannot_emit("Stack slot size overflow"))?;
                    let alignment = i64::try_from(alignment)
                        .map_err(|_| CLangError::cannot_emit("Stack slot alignment overflow"))?;
                    let end = start.checked_add(size)
                        .ok_or_else(|| CLangError::cannot_emit("Stack slot end overflow"))?;
                    if size == 0 || alignment <= 0 || start < i64::from(bottom)
                        || end > 0 || start.rem_euclid(alignment) != 0
                    {
                        return Err(CLangError::cannot_emit(&format!(
                            "Invalid frame slot {}: [{start}, {end}), alignment {alignment}, bottom {bottom}",
                            entry.get_name()
                        )));
                    }
                    slots.push((start, end, entry.get_name().to_string()));
                }
            }
            for child in table.get_sub_tables() {
                collect_table(child, context, bottom, slots)?;
            }
            Ok(())
        }

        fn collect_children(
            ast: &ASTItem,
            context: &CodeGenContext,
            bottom: i32,
            slots: &mut Vec<(i64, i64, String)>,
        ) -> Result<(), CLangError> {
            for child in ast.get_scope_children() {
                if let Some(scope) = child.get_scope_trait() {
                    collect_table(scope.get_symbol_table(), context, bottom, slots)?;
                }
                collect_children(child.get_ast_item(), context, bottom, slots)?;
            }
            Ok(())
        }

        let bottom = self.get_stack_frame_pointer();
        if bottom > 0 {
            return Err(CLangError::cannot_emit("Function frame bottom must not exceed fp"));
        }
        let mut slots = Vec::new();
        collect_table(self.get_symbol_table(), context, bottom, &mut slots)?;
        collect_children(self.get_ast_item(), context, bottom, &mut slots)?;
        slots.sort_by_key(|slot| slot.0);
        for pair in slots.windows(2) {
            if pair[0].1 > pair[1].0 {
                return Err(CLangError::cannot_emit(&format!(
                    "Overlapping frame slots {} and {}", pair[0].2, pair[1].2
                )));
            }
        }
        let bytes = usize::try_from(-i64::from(bottom))
            .map_err(|_| CLangError::cannot_emit("Function frame size overflow"))?;
        let aligned = bytes.checked_add(15)
            .map(|bytes| bytes & !15)
            .ok_or_else(|| CLangError::cannot_emit("Aligned function frame size overflow"))?;
        if aligned > 2048 {
            return Err(CLangError::cannot_emit(
                "Function frame exceeds 2048 bytes; scalar FP-relative templates require signed 12-bit offsets"
            ));
        }
        i32::try_from(aligned)
            .map_err(|_| CLangError::cannot_emit("Function frame exceeds the supported offset range"))?;
        Ok(aligned)
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        r.push((&mut self.declarator).into());

        for x in self.declarations.iter_mut() {
            r.push(x.into());
        }

        r.push((&mut self.compound_statement).into());

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 

        for x in self.declarations.iter_mut() {
            x.access_children(context);
        }

        self.declarator.access_children(context);
        self.compound_statement.access_children(context);

        context.access_clang_item(CLangItem::FunctionDefinition(CLangItem::new(self)));
    }
}

impl ASTItemTrait for FunctionDefinition {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("Fn_{}({})", self.get_function_name(), self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}
