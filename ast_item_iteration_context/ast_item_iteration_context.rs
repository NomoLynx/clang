use crate::languages::clang::*;
use super::*;

#[derive(Debug, Clone)]
pub struct ASTItemIterationContext {
    pub current_scope_element : Option<CLangItem>, 
    pub visited_nodes : Vec<CLangItem>,
    error_nodes : Vec<(CLangItem, ASTItemIterationContextError)>,
}

impl ASTItemIterationContext {
    pub fn new(scope_item:CLangItem) -> Self {
        let mut r = Self::default();
        r.current_scope_element = Some(scope_item);
        r
    }

    pub fn new_from_ast_item(ast_item:&ASTItem) -> Self {
        let scope_item = ast_item.get_current_node().cloned().unwrap();
        Self::new(scope_item)
    }

    /// Adds an error node to the context. This is used to track errors that occur during AST item iteration.
    pub fn add_error_node(&mut self, node:CLangItem, error:ASTItemIterationContextError) {
        error_string(format!("Adding error node: {} with error: {}", node.get_name(), error));
        self.error_nodes.push((node, error));
    }

    pub fn get_error_nodes(&self) -> &Vec<(CLangItem, ASTItemIterationContextError)> {
        &self.error_nodes
    }

    pub fn has_error(&self) -> bool {
        !self.error_nodes.is_empty()
    }

    pub fn output_error(&self) -> Result<(), CLangError>{
        if self.has_error() {
            for (err_item, err_msg) in self.get_error_nodes() {
                error_string(format!("Setup symbol table error from {}: {err_msg}", err_item.get_name()));
                error_string(format!("check infer_and_check_type function in TypeConfiguration"));
            }

            Err(CLangError::GeneralError((file!().to_string(), line!()), 
                self.get_error_nodes()
                    .first()
                    .unwrap()
                    .1
                    .to_string()))
        }
        else {
            Ok(())
        }
    }
}

impl From<&mut ASTItem> for ASTItemIterationContext {
    fn from(ast_item:&mut ASTItem) -> ASTItemIterationContext {
        ASTItemIterationContext::new_from_ast_item(ast_item)
    }
}

impl From<&ASTItem> for ASTItemIterationContext {
    fn from(ast_item:&ASTItem) -> ASTItemIterationContext {
        ASTItemIterationContext::new_from_ast_item(ast_item)
    }
}

impl<'a> Default for ASTItemIterationContext {
    fn default() -> Self {
        Self { 
            current_scope_element: None, 
            visited_nodes : Vec::default(), 
            error_nodes : Vec::default() 
        }
    }
}