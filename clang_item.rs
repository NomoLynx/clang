use super::assignment_expression::AssignmentExpression;
use super::assignment_expression::AssignmentExpressionLeftPart;
use crate::languages::clang::traits::to_symbol_table_elements_trait::ToSymbolTableElements;
use crate::languages::clang::*;
use crate::languages::clang::enum_type::*;
use crate::languages::clang::struct_union::*;
use crate::languages::clang::declaration::*;
use crate::languages::clang::postfix_expression_part::*;
use crate::languages::clang::statement::{ExpressionStatement, Statement};
use crate::languages::clang::symbol_table::SymbolTable;
use crate::languages::clang::selection_statement::*;
use crate::languages::clang::iteration_statement::*;
use crate::languages::clang::jump_statement::*;
use crate::languages::clang::label_statement::*;
use crate::languages::clang::preprocessor_directive::*;
use crate::languages::clang::parameter_declaration::ParameterDeclaration;
use crate::languages::clang::tanry_expression::expression_postfix::*;
use crate::languages::clang::tanry_expression::*;
use crate::languages::clang::constant_expression::*;
use crate::languages::clang::type_specifier::TypeSpecifier;
use crate::languages::clang::ast_item::ASTItem;
use crate::languages::clang::clang_error::CLangError;
use crate::languages::clang::clang_program::CLangProgram;
use crate::languages::clang::compound_statement::CompoundStatement;
use crate::languages::clang::declaration::Declaration;
use crate::languages::clang::external_declaration::ExternalDeclaration;
use crate::languages::clang::function_definition::FunctionDefinition;
use crate::languages::clang::clang_pest::{AbstractDeclarator, DeclatorOrAbstractDeclarator, DirectAbstractDeclarator, DirectAbstractDeclaratorPart, ParameterType, TypeName};
use crate::languages::clang::traits::{scope_trait::ScopeTrait, symbol_table::AccessSymbolTableTrait};

use rust_macro::*;

#[allow(non_camel_case_types)]
#[derive(Debug, Clone, PartialEq, Eq, Hash, GenIsEnumVariant)]
pub enum CLangItem {
    CLangProgram(*mut CLangProgram),  //scope 
    ExternalDeclaration(*mut ExternalDeclaration),
    CastExpression(*mut CastExpression),
    FunctionDefinition(*mut FunctionDefinition), //scope
    CompoundStatement(*mut CompoundStatement),  //scope 
    PreProcessorDirective_Define(*mut PreProcessorDirective_Define),
    
    IterationStatement(*mut IterationStatement),
    IterationStatement_WHILE(*mut IterationStatement_WHILE),
    IterationStatement_DO(*mut IterationStatement_DO),
    IterationStatement_FOR(*mut IterationStatement_FOR),

    AsmStatement(*mut AsmStatement),
    
    SelectionStatement(*mut SelectionStatement),
    SelectionStatement_IF(*mut SelectionStatement_IF),
    SelectionStatement_SWITCH(*mut SelectionStatement_SWITCH),
    
    LabelStatement_Id(*mut LabelStatement_Id),
    LabelStatement_CASE(*mut LabelStatement_CASE),
    LabelStatement(*mut LabelStatement),

    Declaration(*mut Declaration),
    InitDeclarator(*mut InitDeclarator),
    AssignmentExpression(*mut AssignmentExpression),
    AssignmentExpressionLeftPart(*mut AssignmentExpressionLeftPart),
    Declarator(*mut Declarator),

    ParameterType(*mut ParameterType),
    ParameterDeclaration(*mut ParameterDeclaration),

    TypeName(*mut TypeName),
    AbstractDeclarator(*mut AbstractDeclarator),
    DirectDeclarator(*mut DirectDeclarator),

    DirectAbstractDeclarator(*mut DirectAbstractDeclarator), 
    DeclatorOrAbstractDeclarator(*mut DeclatorOrAbstractDeclarator), 
    DirectAbstractDeclaratorPart(*mut DirectAbstractDeclaratorPart),  

    ExpressionStatement(*mut ExpressionStatement),

    TanryExpression(*mut TanryExpression),
    Atom(*mut Atom),
    AtomItem(*mut AtomItem),
    Expression(*mut Expression),
    Expression_Expression(*mut Expression_Expression),
    Expression_Prefix(*mut Expression_Prefix),
    Expression_Postfix(*mut Expression_Postfix),

    PostfixExpressionPart(*mut PostfixExpressionPart),

    SizeOfExpression(*mut SizeOfExpression),

    ConstantExpression(*mut ConstantExpression),
    ConstantExpression_Tanry(*mut ConstantExpression_Tanry),

    DirectDelaratorPart(*mut DirectDelaratorPart),
    DirectDeclarator_Id(*mut DirectDeclarator_Id),
    DirectDeclarator_Declarator(*mut DirectDeclarator_Declarator),

    PreProcessorDirective(*mut PreProcessorDirective),
    Statement(*mut Statement),
    JumpStatement(*mut JumpStatement),

    EnumSpecifier(*mut EnumSpecifier),
    EnumSpecifier_Constant(*mut EnumSpecifier_Constant),

    StructUnionSpecifier(*mut StructUnionSpecifier),
    StructUnionSpecifier_Declarator(*mut StructUnionSpecifier_Declarator),
    StructDeclarator(*mut StructDeclarator),

    TypeSpecifier(*mut TypeSpecifier),
}

