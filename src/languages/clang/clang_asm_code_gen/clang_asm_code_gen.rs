// this file is to generate C code to asm

use crate::languages::{clang::{clang_error::CLangError, constant_expression::ConstantExpression}, r5asm::{directive::Directive, r5asm_pest::SectionItem}};
use core_utils::debug::*;
use rust_macro::Accessors;
use crate::languages::clang::struct_union::*;
use core_utils::traits::generate_code::GenerateCode;

use crate::languages::clang::clang_program::CLangProgram;

use super::code_gen_configuration::CodeGenConfiguration;

#[derive(Debug, Clone)]
pub struct MDTableCell {
    asm_type : String,
    asm_value: String,
    footnote_tags : Vec<String>,
}

impl MDTableCell {
    pub fn new(asm_type:String, asm_value:String, footnote_tag:Option<String>) -> Self {
        if let Some(n) = footnote_tag {
            Self { asm_type, asm_value, footnote_tags : vec![n].to_vec() }
        }
        else {
            Self { asm_type, asm_value, footnote_tags : Vec::default() }
        }
    }

    pub fn to_asm_code(&self) -> String {
        let tags = self.footnote_tags.iter().map(|x| format!("[^{x}]")).collect::<Vec<_>>();
        format!("| {} {} {} |", self.asm_type, self.asm_value, tags.join(" "))
    }

    /// set the asm value
    pub fn set_value(&mut self, value:String) {
        self.asm_value = value;
    }

    /// add a footnote tag
    pub fn add_tag(&mut self, tag:String) {
        self.footnote_tags.push(tag)
    }

    /// get all footnote tags
    pub fn get_tags(&self) -> Vec<String> {
        self.footnote_tags.clone()
    }
}

impl From<(&str, &str)> for MDTableCell {
    fn from(t:(&str, &str)) -> Self {
        Self::new(t.0.to_string(), t.1.to_string(), None)
    }
}

#[derive(Debug, Clone, Accessors)]
pub struct Footnote {
    tag : String,
    content: String,
    comment: String,
}

impl Footnote {
    pub fn new(tag:String, content:String, comment:String) -> Self {
        Self { tag, content, comment }
    }

    pub fn new2(tag:&str) -> Self {
        Self { tag: tag.to_string(), content: tag.to_string(), comment: String::default() }
    }

    pub fn to_asm_code(&self) -> String {
        format!("<!-- {comment} -->\n[^{tag}]: {content}", 
            tag = self.tag, 
            content = self.content, 
            comment = self.comment)
    }
}

impl From<String> for Footnote {
    fn from(s:String) -> Self {
        (&s).into()
    }
}

impl From<&String> for Footnote {
    fn from(s:&String) -> Self {
        Self::new2(s)
    }
}

pub type AsmProgram = Vec<SectionItem>;

/// generate struct or union to md supported in asm, if var_name_option is None, it means generate type definition, otherwise generate variable definition
pub (crate) fn generate_md(struct_type:&StructUnionSpecifier, var_name_option:Option<String>, prog:&CLangProgram, config:&CodeGenConfiguration) -> Result<String, CLangError> {
    let type_name = struct_type.get_name_id().unwrap();
    let struct_or_union = if struct_type.is_struct() { "Struct" } else { "Union" };
    let has_var_name = var_name_option.is_some();
    let var_name = if has_var_name { var_name_option.unwrap() } else { String::default() };

    let header = format!("# {struct_or_union} {type_name} {var_name}\r\n\r\n");
    
    let (items, footnotes) = if struct_type.is_struct() { 
            let mut id = 0;
            let (a, b) = generate_struct_type_items(prog, struct_type, type_name.to_string(), &mut id, config);
            assert!(a.len() == b.len());
            (a,b)
        }
        else {
            let mut offset = 0;
            let type_size = struct_type.get_type_size(config);
            let type_size_in_bytes = type_size / 8;
            let mut memory_layout = vec![MDTableCell::new(".byte".to_string(), "0".to_string(), None); type_size_in_bytes as usize];
            let (_, notes) = generate_union_type_items(&mut memory_layout, prog, struct_type, type_name.to_string(), &mut offset, config);
            (memory_layout, notes)
        };

    

    let header_txt = if has_var_name { &var_name } else { type_name };
    let mut table_header = format!("| {header_txt} |\r\n");
    table_header = format!("{table_header}|----|\r\n");

    let table_rows = items.iter().map(|x| x.to_asm_code()).collect::<Vec<_>>().join("\r\n");
    let footnote = footnotes.iter().map(|x| x.to_asm_code()).collect::<Vec<_>>().join("\r\n");

    Ok(format!("{header}\r\n{table_header}{table_rows}\r\n\r\n{footnote}\r\n\r\n"))
}

