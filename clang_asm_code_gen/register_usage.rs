use std::collections::HashMap;

use lazy_static::lazy_static;

/// a temporary register name for switching two registers
pub const TEMP_REGISTER_NAME: &str = "_temp_REG"; 

lazy_static! {
    pub static ref REGISTER_NAME_MAPPING: HashMap<String, String> = {
        let mut m = HashMap::new();
        m.insert("zero".to_string(), "x0".to_string());
        m.insert("ra".to_string(), "x1".to_string());
        m.insert("sp".to_string(), "x2".to_string());
        m.insert("gp".to_string(), "x3".to_string());
        m.insert("tp".to_string(), "x4".to_string());
        m.insert("t0".to_string(), "x5".to_string());
        m.insert("t1".to_string(), "x6".to_string());
        m.insert("t2".to_string(), "x7".to_string());
        m.insert("fp".to_string(), "x8".to_string());
        m.insert("s0".to_string(), "x8".to_string());
        m.insert("s1".to_string(), "x9".to_string());
        m.insert("a0".to_string(), "x10".to_string());
        m.insert("a1".to_string(), "x11".to_string());
        m.insert("a2".to_string(), "x12".to_string());
        m.insert("a3".to_string(), "x13".to_string());
        m.insert("a4".to_string(), "x14".to_string());
        m.insert("a5".to_string(), "x15".to_string());
        m.insert("a6".to_string(), "x16".to_string());
        m.insert("a7".to_string(), "x17".to_string());
        m.insert("s2".to_string(), "x18".to_string());
        m.insert("s3".to_string(), "x19".to_string());
        m.insert("s4".to_string(), "x20".to_string());
        m.insert("s5".to_string(), "x21".to_string());
        m.insert("s6".to_string(), "x22".to_string());
        m.insert("s7".to_string(), "x23".to_string());
        m.insert("s8".to_string(), "x24".to_string());
        m.insert("s9".to_string(), "x25".to_string());
        m.insert("s10".to_string(), "x26".to_string());
        m.insert("s11".to_string(), "x27".to_string());             //used for storage parent frame pointer
        m.insert("t3".to_string(), "x28".to_string());
        m.insert("t4".to_string(), "x29".to_string());
        m.insert("t5".to_string(), "x30".to_string());
        m.insert("t6".to_string(), "x31".to_string());

        // float registers
        m.insert("ft0".to_string(), "f0".to_string());
        m.insert("ft1".to_string(), "f1".to_string());
        m.insert("ft2".to_string(), "f2".to_string());
        m.insert("ft3".to_string(), "f3".to_string());
        m.insert("ft4".to_string(), "f4".to_string());
        m.insert("ft5".to_string(), "f5".to_string());
        m.insert("ft6".to_string(), "f6".to_string());
        m.insert("ft7".to_string(), "f7".to_string());
        m.insert("fs0".to_string(), "f8".to_string());
        m.insert("fs1".to_string(), "f9".to_string());
        m.insert("fa0".to_string(), "f10".to_string());
        m.insert("fa1".to_string(), "f11".to_string());
        m.insert("fa2".to_string(), "f12".to_string());
        m.insert("fa3".to_string(), "f13".to_string());
        m.insert("fa4".to_string(), "f14".to_string());
        m.insert("fa5".to_string(), "f15".to_string());
        m.insert("fa6".to_string(), "f16".to_string());
        m.insert("fa7".to_string(), "f17".to_string());
        m.insert("fs2".to_string(), "f18".to_string());
        m.insert("fs3".to_string(), "f19".to_string());
        m.insert("fs4".to_string(), "f20".to_string());
        m.insert("fs5".to_string(), "f21".to_string());
        m.insert("fs6".to_string(), "f22".to_string());
        m.insert("fs7".to_string(), "f23".to_string());
        m.insert("fs8".to_string(), "f24".to_string());
        m.insert("fs9".to_string(), "f25".to_string());
        m.insert("fs10".to_string(), "f26".to_string());
        m.insert("fs11".to_string(), "f27".to_string());
        m.insert("ft8".to_string(), "f28".to_string());
        m.insert("ft9".to_string(), "f29".to_string());
        m.insert("ft10".to_string(), "f30".to_string());
        m.insert("ft11".to_string(), "f31".to_string());

        m.insert(TEMP_REGISTER_NAME.to_string(), TEMP_REGISTER_NAME.to_string()); // a temporary register name for switching two registers

        m
    };
}

/// if the name is alais like T0, T1
pub fn is_register_name_alias(reg_name: &str) -> bool {
    REGISTER_NAME_MAPPING.keys().find(|x| *x == reg_name).is_some()
}

/// if the name is like X0, X1
pub fn is_register_name_unique_name(reg_name:&str) -> bool {
    REGISTER_NAME_MAPPING.values().find(|x| *x == reg_name).is_some()
}

/// if the name is valid register name, e.g. either be alias or unique name
pub fn is_register_name(reg_name: &str) -> bool {
    is_register_name_alias(reg_name) || is_register_name_unique_name(reg_name)
}

/// get alias name from unique name
pub fn get_register_alias_name(reg_name: &str) -> Option<&str> {
    REGISTER_NAME_MAPPING.keys().find(|x| REGISTER_NAME_MAPPING.get(*x).unwrap() == reg_name).map(|x| x.as_str())
}

