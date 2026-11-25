use pest::iterators::Pair;

use crate::languages::clang::clang_asm_code_gen::CodeSnippet;
use crate::languages::clang::enum_type::EnumSpecifier;
use crate::languages::clang::symbol_table::SymbolType;
use crate::languages::clang::traits::*;
use core_utils::debug::*;
use crate::languages::clang::clang_types::CLangTypes;
use crate::languages::clang::struct_union::*;
use crate::languages::clang::declaration::*;
use crate::languages::clang::logic::*;
use crate::languages::clang::{ast_item::ASTItem, clang_asm_code_gen::code_gen_context::CodeGenContext, clang_error::CLangError, clang_item::CLangItem, clang_pest::Rule, symbol_table::SymbolTable, type_specifier::TypeSpecifier};

fn find_struct_union_def_in_ast_subtree(ast: &ASTItem, type_name: &str) -> Option<CLangItem> {
    if let Some(node) = ast.get_current_node() {
        if let Some(su) = node.get_struct_or_union() {
            if (su.is_struct_def() || su.is_union_def())
                && su.get_type_name() == type_name
            {
                return Some(su.get_clang_item());
            }
        }

        if let Some(decl) = node.get_declaration() {
            if let Some(su) = decl.get_struct_union() {
                if (su.is_struct_def() || su.is_union_def())
                    && su.get_type_name() == type_name
                {
                    return Some(su.get_clang_item());
                }
            }
        }
    }

    for child in ast.get_children() {
        if let Some(found) = find_struct_union_def_in_ast_subtree(child.get_ast_item(), type_name) {
            return Some(found);
        }
    }

    None
}

/// Walk up the parent chain from `ast_item` and search each ancestor subtree
/// for a struct/union definition whose name matches `type_name`.
/// Returns the `CLangItem` of the matching `StructUnionSpecifier` if found.
/// This works before the symbol table is built (only the AST parent links are needed).
fn find_struct_union_def_by_parent_walk(ast_item: &ASTItem, type_name: &str) -> Option<CLangItem> {
    let mut current = ast_item.get_parent();
    while let Some(parent_citem) = current {
        if let Some(found) = find_struct_union_def_in_ast_subtree(parent_citem.get_ast_item(), type_name) {
            return Some(found);
        }
        current = parent_citem.get_ast_item().get_parent();
    }
    None
}

fn find_enum_def_in_ast_subtree(ast: &ASTItem, enum_name: &str) -> Option<CLangItem> {
    if let Some(node) = ast.get_current_node() {
        if let Some(en) = node.get_enum() {
            if en.has_enum_items() && en.get_type_name() == enum_name {
                return Some(en.get_clang_item());
            }
        }

        if let Some(decl) = node.get_declaration() {
            if let Some(en) = decl.get_enum() {
                if en.has_enum_items() && en.get_type_name() == enum_name {
                    return Some(en.get_clang_item());
                }
            }
        }
    }

    for child in ast.get_children() {
        if let Some(found) = find_enum_def_in_ast_subtree(child.get_ast_item(), enum_name) {
            return Some(found);
        }
    }

    None
}

fn find_enum_def_by_parent_walk(ast_item: &ASTItem, enum_name: &str) -> Option<CLangItem> {
    let mut current = ast_item.get_parent();
    while let Some(parent_citem) = current {
        if let Some(found) = find_enum_def_in_ast_subtree(parent_citem.get_ast_item(), enum_name) {
            return Some(found);
        }
        current = parent_citem.get_ast_item().get_parent();
    }
    None
}

