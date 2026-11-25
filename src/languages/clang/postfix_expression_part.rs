use std::fmt::Display;

use crate::languages::clang::ASTItem;
use crate::languages::clang::CLangError;
use crate::languages::clang::CLangItem;
use crate::languages::clang::assignment_expression::AssignmentExpression;
use crate::languages::clang::clang_asm_code_gen::CodeGenContext;
use crate::languages::clang::clang_pest::Id;
use crate::languages::clang::clang_pest::CLangExpression;
use crate::languages::clang::clang_pest::Rule;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::declaration::resolve_clang_type_for_usage;
use crate::languages::clang::tanry_expression::Expression;
use crate::languages::clang::traits::ASTItemTrait;

use core_utils::debug::error_string;
use pest::iterators::Pair;
use rust_macro::*;

#[derive(Debug, Clone, PartialEq, Eq, EnumAccessors, GenIsEnumVariant)]
pub enum PostfixExpressionPart {
    CLangExpression(ASTItem, CLangExpression),
    Arguments(ASTItem, CLangExpression),
    DotId(ASTItem, Id),
    Ptr(ASTItem, Id),
    Inc(ASTItem),
    Dec(ASTItem),
}

impl PostfixExpressionPart {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> { 
        assert_eq!(pair.as_rule(), Rule::postfix_expression_part); //ensure it is Rule::assignment_expression

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner_pairs = pair.to_owned().into_inner().collect::<Vec<Pair<Rule>>>();
        let pairs = inner_pairs.into_iter().map(|x| (x.as_rule(), x)).collect::<Vec<_>>();

        match pairs.as_slice() {
            [(Rule::left_square_bracket, _), (Rule::clang_expression, p), (Rule::right_square_bracket, _)] => {
                let r = AssignmentExpression::from_clang(p, context)?;
                Ok( Self::CLangExpression(ast_item, r) )
            }
            [(Rule::clang_expression, p)] |
            [(Rule::left_bracket, _), (Rule::clang_expression, p), (Rule::right_bracket, _)] => {
                let r = AssignmentExpression::from_clang(p, context)?;
                Ok( Self::Arguments(ast_item, r) )
            }
            [] | 
            [(Rule::left_bracket, _), (Rule::right_bracket, _)] => {
                Ok( Self::Arguments(ast_item, vec![]) )
            }
            [(Rule::INC_OP, _)] => {
                Ok( Self::Inc(ast_item) )
            }
            [(Rule::DEC_OP, _)] => {
                Ok( Self::Dec(ast_item) )
            }
            [(Rule::identifier, p)] => {
                Ok( Self::DotId(ast_item, p.as_str().to_string()) )
            }
            [(Rule::PTR_OP, _), (Rule::identifier, p)] => {
                Ok( Self::Ptr(ast_item, p.as_str().to_string()) )
            }
            _ => {
                let err_msg = format!("cannot find {pairs:?} in PostfixExpressionPart processing logic"); 
                error_string(err_msg.to_string());
                Err(CLangError::MissingCase(file!().to_string(), line!() as usize, err_msg))
            }
        }
    }

    pub fn is_clangexpr_or_arguments(&self) -> bool {
        self.is_clangexpression() || self.is_arguments()
    }
    
    pub fn is_dot_or_ptr(&self) -> bool {
        self.is_dotid() || self.is_ptr()
    }

    /// if this postfix expression part is a dot member access or pointer member access, return the member name, 
    /// otherwise return None
    pub fn get_dot_or_ptr_member(&self) -> Option<&str> {
        match self {
            Self::DotId(_, member) | Self::Ptr(_, member) => Some(member.as_str()),
            _ => None,
        }
    }

    /// get the parent expression of this postfix expression part, 
    /// it is the expression that this postfix part belongs to
    pub fn get_parent_expression(&self) -> Option<&Expression> {
        let ast_item = self.get_ast_item();
        let expr = ast_item.get_parent()?
                        .get_expression_postfix()?
                        .get_expr();
        Some(expr)
    }

    /// get clang expression if the postfix expression part is CLangExpression or Arguments, otherwise return None
    pub fn get_clang_expression(&self) -> Option<&CLangExpression> {
        match self {
            Self::CLangExpression(_, clang_expression) |
            Self::Arguments(_, clang_expression) => Some(clang_expression),
            _ => None,
        }
    }