impl CLangItem {
    /// get the name of the item, which is the same as the struct name of the item
    pub fn get_name(&self) -> String {
        match self.clone() {
            Self::CLangProgram(n) => {
                CLangItem::from_raw::<CLangProgram>(n).get_name()
            }
            Self::ExternalDeclaration(n) => {
                CLangItem::from_raw::<ExternalDeclaration>(n).get_name()
            }
            Self::CastExpression(n) => {
                CLangItem::from_raw::<CastExpression>(n).get_name()
            }
            Self::FunctionDefinition(n) => {
                CLangItem::from_raw::<FunctionDefinition>(n).get_name()
            }
            Self::CompoundStatement(n) => {
                CLangItem::from_raw::<CompoundStatement>(n).get_name()
            }
            Self::PreProcessorDirective_Define(n) => {
                CLangItem::from_raw::<PreProcessorDirective_Define>(n).get_name()
            }
            Self::IterationStatement(n) => {
                CLangItem::from_raw::<IterationStatement>(n).get_name()
            }
            Self::IterationStatement_WHILE(n) => {
                CLangItem::from_raw::<IterationStatement_WHILE>(n).get_name()
            }
            Self::IterationStatement_DO(n) => {
                CLangItem::from_raw::<IterationStatement_DO>(n).get_name()
            }
            Self::IterationStatement_FOR(n) => {
                CLangItem::from_raw::<IterationStatement_FOR>(n).get_name()
            }
            Self::SelectionStatement(n) => {
                CLangItem::from_raw::<SelectionStatement>(n).get_name()
            }
            Self::SelectionStatement_IF(n) => {
                CLangItem::from_raw::<SelectionStatement_IF>(n).get_name()
            }
            Self::SelectionStatement_SWITCH(n) => {
                CLangItem::from_raw::<SelectionStatement_SWITCH>(n).get_name()
            }
            Self::AsmStatement(n) => {
                CLangItem::from_raw::<AsmStatement>(n).get_name()
            }
            Self::LabelStatement_CASE(n) => {
                CLangItem::from_raw::<LabelStatement_CASE>(n).get_name()
            }
            Self::LabelStatement(n) => {
                CLangItem::from_raw::<LabelStatement>(n).get_name()
            }
            Self::Declaration(n) => {
                CLangItem::from_raw::<Declaration>(n).get_name()
            }
            Self::InitDeclarator(n) => {
                CLangItem::from_raw::<InitDeclarator>(n).get_name()
            }
            Self::AssignmentExpression(n) => {
                CLangItem::from_raw::<AssignmentExpression>(n).get_name()
            }
            Self::AssignmentExpressionLeftPart(n) => {
                CLangItem::from_raw::<AssignmentExpressionLeftPart>(n).get_name()
            }
            Self::Declarator(n) => {
                CLangItem::from_raw::<Declarator>(n).get_name()
            }
            Self::ParameterType(n) => {
                CLangItem::from_raw::<ParameterType>(n).get_name()
            }
            Self::ParameterDeclaration(n) => {
                CLangItem::from_raw::<ParameterDeclaration>(n).get_name()
            }
            Self::TypeName(n) => {
                CLangItem::from_raw::<TypeName>(n).get_name()
            }
            Self::AbstractDeclarator(n) => {
                CLangItem::from_raw::<AbstractDeclarator>(n).get_name()
            }
            Self::DirectAbstractDeclarator(n) => {
                CLangItem::from_raw::<DirectAbstractDeclarator>(n).get_name()
            }
            Self::ExpressionStatement(n) => {
                CLangItem::from_raw::<ExpressionStatement>(n).get_name()
            }
            Self::TanryExpression(n) => {
                CLangItem::from_raw::<TanryExpression>(n).get_name()
            }
            Self::Atom(n) => {
                CLangItem::from_raw::<Atom>(n).get_name()
            }
            Self::Expression_Expression(n) => {
                CLangItem::from_raw::<Expression_Expression>(n).get_name()
            }
            Self::Expression_Prefix(n) => {
                CLangItem::from_raw::<Expression_Prefix>(n).get_name()
            }
            Self::Expression_Postfix(n) => {
                CLangItem::from_raw::<Expression_Postfix>(n).get_name()
            }
            Self::ConstantExpression(n) => {
                CLangItem::from_raw::<ConstantExpression>(n).get_name()
            }
            Self::ConstantExpression_Tanry(n) => {
                CLangItem::from_raw::<ConstantExpression_Tanry>(n).get_name()
            }
            Self::DirectDeclarator_Id(n) => {
                CLangItem::from_raw::<DirectDeclarator_Id>(n).get_name()
            }
            Self::DirectDeclarator_Declarator(n) => {
                CLangItem::from_raw::<DirectDeclarator_Declarator>(n).get_name()
            }
            Self::PreProcessorDirective(n) => {
                CLangItem::from_raw::<PreProcessorDirective>(n).get_name()
            }
            Self::Statement(n) => {
                CLangItem::from_raw::<Statement>(n).get_name()
            }
            Self::JumpStatement(n) => {
                CLangItem::from_raw::<JumpStatement>(n).get_name()
            }
            Self::LabelStatement_Id(n) => {
                CLangItem::from_raw::<LabelStatement_Id>(n).get_name()
            }
            Self::AtomItem(n) => {
                CLangItem::from_raw::<AtomItem>(n).get_name()
            }
            Self::Expression(n) => {
                CLangItem::from_raw::<Expression>(n).get_name()
            }
            Self::DirectDeclarator(n) => {
                CLangItem::from_raw::<DirectDeclarator>(n).get_name()
            }
            Self::DirectDelaratorPart(n) => {
                CLangItem::from_raw::<DirectDelaratorPart>(n).get_name()
            }
            Self::SizeOfExpression(n) => {
                CLangItem::from_raw::<SizeOfExpression>(n).get_name()
            }
            Self::PostfixExpressionPart(n) => {
                CLangItem::from_raw::<PostfixExpressionPart>(n).get_name()
            }
            Self::DeclatorOrAbstractDeclarator(n) => {
                CLangItem::from_raw::<DeclatorOrAbstractDeclarator>(n).get_name()
            }
            Self::DirectAbstractDeclaratorPart(n) => {
                CLangItem::from_raw::<DirectAbstractDeclaratorPart>(n).get_name()
            }
            Self::EnumSpecifier_Constant(n) => {
                CLangItem::from_raw::<EnumSpecifier_Constant>(n).get_name()
            }
            Self::EnumSpecifier(n) => {
                CLangItem::from_raw::<EnumSpecifier>(n).get_name()
            }
            Self::StructUnionSpecifier(n) => {
                CLangItem::from_raw::<StructUnionSpecifier>(n).get_name()
            }
            Self::StructUnionSpecifier_Declarator(n) => {
                CLangItem::from_raw::<StructUnionSpecifier_Declarator>(n).get_name()
            }
            Self::StructDeclarator(n) => {
                CLangItem::from_raw::<StructDeclarator>(n).get_name()
            }
            Self::TypeSpecifier(n) => {
                CLangItem::from_raw::<TypeSpecifier>(n).get_name()
            }
        }
    }

