
// Generated on 2026-09-08 18:41:41
ascent! { 
    struct IterationFor;
    // general code and rules
    
include_source!(base_logic::general);

    
relation id (CLangItem);
relation type_specifier_node (CLangItem);
relation expr_node (CLangItem);
relation for_node (CLangItem);
relation assignment_statement (CLangItem);
relation result(CLangItem, CLangItem, CLangItem);

id(n) <-- node(n), if n.is_atom_item_id();
type_specifier_node(n) <-- node(n), if n.is_typespecifier();
expr_node(n) <-- node(n), if n.is_expression_statement();
for_node(n) <-- node(n), if n.is_for_loop();
assignment_statement(n) <-- node(n), if n.is_assignment_expression();

    
result(id , type_specifier_node , expr_node ) <-- 
	assignment_statement(assignment_statement), 	id(id), 	expr_node(expr_node), 	for_node(for_node), 	type_specifier_node(type_specifier_node),

	child_of(expr_node, for_node),
	child_of(type_specifier_node, expr_node),
	child_of(assignment_statement, expr_node),
	ancestor_of(assignment_statement, id);


}