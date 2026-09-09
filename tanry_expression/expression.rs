use core::panic;
use std::collections::HashSet;
use std::cell::RefCell;
use std::fmt::Display;

use core_utils::debug::*;
use pest::pratt_parser::{Assoc, Op, PrattParser};
use rust_macro::*;
use pest::iterators::Pair;

use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::logic::get_postfix_ref;
use crate::languages::clang::post_operator::*;
use crate::languages::clang::clang_pest::{Rule, UnaryOperator};
use crate::languages::clang::postfix_expression_part::PostfixExpressionPart;
use crate::languages::clang::tanry_expression::expression_postfix::Expression_Postfix;
use crate::languages::clang::tanry_expression::expression_expression::Expression_Expression;
use crate::languages::clang::tanry_expression::expression_prefix::Expression_Prefix;
use crate::languages::clang::traits::ASTItemTrait;
use crate::languages::clang::{ASTItem, CLangError, CLangItem};
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::declaration::Declaration;
use crate::languages::clang::symbol_table::SymbolEntry;
use crate::languages::clang::tanry_expression::*;
use crate::languages::operators::LangOperators;

#[derive(Debug, Clone, PartialEq, Eq, EnumAccessors)]
pub enum Expression {
    Atom(ASTItem, Atom),
    Expression(ASTItem, Expression_Expression),
    Tanry(ASTItem, Box<TanryExpression>),
    Prefix(ASTItem, Expression_Prefix),
    Postfix(ASTItem, Expression_Postfix),
}

impl Expression {

    /// create a new pratt parser for expression which contains order of the operators
    fn new_pratt() -> PrattParser<Rule> {
        let pratt =
            PrattParser::new()
                .op(Op::infix(Rule::logic_or, Assoc::Left))
                .op(Op::infix(Rule::logic_and, Assoc::Left))
                .op(Op::infix(Rule::logic_equal, Assoc::Left) | Op::infix(Rule::logic_not_equal, Assoc::Left))
                .op(Op::infix(Rule::operator_equal_or_greater, Assoc::Left) | Op::infix(Rule::operator_equal_or_less, Assoc::Left) | Op::infix(Rule::operator_greater, Assoc::Left) | Op::infix(Rule::operator_less, Assoc::Left))
                .op(Op::infix(Rule::bit_and, Assoc::Left) | Op::infix(Rule::bit_exclusive_or, Assoc::Left) | Op::infix(Rule::bit_or, Assoc::Left))
                .op(Op::infix(Rule::bit_left_move, Assoc::Left) | Op::infix(Rule::bit_right_move, Assoc::Left))
                .op(Op::infix(Rule::addition, Assoc::Left) | Op::infix(Rule::subtraction, Assoc::Left) | Op::infix(Rule::rem, Assoc::Left))
                .op(Op::infix(Rule::multiplication, Assoc::Left) | Op::infix(Rule::division, Assoc::Left))
                .op(Op::infix(Rule::pow, Assoc::Right)) 
                .op(Op::infix(Rule::PTR_OP, Assoc::Left) | Op::infix(Rule::dot, Assoc::Left))               
                .op(Op::prefix(Rule::unary_op))
                .op(Op::postfix(Rule::post_op))
                .op(Op::postfix(Rule::postfix_expression_part));

        pratt
    }

    fn pratt_map_primary(pair:Pair<Rule>, context:&RefCell<&mut CodeGenContext>) -> Self {
        let ast_item = ASTItem::new(pair.line_col().into());
        match pair.as_rule() {
                Rule::float |
                Rule::bool_value |
                Rule::single_character |
                Rule::string |
                Rule::integer => Expression::Atom(ast_item, Atom::from_pair(&pair, *context.borrow_mut()).unwrap()),
                Rule::identifier => {
                    let atom_ast_item = ast_item.clone();
                    let atom_item_ast_item = ast_item.clone();
                    Expression::Atom(ast_item, Atom::new_id_with_ast_items(pair.as_str(), atom_ast_item, atom_item_ast_item))
                }
                Rule::tenary_expression => Expression::Tanry(ast_item, Box::new(TanryExpression::from_pair(&pair, *context.borrow_mut()).unwrap())),
                _ => todo!("{}", format!("need process {pair:?}")),
            }
    }