    pub fn get_children(&self) -> Vec<CLangItem> {
        match self.clone() {
            Self::CLangProgram(n) => {
                CLangItem::from_raw::<CLangProgram>(n).get_children()
            }
            Self::ExternalDeclaration(n) => {
                CLangItem::from_raw::<ExternalDeclaration>(n).get_children()
            }
            Self::CastExpression(n) => {
                CLangItem::from_raw::<CastExpression>(n).get_children()
            }
            Self::FunctionDefinition(n) => {
                CLangItem::from_raw::<FunctionDefinition>(n).get_children()
            }
            Self::CompoundStatement(n) => {
                CLangItem::from_raw::<CompoundStatement>(n).get_children()
            }
            Self::PreProcessorDirective_Define(n) => {
                CLangItem::from_raw::<PreProcessorDirective_Define>(n).get_children()
            }
            Self::AsmStatement(n) => {
                CLangItem::from_raw::<AsmStatement>(n).get_children()
            }
            Self::IterationStatement(n) => {
                CLangItem::from_raw::<IterationStatement>(n).get_children()
            }
            Self::IterationStatement_WHILE(n) => {
                CLangItem::from_raw::<IterationStatement_WHILE>(n).get_children()
            }
            Self::IterationStatement_DO(n) => {
                CLangItem::from_raw::<IterationStatement_DO>(n).get_children()
            }
            Self::IterationStatement_FOR(n) => {
                CLangItem::from_raw::<IterationStatement_FOR>(n).get_children()
            }
            Self::SelectionStatement(n) => {
                CLangItem::from_raw::<SelectionStatement>(n).get_children()
            }
            Self::SelectionStatement_IF(n) => {
                CLangItem::from_raw::<SelectionStatement_IF>(n).get_children()
            }
            Self::SelectionStatement_SWITCH(n) => {
                CLangItem::from_raw::<SelectionStatement_SWITCH>(n).get_children()
            }
            Self::LabelStatement_CASE(n) => {
                CLangItem::from_raw::<LabelStatement_CASE>(n).get_children()
            }
            Self::LabelStatement(n) => {
                CLangItem::from_raw::<LabelStatement>(n).get_children()
            }
            Self::Declaration(n) => {
                CLangItem::from_raw::<Declaration>(n).get_children()
            }
            Self::InitDeclarator(n) => {
                CLangItem::from_raw::<InitDeclarator>(n).get_children()
            }
            Self::AssignmentExpression(n) => {
                CLangItem::from_raw::<AssignmentExpression>(n).get_children()
            }
            Self::AssignmentExpressionLeftPart(n) => {
                CLangItem::from_raw::<AssignmentExpressionLeftPart>(n).get_children()
            }
            Self::Declarator(n) => {
                CLangItem::from_raw::<Declarator>(n).get_children()
            }
            Self::ParameterType(n) => {
                CLangItem::from_raw::<ParameterType>(n).get_children()
            }
            Self::ParameterDeclaration(n) => {
                CLangItem::from_raw::<ParameterDeclaration>(n).get_children()
            }
            Self::TypeName(n) => {
                CLangItem::from_raw::<TypeName>(n).get_children()
            }
            Self::AbstractDeclarator(n) => {
                CLangItem::from_raw::<AbstractDeclarator>(n).get_children()
            }
            Self::DirectAbstractDeclarator(n) => {
                CLangItem::from_raw::<DirectAbstractDeclarator>(n).get_children()
            }
            Self::ExpressionStatement(n) => {
                CLangItem::from_raw::<ExpressionStatement>(n).get_children()
            }
            Self::TanryExpression(n) => {
                CLangItem::from_raw::<TanryExpression>(n).get_children()
            }
            Self::Atom(n) => {
                CLangItem::from_raw::<Atom>(n).get_children()
            }
            Self::Expression_Expression(n) => {
                CLangItem::from_raw::<Expression_Expression>(n).get_children()
            }
            Self::Expression_Prefix(n) => {
                CLangItem::from_raw::<Expression_Prefix>(n).get_children()
            }
            Self::Expression_Postfix(n) => {
                CLangItem::from_raw::<Expression_Postfix>(n).get_children()
            }
            Self::ConstantExpression(n) => {
                CLangItem::from_raw::<ConstantExpression>(n).get_children()
            }
            Self::ConstantExpression_Tanry(n) => {
                CLangItem::from_raw::<ConstantExpression_Tanry>(n).get_children()
            }
            Self::DirectDeclarator_Id(n) => {
                CLangItem::from_raw::<DirectDeclarator_Id>(n).get_children()
            }
            Self::DirectDeclarator_Declarator(n) => {
                CLangItem::from_raw::<DirectDeclarator_Declarator>(n).get_children()
            }
            Self::PreProcessorDirective(n) => {
                CLangItem::from_raw::<PreProcessorDirective>(n).get_children()
            }
            Self::Statement(n) => {
                CLangItem::from_raw::<Statement>(n).get_children()
            }
            Self::JumpStatement(n) => {
                CLangItem::from_raw::<JumpStatement>(n).get_children()
            }
            Self::LabelStatement_Id(n) => {
                CLangItem::from_raw::<LabelStatement_Id>(n).get_children()
            }
            Self::AtomItem(n) => {
                CLangItem::from_raw::<AtomItem>(n).get_children()
            }
            Self::Expression(n) => {
                CLangItem::from_raw::<Expression>(n).get_children()
            }
            Self::DirectDeclarator(n) => {
                CLangItem::from_raw::<DirectDeclarator>(n).get_children()
            }
            Self::DirectDelaratorPart(n) => {
                CLangItem::from_raw::<DirectDelaratorPart>(n).get_children()
            }
            Self::SizeOfExpression(n) => {
                CLangItem::from_raw::<SizeOfExpression>(n).get_children()
            }
            Self::PostfixExpressionPart(n) => {
                CLangItem::from_raw::<PostfixExpressionPart>(n).get_children()
            }
            Self::DeclatorOrAbstractDeclarator(n) => {
                CLangItem::from_raw::<DeclatorOrAbstractDeclarator>(n).get_children()
            }
            Self::DirectAbstractDeclaratorPart(n) => {
                CLangItem::from_raw::<DirectAbstractDeclaratorPart>(n).get_children()
            }
            Self::EnumSpecifier_Constant(n) => {
                CLangItem::from_raw::<EnumSpecifier_Constant>(n).get_children()
            }
            Self::EnumSpecifier(n) => {
                CLangItem::from_raw::<EnumSpecifier>(n).get_children()
            }
            Self::StructUnionSpecifier(n) => {
                CLangItem::from_raw::<StructUnionSpecifier>(n).get_children()
            }
            Self::StructUnionSpecifier_Declarator(n) => {
                CLangItem::from_raw::<StructUnionSpecifier_Declarator>(n).get_children()
            }
            Self::StructDeclarator(n) => {
                CLangItem::from_raw::<StructDeclarator>(n).get_children()
            }
            Self::TypeSpecifier(n) => {
                CLangItem::from_raw::<TypeSpecifier>(n).get_children()
            }
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self.clone() {
            Self::AbstractDeclarator(n) => {
                CLangItem::from_raw::<AbstractDeclarator>(n).get_ast_item()
            }
            Self::AssignmentExpression(n) => {
                CLangItem::from_raw::<AssignmentExpression>(n).get_ast_item()
            }
            Self::AssignmentExpressionLeftPart(n) => {
                CLangItem::from_raw::<AssignmentExpressionLeftPart>(n).get_ast_item()
            }
            Self::Atom(n) => {
                CLangItem::from_raw::<Atom>(n).get_ast_item()
            }
            Self::CastExpression(n) => {
                CLangItem::from_raw::<CastExpression>(n).get_ast_item()
            }
            Self::CLangProgram(n) => {
                CLangItem::from_raw::<CLangProgram>(n).get_ast_item()
            }
            Self::CompoundStatement(n) => {
                CLangItem::from_raw::<CompoundStatement>(n).get_ast_item()
            }
            Self::ConstantExpression(n) => {
                CLangItem::from_raw::<ConstantExpression>(n).get_ast_item()
            }
            Self::ConstantExpression_Tanry(n) => {
                CLangItem::from_raw::<ConstantExpression_Tanry>(n).get_ast_item()
            }
            Self::Declarator(n) => {
                CLangItem::from_raw::<Declarator>(n).get_ast_item()
            }
            Self::DirectAbstractDeclarator(n) => {
                CLangItem::from_raw::<DirectAbstractDeclarator>(n).get_ast_item()
            }
            Self::DirectDeclarator_Declarator(n) => {
                CLangItem::from_raw::<DirectDeclarator_Declarator>(n).get_ast_item()
            }
            Self::DirectDeclarator_Id(n) => {
                CLangItem::from_raw::<DirectDeclarator_Id>(n).get_ast_item()
            }
            Self::Expression_Expression(n) => {
                CLangItem::from_raw::<Expression_Expression>(n).get_ast_item()
            }
            Self::Expression_Prefix(n) => {
                CLangItem::from_raw::<Expression_Prefix>(n).get_ast_item()
            }
            Self::Expression_Postfix(n) => {
                CLangItem::from_raw::<Expression_Postfix>(n).get_ast_item()
            }
            Self::ExpressionStatement(n) => {
                CLangItem::from_raw::<ExpressionStatement>(n).get_ast_item()
            }
            Self::ExternalDeclaration(n) => {
                CLangItem::from_raw::<ExternalDeclaration>(n).get_ast_item()
            }
            Self::FunctionDefinition(n) => {
                CLangItem::from_raw::<FunctionDefinition>(n).get_ast_item()
            }
            Self::InitDeclarator(n) => {
                CLangItem::from_raw::<InitDeclarator>(n).get_ast_item()
            }
            Self::AsmStatement(n) => {
                CLangItem::from_raw::<AsmStatement>(n).get_ast_item()
            }
            Self::IterationStatement(n) => {
                CLangItem::from_raw::<IterationStatement>(n).get_ast_item()
            }
            Self::IterationStatement_DO(n) => {
                CLangItem::from_raw::<IterationStatement_DO>(n).get_ast_item()
            }
            Self::IterationStatement_FOR(n) => {
                CLangItem::from_raw::<IterationStatement_FOR>(n).get_ast_item()
            }
            Self::IterationStatement_WHILE(n) => {
                CLangItem::from_raw::<IterationStatement_WHILE>(n).get_ast_item()
            }
            Self::LabelStatement(n) => {
                CLangItem::from_raw::<LabelStatement>(n).get_ast_item()
            }
            Self::LabelStatement_CASE(n) => {
                CLangItem::from_raw::<LabelStatement_CASE>(n).get_ast_item()
            }
            Self::ParameterDeclaration(n) => {
                CLangItem::from_raw::<ParameterDeclaration>(n).get_ast_item()
            }
            Self::ParameterType(n) => {
                CLangItem::from_raw::<ParameterType>(n).get_ast_item()
            }
            Self::PreProcessorDirective_Define(n) => {
                CLangItem::from_raw::<PreProcessorDirective_Define>(n).get_ast_item()
            }
            Self::SelectionStatement(n) => {
                CLangItem::from_raw::<SelectionStatement>(n).get_ast_item()
            }
            Self::SelectionStatement_IF(n) => {
                CLangItem::from_raw::<SelectionStatement_IF>(n).get_ast_item()
            }
            Self::SelectionStatement_SWITCH(n) => {
                CLangItem::from_raw::<SelectionStatement_SWITCH>(n).get_ast_item()
            }
            Self::TanryExpression(n) => {
                CLangItem::from_raw::<TanryExpression>(n).get_ast_item()
            }
            Self::TypeName(n) => {
                CLangItem::from_raw::<TypeName>(n).get_ast_item()
            }
            Self::Declaration(n) => {
                CLangItem::from_raw::<Declaration>(n).get_ast_item()
            }
            Self::PreProcessorDirective(n) => {
                CLangItem::from_raw::<PreProcessorDirective>(n).get_ast_item()
            }
            Self::Statement(n) => {
                CLangItem::from_raw::<Statement>(n).get_ast_item()
            }
            Self::JumpStatement(n) => {
                CLangItem::from_raw::<JumpStatement>(n).get_ast_item()
            }
            Self::LabelStatement_Id(n) => {
                CLangItem::from_raw::<LabelStatement_Id>(n).get_ast_item()
            }
            Self::AtomItem(n) => {
                CLangItem::from_raw::<AtomItem>(n).get_ast_item()
            }
            Self::Expression(n) => {
                CLangItem::from_raw::<Expression>(n).get_ast_item()
            }
            Self::DirectDeclarator(n) => {
                CLangItem::from_raw::<DirectDeclarator>(n).get_ast_item()
            }
            Self::DirectDelaratorPart(n) => {
                CLangItem::from_raw::<DirectDelaratorPart>(n).get_ast_item()
            }
            Self::SizeOfExpression(n) => {
                CLangItem::from_raw::<SizeOfExpression>(n).get_ast_item()
            }
            Self::PostfixExpressionPart(n) => {
                CLangItem::from_raw::<PostfixExpressionPart>(n).get_ast_item()
            }
            Self::DeclatorOrAbstractDeclarator(n) => {
                CLangItem::from_raw::<DeclatorOrAbstractDeclarator>(n).get_ast_item()
            }
            Self::DirectAbstractDeclaratorPart(n) => {
                CLangItem::from_raw::<DirectAbstractDeclaratorPart>(n).get_ast_item()
            }
            Self::EnumSpecifier_Constant(n) => {
                CLangItem::from_raw::<EnumSpecifier_Constant>(n).get_ast_item()
            }
            Self::EnumSpecifier(n) => {
                CLangItem::from_raw::<EnumSpecifier>(n).get_ast_item()
            }
            Self::StructUnionSpecifier(n) => {
                CLangItem::from_raw::<StructUnionSpecifier>(n).get_ast_item()
            }
            Self::StructUnionSpecifier_Declarator(n) => {
                CLangItem::from_raw::<StructUnionSpecifier_Declarator>(n).get_ast_item()
            }
            Self::StructDeclarator(n) => {
                CLangItem::from_raw::<StructDeclarator>(n).get_ast_item()
            }
            Self::TypeSpecifier(n) => {
                CLangItem::from_raw::<TypeSpecifier>(n).get_ast_item()
            }
        }
    }

    pub fn get_ast_item_mut(&self) -> &mut ASTItem {
        match self.clone() {
            Self::AbstractDeclarator(n) => {
                CLangItem::from_raw::<AbstractDeclarator>(n).get_ast_item_mut()
            }
            Self::AssignmentExpression(n) => {
                CLangItem::from_raw::<AssignmentExpression>(n).get_ast_item_mut()
            }
            Self::AssignmentExpressionLeftPart(n) => {
                CLangItem::from_raw::<AssignmentExpressionLeftPart>(n).get_ast_item_mut()
            }
            Self::Atom(n) => {
                CLangItem::from_raw::<Atom>(n).get_ast_item_mut()
            }
            Self::CastExpression(n) => {
                CLangItem::from_raw::<CastExpression>(n).get_ast_item_mut()
            }
            Self::CLangProgram(n) => {
                CLangItem::from_raw::<CLangProgram>(n).get_ast_item_mut()
            }
            Self::CompoundStatement(n) => {
                CLangItem::from_raw::<CompoundStatement>(n).get_ast_item_mut()
            }
            Self::ConstantExpression(n) => {
                CLangItem::from_raw::<ConstantExpression>(n).get_ast_item_mut()
            }
            Self::ConstantExpression_Tanry(n) => {
                CLangItem::from_raw::<ConstantExpression_Tanry>(n).get_ast_item_mut()
            }
            Self::Declarator(n) => {
                CLangItem::from_raw::<Declarator>(n).get_ast_item_mut()
            }
            Self::DirectAbstractDeclarator(n) => {
                CLangItem::from_raw::<DirectAbstractDeclarator>(n).get_ast_item_mut()
            }
            Self::DirectDeclarator_Declarator(n) => {
                CLangItem::from_raw::<DirectDeclarator_Declarator>(n).get_ast_item_mut()
            }
            Self::DirectDeclarator_Id(n) => {
                CLangItem::from_raw::<DirectDeclarator_Id>(n).get_ast_item_mut()
            }
            Self::Expression_Expression(n) => {
                CLangItem::from_raw::<Expression_Expression>(n).get_ast_item_mut()
            }
            Self::Expression_Prefix(n) => {
                CLangItem::from_raw::<Expression_Prefix>(n).get_ast_item_mut()
            }
            Self::Expression_Postfix(n) => {
                CLangItem::from_raw::<Expression_Postfix>(n).get_ast_item_mut()
            }
            Self::ExpressionStatement(n) => {
                CLangItem::from_raw::<ExpressionStatement>(n).get_ast_item_mut()
            }
            Self::ExternalDeclaration(n) => {
                CLangItem::from_raw::<ExternalDeclaration>(n).get_ast_item_mut()
            }
            Self::FunctionDefinition(n) => {
                CLangItem::from_raw::<FunctionDefinition>(n).get_ast_item_mut()
            }
            Self::InitDeclarator(n) => {
                CLangItem::from_raw::<InitDeclarator>(n).get_ast_item_mut()
            }
            Self::IterationStatement(n) => {
                CLangItem::from_raw::<IterationStatement>(n).get_ast_item_mut()
            }
            Self::AsmStatement(n) => {
                CLangItem::from_raw::<AsmStatement>(n).get_ast_item_mut()
            }
            Self::IterationStatement_DO(n) => {
                CLangItem::from_raw::<IterationStatement_DO>(n).get_ast_item_mut()
            }
            Self::IterationStatement_FOR(n) => {
                CLangItem::from_raw::<IterationStatement_FOR>(n).get_ast_item_mut()
            }
            Self::IterationStatement_WHILE(n) => {
                CLangItem::from_raw::<IterationStatement_WHILE>(n).get_ast_item_mut()
            }
            Self::LabelStatement(n) => {
                CLangItem::from_raw::<LabelStatement>(n).get_ast_item_mut()
            }
            Self::LabelStatement_CASE(n) => {
                CLangItem::from_raw::<LabelStatement_CASE>(n).get_ast_item_mut()
            }
            Self::ParameterDeclaration(n) => {
                CLangItem::from_raw::<ParameterDeclaration>(n).get_ast_item_mut()
            }
            Self::ParameterType(n) => {
                CLangItem::from_raw::<ParameterType>(n).get_ast_item_mut()
            }
            Self::PreProcessorDirective_Define(n) => {
                CLangItem::from_raw::<PreProcessorDirective_Define>(n).get_ast_item_mut()
            }
            Self::SelectionStatement(n) => {
                CLangItem::from_raw::<SelectionStatement>(n).get_ast_item_mut()
            }
            Self::SelectionStatement_IF(n) => {
                CLangItem::from_raw::<SelectionStatement_IF>(n).get_ast_item_mut()
            }
            Self::SelectionStatement_SWITCH(n) => {
                CLangItem::from_raw::<SelectionStatement_SWITCH>(n).get_ast_item_mut()
            }
            Self::TanryExpression(n) => {
                CLangItem::from_raw::<TanryExpression>(n).get_ast_item_mut()
            }
            Self::TypeName(n) => {
                CLangItem::from_raw::<TypeName>(n).get_ast_item_mut()
            }
            Self::Declaration(n) => {
                CLangItem::from_raw::<Declaration>(n).get_ast_item_mut()
            }
            Self::PreProcessorDirective(n) => {
                CLangItem::from_raw::<PreProcessorDirective>(n).get_ast_item_mut()
            }
            Self::Statement(n) => {
                CLangItem::from_raw::<Statement>(n).get_ast_item_mut()
            }
            Self::JumpStatement(n) => {
                CLangItem::from_raw::<JumpStatement>(n).get_ast_item_mut()
            }
            Self::LabelStatement_Id(n) => {
                CLangItem::from_raw::<LabelStatement_Id>(n).get_ast_item_mut()
            }
            Self::AtomItem(n) => {
                CLangItem::from_raw::<AtomItem>(n).get_ast_item_mut()
            }
            Self::Expression(n) => {
                CLangItem::from_raw::<Expression>(n).get_ast_item_mut()
            }
            Self::DirectDeclarator(n) => {
                CLangItem::from_raw::<DirectDeclarator>(n).get_ast_item_mut()
            }
            Self::DirectDelaratorPart(n) => {
                CLangItem::from_raw::<DirectDelaratorPart>(n).get_ast_item_mut()
            }
            Self::SizeOfExpression(n) => {
                CLangItem::from_raw::<SizeOfExpression>(n).get_ast_item_mut()
            }
            Self::PostfixExpressionPart(n) => {
                CLangItem::from_raw::<PostfixExpressionPart>(n).get_ast_item_mut()
            }
            Self::DeclatorOrAbstractDeclarator(n) => {
                CLangItem::from_raw::<DeclatorOrAbstractDeclarator>(n).get_ast_item_mut()
            }
            Self::DirectAbstractDeclaratorPart(n) => {
                CLangItem::from_raw::<DirectAbstractDeclaratorPart>(n).get_ast_item_mut()
            }
            Self::EnumSpecifier_Constant(n) => {
                CLangItem::from_raw::<EnumSpecifier_Constant>(n).get_ast_item_mut()
            }
            Self::EnumSpecifier(n) => {
                CLangItem::from_raw::<EnumSpecifier>(n).get_ast_item_mut()
            }
            Self::StructUnionSpecifier(n) => {
                CLangItem::from_raw::<StructUnionSpecifier>(n).get_ast_item_mut()
            }
            Self::StructUnionSpecifier_Declarator(n) => {
                CLangItem::from_raw::<StructUnionSpecifier_Declarator>(n).get_ast_item_mut()
            }
            Self::StructDeclarator(n) => {
                CLangItem::from_raw::<StructDeclarator>(n).get_ast_item_mut()
            }
            Self::TypeSpecifier(n) => {
                CLangItem::from_raw::<TypeSpecifier>(n).get_ast_item_mut()
            }
        }
    }

    /// Returns the unique sequence id of the current AST item, 
    /// this can be used to identify the AST item in the AST tree
    pub fn get_ast_item_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
    
    /// Returns true if the current node is a scope element
    /// currently it supports CLangProgram, FunctionDefinition, CompoundStatement as scope elements
    pub fn is_scope(&self) -> bool {
        match self.clone() {
            Self::CLangProgram(_) => true,
            Self::FunctionDefinition(_) => true,
            Self::CompoundStatement(_) => true,
            _ => false,
        }
    }

    pub fn is_for_loop(&self) -> bool {
        match self {
            Self::IterationStatement_FOR(_) => true,
            _ => false,
        }
    }

    /// Returns true if the current node is a type node and will requires type evaluation and NOT check
    /// statements like WHILE, IF, FOR are not type nodes
    pub fn is_type_node(&self) -> bool {
        match self.clone() {
            Self::TanryExpression(_) |
            Self::CastExpression(_) |
            Self::SizeOfExpression(_) |
            Self::AtomItem(_) |
            Self::Atom(_) | 
            Self::Expression_Postfix(_) |
            Self::Expression_Prefix(_) |
            Self::Expression_Expression(_) |
            Self::Expression(_) |
            Self::ConstantExpression(_) |
            Self::AssignmentExpressionLeftPart(_) |
            Self::ConstantExpression_Tanry(_) => true,
            _ => false,
        }
    }

    /// Returns true if the current node is a symbol entry element, which has symbol table or it is a declaration
    pub fn is_symbol_entry_element(&self) -> bool {
        match self.clone() {
            Self::CLangProgram(_) |
            Self::FunctionDefinition(_) |
            Self::CompoundStatement(_) | 
            Self::ParameterDeclaration(_) |
            Self::Declaration(_ ) => true, 
            _ => false,
        }
    }

    /// Returns the symbol entry element trait object if the current node is a symbol entry element
    /// otherwise returns None
    /// each of the symbol entry elements implement ToSymbolTableElements trait, it means they can provide symbol table elements
    pub fn get_symbol_entry_elements(&mut self) -> Option<&mut dyn ToSymbolTableElements> {
        match self.clone() {
            Self::CLangProgram(n) => {
                let k = Self::from_raw(n);
                Some(k)
            }
            Self::FunctionDefinition(n) => {
                let k = Self::from_raw(n);
                Some(k)
            }
            Self::CompoundStatement(n) => {
                let k = Self::from_raw(n);
                Some(k)
            }
            Self::Declaration(n) => {
                let k = Self::from_raw(n);
                Some(k)
            }
            Self::ParameterDeclaration(n) => {
                let k  = Self::from_raw(n);
                Some(k)
            }
            Self::EnumSpecifier(n) => {
                let k = Self::from_raw(n);
                Some(k)
            }
            _ => None,
        }
    }

    pub fn get_scope_trait(&self) -> Option<&mut dyn ScopeTrait> {
        match self.clone() {
            Self::CLangProgram(n) => Some(Self::from_raw::<CLangProgram>(n)),
            Self::FunctionDefinition(n) => Some(Self::from_raw::<FunctionDefinition>(n)),
            Self::CompoundStatement(n) => Some(Self::from_raw::<CompoundStatement>(n)),
            _ => None,
        }
    }

    pub fn get_all_symbol_tables(&self) -> Result<Vec<&SymbolTable>, CLangError> {
        match self.clone() {
            Self::CLangProgram(n) => {
                CLangItem::from_raw::<CLangProgram>(n).get_all_symbol_tables()
            }
            Self::FunctionDefinition(n) => {
                CLangItem::from_raw::<FunctionDefinition>(n).get_all_symbol_tables()
            }
            Self::CompoundStatement(n) => {
                CLangItem::from_raw::<CompoundStatement>(n).get_all_symbol_tables()
            }
            _ => {
                let err_msg = format!("get_all_symbol_tables is not supported for node type: {:?}", self.get_clang_item_real_type_name());
                Err(CLangError::MissingCase(file!().to_string(), line!() as usize, err_msg))
            }
        }
    }

    pub fn get_composite_type_symbol_table(&self) -> Option<&SymbolTable> {
        self.get_struct_or_union()
            .and_then(|x| Some(x.get_symbol_table()))
    }

    /// Sets up the symbol table for the current scope node and its children recursively
    /// this only applies to scope nodes like CLangProgram, FunctionDefinition, CompoundStatement
    pub fn setup_symbol_table(&self, config:&CodeGenConfiguration) -> Result<(), CLangError> {
        match self.clone() {
            Self::CLangProgram(n) => {
                CLangItem::from_raw::<CLangProgram>(n).setup_symbol_table(config)
            }
            Self::FunctionDefinition(n) => {
                CLangItem::from_raw::<FunctionDefinition>(n).setup_symbol_table(config)
            }
            Self::CompoundStatement(n) => {
                CLangItem::from_raw::<CompoundStatement>(n).setup_symbol_table(config)
            }
            _ => Err(CLangError::MissingCase(file!().to_string(), line!() as usize, "setup_symbol_table is not supported".to_string())),
        }
    }

    pub fn to_clang_prog(&self) -> Option<&mut CLangProgram> {
        match self.clone() {
            Self::CLangProgram(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_declaration(&self) -> Option<&mut Declaration> {
        match self.clone() {
            Self::Declaration(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_function_definition(&self) -> Option<&mut FunctionDefinition> {
        match self.clone() {
            Self::FunctionDefinition(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_struct_union_declarator(&self) -> Option<&mut StructUnionSpecifier_Declarator> {
        match self.clone() {
            Self::StructUnionSpecifier_Declarator(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_for_loop(&self) -> Option<&mut IterationStatement_FOR> {
        match self.clone() {
            Self::IterationStatement_FOR(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_compound_statement(&self) -> Option<&mut CompoundStatement> {
        match self.clone() {
            Self::CompoundStatement(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn is_direct_declarator_id(&self) -> bool {
        match self {
            Self::DirectDeclarator_Id(_) => true,
            _ => false,
        }
    }

    pub fn is_parameter_type(&self) -> bool {
        match self {
            Self::ParameterType(_) => true,
            _ => false,
        }
    }

    pub fn is_direct_declarator_part(&self) -> bool {
        match self {
            Self::DirectDelaratorPart(_) => true,
            _ => false,
        }
    }

    pub fn is_expression_statement(&self) -> bool {
        match self {
            Self::ExpressionStatement(_) => true,
            _ => false,
        }
    }

    pub fn is_constant_expression(&self) -> bool {
        match self {
            Self::ConstantExpression(_) => true,
            _ => false,
        }
    }

    pub fn is_tanry_expression(&self) -> bool {
        match self {
            Self::TanryExpression(_) => true,
            _ => false,
        }
    }

    pub fn is_expression_prefix_addressing(&self) -> bool {
        if let Some(prefix) = self.get_expression_prefix() {
            prefix.is_addressing()
        } 
        else {
            false
        }
    }

    pub fn is_parent_of(&self, child: &CLangItem) -> bool {
        let children = self.get_children();
        for c in children.iter() {
            if c == child {
                return true;
            }
        }
        false
    }

    pub fn get_type_specifier(&self) -> Option<&mut TypeSpecifier> {
        match self.clone() {
            Self::TypeSpecifier(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_atom_item(&self) -> Option<&mut AtomItem> {
        match self.clone() {
            Self::AtomItem(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_expression(&self) -> Option<&mut Expression> {
        match self.clone() {
            Self::Expression(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the expression prefix if the current node is Expression_Prefix
    pub fn get_expression_prefix(&self) -> Option<&mut Expression_Prefix> {
        match self.clone() {
            Self::Expression_Prefix(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the expression postfix if the current node is Expression_Postfix
    pub fn get_expression_postfix(&self) -> Option<&mut Expression_Postfix> {
        match self.clone() {
            Self::Expression_Postfix(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    pub fn get_expression_statement(&self) -> Option<&mut ExpressionStatement> {
        match self.clone() {
            Self::ExpressionStatement(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the enum if the current node is EnumSpecifier
    pub fn get_enum(&self) -> Option<&mut EnumSpecifier> {
        match self.clone() {
            Self::EnumSpecifier(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the enum item if the current node is EnumSpecifier_Constant
    pub fn get_enum_item(&self) -> Option<&mut EnumSpecifier_Constant> {
        match self.clone() {
            Self::EnumSpecifier_Constant(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the structUnionSpecifier if the current node is StructUnionSpecifier
    pub fn get_struct_or_union(&self) -> Option<&mut StructUnionSpecifier> {
        match self.clone() {
            Self::StructUnionSpecifier(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the postfix expression part if the current node is PostfixExpressionPart
    pub fn get_postfix_expression_part(&self) -> Option<&mut PostfixExpressionPart>{
        match self.clone() {
            Self::PostfixExpressionPart(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the assignment expression if the current node is AssignmentExpression
    pub fn get_assignment_expression(&self) -> Option<&mut AssignmentExpression> {
        match self.clone() {
            Self::AssignmentExpression(n) => Some(Self::from_raw(n)),
            _ => None,
        }
    }

    /// Returns the real struct or union definition if the current node is a struct or union specifier or 
    /// it is a declaration with struct or union specifier
    pub fn get_real_struct_or_union(&self) -> Option<&mut StructUnionSpecifier> {
        if self.is_struct_or_union() {
            let r = self.get_struct_or_union()?
                .get_real_definition_mut();
            Some(r)
        }
        else if self.is_declaration() {
            self.get_declaration()
                .and_then(|x| x.get_struct_union_mut())
                .and_then(|x| Some(x.get_real_definition_mut()))
        }
        else {
            None
        }
    }

    pub fn is_struct_or_union(&self) -> bool {
        match self {
            Self::StructUnionSpecifier(_) => true,
            _ => false,
        }
    }

    pub fn is_struct(&self) -> bool {
        match self.clone() {
            Self::StructUnionSpecifier(n) => {
                let k = Self::from_raw(n);
                k.is_struct()
            }
            _ => false,
        }
    }

    pub fn is_union(&self) -> bool {
        match self.clone() {
            Self::StructUnionSpecifier(n) => {
                let k = Self::from_raw(n);
                k.is_union()
            }
            _ => false,
        }
    }

    pub fn is_clang_prog(&self) -> bool {
        match self {
            Self::CLangProgram(_) => true,
            _ => false,
        }
    }

    pub fn is_function_definition(&self) -> bool {
        match self {
            Self::FunctionDefinition(_) => true,
            _ => false,
        }
    }

    pub fn is_atom_item(&self) -> bool {
        match self {
            Self::AtomItem(_) => true,
            _ => false,
        }
    }

    pub fn is_atom_item_id(&self) -> bool {
        if self.is_atom_item() {
            let r = self.get_atom_item().and_then(|x| x.get_id_value());
            r.is_some()
        }
        else {
            false
        }
    }

    pub fn is_assignment_expression(&self) -> bool {
        match self {
            Self::AssignmentExpression(_) => true,
            _ => false,
        }
    }

    /// checkf self is expression_postfix 
    pub fn is_expr_post(&self) -> bool {
        match self {
            Self::Expression_Postfix(_) => true,
            _ => false,
        }
    }

    pub fn is_postfix_expression_part(&self) -> bool {
        match self {
            Self::PostfixExpressionPart(_) => true,
            _ => false,
        }
    }

    /// Checks if the current CLangItem represents function arguments 
    /// within a postfix expression part.
    pub fn is_arguments(&self) -> bool {
        if self.is_postfix_expression_part() {
            self.get_postfix_expression_part().unwrap().is_arguments()
        }
        else {
            false
        }
    }

    pub fn set_self_in_ast_item(&mut self) {
        self.get_ast_item_mut().set_current_node(self.clone());
    }

    pub fn set_children_in_ast_item(&mut self) {
        self.get_ast_item_mut().set_children(self.get_children());
    }

    pub fn set_parent(&mut self, parent: Option<&CLangItem>) {
        self.get_ast_item_mut().set_parent(parent.map(|x| x.clone()));
    }
    
    pub fn build_ast_item_tree(&mut self, f:fn(&CLangItem) -> ()) {
        f(self);
        let ast_item = self.get_ast_item_mut();
        ast_item.set_current_node(self.clone());
        
        for n in self.get_children().iter_mut() {
            n.get_ast_item_mut().set_parent(Some(self.clone()));
            ast_item.add_child(n.clone());
            n.build_ast_item_tree(f);
        }
    }

    /// Returns the real type name of the current CLangItem, 
    /// this is used for debugging and error reporting
    pub fn get_clang_item_real_type_name(&self) -> String { 
        match self {
            Self::AbstractDeclarator(_) => "AbstractDeclarator".to_string(),
            Self::AssignmentExpression(_) => "AssignmentExpression".to_string(),
            Self::AssignmentExpressionLeftPart(_) => "AssignmentExpressionLeftPart".to_string(),
            Self::Atom(_) => "Atom".to_string(),
            Self::CastExpression(_) => "CastExpression".to_string(),
            Self::CLangProgram(_) => "CLangProgram".to_string(),
            Self::CompoundStatement(_) => "CompoundStatement".to_string(),
            Self::ConstantExpression(_) => "ConstantExpression".to_string(),
            Self::ConstantExpression_Tanry(_) => "ConstantExpression_Tanry".to_string(),
            Self::Declarator(_) => "Declarator".to_string(),
            Self::DirectAbstractDeclarator(_) => "DirectAbstractDeclarator".to_string(),
            Self::DirectDeclarator_Declarator(_) => "DirectDeclarator_Declarator".to_string(),
            Self::DirectDeclarator_Id(_) => "DirectDeclarator_Id".to_string(),
            Self::Expression_Expression(_) => "Expression_Expression".to_string(),
            Self::Expression_Prefix(_) => "Expression_Prefix".to_string(),
            Self::Expression_Postfix(_) => "Expression_Postfix".to_string(),
            Self::ExpressionStatement(_) => "ExpressionStatement".to_string(),
            Self::ExternalDeclaration(_) => "ExternalDeclaration".to_string(),
            Self::FunctionDefinition(_) => "FunctionDefinition".to_string(),
            Self::InitDeclarator(_) => "InitDeclarator".to_string(),
            Self::IterationStatement(_) => "IterationStatement".to_string(),
            Self::IterationStatement_DO(_) => "IterationStatement_DO".to_string(),
            Self::IterationStatement_FOR(_) => "IterationStatement_FOR".to_string(),
            Self::IterationStatement_WHILE(_) => "IterationStatement_WHILE".to_string(),
            Self::LabelStatement(_) => "LabelStatement".to_string(),
            Self::LabelStatement_CASE(_) => "LabelStatement_CASE".to_string(),
            Self::ParameterDeclaration(_) => "ParameterDeclaration".to_string(),
            Self::ParameterType(_) => "ParameterType".to_string(),
            Self::PreProcessorDirective_Define(_) => "PreProcessorDirective_Define".to_string(),
            Self::SelectionStatement(_) => "SelectionStatement".to_string(),
            Self::SelectionStatement_IF(_) => "SelectionStatement_IF".to_string(),
            Self::SelectionStatement_SWITCH(_) => "SelectionStatement_SWITCH".to_string(),
            Self::TanryExpression(_) => "TanryExpression".to_string(),
            Self::TypeName(_) => "TypeName".to_string(),
            Self::Declaration(_) => "Declaration".to_string(),
            Self::PreProcessorDirective(_) => "PreProcessorDirective".to_string(),
            Self::Statement(_) => "Statement".to_string(),
            Self::JumpStatement(_) => "JumpStatement".to_string(),
            Self::LabelStatement_Id(_) => "LabelStatement_Id".to_string(),
            Self::AtomItem(_) => "AtomItem".to_string(),
            Self::Expression(_) => "Expression".to_string(),
            Self::DirectDeclarator(_) => "DirectDeclarator".to_string(),
            Self::DirectDelaratorPart(_) => "DirectDelaratorPart".to_string(),
            Self::SizeOfExpression(_) => "SizeOfExpression".to_string(),
            Self::PostfixExpressionPart(_) => "PostfixExpressionPart".to_string(),
            Self::DeclatorOrAbstractDeclarator(_) => "DeclatorOrAbstractDeclarator".to_string(),
            Self::DirectAbstractDeclaratorPart(_) => "DirectAbstractDeclaratorPart".to_string(),
            Self::EnumSpecifier_Constant(_) => "EnumSpecifier_Constant".to_string(),
            Self::EnumSpecifier(_) => "EnumSpecifier".to_string(),
            Self::StructUnionSpecifier(_) => "StructUnionSpecifier".to_string(),
            Self::StructUnionSpecifier_Declarator(_) => "StructUnionSpecifier_Declarator".to_string(),
            Self::StructDeclarator(_) => "StructDeclarator".to_string(),
            Self::TypeSpecifier(_) => "TypeSpecifier".to_string(), 
            Self::AsmStatement(_) => "AsmStatement".to_string(),
        }
    }

    pub fn get_sequence_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
    
    pub fn new<T>(v:& mut T) -> *mut T {
        v as *mut T
    }

    pub fn from_raw<'a, T>(ptr: *mut T) -> &'a mut T {
        unsafe {
            // Dereference the raw pointer to obtain a mutable reference
            &mut *ptr
        }
    }

}

impl From<&mut CLangProgram> for CLangItem {
    fn from(value: &mut CLangProgram) -> Self {
        CLangItem::CLangProgram(CLangItem::new(value))
    }
}

impl From<&mut ExternalDeclaration> for CLangItem {
    fn from(value: &mut ExternalDeclaration) -> Self {
        CLangItem::ExternalDeclaration(CLangItem::new(value))
    }
}

impl From<&mut CastExpression> for CLangItem {
    fn from(value: &mut CastExpression) -> Self {
        CLangItem::CastExpression(CLangItem::new(value))
    }
}

impl From<&mut FunctionDefinition> for CLangItem {
    fn from(value: &mut FunctionDefinition) -> Self {
        CLangItem::FunctionDefinition(CLangItem::new(value))
    }
}

impl From<&mut CompoundStatement> for CLangItem {
    fn from(value: &mut CompoundStatement) -> Self {
        CLangItem::CompoundStatement(CLangItem::new(value))
    }
}

impl From<&mut PreProcessorDirective_Define> for CLangItem {
    fn from(value: &mut PreProcessorDirective_Define) -> Self {
        CLangItem::PreProcessorDirective_Define(CLangItem::new(value))
    }
}

impl From<&mut IterationStatement> for CLangItem {
    fn from(value: &mut IterationStatement) -> Self {
        CLangItem::IterationStatement(CLangItem::new(value))
    }
}

impl From<&mut IterationStatement_WHILE> for CLangItem {
    fn from(value: &mut IterationStatement_WHILE) -> Self {
        CLangItem::IterationStatement_WHILE(CLangItem::new(value))
    }
}

impl From<&mut IterationStatement_DO> for CLangItem {
    fn from(value: &mut IterationStatement_DO) -> Self {
        CLangItem::IterationStatement_DO(CLangItem::new(value))
    }
}

impl From<&mut IterationStatement_FOR> for CLangItem {
    fn from(value: &mut IterationStatement_FOR) -> Self {
        CLangItem::IterationStatement_FOR(CLangItem::new(value))
    }
}

impl From<&mut SelectionStatement> for CLangItem {
    fn from(value: &mut SelectionStatement) -> Self {
        CLangItem::SelectionStatement(CLangItem::new(value))
    }
}

impl From<&mut SelectionStatement_IF> for CLangItem {
    fn from(value: &mut SelectionStatement_IF) -> Self {
        CLangItem::SelectionStatement_IF(CLangItem::new(value))
    }
}

impl From<&mut SelectionStatement_SWITCH> for CLangItem {
    fn from(value: &mut SelectionStatement_SWITCH) -> Self {
        CLangItem::SelectionStatement_SWITCH(CLangItem::new(value))
    }
}

impl From<&mut LabelStatement_CASE> for CLangItem {
    fn from(value: &mut LabelStatement_CASE) -> Self {
        CLangItem::LabelStatement_CASE(CLangItem::new(value))
    }
}

impl From<&mut LabelStatement> for CLangItem {
    fn from(value: &mut LabelStatement) -> Self {
        CLangItem::LabelStatement(CLangItem::new(value))
    }
}

impl From<&mut Declaration> for CLangItem {
    fn from(value: &mut Declaration) -> Self {
        CLangItem::Declaration(CLangItem::new(value))
    }
}

impl From<&mut InitDeclarator> for CLangItem {
    fn from(value: &mut InitDeclarator) -> Self {
        CLangItem::InitDeclarator(CLangItem::new(value))
    }
}

impl From<&mut AssignmentExpression> for CLangItem {
    fn from(value: &mut AssignmentExpression) -> Self {
        CLangItem::AssignmentExpression(CLangItem::new(value))
    }
}

impl From<&mut AssignmentExpressionLeftPart> for CLangItem {
    fn from(value: &mut AssignmentExpressionLeftPart) -> Self {
        CLangItem::AssignmentExpressionLeftPart(CLangItem::new(value))
    }
}

impl From<&mut Declarator> for CLangItem {
    fn from(value: &mut Declarator) -> Self {
        CLangItem::Declarator(CLangItem::new(value))
    }
}

impl From<&mut ParameterType> for CLangItem {
    fn from(value: &mut ParameterType) -> Self {
        CLangItem::ParameterType(CLangItem::new(value))
    }
}

impl From<&mut ParameterDeclaration> for CLangItem {
    fn from(value: &mut ParameterDeclaration) -> Self {
        CLangItem::ParameterDeclaration(CLangItem::new(value))
    }
}

impl From<&mut TypeName> for CLangItem {
    fn from(value: &mut TypeName) -> Self {
        CLangItem::TypeName(CLangItem::new(value))
    }
}

impl From<&mut AbstractDeclarator> for CLangItem {
    fn from(value: &mut AbstractDeclarator) -> Self {
        CLangItem::AbstractDeclarator(CLangItem::new(value))
    }
}

impl From<&mut DirectAbstractDeclarator> for CLangItem {
    fn from(value: &mut DirectAbstractDeclarator) -> Self {
        CLangItem::DirectAbstractDeclarator(CLangItem::new(value))
    }
}

impl From<&mut ExpressionStatement> for CLangItem {
    fn from(value: &mut ExpressionStatement) -> Self {
        CLangItem::ExpressionStatement(CLangItem::new(value))
    }
}

impl From<&mut TanryExpression> for CLangItem {
    fn from(value: &mut TanryExpression) -> Self {
        CLangItem::TanryExpression(CLangItem::new(value))
    }
}

impl From<&mut Atom> for CLangItem {
    fn from(value: &mut Atom) -> Self {
        CLangItem::Atom(CLangItem::new(value))
    }
}

impl From<&mut Expression_Expression> for CLangItem {
    fn from(value: &mut Expression_Expression) -> Self {
        CLangItem::Expression_Expression(CLangItem::new(value))
    }
}

impl From<&mut Expression_Prefix> for CLangItem {
    fn from(value: &mut Expression_Prefix) -> Self {
        CLangItem::Expression_Prefix(CLangItem::new(value))
    }
}

impl From<&mut Expression_Postfix> for CLangItem {
    fn from(value: &mut Expression_Postfix) -> Self {
        CLangItem::Expression_Postfix(CLangItem::new(value))
    }
}

impl From<&mut ConstantExpression> for CLangItem {
    fn from(value: &mut ConstantExpression) -> Self {
        CLangItem::ConstantExpression(CLangItem::new(value))
    }
}

impl From<&mut ConstantExpression_Tanry> for CLangItem {
    fn from(value: &mut ConstantExpression_Tanry) -> Self {
        CLangItem::ConstantExpression_Tanry(CLangItem::new(value))
    }
}

impl From<&mut DirectDeclarator_Id> for CLangItem {
    fn from(value: &mut DirectDeclarator_Id) -> Self {
        CLangItem::DirectDeclarator_Id(CLangItem::new(value))
    }
}

impl From<&mut DirectDeclarator_Declarator> for CLangItem {
    fn from(value: &mut DirectDeclarator_Declarator) -> Self {
        CLangItem::DirectDeclarator_Declarator(CLangItem::new(value))
    }
}

impl From<&mut PreProcessorDirective> for CLangItem {
    fn from(value: &mut PreProcessorDirective) -> Self {
        CLangItem::PreProcessorDirective(CLangItem::new(value))
    }
}

impl From<&mut Statement> for CLangItem {
    fn from(value: &mut Statement) -> Self {
        CLangItem::Statement(CLangItem::new(value))
    }
}

impl From<&mut JumpStatement> for CLangItem {
    fn from(value: &mut JumpStatement) -> Self {
        CLangItem::JumpStatement(CLangItem::new(value))
    }
}

impl From<&mut LabelStatement_Id> for CLangItem {
    fn from(value: &mut LabelStatement_Id) -> Self {
        CLangItem::LabelStatement_Id(CLangItem::new(value))
    }
}

impl From<&mut AtomItem> for CLangItem {
    fn from(value: &mut AtomItem) -> Self {
        CLangItem::AtomItem(CLangItem::new(value))
    }
}

impl From<&mut Expression> for CLangItem {
    fn from(value: &mut Expression) -> Self {
        CLangItem::Expression(CLangItem::new(value))
    }
}

impl From<&mut DirectDeclarator> for CLangItem {
    fn from(value: &mut DirectDeclarator) -> Self {
        CLangItem::DirectDeclarator(CLangItem::new(value))
    }
}

impl From<&mut DirectDelaratorPart> for CLangItem {
    fn from(value: &mut DirectDelaratorPart) -> Self {
        CLangItem::DirectDelaratorPart(CLangItem::new(value))
    }
}

impl From<&mut SizeOfExpression> for CLangItem {
    fn from(value: &mut SizeOfExpression) -> Self {
        CLangItem::SizeOfExpression(CLangItem::new(value))
    }
}

impl From<&mut PostfixExpressionPart> for CLangItem {
    fn from(value: &mut PostfixExpressionPart) -> Self {
        CLangItem::PostfixExpressionPart(CLangItem::new(value))
    }
}

impl From<&mut DeclatorOrAbstractDeclarator> for CLangItem {
    fn from(value: &mut DeclatorOrAbstractDeclarator) -> Self {
        CLangItem::DeclatorOrAbstractDeclarator(CLangItem::new(value))
    }
}

impl From<&mut DirectAbstractDeclaratorPart> for CLangItem {
    fn from(value: &mut DirectAbstractDeclaratorPart) -> Self {
        CLangItem::DirectAbstractDeclaratorPart(CLangItem::new(value))
    }
}

impl From<&mut EnumSpecifier_Constant> for CLangItem {
    fn from(value: &mut EnumSpecifier_Constant) -> Self {
        CLangItem::EnumSpecifier_Constant(CLangItem::new(value))
    }
}

impl From<&mut EnumSpecifier> for CLangItem {
    fn from(value: &mut EnumSpecifier) -> Self {
        CLangItem::EnumSpecifier(CLangItem::new(value))
    }
}

impl From<&mut StructUnionSpecifier> for CLangItem {
    fn from(value: &mut StructUnionSpecifier) -> Self {
        CLangItem::StructUnionSpecifier(CLangItem::new(value))
    }
}

impl From<&mut StructUnionSpecifier_Declarator> for CLangItem {
    fn from(value: &mut StructUnionSpecifier_Declarator) -> Self {
        CLangItem::StructUnionSpecifier_Declarator(CLangItem::new(value))
    }
}

impl From<&mut StructDeclarator> for CLangItem {
    fn from(value: &mut StructDeclarator) -> Self {
        CLangItem::StructDeclarator(CLangItem::new(value))
    }
}

impl From<&mut TypeSpecifier> for CLangItem {
    fn from(value: &mut TypeSpecifier) -> Self {
        CLangItem::TypeSpecifier(CLangItem::new(value))
    }
}

impl From<&mut AsmStatement> for CLangItem {
    fn from(value: &mut AsmStatement) -> Self {
        CLangItem::AsmStatement(CLangItem::new(value))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParentChildrenLinker {
    pub parent : Option<CLangItem>,
    pub children : Box<Vec<Box<CLangItem>>>,
}

impl ParentChildrenLinker {
    pub fn new(parent: Option<CLangItem>, children: Box<Vec<Box<CLangItem>>>) -> Self {
        Self { parent, children }
    }

    pub fn get_parent(&self) -> Option<&CLangItem> {
        self.parent.as_ref()
    }

    pub fn get_children(&self) -> &Vec<Box<CLangItem>> {
        &self.children
    }

    pub fn get_children_mut(&mut self) -> &mut Vec<Box<CLangItem>> {
        &mut self.children
    }

    pub fn get_parent_mut(&mut self) -> &mut Option<CLangItem> {
        &mut self.parent
    }

    pub fn set_parent(&mut self, parent: Option<CLangItem>) {
        self.parent = parent;
    }

    pub fn set_children(&mut self, children: Box<Vec<Box<CLangItem>>>) {
        self.children = children;
    }

    pub fn add_child(&mut self, child: CLangItem) {
        self.children.push(Box::new(child));
    }
}

impl Default for ParentChildrenLinker {
    fn default() -> Self {
        Self { parent: None, children: Box::new(Vec::default()), }
    }
}