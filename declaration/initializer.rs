use pest::iterators::Pair;

use crate::ast::decl;
use crate::languages::clang::clang_pest::ParameterType;
use crate::languages::clang::{ASTItem, CLangItem, assignment_expression::*, clang_pest::Rule};
use crate::languages::clang::CLangError;
use crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext;
use crate::languages::clang::constant_expression::*;
use crate::languages::clang::symbol_table::*;
use core_utils::debug::*;
use crate::languages::clang::traits::*;
use crate::languages::clang::declaration::*;

pub type InitiaLizers = Vec<AssignmentExpression>;

/// init_declarator struct, Rule::init_declarator
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InitDeclarator {
    declarator: Declarator,
    initializer: InitiaLizers,
    ast_item: ASTItem,
}

impl InitDeclarator {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::init_declarator); //ensure it is the rule:init_declarator

        let ast_item = ASTItem::new(pair.line_col().into());
        let inner_pairs: Vec<Pair<Rule>> = pair.to_owned().into_inner().collect();

        if inner_pairs.len() != 1 && inner_pairs.len() != 2 {
            return Err(CLangError::ParsingConversionError);
        }

        //parse the rule:declarator
        let declarator = Declarator::from_pair(&inner_pairs[0], context)?;
        
        //add var to symbol table
        let name = declarator.get_id_name();
        let id = ast_item.get_sequence_id();
        let entry = SymbolEntry::new_var(id, &name, &context.get_scope_list().iter().map(|x| x.to_string()).collect::<Vec<_>>().join(","));
        context.get_symbol_table_mut().insert(entry);

        let items = inner_pairs.iter().map(|x| (x.as_rule(), x.clone())).collect::<Vec<_>>();
        match items.as_slice() {
            [(Rule::declarator, _), (Rule::initializer, p)] => {
                //contains the rule:declarator and initializer
                let initializer_list = p.clone()
                    .into_inner()
                    .map(|pair| {
                        match pair.as_rule()
                        {
                            Rule::assignment_expression => AssignmentExpression::from_pair(&pair, context),
                            Rule::initializer => {
                                //handle nested initializer
                                let inner_init_pair = pair.into_inner().next().unwrap();
                                match inner_init_pair.as_rule() {
                                    Rule::assignment_expression => AssignmentExpression::from_pair(&inner_init_pair, context),
                                    _ => {
                                        error_string(format!("expected assignment_expression in nested initializer, but got {:?} at {:?}", inner_init_pair.as_rule(), inner_init_pair.line_col()));
                                        panic!();
                                    }
                                }
                            },
                            _ => {
                                error_string(format!("expected assignment_expression or initializer, but got {:?} at {:?}", pair.as_rule(), pair.line_col()));
                                panic!();
                            }
                        }                        
                    })
                    .collect::<Result<Vec<AssignmentExpression>, _>>()?; //how to handle initializer_list ?

                return Ok(InitDeclarator {
                    declarator: declarator,
                    initializer: initializer_list,
                    ast_item,
                });
            },
            [(Rule::declarator, _)] => {
                //only contains the rule:declarator
                return Ok(InitDeclarator {
                    declarator: declarator,
                    initializer: vec![],
                    ast_item,
                });
            },
            _ => {
                todo!("handle init_declarator with unexpected rules: {:?}", items);
            }
        }
    }

    pub fn from_str(id_name:&str, ast_item:ASTItem) -> Self {
        let declarator = Declarator::new_with_id(ast_item.clone(), id_name);

        Self { declarator, 
                initializer: vec![], 
                ast_item: ast_item.clone(),
        }
    }

    pub fn get_declarator (&self) -> &Declarator {
        &self.declarator
    }

    pub fn get_declarator_parameters(&self) -> Vec<&ParameterType> {
        self.declarator.get_parameters()
    }

    pub fn get_id_name(&self) -> String {
        let name = self.declarator.get_id_name();
        name
    }

    /// used to check if declarator has value, it can be used to check if the definition is function
    pub fn declarator_has_values(&self) -> bool {
        self.declarator.direct_decl_has_values()
    }

    pub fn get_init(&self) -> &InitiaLizers {
        &self.initializer
    }

    /// check if the init_declarator has initializer, like i=9, the 9 is the initializer
    pub fn has_initializer(&self) -> bool {
        !self.initializer.is_empty()
    }

    pub fn get_init_constant_expr(&self) -> Option<&ConstantExpression> {
        if let Some(n) = self.get_init().first() {
            Some(n.get_constant_expr())
        }
        else {
            None
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![(&mut self.declarator).into()];

        for x in self.initializer.iter_mut() {
            r.push(x.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.declarator.access_children(context);
        for x in self.initializer.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::InitDeclarator(CLangItem::new(self)));
    }
}

impl ASTItemTrait for InitDeclarator {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        format!("InitDecl({})", self.get_id())
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}