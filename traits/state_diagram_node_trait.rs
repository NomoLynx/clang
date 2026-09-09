use crate::languages::clang::{ASTItem, CLangItem};

pub (self) fn escape_string(s:&str) -> String {
    s.replace(" ", "_")
     .replace(":", "_")
}

pub trait StateDiagramNodeTrait {
    fn get_diagram_node_name(&self) -> String;
}

impl StateDiagramNodeTrait for ASTItem {
    fn get_diagram_node_name(&self) -> String {
        let node_name = self.get_current_node().unwrap().get_name();
        if let Some(type_name) = self.get_runtime_type() {
            let r = format!("{}_{}", escape_string(&node_name), escape_string(& type_name.to_string()));
            r
        }
        else {
            escape_string(&node_name)
        }
    }
}

impl StateDiagramNodeTrait for CLangItem {
    fn get_diagram_node_name(&self) -> String {
        let ast_item = self.get_ast_item();
        ast_item.get_diagram_node_name()
    }
}