    /// Resolve the resulting type after applying this postfix part to `base_type`.
    pub fn resolve_result_type(&self, base_type:&CLangTypes) -> CLangTypes {
        match self {
            Self::DotId(_, member) => {
                base_type
                    .get_struct_union()
                    .and_then(|su| su.get_member_type(member))
                    .or_else(|| self.resolve_member_type_from_symbol_table(base_type, member))
                    .unwrap_or(CLangTypes::Invalid)
            }
            Self::Ptr(_, member) => {
                let deref_type = base_type
                    .dereference_pointer_type(1)
                    .unwrap_or(CLangTypes::Invalid);

                deref_type
                    .get_struct_union()
                    .and_then(|su| su.get_member_type(member))
                    .or_else(|| self.resolve_member_type_from_symbol_table(&deref_type, member))
                    .unwrap_or(CLangTypes::Invalid)
            }
            Self::CLangExpression(_, _) => {
                match base_type {
                    // array[index] => drop one dimension; fully indexed arrays become element type
                    CLangTypes::Array(element_type, array_name, dimensions) => {
                        if dimensions.get_dimension_count() > 1 {
                            let remaining_dims = dimensions.get_dimensions_values()[1..].to_vec();
                            CLangTypes::create_array_type(*element_type.clone(), array_name.clone(), remaining_dims.into())
                        }
                        else {
                            *element_type.clone()
                        }
                    }
                    // pointer[index] => dereferenced element type
                    CLangTypes::Pointer(_, _) => base_type.dereference_pointer_type(1).unwrap_or(CLangTypes::Invalid),
                    _ => CLangTypes::Invalid,
                }
            }
            // Function-call result type: callable(args) yields callable's return type.
            Self::Arguments(_, _) => {
                match base_type {
                    CLangTypes::FunctionPointer(_, ret_type, _) => *ret_type.clone(),
                    _ => base_type.clone(),
                }
            }
            // Post inc/dec do not change the expression's type.
            Self::Inc(_) |
            Self::Dec(_) => base_type.clone(),
        }
    }

    fn resolve_member_type_from_symbol_table(&self, composite_type:&CLangTypes, member:&str) -> Option<CLangTypes> {
        let resolved_type = resolve_clang_type_for_usage(composite_type, self.get_ast_item());
        let composite_type = &resolved_type;

        let normalized_type = if composite_type.is_composite() {
            Some(composite_type.clone())
        }
        else if composite_type.is_composite_pointer() {
            composite_type.dereference_pointer_type(1)
        }
        else {
            None
        }?;

        let type_name = normalized_type.get_composite_type_name()?;
        let entry = self.get_ast_item().get_symbol_recursive_type_definition(&type_name)?;

        entry
            .get_clang_item_link()
            .and_then(|item| item.get_declaration())
            .and_then(|decl| decl.get_struct_union())
            .and_then(|su| su.get_member_type(member))
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Arguments(_, n) |
            Self::CLangExpression(_, n) => n.iter_mut().map(|x| x.into()).collect(),
            Self::Dec(_) |
            Self::DotId(_, _) |
            Self::Inc(_) |
            Self::Ptr(_, _) => vec![],
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) {
        match self {
            Self::Arguments(_, n) => {
                for x in n {
                    x.access_children(context);
                }
            }
            Self::CLangExpression(_, n) => {
                for x in n {
                    x.access_children(context);
                } 
            }
            _ => (),
        }

        context.access_clang_item(CLangItem::PostfixExpressionPart(CLangItem::new(self)));
    }

    /// Returns a label string safe for use in Mermaid diagrams.
    /// Unlike Display, this avoids `->` which conflicts with Mermaid arrow syntax.
    pub fn to_mermaid_name(&self) -> String {
        match self {
            Self::Ptr(_, n) => format!("Ptr({n})"),
            other => other.to_string(),
        }
    }
}

impl Display for PostfixExpressionPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Arguments(_, clang_expr) => {
                let s = clang_expr.iter()
                                    .map(|x| format!("{x}"))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                format!("({s})")
            }
            Self::CLangExpression(_, clang_expr) => {
                let s = clang_expr.iter()
                                    .map(|x| format!("[{x}]"))
                                    .collect::<Vec<_>>()
                                    .join(", ");
                s
            }
            Self::Dec(_) => "--".to_string(),
            Self::DotId(_, n) => format!(".{n}"),
            Self::Inc(_) => "++".to_string(),
            Self::Ptr(_, n) => format!("->{n}"),
        };

        write!(f, "{}", s)
    }
}

impl ASTItemTrait for PostfixExpressionPart {
    fn get_name(&self) -> String {
        match self {
            Self::Arguments(_, _) | 
            Self::CLangExpression(_, _) => format!("PostfixExprPart({})", self.get_id()),
            Self::Dec(_) => format!("PostfixExprPart_Dec({})", self.get_id()),
            Self::DotId(_, n) => format!("PostfixExprPart_DotId({n}, {})", self.get_id()),
            Self::Inc(_) => format!("PostfixExprPart_Inc({})", self.get_id()),
            Self::Ptr(_, n) => format!("PostfixExprPart_PTR({n}, {})", self.get_id()),
        }
    }

    fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Arguments(n, _) |
            Self::CLangExpression(n, _) |
            Self::Dec(n) |
            Self::DotId(n, _) |
            Self::Inc(n) |
            Self::Ptr(n, _) => n,
        }
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Arguments(n, _) |
            Self::CLangExpression(n, _) |
            Self::Dec(n) |
            Self::DotId(n, _) |
            Self::Inc(n) |
            Self::Ptr(n, _) => n,
        }
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}
