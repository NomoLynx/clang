use std::{fmt::{Debug, Display}, path::PathBuf, vec};
use super::assignment_expression::AssignmentExpression;
use core_utils::debug::*;
use crate::{languages::clang::{ast_item::ASTItem, parameter_declaration::ParameterDeclaration, traits::{ASTItemTrait, symbol_table::AccessSymbolTableTrait}}};

use super::{clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_program::CLangProgram, constant_expression::ConstantExpression, type_specifier::TypeSpecifier};
use pest::{iterators::Pair, Parser};
use pest_derive::Parser;
use crate::languages::clang::declaration::*;
use rust_macro::*;

#[derive(Parser)]
#[grammar = "languages/clang/clang.pest"]
pub(crate) struct CLangParser;

pub(crate) fn parse_clang(input: &str, context:&mut CodeGenContext) -> Result<CLangProgram, CLangError> {
    let mut pairs = CLangParser::parse(Rule::clang_program, input)
        .map_err(|e| CLangError::get_location_from_pest_input_location(e.line(), &e.line_col))?;
    if let Some(pair) = pairs.find(|n| n.as_str().len() == input.len()) {
        let prog_r = CLangProgram::from_pair(&pair, context);
        if prog_r.is_err() {
            error_string(format!("cannot get program from Rule::clang_program, the error = {prog_r:?}"));
            Err(CLangError::ParsingConversionError)
        } else {
            let prog = prog_r.unwrap();
            Ok(prog)
        }
    } 
    else {
        error_string(format!(
            "Error: {} at {}",
            "does not catch all string",
            input.to_owned()
        ));
        
        debug_string(format!("input: {}\r\nParsed: {:#?}", input, pairs));
        let count = pairs.count();
        debug_string(format!("Pairs count = {count}\r\n"));
        Err(CLangError::ParsingConversionError)
    }
}

/// build symbol table for the clang program
/// set up the sequence id, build up clang item in ast item, 
/// fix array type, and setup symbol table and semantic linkages
pub fn parse_clang_symbol_table(prog:&mut CLangProgram, context:&mut CodeGenContext) -> Result<(), CLangError> {
    prog.access_children_deep_first(context);

    //build ast item tree mainly set clang item
    let mut citem : CLangItem = prog.into();
    citem.build_ast_item_tree(|_x| { 
        //output_string(format!("{}", x.get_name())); 
    });

    // fix array types
    prog.fix_types(context);

    // fix struct/union declaration
    prog.fix_struct_union_declaration(context)?;

    // set up symbol table & semantic linkages
    prog.setup_symbol_table(context.get_config())?;

    Ok(())
}

/// iterate the parsing AST tree, set up the unique id for each node
/// also set up symbol table, semantic linkages, type system, etc.
pub (crate) fn parse_clang_2nd(prog:&mut CLangProgram, context:&mut CodeGenContext) -> Result<(), CLangError> {
    parse_clang_symbol_table(prog, context)?;
    
    context.write_diagram_file(prog)?;
    context.write_symbol_table_md_file(prog);

    // semantic linkages
    let context0 = prog.setup_semantic_linkages();
    context0.output_error()?;

    // type system and type checking
    let context1 = prog.setup_type_system(context);
    context1.output_error()?;

    // enrich symbol tables, like address_taken field
    prog.enrich_symbol_table(context);

    // assign registers
    prog.assign_registers(context);

    // Persist symbol tables after storage locations are assigned.
    context.write_symbol_table_md_file(prog);

    Ok(())    
}

pub type Id = String;
pub type CLangExpression = Vec<AssignmentExpression>;
pub type FileName = String;
pub type PointerLevel = u8; //0 means no pointer

///struct of Full Path of the File
pub struct ProcessorFile(FileName);

impl ProcessorFile {
    pub fn from_pair(pair: &Pair<Rule>, _context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::path); //esnure parsed rule is Path

        let mut dir = "".to_string();

        let mut inner_pairs = pair.to_owned().into_inner();