/// get unique name from alias name
pub fn get_register_unique_name(reg_name: &str) -> Option<&str> {
    REGISTER_NAME_MAPPING.get(reg_name).map(|x| x.as_str())
}

/// get register name from either alias or unique name and return unique name and alias name tuple
pub fn get_register_names(reg_name: &str) -> Option<(&str, &str)> {
    if is_register_name_alias(reg_name) {
        let unique_name = get_register_unique_name(reg_name).unwrap();
        Some((unique_name, reg_name))
    }
    else if is_register_name_unique_name(reg_name) {
        let alias_name = get_register_alias_name(reg_name).unwrap();
        Some((reg_name, alias_name))
    }
    else {
        None
    }
}

#[derive(Debug, Clone, Eq, Hash)]
pub struct RegisterUsage {
    /// register name
    reg_name: String,
    /// register alias,
    reg_alias: String,
    /// register usage
    is_in_use: bool,
    /// register value which store which variable
    reg_value : String, 
    /// register valvue is address
    is_reg_value_address: bool,
}

impl RegisterUsage {
    pub fn new(reg_name: &str) -> Self {
        let (reg_alias, unique_name) = if is_register_name_alias(reg_name) {
            (reg_name.to_string(), get_register_unique_name(reg_name).unwrap().to_string())
        }
        else {
            if let Some(reg_alias) = get_register_alias_name(reg_name) {
                (reg_alias.to_string(), reg_name.to_string())
            }
            else {
                panic!("cannot find {reg_name} alias");
            }
        };
        
        Self {
            reg_name : unique_name,
            reg_alias : reg_alias,
            is_in_use: false,
            reg_value: "".to_string(),
            is_reg_value_address: false,
        }
    }

    pub fn set_reg_value(&mut self, reg_value: &str) {
        self.reg_value = reg_value.to_string();
    }

    pub fn set_reg_value_address(&mut self, is_reg_value_address: bool) {
        self.is_reg_value_address = is_reg_value_address;
    }

    pub fn set_in_use(&mut self, is_in_use: bool) {
        self.is_in_use = is_in_use;
    }

    pub fn get_reg_value(&self) -> &str {
        &self.reg_value
    }

    pub fn get_reg_name(&self) -> &str {
        assert!(is_register_name(&self.reg_name));
        &self.reg_name
    }

    pub fn get_reg_alias(&self) -> &str {
        assert!(is_register_name_alias(&self.reg_alias));
        &self.reg_alias
    }

    pub fn is_reg_value_address(&self) -> bool {
        self.is_reg_value_address
    }

    pub fn is_in_use(&self) -> bool {
        self.is_in_use
    }

    /// find the memorized name of a register, like t0 or sp, use get_reg_name to find the name with x only
    pub fn get_register_memorized_name(&self, reg_name: &str) -> String {
        get_register_alias_name(reg_name).map_or(reg_name.to_string(), |v| v.to_string())
    }

    /// check if a register name is a temp register
    pub fn is_temp_register(&self) -> bool {
        let reg_name = self.get_reg_name();
        let name = self.get_register_memorized_name(reg_name);
        name!="tp" && (name.starts_with("t") || name.starts_with("ft"))
    }

    /// is paramter register, a0-a7, fa0-fa7
    pub fn is_parameter_register(&self) -> bool {
        let reg_name = self.get_reg_name();
        let name = self.get_register_memorized_name(reg_name);
        super::RegisterSet::get_riscv_parameter_register_names().contains(&name) || 
        super::RegisterSet::get_riscv_float_parameter_register_names().contains(&name)
    }

    /// check if regsiter is float register
    pub fn is_float_register(&self) -> bool {
        let reg_name = self.get_reg_name();
        if reg_name.starts_with("f") {
            return true;
        }
        
        let name = get_register_unique_name(reg_name);
        match name {
            Some(n) => n.starts_with("f"),
            None => false,
        }
    }

    /// check if regsiter is integer register
    pub fn is_integer_register(&self) -> bool {
        let reg_name = self.get_reg_name();
        if reg_name.starts_with("x") {
            return true;
        }

        let name = get_register_unique_name(reg_name);
        match name {
            Some(n) => n.starts_with("x"),
            None => false,
        }
    }

    /// is s register
    pub fn is_s_register(&self) -> bool {
        let reg_name = self.get_reg_name();
        let name = self.get_register_memorized_name(reg_name);
        (name.starts_with("s") && name != "sp") || name.starts_with("fs")
    }
}

impl PartialEq for RegisterUsage {
    fn eq(&self, other: &Self) -> bool {
        let self_name = get_register_unique_name(self.get_reg_name()).map_or(self.reg_name.to_string(), |x| x.to_string());
        let other_name = get_register_unique_name(other.get_reg_name()).map_or(other.reg_name.to_string(), |x| x.to_string());
        self_name == other_name
    }
}

impl From<String> for RegisterUsage {
    fn from(s: String) -> Self {
        RegisterUsage::new(s.as_str())
    }
}

impl From<&str> for RegisterUsage {
    fn from(s: &str) -> Self {
        RegisterUsage::new(s)
    }
}

/// test for register usage
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_int_register() {
        let reg = RegisterUsage::new("t0");
        assert!(reg.is_integer_register());
        assert!(!reg.is_float_register());
    }

    #[test]
    fn test_float_register() {
        let reg = RegisterUsage::new("ft0");
        assert!(reg.is_float_register());
        assert!(!reg.is_integer_register());
    }
}