    fn pratt_map_postfix(v:Self, op:Pair<Rule>, context:&RefCell<&mut CodeGenContext>) -> Self {
        let ast_item = ASTItem::new(op.line_col().into());

        if op.as_rule() == Rule::postfix_expression_part {
            let op = PostfixExpressionPart::from_pair(&op, *context.borrow_mut()).unwrap();
            Expression::Postfix(ast_item, Expression_Postfix::new(Box::new(v), PostOperator::PostfixExpressionPart(op)))
        }
        else {
            let inner = op.into_inner().into_iter().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
        
            match inner.as_slice() {
                [(Rule::plusplus, _)] => Expression::Postfix(ast_item, Expression_Postfix::new(Box::new(v), PostOperator::Inc)),
                [(Rule::minusminus, _)] => Expression::Postfix(ast_item, Expression_Postfix::new(Box::new(v), PostOperator::Desc)),
                _ => {
                    let len = inner.len();
                    error_string(format!("len = {}", len));
                    error_string(format!("inner = {:?}", inner));
                    error_string(format!("v = {:?}", v));
                    todo!("{}", format!("need process postfix {inner:?} and {v:?}"))
                }
            }
        }
    }

    fn pratt_map_infix(left:Self, p:Pair<Rule>, right:Self, _context:&RefCell<&mut CodeGenContext>) -> Self {
        let op = match p.as_rule() {
            Rule::addition => LangOperators::Add,
            Rule::subtraction => LangOperators::Sub,
            Rule::multiplication => LangOperators::Mul,
            Rule::division => LangOperators::Div,
            Rule::logic_equal => LangOperators::LogicEqual,
            Rule::logic_not_equal => LangOperators::NotEqual,
            Rule::logic_and => LangOperators::LogicAnd,
            Rule::logic_or => LangOperators::LogicOr,
            Rule::operator_equal_or_greater => LangOperators::GreaterEuqal,
            Rule::operator_equal_or_less => LangOperators::LessEqual,
            Rule::operator_greater => LangOperators::Greater,
            Rule::operator_less => LangOperators::Less,
            Rule::rem => LangOperators::Rem,
            Rule::bit_and => LangOperators::BitAnd,
            Rule::bit_or => LangOperators::BitOr,
            Rule::bit_exclusive_or => LangOperators::BitXor,
            Rule::bit_left_move => LangOperators::BitLeftMove,
            Rule::bit_right_move => LangOperators::BitRightMove,
            Rule::pow => LangOperators::Pow,
            Rule::dot => LangOperators::AddressDot,
            Rule::PTR_OP => LangOperators::AddressArrow,
            _ => todo!("{}", format!("need infix {p:?} and (left = {left:?}, {right:?})")),
        };

        let left_expr = Box::new(left);
        let right_expr = Box::new(right);
        Expression::Expression(ASTItem::new(p.line_col().into()), Expression_Expression::new(left_expr, right_expr, op))
    }

    fn pratt_map_prefix(op:Pair<Rule>, v:Self, _context:&RefCell<&mut CodeGenContext>) -> Self {
        let inner = op.into_inner().into_iter().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();
                    
        match inner.as_slice() {
            [(Rule::plusplus, p)] |
            [(Rule::minusminus, p)] |
            [(Rule::subtraction, p)] |
            [(Rule::logic_not, p)] |
            [(Rule::bit_not, p)] |
            [(Rule::bit_and, p)] |
            [(Rule::pointer, p)] => {
                let op = UnaryOperator::from_pair2(p).unwrap();
                Expression::Prefix(ASTItem::new(p.line_col().into()), Expression_Prefix::new(op, Box::new(v)))
            }
            _ => {
                let len = inner.len();
                error_string(format!("len = {}", len));
                todo!("{}", format!("need process prefix {inner:?} and {v:?}"))
            }
        }
    }

    pub fn from_pair(pair:&Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        let pairs = pair.to_owned().into_inner();
        let pratt = Self::new_pratt();
        let context_ref_cell = RefCell::new(context);
        let r = pratt.map_primary(|primary| Self::pratt_map_primary(primary, &context_ref_cell))
                .map_prefix(|op, v| Self::pratt_map_prefix(op, v, &context_ref_cell))
                .map_postfix(|v, op| Self::pratt_map_postfix(v, op, &context_ref_cell))
                .map_infix(|left, p, right| Self::pratt_map_infix(left, p, right, &context_ref_cell))
                .parse(pairs);

        Ok(r)
    }

    /// get the atom id if the expression is atom; otherwise return None
    pub fn get_atom_id(&self) -> Option<&String> {
        match self {
            Self::Atom(_, n) => n.get_item_id(),
            _ => None,
        }
    }

