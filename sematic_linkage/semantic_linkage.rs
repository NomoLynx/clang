use crate::languages::clang::CLangItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SemanticLinkage {
    Definition(CLangItem), // Link to a declaration
    DefinitionUsage(CLangItem), // Link to a usage of a declaration
}

impl SemanticLinkage {
    pub fn as_definition(&self) -> Option<&CLangItem> {
        match self {
            SemanticLinkage::Definition(item) => Some(item),
            _ => None,
        }
    }

    pub fn to_mermaid_label(&self) -> String {
        match self {
            SemanticLinkage::Definition(_) => format!("Def"),
            SemanticLinkage::DefinitionUsage(_) => format!("DefUse"),
        }
    }

    pub fn get_target(&self) -> &CLangItem {
        match self {
            Self::Definition(item) | 
            Self::DefinitionUsage(item) => item,
        }
    }
}