        //if rule::path contains the rule::directory
        if let Some(_) = inner_pairs
            .peek()
            .filter(|pair| pair.as_rule() == Rule::directory)
        {
            let pair = inner_pairs.next().unwrap(); //consumed this rule
            let inner = pair.into_inner().next().unwrap();
            match inner.as_rule() {
                Rule::win_directory => {
                    dir = inner
                        .into_inner()
                        .map(|c| c.as_str().to_string())
                        .collect::<Vec<String>>()
                        .join("\\")
                }

                Rule::unix_directory => {
                    dir = inner
                        .into_inner()
                        .map(|c| c.as_str().to_string())
                        .collect::<Vec<String>>()
                        .join("/")
                }

                _ => {
                    let str = format!("cannot find {inner:?} in Path processing logic");
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }

        // Parse the rule:file_name
        let file_name = inner_pairs.next().unwrap().as_str().to_string();

        //concat the dir & file
        let mut new_dir = PathBuf::from(dir);
        new_dir.set_file_name(file_name);
        Ok(ProcessorFile(
            new_dir.as_path().to_str().unwrap().to_string(),
        ))
    }

    pub fn get_file_name(&self) -> FileName {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, GenIsEnumVariant)]
pub enum UnaryOperator {
    AddressOf,   // "&"
    Dereference(PointerLevel), // "*" or multiple "*"
    Plus,        // "+"
    Minus,       // "-"
    BitwiseNot,  // "~"
    LogicalNot,  // "!"
    Inc,         // "++"
    Dec,         // "--"
}

impl UnaryOperator {
    pub fn from_pair(pair: &Pair<Rule>, _context:&mut CodeGenContext) -> Result<Self, CLangError> {
        Self::from_pair2(pair)
    }

