use crate::languages::clang::ASTItem;
use crate::languages::clang::clang_item::CLangItem;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;

use crate::languages::clang::clang_pest::CLangExpression;
use crate::languages::clang::traits::*;
use crate::languages::clang::statement::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IterationStatement_DO {
    pub statement: Box<Statement>,
    pub expression: CLangExpression,
    pub ast_item: ASTItem,
}

impl IterationStatement_DO {
    pub fn new(statement: Box<Statement>, expression: CLangExpression) -> Self {
        Self { statement, expression, ast_item: ASTItem::default() }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![self.statement.as_mut().into()];
        r.append(&mut self.expression.iter_mut().map(|x| x.into()).collect());
        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.statement.access_children(context);
        for x in self.expression.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(self.into());
    }
}

impl ASTItemTrait for IterationStatement_DO {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("DO({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.ast_item.get_sequence_id()
    }
}