/// Recursively resolve the struct/union CLangItem back-link inside `ty`, handling
/// `Pointer` and `Array` wrappers so that pointer-to-struct members are also fixed.
/// Uses parent-chain traversal so it works even before the symbol table is built.
/// Falls back to symbol-table lookup when a parent walk finds nothing (e.g. cross-file).
pub(crate) fn fix_struct_union_link_in_type(ty: &mut CLangTypes, ast_item: &ASTItem) {
    match ty {
        CLangTypes::TypeAlias(alias_name, real_type) => {
            if real_type.is_none() {
                if let Some(entry) = ast_item.get_symbol_recursive_all_types(alias_name) {
                    if let Some(data_type) = entry.get_data_type() {
                        // Avoid self-referential aliases and resolve nested aliases recursively.
                        if let CLangTypes::TypeAlias(inner_alias_name, _) = data_type {
                            if inner_alias_name == alias_name {
                                return;
                            }
                        }

                        let mut resolved = data_type.clone();
                        fix_struct_union_link_in_type(&mut resolved, ast_item);
                        *real_type = Some(Box::new(resolved));
                    }
                }
            }
            else if let Some(real) = real_type.as_mut() {
                fix_struct_union_link_in_type(real, ast_item);
            }
        }
        CLangTypes::Struct(_, _) | CLangTypes::Union(_, _) => {
            if let Some(type_name) = ty.get_struct_union().map(|x| x.get_type_name()) {
                if ty.contains_invalid_clang_item_link() {
                    // Try parent-chain walk first (works before symbol table is built)
                    if let Some(clang_item) = find_struct_union_def_by_parent_walk(ast_item, &type_name) {
                        ty.set_struct_union_clang_item(clang_item);
                    } else if let Some(entry) = ast_item.get_symbol_recursive_type_definition(&type_name) {
                        // Fallback: symbol table lookup (works after setup_symbol_table)
                        if let Some(real_type) = entry.get_clang_item_link()
                                                        .and_then(|x| x.get_declaration())
                                                        .and_then(|x| x.get_struct_union()) {
                            ty.set_struct_union_clang_item(real_type.get_clang_item());
                        }
                    }
                }
            }
        }
        CLangTypes::Enum(_, _) => {
            if let Some(type_name) = ty.get_enum().map(|x| x.get_type_name()) {
                if ty.contains_invalid_clang_item_link() {
                    // Try parent-chain walk first (works before symbol table is built)
                    if let Some(clang_item) = find_enum_def_by_parent_walk(ast_item, &type_name) {
                        ty.set_enum_clang_item(clang_item);
                    } else if let Some(entry) = ast_item.get_symbol_recursive_type_definition(&type_name) {
                        // Fallback: symbol table lookup (works after setup_symbol_table)
                        if let Some(real_type) = entry.get_clang_item_link()
                                                        .and_then(|x| x.get_declaration())
                                                        .and_then(|x| x.get_enum()) {
                            ty.set_enum_clang_item(real_type.get_clang_item());
                        }
                    }
                }
            }
        }
        CLangTypes::Pointer(_, base_type) => {
            fix_struct_union_link_in_type(base_type, ast_item);
        }
        CLangTypes::Array(base_type, _, _) => {
            fix_struct_union_link_in_type(base_type, ast_item);
        }
        CLangTypes::FunctionPointer(_, ret_type, param_types) => {
            fix_struct_union_link_in_type(ret_type, ast_item);
            for param_type in param_types.iter_mut() {
                fix_struct_union_link_in_type(param_type, ast_item);
            }
        }
        _ => {}
    }
}

/// Central entry point for code generation and semantic computations that need
/// the concrete type while preserving typedef wrappers in stored metadata.
pub(crate) fn resolve_clang_type_for_usage(ty: &CLangTypes, ast_item: &ASTItem) -> CLangTypes {
    let mut resolved = ty.clone();
    fix_struct_union_link_in_type(&mut resolved, ast_item);
    resolved.collapse_type_alias_for_usage()
}

/// type declaration struct, Rule::declaration
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Declaration {
    clang_type: CLangTypes,  //struct, union, and enum types can be here.
    is_extern : bool,
    is_typedef : bool,
    init_declarator_list: Vec<InitDeclarator>,
    ast_item : ASTItem
}

