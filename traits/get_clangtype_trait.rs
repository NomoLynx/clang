use crate::languages::clang::{assignment_expression::AssignmentExpressionLeftPart, clang_types::CLangTypes, constant_expression::{CastExpression, ConstantExpression, ConstantExpression_Tanry, SizeOfExpression}, symbol_table::SymbolTable, traits::ASTItemTrait};
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::tanry_expression::expression_postfix::*;

pub trait GetCLangTypeTrait 
    where Self : ASTItemTrait
{
    fn get_clang_type(&self) -> CLangTypes;
    fn get_type_string(&self) -> Option<String> {
        let clang_type = self.get_clang_type();
        if clang_type.is_none_type() || clang_type.is_invalid_type() {
            None
        }
        else {
            let r = clang_type.to_string();
            Some(r)
        }
    }
    
    fn set_clang_type(&mut self, clang_type:CLangTypes) {
        self.get_ast_item_mut().set_runtime_type(clang_type);
    }
}

impl GetCLangTypeTrait for TanryExpression {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for AtomItem {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for Expression {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for Expression_Expression {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for Atom {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for ConstantExpression_Tanry {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for SizeOfExpression {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for CastExpression {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for ConstantExpression {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for Expression_Postfix {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for Expression_Prefix {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}

impl GetCLangTypeTrait for AssignmentExpressionLeftPart {
    fn get_clang_type(&self) -> CLangTypes {
        self.get_clang_type()
    }
}