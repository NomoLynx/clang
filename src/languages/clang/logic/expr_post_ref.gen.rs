
// Generated on 2026-09-07 18:15:02
ascent! { 
    struct ExprPostfixRefLogic;
    // general code and rules
    
include_source!(base_logic::general);

    
relation expr_post2 (CLangItem);
relation expr (CLangItem);
relation postfix_expr_part (CLangItem);
relation expr_post (CLangItem);
relation atom (CLangItem);
relation id (CLangItem);
relation result(CLangItem, CLangItem);

expr_post2(n) <-- node(n), if n.is_expr_post();
expr(n) <-- node(n), if n.is_expression();
postfix_expr_part(n) <-- node(n), if n.is_postfix_expression_part();
expr_post(n) <-- node(n), if n.is_expr_post();
atom(n) <-- node(n), if n.is_atom();
id(n) <-- node(n), if n.is_atom_item_id();

    
result(expr_post2 , postfix_expr_part ) <-- 
	expr_post2(expr_post2), 	expr_post(expr_post), 	postfix_expr_part(postfix_expr_part), 	expr(expr),

	child_of(expr, expr_post),
	child_of(postfix_expr_part, expr_post),
	child_of(expr_post2, expr);

result(expr , postfix_expr_part ) <-- 
	expr_post(expr_post), 	postfix_expr_part(postfix_expr_part), 	expr(expr), 	id(id), 	atom(atom),

	child_of(expr, expr_post),
	child_of(postfix_expr_part, expr_post),
	child_of(atom, expr),
	child_of(id, atom);


}