fn generate_struct_type_items(prog:&CLangProgram, struct_type:&StructUnionSpecifier, name:String, id:&mut u32, config:&CodeGenConfiguration) -> (Vec<MDTableCell>, Vec<Footnote>) {
    let mut items = Vec::default();
    let mut footnotes = Vec::default();

    for nn in struct_type.get_declares().iter() {
        let tys = nn.get_item_type();
        let id_and_or_init = nn.get_declarators();
        let is_builtin_type = tys.is_built_in();
        let ids = id_and_or_init.iter()
                                        .filter_map(|x| x.get_name_id())
                                        .map(|x| x.get_id_name())
                                        .collect::<Vec<_>>();
        let id_name = ids.first().unwrap();

        if is_builtin_type {
            let asm_type = config.get_type_asm(tys).unwrap();
            let asm_value = if id_and_or_init.iter().any(|declarator| declarator.get_init().is_some()) {
                let vs = id_and_or_init.iter().filter_map(|declarator| declarator.get_init()).collect::<Vec<_>>();
                let v = vs.first().unwrap().get_const_value().unwrap();
                v 
            }
            else {
                let v = config.get_type_default_value(tys).unwrap();
                v
            };

            items.push(MDTableCell::new(asm_type, asm_value, Some(id_name.to_string())));
            footnotes.push(Footnote::new(id_name.to_string(), id_name.to_string(),  format!("for {name}{id}")));

            *id += 1;
        }
        else {
            let (mut m, mut n) = generate_struct_type_items(prog, tys.get_struct_union().unwrap(), id_name.to_string(), id, config);
            items.append(&mut m);
            footnotes.append(&mut n);
        }
    }

    (items, footnotes)
}

/// generate union to md supported in asm
/// this function is to set the lables to right location. The memroy layout has been set up at beginning
fn generate_union_type_items(memory_layout:&mut Vec<MDTableCell>, prog:&CLangProgram, union_type:&StructUnionSpecifier, name:String, current_offset:&mut usize, config:&CodeGenConfiguration) -> (Vec<MDTableCell>, Vec<Footnote>) {
    let is_struct = union_type.is_struct();

    let mut footnotes = Vec::default();

    for nn in union_type.get_declares().iter() {
        let tys = nn.get_item_type();
        let id_and_or_init = nn.get_declarators();
        let is_builtin_type = tys.is_built_in();
        let ids = id_and_or_init.iter()
                                        .filter_map(|x| x.get_name_id())
                                        .map(|x| x.get_id_name())
                                        .collect::<Vec<_>>();
        let id_name = ids.first().unwrap();

        if is_builtin_type {
            let cell = memory_layout.into_iter().nth(*current_offset).unwrap();
            cell.add_tag(id_name.to_string());
            footnotes.push(Footnote::new(id_name.to_string(), id_name.to_string(),  format!("comment")));
        }
        else {
            if tys.is_struct() {
                let (_, notes) = generate_union_type_items(memory_layout, prog, tys.get_struct_union().unwrap(), name.to_string(), current_offset, config);
                footnotes.extend(notes);
            }
        }

        if is_struct {
            *current_offset += tys.get_type_size_in_bytes2(config).unwrap();
        }
    }

    (Vec::default(), footnotes)
}