    pub fn from_pair2(pair: &Pair<Rule>) -> Result<Self, CLangError> {
        match pair.as_rule() {
            Rule::bit_and => Ok(UnaryOperator::AddressOf),
            Rule::addition => Ok(UnaryOperator::Plus),
            Rule::subtraction => Ok(UnaryOperator::Minus),
            Rule::logic_not => Ok(UnaryOperator::LogicalNot),
            Rule::bit_not => Ok(UnaryOperator::BitwiseNot),
            Rule::plusplus => Ok(UnaryOperator::Inc),
            Rule::minusminus => Ok(UnaryOperator::Dec),
            Rule::INC_OP => Ok(UnaryOperator::Inc),
            Rule::DEC_OP => Ok(UnaryOperator::Dec),
            Rule::pointer => { 
                let pointer = Pointer::from_pair2(pair)?;
                Ok(UnaryOperator::Dereference(pointer.get_level()))
            }
            _ => {
                let str = format!("cannot find {pair:?} in UnaryOperator processing logic");
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn get_mermaid_name(&self) -> String {
        match self {
            Self::AddressOf => "AddressOf".to_string(),
            Self::Dereference(n) => format!("Dereference({n})"),
            Self::Plus => "Plus".to_string(),
            Self::Minus => "Minus".to_string(),
            Self::BitwiseNot => "BitwiseNot".to_string(),
            Self::LogicalNot => "LogicalNot".to_string(),
            Self::Inc => "Inc".to_string(),
            Self::Dec => "Dec".to_string(),
        }
    }
}

impl Display for UnaryOperator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::AddressOf => "&".to_string(),
            Self::Dereference(n) => "*".repeat(*n as usize),
            Self::Plus => "+".to_string(),
            Self::Minus => "-".to_string(),
            Self::BitwiseNot => "~".to_string(),
            Self::LogicalNot => "!".to_string(),
            Self::Inc => "++".to_string(),
            Self::Dec => "--".to_string(),
        };

        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AssignmentOp {
    SimpleAssign, //add it
    MulAssign,
    DivAssign,
    ModAssign,
    AddAssign,
    SubAssign,
    LeftAssign,
    RightAssign,
    AndAssign,
    XorAssign,
    OrAssign,
}

impl AssignmentOp {
    pub fn from_pair(pair: &Pair<Rule>) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::assignment_operator); //ensure it is rule:assignment_operator

        if pair.as_str().eq("=") {
            Ok(AssignmentOp::SimpleAssign)
        } else {
            let inner_pair = pair.to_owned().into_inner().next().unwrap();

            match inner_pair.as_rule() {
                Rule::MUL_ASSIGN => Ok(AssignmentOp::MulAssign),
                Rule::DIV_ASSIGN => Ok(AssignmentOp::DivAssign),
                Rule::MOD_ASSIGN => Ok(AssignmentOp::ModAssign),
                Rule::ADD_ASSIGN => Ok(AssignmentOp::AddAssign),
                Rule::SUB_ASSIGN => Ok(AssignmentOp::SubAssign),
                Rule::LEFT_ASSIGN => Ok(AssignmentOp::LeftAssign),
                Rule::RIGHT_ASSIGN => Ok(AssignmentOp::RightAssign),
                Rule::AND_ASSIGN => Ok(AssignmentOp::AndAssign),
                Rule::XOR_ASSIGN => Ok(AssignmentOp::XorAssign),
                Rule::OR_ASSIGN => Ok(AssignmentOp::OrAssign),
                _ => {
                    let str = format!("cannot find {pair:?} in AssignmentOp processing logic");
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }
    }

    pub fn get_name(&self) -> String {
        let r = match self {
            Self::SimpleAssign => "SimpleAssign",
            Self::MulAssign => "MulAssign",
            Self::DivAssign => "DivAssign",
            Self::ModAssign => "ModAssign",
            Self::AddAssign => "AddAssign",
            Self::SubAssign => "SubAssign",
            Self::LeftAssign => "LeftAssign",
            Self::RightAssign => "RightAssign",
            Self::AndAssign => "AndAssign",
            Self::XorAssign => "XorAssign",
            Self::OrAssign => "OrAssign",
        };

        r.to_string()
    }
}

impl Display for AssignmentOp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let r = match self {
            Self::SimpleAssign => "=",
            Self::MulAssign => "*=",
            Self::DivAssign => "/=",
            Self::ModAssign => "%=",
            Self::AddAssign => "+=",
            Self::SubAssign => "-=",
            Self::LeftAssign => "<<=",
            Self::RightAssign => ">>=",
            Self::AndAssign => "&=",
            Self::XorAssign => "^=",
            Self::OrAssign => "|=",
        };

        write!(f, "{}", r)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pointer {
    poniter_level: PointerLevel, //0 means no pointer
    is_const: bool,
    is_volaitile: bool,
}

impl Pointer {
    pub fn from_pair2(pair: &Pair<Rule>) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::pointer); //ensure it is rule:pointer

        let mut pointer = Pointer {
            poniter_level: 0,
            is_const: false,
            is_volaitile: false,
        };
        let pair_content = pair.as_str();

        // pointer level is how many '*'
        pointer.poniter_level = pair_content.chars().filter(|c| *c == '*').count() as PointerLevel;

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::CONST => pointer.is_const = true,

                Rule::VOLATILE => pointer.is_volaitile = true,

                _ => {
                    let str = format!("cannot find {inner_pair:?} in pointer processing logic");
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }
        Ok(pointer)
    }

    pub fn from_pair(pair: &Pair<Rule>, _context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::pointer); //ensure it is rule:pointer
        Self::from_pair2(pair)    
    }

    pub fn get_level(&self) -> PointerLevel {
        self.poniter_level
    }
}

impl Display for Pointer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        for _ in 0..self.poniter_level {
            r.push_str("*");
        }
        if self.is_const {
            r.push_str(" const");
        }
        if self.is_volaitile {
            r.push_str(" volatile");
        }
        write!(f, "{}", r)
    }
}

///It has implemented the rule:parameter_type_list and can handle the ELLIPSIS (...)
/// has all parameters of a function
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterType {
    parameters: Vec<ParameterDeclaration>,
    eclipse: bool,
    ast_item : ASTItem,
}

