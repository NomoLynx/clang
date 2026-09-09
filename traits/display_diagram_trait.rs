use std::collections::HashSet;

use crate::languages::clang::ast_item::ASTItem;
use crate::languages::clang::clang_program::CLangProgram;
use crate::languages::clang::compound_statement::CompoundStatement;
use crate::languages::clang::function_definition::FunctionDefinition;
use super::*;

fn append_ast_state_diagram(
    node: &ASTItem,
    r: &mut Vec<String>,
    show_semantic_links: bool,
    visited: &mut HashSet<usize>,
) {
    if !visited.insert(node.get_sequence_id()) {
        return;
    }

    let Some(current_node) = node.get_current_node() else {
        return;
    };

    let current_node_name = current_node.get_diagram_node_name();

    for child in node.get_children().iter() {
        let child_name = child.get_diagram_node_name();
        r.push(format!("{current_node_name} --> {child_name}"));
        append_ast_state_diagram(child.get_ast_item(), r, show_semantic_links, visited);
    }

    if show_semantic_links {
        for link in node.get_semantic_linkages().iter() {
            let target = link.get_target();
            let label = link.to_mermaid_label();
            let target_name = target.get_diagram_node_name();
            r.push(format!("{current_node_name} --> {target_name} : {label}"));
        }
    }
}

fn append_scope_link_diagram(node: &ASTItem, r: &mut Vec<String>, visited: &mut HashSet<usize>) {
    if !visited.insert(node.get_sequence_id()) {
        return;
    }

    let Some(current_node) = node.get_current_node() else {
        return;
    };

    if current_node.is_scope() {
        let current_node_name = current_node.get_diagram_node_name();

        if let Some(target_node) = node.get_scope_linker().get_parent() {
            let target_node_name = target_node.get_diagram_node_name();
            r.push(format!("{current_node_name} --> {target_node_name} : parent"));
        }

        node.get_scope_children().iter().for_each(|child_node| {
            let child_node_name = child_node.get_diagram_node_name();
            r.push(format!("{current_node_name} --> {child_node_name} : child"));
        });
    }

    for child in node.get_children().iter() {
        append_scope_link_diagram(child.get_ast_item(), r, visited);
    }
}

/// generate the AST tree in state diagram format (mermaid)
pub trait StateDiagramDisplay {
    fn to_state_diagram(&mut self, show_semantic_links:bool) -> String
        where Self : ASTItemTrait,  
    {
        let mut r = vec!["stateDiagram-v2".to_string()];
        let citem = self.get_ast_item().get_current_node().unwrap().clone();
        let ast_item = citem.get_ast_item();
        let mut visited = HashSet::new();
        append_ast_state_diagram(ast_item, &mut r, show_semantic_links, &mut visited);

        r.join("\r\n")
    }

    fn to_scope_link_diagram(&self) -> String 
        where Self : ASTItemTrait,
    {
        let mut r = vec!["stateDiagram-v2".to_string()];

        let citem = self.get_ast_item().get_current_node().unwrap().clone();
        let ast_item = citem.get_ast_item();
        let mut visited = HashSet::new();
        append_scope_link_diagram(ast_item, &mut r, &mut visited);

        r.join("\r\n")
    }
}

impl StateDiagramDisplay for CLangProgram { }

impl StateDiagramDisplay for FunctionDefinition { }

impl StateDiagramDisplay for CompoundStatement { }