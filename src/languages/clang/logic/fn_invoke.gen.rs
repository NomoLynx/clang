
// Generated on 2026-09-08 04:16:39
ascent! { 
    struct FunctionInvokeLogic;
    // general code and rules
    
include_source!(base_logic::general);

    
relation expr_post (CLangItem);
relation expr (CLangItem);
relation id (CLangItem);
relation postfix_expr_part (CLangItem);
relation result(CLangItem, CLangItem);

expr_post(n) <-- node(n), if n.is_expr_post();
expr(n) <-- node(n), if n.is_expression();
id(n) <-- node(n), if n.is_atom_item_id();
postfix_expr_part(n) <-- node(n), if n.is_arguments();

    
result(id , postfix_expr_part ) <-- 
	id(id), 	expr_post(expr_post), 	postfix_expr_part(postfix_expr_part), 	expr(expr),

	child_of(expr, expr_post),
	unilinage_of(expr, id),
	child_of(postfix_expr_part, expr_post);


}