impl Declaration {
    pub fn from_pair(pair: &Pair<Rule>, context:&mut CodeGenContext) -> Result<Self, CLangError> {
        assert_eq!(pair.as_rule(), Rule::declaration); //ensure it is rule:declaration

        let ast_item = ASTItem::new(pair.line_col().into());
        let mut declaration = Declaration::default();
        declaration.ast_item = ast_item;

        let inner_pairs: Vec<Pair<Rule>> = pair.to_owned().into_inner().collect();

        if inner_pairs.len() != 1 && inner_pairs.len() != 2 {
            error_string(format!("require innner pair to be 1 or 2, actually value = {}", inner_pairs.len()));
            return Err(CLangError::ParsingConversionError);
        }

        //parse the rule: declaration_specifiers
        let types = inner_pairs[0]
            .to_owned()
            .into_inner()
            .map(|pair| TypeSpecifier::from_pair(&pair, context).unwrap())
            .collect::<Vec<_>>();

        // check if types has extern and get its index
        declaration.is_extern = types.iter().any(|x| x.is_extern());
        declaration.is_typedef = types.iter().any(|x| x.is_typedef());

        //parse the rule:init_declarator_list if exists
        if inner_pairs.len() == 2 {
            let init_declarator_list_pair = inner_pairs[1].to_owned();
            let init_declarator_list_pair_inner = init_declarator_list_pair.into_inner();
            if init_declarator_list_pair_inner.len() > 0 {
                for init_declarator_pair in init_declarator_list_pair_inner {
                    let n = InitDeclarator::from_pair(&init_declarator_pair, context)?;
                    declaration
                        .init_declarator_list
                        .push(n);
                }

                // remove extern from types
                let types = types.into_iter().filter(|x| !x.is_extern()).collect::<Vec<_>>();
                declaration.clang_type = types.into();

                // Normalize declarator-driven array type early so no AST raw pointers are invalidated later.
                declaration.fix_type();

                Ok(declaration)
            }
            else {
                let items = inner_pairs.iter().map(|x| (x.as_rule(), x.clone())).collect::<Vec<_>>();
                match items.as_slice() {
                    [(Rule::type_alias, alias_p), (Rule::identifier, id_p)] => {
                        let alias_name = alias_p.as_str().to_owned();
                        let id_name = id_p.as_str().to_owned();
                        let alias_type = CLangTypes::TypeAlias(alias_name.clone(), None);
                        declaration.clang_type = alias_type;
                        let init_declarator_obj = InitDeclarator::from_str(&id_name, ASTItem::new(id_p.line_col().into()));
                        declaration.init_declarator_list.push(init_declarator_obj);
                        Ok(declaration)
                    }
                    _ => {
                        let str = format!("cannot find {items:?} in Declaration processing logic");
                        error_string(str.to_string());
                        return Err(CLangError::NoFound((file!().to_string(), line!()), str));
                    }
                }
            }
        }
        else {
            // declaration with only type specifiers, no init_declarator_list
            // e.g. `struct A { ... };` or `union B { ... };`
            let types = types.into_iter().filter(|x| !x.is_extern()).collect::<Vec<_>>();
            declaration.clang_type = types.into();
            Ok(declaration)
        }
    }

    /// fix type for array declaration
    pub fn fix_type(&mut self) {
        if self.is_function_def() {
            return;
        }

        let broken_element_option = self.get_variable_init_values().first()
                                                    .and_then(|x| x.get_declarator()
                                                                                    .get_direct_declarator()
                                                                                    .get_direct_decl_id());         
        if let Some(n) = broken_element_option {
            if let Some((id, dimension)) = n.get_array_info() {
                self.clang_type = CLangTypes::create_array_type(self.clang_type.clone(), id, dimension);
            }
        }
    }

    /// fix the type defintion for member reference to another struct (e.g. struct A { struct B b; }, 
    /// the B need to be fixed to the actual struct definition)
    pub fn fix_struct_union_declaration(&mut self) -> Result<(), CLangError> {
        let decl = self.get_clang_item();
        // Clone ast_item before any mutable borrows to avoid borrow-checker conflicts
        let ast_item = self.get_ast_item().clone();
        if let Some(struct_union) = self.get_struct_union_mut() {
            if struct_union.is_union_def() || struct_union.is_struct_def() {
                for member in struct_union.get_declares_mut() {
                    let ty = member.get_item_type_mut();
                    fix_struct_union_link_in_type(ty, &ast_item);
                }
            }
            else { //the struct/union is declaration without defintion (e.g. like struct A a;), now need to find the real definition
                // the action should be link struct/union to its real definition
                // chagne self.CLangType is NOT good, it will duplicate the symbol table and mess up the ast tree mermaid output
                if let Some(real_type) = decl.get_declaration().unwrap()
                                            .get_ast_item()
                                            .get_symbol_recursive_type_definition(struct_union.get_name_id().unwrap())
                                            .and_then(|x| x.get_clang_item_link())
                                            .and_then(|x| x.get_declaration())
                                            .and_then(|x| x.get_struct_union()) {
                    let link = real_type.get_clang_item();
                    struct_union.set_linkage(link);
                }
            }
        }        

        // Also fix the declaration's own type (e.g., struct S2 *p; where clang_type is Pointer(_, Struct(_, None)))
        fix_struct_union_link_in_type(self.get_type_mut(), &ast_item);

        Ok(())
    }