    /// set id value changed flag for atom, prefix, postfix expression
    pub fn set_id_value_changed(&mut self, changed:bool) {
        match self {
            Self::Atom(_, n) => n.set_id_value_changed(changed),
            Self::Prefix(_, n) => n.get_expr_mut().set_id_value_changed(changed),
            Self::Postfix(_, n) => n.get_expr_mut().set_id_value_changed(changed),
            Self::Expression(_, n) => {
                match n.get_op() {
                    LangOperators::AddressDot | 
                    LangOperators::AddressArrow => {
                        n.get_left_mut().set_id_value_changed(changed);
                        n.get_right_mut().set_id_value_changed(changed);
                    }
                    _ => {
                        panic!("set_id_value_changed only support Atom, Prefix, Postfix Expression and Addressing Expression, but current is {:?}", self)
                    }
                }
            }
            _ => {
                panic!("set_id_value_changed only support Atom Expression, but current is {:?}", self)
            }
        }
    }

    pub fn get_composite_definition(&self) -> Option<CLangItem> {
        if self.is_atom() {
            self.get_atom_item()
                .and_then(|n| n.get_composite_definition())
        }
        else {
            None
        }
    }

    pub fn get_is_highest_addressing_expression(&self) -> Option<bool> {
        if self.is_addressing_expression() {
            match self {
                Self::Expression(_, n) => Some(n.get_is_highest_addressing_expression()),
                _ => None,
            }
        }
        else {
            None
        }
    }

    pub fn set_is_highest_addressing_expression(&mut self, is_highest:bool) {
        if self.is_addressing_expression() {
            match self {
                Self::Expression(_, n) => n.set_is_highest_addressing_expression(is_highest),
                _ => {},
            }
        }
    }

    /// set semantic linkage for addressing expression, 
    /// it's to link the left expression to the composite definition
    /// also link the right expression (member) to the member definition in the composite definition
    pub fn set_semantic_linkage_for_addressing_expression(&mut self) -> Result<CLangTypes, CLangError>{
        if self.is_addressing_expression() {
            let (_, expr_expr) = self.get_expression_mut().unwrap();

            let mut type_def_clang_item_value = None;

            let left = expr_expr.get_left_mut();
            match &mut **left {
                Expression::Atom(_, atom) => {
                    let type_def_clang_item = atom.get_item().get_declaration_type().cloned();
                    if let Some(type_def_clang_item) = type_def_clang_item {
                        type_def_clang_item_value = Some(type_def_clang_item.clone());
                        atom.get_item_mut().add_semantic_linkage(type_def_clang_item);
                    }
                    else {
                        return Err(CLangError::GeneralError((file!().to_string(), line!()), format!("Failed to find declaration type for atom in addressing expression: {:?}", atom)));
                    }
                }
                Expression::Expression(_, _) => {
                    if left.is_addressing_expression() {
                        let composite_type = left.set_semantic_linkage_for_addressing_expression()?;
                        if let Some(struct_union) = composite_type.get_struct_union() {
                            type_def_clang_item_value = match &composite_type {
                                CLangTypes::Struct(_, Some(type_def_clang_item)) |
                                CLangTypes::Union(_, Some(type_def_clang_item))
                                    if type_def_clang_item.is_struct_or_union() => Some(type_def_clang_item.clone()),
                                _ => left.get_ast_item().get_struct_or_union_clang_item(struct_union).cloned(),
                            };

                            if type_def_clang_item_value.is_none() {
                                return Err(CLangError::GeneralError((file!().to_string(), line!()), format!("Failed to resolve struct/union clang item for left expression in addressing expression: {:?} with resolved type {:?}", left, composite_type)));
                            }
                        }
                        else {
                            return Err(CLangError::GeneralError((file!().to_string(), line!()), format!("Failed to find struct/union definition for left expression in addressing expression: {:?} with resolved type {:?}", left, composite_type)));
                        }
                    }
                    else {
                        return Err(CLangError::GeneralError((file!().to_string(), line!()), format!("Unexpected left expression for addressing expression, expected atom or addressing expression, but got {:?}", left)));
                    }
                }
                _ => {
                    return Err(CLangError::GeneralError((file!().to_string(), line!()), format!("Unexpected left expression for addressing expression, expected atom, but got {:?}", left)));
                }
            }

            assert!(type_def_clang_item_value.is_some(), "Failed to find type definition for left expression in addressing expression: {:?}", left);

            let right = &mut **expr_expr.get_right_mut();
            let right_type = right.set_semantic_linkage_for_expression(type_def_clang_item_value.unwrap())?;
            
            Ok(right_type)
        }
        else {
            Err(CLangError::GeneralError((file!().to_string(), line!()), format!("set_semantic_linkage_for_addressing_expression only support addressing expression, but current is {:?}", self)))
        }
    }