impl ParameterType {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::parameter_type_list); //ensure it is rule:parameter_type_list

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut parameter_type_list = ParameterType {
            parameters: vec![],
            eclipse: false,
            ast_item,
        };

        let inner_pairs: Vec<Pair<Rule>> = pair.to_owned().into_inner().collect();

        //parse the rule:parameter_list
        parameter_type_list.parameters = inner_pairs[0]
            .to_owned()
            .into_inner()
            .map(|pair| ParameterDeclaration::from_pair(&pair, context).unwrap())
            .collect();
        //parse the rule:ELLIPSIS if exist
        if inner_pairs.len() == 2 {
            parameter_type_list.eclipse = true;
        }

        Ok(parameter_type_list)
    }

    pub fn get_eclipse(&self) -> bool {
        self.eclipse
    }

    pub fn get_parameters(&self) -> &Vec<ParameterDeclaration> {
        &self.parameters
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        for x in self.parameters.iter_mut() {
            r.push(x.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        for x in self.parameters.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::ParameterType(CLangItem::new(self)));
    }
}

impl ASTItemTrait for ParameterType {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        if self.eclipse {
            format!("ParamType_Ellipse({})", self.get_id())
        } else {
            format!("ParamType({})", self.get_id())
        }
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Display for ParameterType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        for x in self.parameters.iter() {
            r.push_str(&format!("{}, ", x));
        }
        if self.eclipse {
            r.push_str("...");
        }
        write!(f, "{}", r)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclatorOrAbstractDeclarator {
    Declarator(ASTItem, Declarator),
    AbstractDeclarator(ASTItem, AbstractDeclarator),
}

impl DeclatorOrAbstractDeclarator {
    pub fn get_declarator(&self) -> Option<&Declarator> {
        match self {
            Self::Declarator(_, n) => Some(n),
            Self::AbstractDeclarator(_, _) => None,
        }
    }

    pub fn get_pointer_level(&self) -> u8 {
        match self {
            Self::Declarator(_, n) => n.get_pointer_level(),
            Self::AbstractDeclarator(_, n) => n.get_pointer_level(),
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::Declarator(n, _) => n,
            Self::AbstractDeclarator(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::Declarator(n, _) => n,
            Self::AbstractDeclarator(n, _) => n,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::Declarator(_, n) => vec![n.into()],
            Self::AbstractDeclarator(_, n) => vec![n.into()],
        }
    }

    pub fn get_name(&self) -> String {
        format!("DeclOrAbstractDecl({})", self.get_ast_item().get_sequence_id())
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::AbstractDeclarator(_, n) => n.access_children(context),
            Self::Declarator(_, n) => n.access_children(context),
        }

        context.access_clang_item(CLangItem::DeclatorOrAbstractDeclarator(CLangItem::new(self)));
    }
}

impl Display for DeclatorOrAbstractDeclarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Declarator(_, n) => write!(f, "{}", n),
            Self::AbstractDeclarator(_, n) => write!(f, "{}", n),
        }
    }
}

impl ASTItemTrait for DeclatorOrAbstractDeclarator {
    fn get_ast_item(&self) -> &ASTItem {
        self.get_ast_item()
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        self.get_ast_item_mut()
    }

