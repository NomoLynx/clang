use crate::languages::clang::ASTItem;
use crate::languages::clang::CLangError;
use crate::languages::clang::clang_asm_code_gen::CodeGenConfiguration;
use crate::languages::clang::clang_item::*;
use crate::languages::clang::assignment_expression::*;
use crate::languages::clang::compound_statement::*;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use crate::languages::clang::symbol_table::SymbolTable;
use crate::languages::clang::traits::*;
use crate::languages::clang::statement::*;
use crate::languages::clang::logic::iterator_var_def::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IterationStatement_FOR {
    pub expression_statements: Vec<ExpressionStatement>,
    pub clang_expression: Option<Vec<AssignmentExpression>>,
    pub statement: Box<Statement>,
    pub ast_item: ASTItem,
}

impl IterationStatement_FOR {
    pub fn new(
        expression_statements: Vec<ExpressionStatement>,
        clang_expression: Option<Vec<AssignmentExpression>>,
        statement: Box<Statement>,
    ) -> Self {
        Self {
            expression_statements,
            clang_expression,
            statement,
            ast_item: ASTItem::default(),
        }
    }

    fn get_compund_symbol_table_mut(&mut self) -> Option<&mut SymbolTable> {
        let compound_stmt_option = self.get_embedded_compound_statement_mut();
        if let Some(compound_stmt) = compound_stmt_option {
            Some(compound_stmt.get_symbol_table_mut())
        } else {
            None
        }
    }

    pub fn process_definitions_in_init_statement(&mut self, _context:&CodeGenConfiguration) -> Result<(), CLangError> {
        let result = invoke_iterator_var_def(self.get_all_clangitems());

        // return early if no id definition found
        if result.is_empty() {
            return Ok(());
        }

        let binding = result
                                            .iter()
                                            .map(|(id, ty, expr)| {
                                                (id.get_atom_item(), ty.get_type_specifier(), expr.get_expression_statement())
                                            }).collect::<Vec<_>>();
        let (atom_item_option, type_specifier_option, expr_option) = binding
                                            .first()
                                            .unwrap();
        if let (Some(atom_item), Some(type_specifier), Some(_expr)) = (atom_item_option, type_specifier_option, expr_option) {
            let id_name = atom_item.get_id_value().unwrap();
            let ty = (&*(*type_specifier)).into();
            let scope_name = self.get_name();
            let clang_item = self.get_clang_item();

            let symbol_table = self.get_compund_symbol_table_mut();
            symbol_table
                .unwrap()
                .add_var_definition(id_name, ty, atom_item.get_id(), &scope_name, clang_item);
        }

        Ok(())
    }

    /// Get the embedded compound statement in FOR statement
    pub fn get_embedded_compound_statement_mut(&mut self) -> Option<&mut CompoundStatement> {
        let stmt = self.statement.as_mut();
        stmt.get_compound_statement_mut()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        // child nodes in the order of for elements to instructions in compound statement
        r.append(&mut self.expression_statements.iter_mut().map(|x| x.into()).collect());
        if let Some(n) = self.clang_expression.as_mut() {
            r.append(&mut n.iter_mut().map(|x| x.into()).collect::<Vec<_>>());
        }
        r.push(self.statement.as_mut().into());

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        for x in self.expression_statements.iter_mut() {
            x.access_children(context);
        }

        if let Some(n) = self.clang_expression.as_mut() {
            for x in n.iter_mut() {
                x.access_children(context);
            }
        }

        self.statement.access_children(context);

        context.access_clang_item(CLangItem::IterationStatement_FOR(CLangItem::new(self)));
    }
}

impl ASTItemTrait for IterationStatement_FOR {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("FOR({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.ast_item.get_sequence_id()
    }
}