    /// link the current expression to type_def_clang_item's member, type_def_clang_item is a struct/union definition clang item
    /// the function also perform semantic check to make sure the member exists in the composite definition; 
    /// it's used for both AddressDot and AddressArrow operator
    /// self is a member of the composite definition represented by type_def_clang_item, 
    /// like structA.member, member is self and linked to StructA's definition. 
    /// RETURN the member type if the linkage is successful, otherwise return error
    fn set_semantic_linkage_for_expression(&mut self, composite_definition:CLangItem) -> Result<CLangTypes, CLangError>{
        let n = self.get_atom_id()
                            .expect(& format!("cannot get atom_id from {self:?}"))
                            .to_string();

        let composite_option = composite_definition.get_declaration()
                                            .and_then(|x| Some(x.get_type_mut()))
                                            .and_then(|x| x.get_struct_union_mut())
                                .or_else(|| composite_definition.get_struct_or_union());

        if let Some(composite) = composite_option {
            for decl in composite.get_declares() {
                let name = decl.get_struct_item_name();
                if name == n {
                    if let Some(atom_item) = self.get_atom_mut()
                        .and_then(|x| Some(x.1.get_item_mut())) {
                            atom_item.add_semantic_linkage(composite_definition.clone());

                            let item_type = decl.get_item_type().clone();
                            return Ok(item_type);
                        }
                }
            }

            let composite_def_ast_item = composite_definition.get_ast_item();
            error_string(format!("cannot find member {n} in definition ast item {:?}", composite_def_ast_item));
            Err(CLangError::GeneralError((file!().to_string(), line!()), format!("Failed to find member {} in composite definition {:?} for expression {:?}", n, composite_definition, self)))
        }
        else {
            Err(CLangError::GeneralError((file!().to_string(), line!()), format!("Failed to find composite definition for expression {:?} with composite definition {:?}", self, composite_definition)))
        }
    }

    /// return true if the expression is atom
    pub fn is_atom(&self) -> bool {
        self.get_atom_item().is_some()
    }

    /// return true if the expression is addressing, i.e., using & operator
    pub fn is_addressing(&self) -> bool {
        self.get_atom_item()
            .map_or(false, |n| n.is_addressing())
    }

    /// return true if expression is Expression_Exprssion and operator is AddressDot or AddressArrow
    pub fn is_addressing_expression(&self) -> bool {
        match self {
            Self::Expression(_, n) => {
                matches!(n.get_op(), LangOperators::AddressDot | LangOperators::AddressArrow)
            }
            _ => false,
        }
    }

    pub fn get_atom_item(&self) -> Option<&Atom> {
        match self {
            Self::Atom(_, n) => Some(n),
            _ => None,
        }
    }

    pub fn get_clang_type(&self) -> CLangTypes {
        if self.get_ast_item().get_runtime_type().is_some() {
            return self.get_ast_item().get_runtime_type().unwrap().clone();
        }
        
        match self {
            Self::Atom(_, n) => n.get_clang_type(),
            Self::Expression(_, n) => n.get_clang_type(),
            Self::Tanry(_, n) => n.get_clang_type(),
            Self::Postfix(_, n) => n.get_clang_type(),
            Self::Prefix(_, n) => n.get_clang_type(),
        }
    }

    /// get string value if self is Atom; otherwise return None
    pub fn get_string_value(&self) -> Option<String> {
        if self.only_atom() {
            match self {
                Self::Atom(_, n) => n.get_string_value(),
                _ => None,
            }
        }
        else {
            None
        }
    }

    pub fn only_atom(&self) -> bool {
        match self {
            Self::Atom(_, _) => true,
            _ => false,
        }
    }

    /// get storage_location of the expression
    /// structA.structB.xx, the storage location is structA.structB.xx
    pub fn get_storage_location(&self) -> isize {
        let patterns = get_postfix_ref(self.get_all_clangitems());
        get_offset(&patterns) as isize
    }

