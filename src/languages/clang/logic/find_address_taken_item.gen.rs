
// Generated on 2026-09-08 16:24:15
ascent! { 
    struct FindAddressTakenItem;
    // general code and rules
    
include_source!(base_logic::general);

    
relation expr (CLangItem);
relation id (CLangItem);
relation expr_prefix (CLangItem);
relation result(CLangItem);

expr(n) <-- node(n), if n.is_expression();
id(n) <-- node(n), if n.is_atom_item_id();
expr_prefix(n) <-- node(n), if n.is_expression_prefix_addressing();

    
result(id ) <-- 
	expr_prefix(expr_prefix), 	expr(expr), 	id(id),

	unilinage_of(expr_prefix, expr),
	unilinage_of(expr, id);


}