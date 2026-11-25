use ascent::ascent;

use crate::languages::clang::CLangItem;

ascent! {
    pub struct DeclarationFacts;

    // facts
    relation node(CLangItem);
    relation parameter(CLangItem);
    relation single_child_of(CLangItem, CLangItem);
    relation unilinage_of(CLangItem, CLangItem);

    // rules
    single_child_of(child, parent) <-- 
        node(parent), node(child),
        if parent.is_parent_of(child) && parent.get_children().len() == 1;
    
    unilinage_of(parent,child) <-- 
        single_child_of(child, parent);
    unilinage_of(parent,grandchild) <--
        single_child_of(child, parent),
        unilinage_of(child, grandchild);

    parameter(x) <-- node(root), if root.is_declaration(),
        unilinage_of(root, x), if x.is_parameter_type();
}