    /// collect all identifier names used in this expression.
    /// this is a raw collection step and does not validate whether the identifier is a variable.
    pub fn get_used_variable_names(&self) -> Vec<String> {
        let mut seen = HashSet::new();
        let mut names = Vec::new();

        for item in self.get_all_clangitems() {
            if let Some(atom_item) = item.get_atom_item() {
                if let Some(id) = atom_item.get_id_value() {
                    if seen.insert(id.clone()) {
                        names.push(id.clone());
                    }
                }
            }
        }

        names
    }

    /// verify each used identifier is a variable-like symbol and return associated symbol entries.
    /// variable-like symbols include `VAR` and `DECLARATION` entries (parameters are declarations).
    pub fn get_used_variable_symbol_entries(&self) -> Result<Vec<(String, &SymbolEntry)>, CLangError> {
        let mut result = Vec::new();

        for name in self.get_used_variable_names() {
            let entry = self.resolve_variable_symbol_entry(&name)
                .ok_or_else(|| {
                    CLangError::GeneralError(
                        (file!().to_string(), line!()),
                        format!("Identifier '{name}' is used in expression but is not a variable symbol"),
                    )
                })?;

            result.push((name, entry));
        }

        Ok(result)
    }

    fn resolve_variable_symbol_entry(&self, name:&str) -> Option<&SymbolEntry> {
        if let Some(entry) = self.get_ast_item().get_var_symbol_entry_recursive(name) {
            return Some(entry);
        }

        if let Some(entry) = self.get_ast_item().get_parameter_symbol_entry(name) {
            return Some(entry);
        }

        self.get_ast_item()
            .get_symbol_recursive_all_types(name)
            .and_then(|entry| {
                if entry.get_symbol_type().is_var() || entry.get_symbol_type() == &Declaration::get_symbol_table_type_declaration() {
                    Some(entry)
                }
                else {
                    None
                }
            })
    }

    /// return true if this expression references the first function parameter
    /// and that parameter has storage in register `a0`.
    pub fn uses_first_parameter_in_a0(&self) -> bool {
        let first_parameter_name = self.get_ast_item()
            .get_current_function()
            .and_then(|function| {
                function.get_parameters()
                    .first()
                    .and_then(|parameter| parameter.get_var_name())
            });

        let Some(first_parameter_name) = first_parameter_name else {
            return false;
        };

        if !self.get_used_variable_names().iter().any(|name| name == &first_parameter_name) {
            return false;
        }

        self.get_ast_item()
            .get_parameter_symbol_entry(&first_parameter_name)
            .map(|entry| {
                entry.get_storage_location()
                    .iter()
                    .any(|location| location.get_register_name() == Some("a0"))
            })
            .unwrap_or(false)
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Atom(_, n) => vec![n.into()],
            Self::Tanry(_, n) => vec![n.as_mut().into()],
            Self::Expression(_, n) => vec![n.into()],
            Self::Postfix(_, n) => vec![n.into()],
            Self::Prefix(_, n) => vec![n.into()],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Atom(_, n) => n.access_children(context),
            Self::Tanry(_, n) => {
                n.access_children(context);
            }
            Self::Expression(_, n) => {
                n.access_children(context);
            }
            Self::Postfix(_, n) => {
                n.access_children(context);
            }
            Self::Prefix(_, n) => {
                n.access_children(context);
            }
        }

        context.access_clang_item(CLangItem::Expression(CLangItem::new(self)));
    }
}

impl Display for Expression {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Atom(_, n) => format!("{n}"),
            Self::Tanry(_, n) => format!("{n}"),
            Self::Expression(_, n) => format!("{n}"),
            Self::Postfix(_, n) => format!("{n}"),
            Self::Prefix(_, n) => format!("{n}"),
        };

        write!(f, "{}", s)
    }
}

impl ASTItemTrait for Expression {
    fn get_name(&self) -> String {
        let id = self.get_id();

        let base = format!("Expr({})", id);

        if let Some((_, postfix)) = self.get_postfix() {
            if let Some(id) = postfix.get_expr().get_atom_id() {
                return format!("{}_Fn_{id}", base);
            }
        }

        base
    }

    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Atom(n, _) |
            Self::Tanry(n, _) |
            Self::Expression(n, _) => n,
            Self::Postfix(n, _) |
            Self::Prefix(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Atom(n, _) |
            Self::Tanry(n, _) |
            Self::Expression(n, _) => n,
            Self::Postfix(n, _) => n,
            Self::Prefix(n, _) => n,
        }
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}