/// generate struct or union to md supported in asm
pub (crate) fn struct_to_asm(s:&StructUnionSpecifier, config:&CodeGenConfiguration) -> Result<AsmProgram, CLangError> {
    let mut code = Vec::default();

    let name = s.get_name_id();
    if name.is_some() { 
        code.push(SectionItem::new_label(name.unwrap()));
    }

    let is_struct = s.is_struct();
    if is_struct {
        let items = s.get_declares();

        for nn in items.iter() {
            let ty = nn.get_item_type();
            let n = nn.get_declarators();
            let type_str_option = config.get_type_asm(ty);
            if let Some(type_str) = type_str_option {
                let last_item = n.last();
                if let Some(declarator) = last_item {
                    let lable_option = declarator.get_name_id();
                    let init_option = declarator.get_init();
                    if let Some(lable) = lable_option {
                        let label_str = lable.get_id_name();
                        code.push(SectionItem::new_label(&label_str));
                        if let Some(expr) = init_option {
                            let expr_str = constant_expression_to_asm(expr, config)?;
                            let item_option = Directive::new(&type_str, [expr_str.to_string()].to_vec());
                            if let Some(item) = item_option {
                                code.push(SectionItem::Directive(item));
                            }
                            else {
                                error_string(format!("cannot convert {type_str} {expr_str} to asm Directive"));
                                panic!();
                            }
                        }
                        else {
                            let default_str = config.get_type_default_value(ty);
                            if default_str.is_some() {
                                let default_value_str = default_str.unwrap();
                                let item_option = Directive::new(&type_str, [default_value_str.to_string()].to_vec());
                                if let Some(item) = item_option {
                                    code.push(SectionItem::Directive(item));
                                }
                                else {
                                    error_string(format!("cannot convert {type_str} {default_value_str} to asm Directive"));
                                    panic!();
                                }
                            }
                        }
                    } 
                } 
            }
        }
    }
    else {  //union case
        let items = s.get_declares();

        let mut max_type_len = 0;
        let mut max_type_str = String::default();
        let mut init_value = String::default();

        for nn in items.iter() {
            let ty = nn.get_item_type();
            let n = nn.get_declarators();
            let type_str_option = config.get_type_asm(ty);

            if let Some(type_str) = type_str_option {
                if max_type_len < ty.get_type_size_in_bytes2(config).unwrap() {
                    max_type_len = ty.get_type_size_in_bytes2(config).unwrap();
                    max_type_str = type_str;
                    init_value = config.get_type_default_value(ty).unwrap();
                }

                let last_item = n.last();
                if let Some(declarator) = last_item {
                    let lable_option = declarator.get_name_id();
                    let init_option = declarator.get_init();
                    if let Some(lable) = lable_option {
                        let label_str = lable.get_id_name();
                        code.push(SectionItem::new_label(&label_str));

                        if let Some(expr) = init_option {
                            let expr_str = constant_expression_to_asm(expr, config)?;
                            init_value = expr_str;
                        }
                        else {
                            let default_str = config.get_type_default_value(ty);
                            if default_str.is_some() {
                                init_value = default_str.unwrap();
                            }
                        }
                    } 
                } 
            }
        }

        let directive_option = Directive::new(&max_type_str, [init_value.to_string()].to_vec());
        if let Some(directive) = directive_option {
            code.push(SectionItem::Directive(directive));
        }
        else {
            error_string(format!("cannot convert {max_type_str} {init_value} to asm Directive"));
            panic!();
        }
    }

    Ok(code)
}

pub (crate) fn struct_to_asm_string(s:&StructUnionSpecifier, config:&CodeGenConfiguration) -> Result<Vec<String>, CLangError> {
    let items = struct_to_asm(s, config)?;
    let r = items.iter().map(|x| x.generate_code_string()).collect::<Vec<_>>();
    Ok(r)
}

pub (crate) fn constant_expression_to_asm(_s:&ConstantExpression, _config:&CodeGenConfiguration) -> Result<String, CLangError> {
    Ok("0".to_string())
}