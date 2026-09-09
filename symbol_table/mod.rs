pub (crate) mod symbol_table;
pub(crate) mod symbol_entry;
pub(crate) mod symbol_location;
pub(crate) mod symbol_type;
pub(crate) mod register_use_types;
pub(crate) mod type_alias;

pub use symbol_table::*;
pub use symbol_entry::*;
pub use symbol_location::*;
pub use symbol_type::*;
pub use register_use_types::*;

pub type TypeName = String;
pub type ComplexType = String;
pub type VarialbeName = String;
pub type RegisterName = String;
pub type StackOffset = i32;
pub type HeapOffset = i32;
pub type MemberOffset = i32;   // the offset of the member from the start of the struct/union variable, this is used for struct/union member