    /// generate big data type initialization code for array and struct/union, it's to modify the sp for large data type allocation,
    /// please note that initial value can change the actual sp offset value (sp_change_amount),
    /// the assumption is the initial code already move the sp register value
    pub fn generate_big_data_type_init_code(&self, context:&mut CodeGenContext) -> Result<CodeSnippet, CLangError> {
        let mut result = CodeSnippet::new_with_debug_info("array initialization code", "array initialization code");

        let ty = self.get_type();
        assert!(ty.is_array() || ty.is_composite()); //only array and struct/union can be big data type

        let sp_change_amount = if ty.is_array() {
            let array_size = ty.get_type_size_in_bytes(context).unwrap();
            let (ty_base, _name, _dimension) = ty.get_array_type().unwrap();
            let init_value_count = self.get_initial_value_count();
            array_size - init_value_count * ty_base.get_type_size_in_bytes(context).unwrap()
        }
        else { // struct or union
            ty.get_type_size_in_bytes(context).unwrap()
        };

        if sp_change_amount == 0 {
            return Ok(result); //no need to change sp
        }

        result.add_asm_inc("addi", vec![Some("sp"), Some("sp"), Some(format!("-{sp_change_amount}").as_ref())])?;
        Ok(result)
    }

    /// get if this declaration is extern
    pub fn is_extern(&self) -> bool {
        self.is_extern
    }

    /// get if this declaration is typedef
    pub fn is_typedef(&self) -> bool {
        self.is_typedef
    }

    /// checkf if declaration is global by checking its belonging function is None
    pub fn is_global(&self) -> bool {
        self.get_ast_item().get_current_function().is_none()
    }

    /// get declaration type CLangTypes
    pub fn get_type(&self) -> &CLangTypes {
        &self.clang_type
    }

    pub fn get_type_mut(&mut self) -> &mut CLangTypes {
        &mut self.clang_type
    }

    /// return const "SymbolType::Var"
    pub fn get_symbol_table_type_var() -> SymbolType {
        SymbolType::Var
    }

    /// return const "SymbolType::Declaration"
    pub fn get_symbol_table_type_declaration() -> SymbolType {
        SymbolType::Declaration
    }

    pub fn get_type_name_string(&self) -> String {
        self.get_type().to_string()
    }

    pub fn get_variable_init_values(&self) -> &Vec<InitDeclarator> {
        &self.init_declarator_list
    }

    /// resolve concrete data type for one declarator under this declaration.
    /// this applies declarator-level shape (pointer/array/function-pointer) on top of base declaration type.
    pub fn get_declarator_data_type(&self, init_decl:&InitDeclarator) -> CLangTypes {
        let declarator = init_decl.get_declarator();
        let direct_decl = declarator.get_direct_declarator();
        let base_type = self.get_type().clone();

        // function pointer declarator pattern: direct declarator wraps an inner declarator
        // that carries pointer stars, and the outer declarator carries parameter list.
        if let DirectDeclarator::Declarator(_, nested) = direct_decl {
            let has_function_parameters = !declarator.get_parameters().is_empty();
            let has_pointer_to_function = nested.get_declarator().get_pointer_level() > 0;
            if has_function_parameters && has_pointer_to_function {
                let param_types = declarator.get_parameters().iter()
                    .flat_map(|parameter_type| parameter_type.get_parameters().iter())
                    .map(|param_decl| param_decl.get_symbol_table_clang_type())
                    .collect::<Vec<_>>();
                let var_names = self.get_var_names();
                assert_eq!(var_names.len(), 1, "Function pointer declarator should have exactly one variable name");
                return CLangTypes::FunctionPointer(var_names[0].clone(), Box::new(base_type), param_types);
            }
        }

        let mut data_type = base_type;

        let pointer_level = declarator.get_pointer_level();
        if pointer_level > 0 {
            data_type = CLangTypes::make_pointer_type(data_type, pointer_level as usize);
        }

        // array suffix belongs to the direct declarator id.
        if let Some(direct_decl_id) = direct_decl.get_direct_decl_id() {
            if let Some((id, _dimension)) = direct_decl_id.get_array_info() {
                assert!(data_type.is_array(), 
                    "Array type should be built in fix_type() before get_declarator_data_type(), but got NON-Array type here for id = {id}");
            }
        }

        data_type
    }

    pub fn get_var_names(&self) -> Vec<String> {
        let init = self.get_variable_init_values();
        init.iter().map(|x| x.get_id_name()).collect::<Vec<_>>()
    }

    pub fn has_var_name(&self, name:&str) -> bool {
        let r = self.get_var_names().iter().any(|x| x==name);
        r
    }

    /// check if this declaration has initial value for any variable
    pub fn has_initial_value(&self) -> bool {
        let r = self.get_variable_init_values().iter().any(|x| x.has_initializer());
        r
    }