    fn get_name(&self) -> String {
        self.get_name()
    }
    
    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeName {
    specifiers_qualifiers: TypeSpecifier,
    abstractor_declarator: Option<AbstractDeclarator>,
    ast_item : ASTItem,
}
impl TypeName {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::type_name); //ensure it is rule:type_name

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut type_name = TypeName {
            specifiers_qualifiers: TypeSpecifier::Const(ASTItem::default()), //for initial value
            abstractor_declarator: None,
            ast_item,
        };

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::specifier_qualifier_list => {
                    // collect inner pairs so we can inspect and reuse them
                    let inners: Vec<Pair<Rule>> = inner_pair.into_inner().collect();
                    let inner_count = inners.len();
                    assert!(inner_count == 1); //only one specifier_qualifier_list, other cases will need to be handled
                    let ty_pair = &inners[0];
                    type_name.specifiers_qualifiers = TypeSpecifier::from_pair(ty_pair, context)?;
                }
                Rule::abstract_declarator => {
                    type_name.abstractor_declarator =
                        Some(AbstractDeclarator::from_pair(&inner_pair, context)?)
                }
                _ => {
                    let str = format!(
                        "cannot find {inner_pair:?} in primary_expression processing logic"
                    );
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }
        Ok(type_name)
    }

    pub fn get_specifiers_qualifiers(&self) -> &TypeSpecifier {
        &self.specifiers_qualifiers
    }

    pub fn get_type_string(&self) -> Option<String> {
        self.get_specifiers_qualifiers().
            get_type_name_string()
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_name(&self) -> String {
        format!("TypeName({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        r.push((&mut self.specifiers_qualifiers).into());
        if let Some(n) = self.abstractor_declarator.as_mut() {
            r.push(n.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        self.specifiers_qualifiers.access_children(context);
        if let Some(n) = self.abstractor_declarator.as_mut() {
            n.access_children(context);
        }

        context.access_clang_item(CLangItem::TypeName(CLangItem::new(self)));
    }
}

impl Display for TypeName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        r.push_str(&format!("{}", self.specifiers_qualifiers));
        if let Some(n) = &self.abstractor_declarator {
            r.push_str(&format!("{}", n));
        }
        write!(f, "{}", r)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbstractDeclarator {
    pointer: Option<Pointer>,
    direct_abstract_declarator: Option<DirectAbstractDeclarator>,
    ast_item : ASTItem,
}

impl AbstractDeclarator {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::abstract_declarator); //ensure it is rule:abstract_declarator

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut abstract_declarator = AbstractDeclarator {
            pointer: None,
            direct_abstract_declarator: None,
            ast_item,
        };

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::pointer => {
                    abstract_declarator.pointer = Some(Pointer::from_pair(&inner_pair, context)?)
                }

                Rule::direct_abstract_declarator => {
                    abstract_declarator.direct_abstract_declarator =
                        Some(DirectAbstractDeclarator::from_pair(&inner_pair, context)?)
                }

                _ => {
                    let str = format!(
                        "cannot find {inner_pair:?} in abstract_declarator processing logic"
                    );
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }
        
        Ok(abstract_declarator)
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_name(&self) -> String {
        format!("AbstractDecl({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_pointer_level(&self) -> u8 {
        self.pointer
            .as_ref()
            .map(|p| p.get_level())
            .unwrap_or(0)
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        if let Some(n) = self.direct_abstract_declarator.as_mut() {
            r.push(n.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        if let Some(n) = self.direct_abstract_declarator.as_mut() {
            n.access_children(context);
        }

        context.access_clang_item(CLangItem::AbstractDeclarator(CLangItem::new(self)));
    }
}

impl Display for AbstractDeclarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        if let Some(p) = &self.pointer {
            r.push_str(&format!("{} ", p));
        }
        if let Some(d) = &self.direct_abstract_declarator {
            r.push_str(&format!("{}", d));
        }
        write!(f, "{}", r)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirectAbstractDeclarator {
    abstractor_declaractor: Option<Box<AbstractDeclarator>>,
    declarator_parts: Vec<DirectAbstractDeclaratorPart>,
    ast_item: ASTItem,
}
impl DirectAbstractDeclarator {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::direct_abstract_declarator); //ensure it is rule:direct_abstract_declarator

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut direct_abstract_declarator = DirectAbstractDeclarator {
            abstractor_declaractor: None,
            declarator_parts: vec![],
            ast_item,
        };

        for inner_pair in pair.to_owned().into_inner() {
            match inner_pair.as_rule() {
                Rule::abstract_declarator => {
                    direct_abstract_declarator.abstractor_declaractor =
                        Some(Box::new(AbstractDeclarator::from_pair(&inner_pair, context)?))
                }

                Rule::direct_abstract_declarator_part => direct_abstract_declarator
                    .declarator_parts
                    .push(DirectAbstractDeclaratorPart::from_pair(&inner_pair, context)?),

                _ => {
                    let str = format!(
                        "cannot find {inner_pair:?} in direct_abstract_declarator processing logic"
                    );
                    error_string(str.to_string());
                    return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                }
            }
        }
        Ok(direct_abstract_declarator)
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    pub fn get_name(&self) -> String {
        format!("DirectAbstractDecl({})", self.get_ast_item().get_sequence_id())
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        if let Some(n) = self.abstractor_declaractor.as_mut() {
            r.push(n.as_mut().into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        if let Some(n) = self.abstractor_declaractor.as_mut() {
            n.access_children(context);
        }

        for x in self.declarator_parts.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::DirectAbstractDeclarator(CLangItem::new(self)));
    }
}

impl Display for DirectAbstractDeclarator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut r = String::default();
        if let Some(p) = &self.abstractor_declaractor {
            r.push_str(&format!("{}", p));
        }
        for x in self.declarator_parts.iter() {
            r.push_str(&format!("{}", x));
        }
        write!(f, "{}", r)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DirectAbstractDeclaratorPart {
    Const(ASTItem, ConstantExpression),
    // Parameter(Vec<ParameterType>),
    Parameter(ASTItem, ParameterType), //ParaterType has been implemented the rule:parameter_type_list, no need define Vec here
    BracketOnly(ASTItem),
}
///direct_declarator_part is a slient rule and will not be outputed
impl DirectAbstractDeclaratorPart {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        let ast_item = ASTItem::new(pair.line_col().into());
        match pair.as_rule() {
            Rule::constant_expression => Ok(DirectAbstractDeclaratorPart::Const(
                ast_item,
                ConstantExpression::from_pair(&pair, context)?,
            )),

            Rule::parameter_type_list => Ok(DirectAbstractDeclaratorPart::Parameter(
                ast_item,
                ParameterType::from_pair(&pair, context)?,
            )),

            _ => {
                let str = format!(
                    "cannot find {pair:?} in direct_abstract_declarator_part processing logic"
                );
                error_string(str.to_string());
                return Err(CLangError::NoFound((file!().to_string(), line!()), str));
            }
        }
    }

    pub fn get_ast_item(&self) -> &ASTItem {
        match self {
            Self::BracketOnly(n) |
            Self::Const(n, _) |
            Self::Parameter(n, _) => n,
        }
    }

    pub fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        match self {
            Self::BracketOnly(n) |
            Self::Const(n, _) |
            Self::Parameter(n, _) => n,
        }
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        match self {
            Self::BracketOnly(_) => vec![],
            Self::Const(_, n) => vec![n.into()],
            Self::Parameter(_, n) => vec![n.into()],
        }
    }

    pub fn get_name(&mut self) -> String {
        match self {
            Self::BracketOnly(_) => format!("DeclOrAbstractDecl_BrackOnly({})", self.get_ast_item().get_sequence_id()),
            Self::Const(_, _) => format!("DeclOrAbstractDecl_Const({})", self.get_ast_item().get_sequence_id()),
            Self::Parameter(_, _) => format!("DeclOrAbstractDecl_Param({})", self.get_ast_item().get_sequence_id()),
        }
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self {
            Self::Const(_, n) => n.access_children(context),
            Self::Parameter(_, n) => n.access_children(context),
            Self::BracketOnly(_) => (),
        }

        context.access_clang_item(CLangItem::DirectAbstractDeclaratorPart(CLangItem::new(self)));
    }
}

impl Display for DirectAbstractDeclaratorPart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BracketOnly(_) => write!(f, "[]"),
            Self::Const(_, n) => write!(f, "[{}]", n),
            Self::Parameter(_, n) => write!(f, "({})", n),
        }
    }
}