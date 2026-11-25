use pest::iterators::Pair;

use core_utils::debug::*;
use crate::languages::clang::traits::*;

use super::{ast_item::{ASTItem}, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule, declaration::Declaration, function_definition::FunctionDefinition, statement::Statement, symbol_table::SymbolTable};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompoundStatement {
    compound_statement_items: Vec<CompoundStatementItem>,
    ast_item: ASTItem,
    symbol_table : SymbolTable,
}

impl CompoundStatement {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::compound_statement); //Ensure it is Rule::compound_statement

        let mut compound_statment = CompoundStatement::default();

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::declaration_list => {
                    compound_statment.compound_statement_items.extend(inner_pair
                        .into_inner()
                        .map(|pair| {
                            let decl = Declaration::from_pair(&pair, context);
                            match decl {
                                Ok(d) => d.into(),
                                Err(e) => {
                                    error_string(format!("Error parsing declaration: {:?}", e));
                                    panic!("Error parsing declaration");
                                }
                            }
                        })
                        .collect::<Vec<_>>())
                }
                Rule::statement => compound_statment
                    .compound_statement_items
                    .push(Statement::from_pair(&inner_pair, context)?.into()),

                _ => {
                    let str = format!("cannot find {inner_pair:?} in CompoundStatement processing logic");
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }

        Ok(compound_statment)
    }

    pub fn get_statements(&self) -> Vec<&Statement> {
        self.compound_statement_items
            .iter()
            .filter_map(|x| match x {
                CompoundStatementItem::Statement(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    pub fn get_statements_mut(&mut self) -> Vec<&mut Statement> {
        self.compound_statement_items
            .iter_mut()
            .filter_map(|x| match x {
                CompoundStatementItem::Statement(s) => Some(s),
                _ => None,
            })
            .collect()
    }

    /// get the container function of current compound statement
    pub fn get_container_function(&self) -> Option<&FunctionDefinition> {
        let mut parent_option = self.get_ast_item().get_parent();
        loop {
            if let Some(parent) = parent_option {
                if let Some(parent) = parent.get_function_definition() {
                    return Some(parent);
                } 
                else {
                    parent_option = parent.get_ast_item().get_parent();
                }
            }
            else {
                return None;
            }
        }
    }

    /// get declarations
    pub fn get_declarations(&self) -> Vec<&Declaration> {
        self.compound_statement_items
            .iter()
            .filter_map(|x| match x {
                CompoundStatementItem::Declaration(d) => Some(d),
                _ => None,
            })
            .collect()
    }

    /// get declarations mut
    pub fn get_declarations_mut(&mut self) -> Vec<&mut Declaration> {
        self.compound_statement_items
            .iter_mut()
            .filter_map(|x| match x {
                CompoundStatementItem::Declaration(d) => Some(d),
                _ => None,
            })
            .collect()
    }

    /// check if the self has declaration
    pub fn has_declaration(&self) -> bool {
        self.compound_statement_items
            .iter()
            .any(|x| match x {
                CompoundStatementItem::Declaration(_) => true,
                _ => false,
            })
    }

    /// get compound statement items
    pub fn get_compound_statement_items(&self) -> &Vec<CompoundStatementItem> {
        &self.compound_statement_items
    }

    /// get compound statement items mut
    pub fn get_compound_statement_items_mut(&mut self) -> &mut Vec<CompoundStatementItem> {
        &mut self.compound_statement_items
    }

    /// get symbol table
    pub fn get_symbol_table(&self) -> &SymbolTable {
        &self.symbol_table
    }

    /// get last statement
    pub fn get_last_statement(&self) -> Option<&Statement> {
        self.get_statements().last().map(|v| &**v)
    }

    /// Returns true when a function body represented by this compound statement
    /// should receive an implicit fallback return epilogue.
    pub fn needs_fallback_return_epilogue(&self) -> bool {
        self.get_last_statement()
            .map(|statement| !statement.get_jump().map_or(false, |jump| jump.is_return()))
            .unwrap_or(true)
    }

    pub fn get_symbol_table_mut(&mut self) -> &mut SymbolTable {
        &mut self.symbol_table
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        for item in self.compound_statement_items.iter_mut() {
            match item {
                CompoundStatementItem::Declaration(d) => r.push(d.into()),
                CompoundStatementItem::Statement(s) => r.push(s.into()),
            }
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        for item in self.compound_statement_items.iter_mut() {
            match item {
                CompoundStatementItem::Declaration(d) => d.access_children(context),
                CompoundStatementItem::Statement(s) => s.access_children(context),
            }
        }

        context.access_clang_item(CLangItem::CompoundStatement(CLangItem::new(self)));
    }
}

impl Default for CompoundStatement {
    fn default() -> Self {
        Self { 
            compound_statement_items: vec![], 
            ast_item: ASTItem::default(), 
            symbol_table: SymbolTable::default(), 
        }
    }
}

impl ASTItemTrait for CompoundStatement {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("Compound({})", self.get_ast_item().get_sequence_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompoundStatementItem {
    Declaration(Declaration),
    Statement(Statement),
}

impl From<Declaration> for CompoundStatementItem {
    fn from(item: Declaration) -> Self {
        Self::Declaration(item)
    }
}

impl From<Statement> for CompoundStatementItem {
    fn from(item: Statement) -> Self {
        Self::Statement(item)
    }
} 