    /// get total initial value count for all variables in this declaration, like int a[3] = {1, 2, 3}, the count is 3
    fn get_initial_value_count(&self) -> usize {
        let r = self.get_variable_init_values()
                                            .iter()
                                            .map(|x| x.get_init().len())
                                            .sum();
        r
    }

    /// check if this declaration is function definition, that function can only define the function signature without body
    /// one sample is lib.h in test data, it defines printf function signature only.
    pub fn is_function_def(&self) -> bool {
        let has_clang_item = self.get_ast_item().get_current_node().is_some();
        if has_clang_item {
            let mut prog = DeclarationFacts::default();
            prog.node = self.get_all_clangitems_for_logic();
            prog.run();

            let r = !prog.parameter.is_empty();
            r
        }
        else {
            let has_parameter = self.get_variable_init_values().iter()
                .any(|x| x.get_declarator_parameters().len() > 0);
            has_parameter
        }
    }

    /// get struct definition if this declaration is struct
    pub fn get_struct(&self) -> Option<&StructUnionSpecifier> {
        self.get_type().get_struct()
    }

    /// get union definition if this declaration is union
    pub fn get_union(&self) -> Option<&StructUnionSpecifier> {
        self.get_type().get_union()
    }

    /// get struct/union definition if this declaration is struct or union
    pub fn get_struct_union(&self) -> Option<&StructUnionSpecifier> {
        self.get_type().get_struct_union()
    }

    /// get struct/union definition if this declaration is struct or union, mutable version
    pub fn get_struct_union_mut(&mut self) -> Option<&mut StructUnionSpecifier> {
        self.get_type_mut().get_struct_union_mut()
    }

    /// get enum definition if this declaration is enum
    pub fn get_enum(&self) -> Option<&EnumSpecifier> {
        self.get_type().get_enum()
    }

    /// get member type for struct/union, the member is identified by id, and the struct/union definition is from this declaration's type
    pub fn get_struct_union_member_type(&self, id:&str) -> Option<CLangTypes> {
        self.get_struct_union()
            .and_then(|x| x.get_member_type(id))
    }

    /// get symbol table this declaration belongs to
    pub fn get_symbol_table(&self) -> Option<&SymbolTable> {
        self.get_ast_item().get_symbol_table()
    }

    pub fn get_ast_item_mut_from_ref(&self) -> &mut ASTItem {
        let r = self.get_ast_item().get_current_node().unwrap();
        r.get_ast_item_mut()
    }

    pub fn get_children(&mut self) -> Vec<CLangItem> {
        let mut r = vec![];

        match self.get_type_mut() {
            CLangTypes::Struct(n, _) |
            CLangTypes::Union(n, _) => r.push(n.into()),
            CLangTypes::Enum(n, _) => r.push(n.into()),
            _ => {}
        }

        for x in self.init_declarator_list.iter_mut() {
            r.push(x.into());
        }

        r
    }

    pub fn access_children(&mut self, context:&mut CodeGenContext) { 
        match self.get_type_mut() {
            CLangTypes::Struct(n, _) |
            CLangTypes::Union(n, _) => n.access_children(context),
            CLangTypes::Enum(n, _) => n.access_children(context),
            _ => {}
        }

        for x in self.init_declarator_list.iter_mut() {
            x.access_children(context);
        }

        context.access_clang_item(CLangItem::Declaration(CLangItem::new(self)));
    }
}

impl ASTItemTrait for Declaration {
    fn get_ast_item(&self) -> &ASTItem {
        &self.ast_item
    }

    fn get_ast_item_mut(&mut self) -> &mut ASTItem {
        &mut self.ast_item
    }

    fn get_name(&self) -> String {
        if self.is_typedef() {
            let alias_name = self
                .get_var_names()
                .first()
                .cloned()
                .unwrap_or_else(|| "UnknownAlias".to_string())
                .replace(" ", "_");
            format!("TypeDef_{alias_name}_({})", self.get_id())
        }
        else {
            let type_str = self.get_type().to_string().replace(" ", "_");
            format!("Decl_{type_str}({})", self.get_id())
        }
    }

    fn get_id(&self) -> usize {
        self.get_ast_item().get_sequence_id()
    }
}

impl Default for Declaration {
    fn default() -> Self {
        Declaration {
            clang_type: CLangTypes::Invalid,
            is_extern : false,
            is_typedef : false,
            init_declarator_list: vec![],
            ast_item : ASTItem::default(),
        }
    }
}