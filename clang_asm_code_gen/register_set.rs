use std::collections::HashSet;
use std::ops::{Add, AddAssign, Index, Mul, Sub};

use crate::languages::clang::clang_asm_code_gen::register_usage::RegisterUsage;
use crate::languages::clang::clang_types::CLangTypes;

#[derive(Debug, Clone)]
pub enum RegisterSetReplacementItemAction {
    Replace,   //replace old_reg with new_reg
    Switch,    //switch old_reg with new_reg, and new_reg with old_reg
}

#[derive(Debug, Clone)]
pub struct RegisterSetReplacementItem {
    pub old_reg: String,
    pub new_reg: String,
    pub action: RegisterSetReplacementItemAction,
}

impl RegisterSetReplacementItem {
    pub fn new(old_reg: String, new_reg: String, action: RegisterSetReplacementItemAction) -> Self {
        Self {
            old_reg,
            new_reg,
            action,
        }
    }

    pub fn to_tuple(&self) -> (String, String) {
        (self.old_reg.clone(), self.new_reg.clone())
    }

    /// Check if the action is Switch
    pub fn is_replace_action(&self) -> bool {
        matches!(self.action, RegisterSetReplacementItemAction::Replace)
    }
}

impl Default for RegisterSetReplacementItem {
    fn default() -> Self {
        Self {
            old_reg: String::default(),
            new_reg: String::default(),
            action: RegisterSetReplacementItemAction::Replace,
        }
    }
}

impl From<(String, String)> for RegisterSetReplacementItem {
    fn from(value: (String, String)) -> Self {
        Self::new(value.0, value.1, RegisterSetReplacementItemAction::Replace)
    }
}

#[derive(Debug, Clone)]
pub struct RegisterSet(Vec<RegisterUsage>);

impl RegisterSet {
    pub fn get_float_registers(&self) -> RegisterSet {
        let v: Vec<RegisterUsage> = self.0.iter()
            .filter(|x| x.is_float_register())
            .map(|x| x.clone())
            .collect();
        RegisterSet(v)
    }

    pub fn get_int_registers(&self) -> RegisterSet {
        let v: Vec<RegisterUsage> = self.0.iter()
            .filter(|x| x.is_integer_register())
            .map(|x| x.clone())
            .collect();
        RegisterSet(v)
    }
    
    pub fn get_s_registers(&self) -> RegisterSet {
        let v: Vec<RegisterUsage> = self.0.iter()
            .filter(|x| x.is_s_register())
            .map(|x| x.clone())
            .collect();
        RegisterSet(v)
    }

    /// Remove registers from the set by their names or aliases
    pub fn remove_regs(&mut self, regs:&Vec<String>) {
        self.0.retain(|x| !regs.contains(&x.get_reg_name().to_string()));
        self.0.retain(|x| !regs.contains(&x.get_reg_alias().to_string()));
    }

    pub fn get_temp_registers(&self) -> RegisterSet {
        let v: Vec<RegisterUsage> = self.0.iter()
            .filter(|x| x.is_temp_register())
            .map(|x| x.clone())
            .collect();
        RegisterSet(v)
    }

    pub fn get_int_float_registers(&self) -> (RegisterSet, RegisterSet) {
        let (int_regs, float_regs) = (self.get_int_registers(), self.get_float_registers());
        (int_regs, float_regs)
    }

    pub fn is_integer_register_set(&self) -> bool {
        self.0.iter().all(|x| x.is_integer_register())
    }

    pub fn is_float_register_set(&self) -> bool {
        self.0.iter().all(|x| x.is_float_register())
    }

    pub fn get_registers(&self) -> Vec<&RegisterUsage> {
        self.0.iter().collect()
    }

    pub fn get_sorted_reg_names(&self) -> Vec<String> {
        let mut names: Vec<String> = self.0.iter()
            .map(|x| x.get_reg_name().to_string())
            .collect();
        names.sort();
        names
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn insert(&mut self, value:RegisterUsage) {
        self.0.push(value)
    }

    pub fn to_hashset(&self) -> HashSet<RegisterUsage> {
        self.0.iter().map(|x| x.clone())
            .collect::<HashSet<_>>()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn set_values(&mut self, v:&Vec<RegisterUsage>) {
        self.0 = v.iter().map(|x| x.clone()).collect();
    }

    pub fn new_from_vec(v:&Vec<RegisterUsage>) -> Self {
        let mut r = Self::default();
        r.set_values(v);
        r
    }

    pub fn new_from_string_vec(v:&Vec<String>) -> Self {
        let mut r = Self::default();
        r.0 = v.iter().map(|x| RegisterUsage::new(x)).collect();
        r
    }

    pub fn replace_by_tuples(&mut self, replacements:&Vec<(String, String)>) {
        self.0.iter_mut().for_each(|x| { 
            let y = replacements.iter().find(|(a, _)| a==x.get_reg_name());
            if let Some((_,b)) = y {
                *x = RegisterUsage::new(b);
            }
        });
    }

    pub fn has_conflict(&self, other:&RegisterSet) -> bool {
        self.to_hashset().intersection(&other.to_hashset()).count() > 0
    }

    pub fn get_register_alias_name(&self) -> Vec<String> {
        self.0.iter().map(|x| x.get_reg_alias().to_string()).collect()
    }

    pub fn zip_with(&self, other:&RegisterSet) -> Vec<(String, String)> {
        self.get_register_alias_name().into_iter()
            .zip(other.get_register_alias_name().into_iter())
            .collect::<Vec<_>>()
    }

    /// Get two registersets (A, B) as input
    /// return a vector of tuples of register names (a, b) where a is from A and b is from B, and a != b
    pub fn get_non_conflicting_pairs(&self, other:&RegisterSet) -> Vec<(String, String)> {
        let mut r = Vec::new();
        let mut other_register_aliases = other.get_register_alias_name();

        for n in self.get_register_alias_name().into_iter() {
            if let Some(pos) = other_register_aliases.iter().position(|x| x != &n) {
                let other_reg_name = other_register_aliases.remove(pos); // use swap_remove(pos) if order does not matter
                r.push((n, other_reg_name));
            } else {
                panic!(
                    "get_non_conflicting_pairs: no non-conflicting register found for register '{}'",
                    n
                );
            }
        }

        r
    }

    /// Move away registers in self that conflict with registers in b, considering fixed_regs that must not be moved.
    /// protected_regs are registers that must not be used as spill/replacement slots either.
    /// Returns (replacements for self's registers, registers that must be push/popped instead).
    /// - conflicted registers that have a free slot get a rename replacement
    /// - conflicted registers without a free slot are returned as spillover (caller must push/pop them)
    pub fn move_away_from(&self, b: &Self, available_set: &Self, fixed_regs: &Self, protected_regs: &Self) -> (Vec<RegisterSetReplacementItem>, Self) {
        let conflicted_regs = self.clone() * b.clone() - fixed_regs.clone();
        if conflicted_regs.is_empty() {
            return (vec![], Self::default());
        }

        let mut free_regs = available_set.clone() - b.clone() - self.clone() - fixed_regs.clone() - protected_regs.clone();
        let mut reg_to_push = Self::default();
        if conflicted_regs.len() > free_regs.len() {
            reg_to_push = conflicted_regs.clone() - free_regs.clone();
            free_regs += reg_to_push.clone();
        }

        let mut replacements = conflicted_regs.get_register_alias_name().into_iter()
            .zip(free_regs.get_register_alias_name().into_iter())
            .filter(|(m,n)| m != n)  // only keep replacements where the names differ
            .map(|x| x.into())
            .collect::<Vec<_>>();

        // check if reg_to_push contains any protected registers
        let protected_conflicts = reg_to_push.clone() * protected_regs.clone();
        if !protected_conflicts.is_empty() {
            // If there are protected registers in reg_to_push, we need to handle this case.
            // candidate set does not need exclude reg_to_push, reg_to_push should be pushed to stack. it means registers in
            // reg_to_push is protected and can be used freely between push-pop range.
            let mut switch_candidate_regset = self.clone() - fixed_regs.clone() - protected_regs.clone();
            switch_candidate_regset.retain_only_temp_registers(); // only keep temp registers for switching 
            
            if protected_conflicts.len() > switch_candidate_regset.len() {
                panic!("Not enough candidate registers to switch with protected registers. Protected conflicts: {:?}, Switch candidates: {:?}", protected_conflicts, switch_candidate_regset);
            }

            protected_conflicts.get_non_conflicting_pairs(&switch_candidate_regset)
                .iter()
                .for_each(|(old, new)| {
                    replacements.push(RegisterSetReplacementItem::new(old.clone(), new.clone(), RegisterSetReplacementItemAction::Switch));
                });
        }

        (replacements, reg_to_push)
    }

    fn retain_only_temp_registers(&mut self) {
        self.0.retain(|x| x.is_temp_register());
    }

    /// Get available RISC-V register names (t0-t6, s0-s11, ft0-ft11, fs0-fs11)
    pub fn get_riscv_available_register_names() -> Vec<String> {
        // Generate "t0" to "t6"
        let t_range: Vec<String> = (0..=6).map(|i| format!("t{}", i)).collect();

        // Generate "s0" to "s11"
        let s_range: Vec<String> = (0..=11).map(|i| format!("s{}", i)).collect();

        // generate "ft0" to "ft11"
        let ft_range: Vec<String> = (0..=11).map(|i| format!("ft{}", i)).collect();

        // generate "fs0" to "fs11"
        let fs_range: Vec<String> = (0..=11).map(|i| format!("fs{}", i)).collect();

        // Combine all ranges into a single vector
        let mut available_registers = Vec::new();
        available_registers.extend(t_range);
        available_registers.extend(s_range);
        available_registers.extend(ft_range);
        available_registers.extend(fs_range);
        available_registers
    }

    /// get parameter register names according to RISC-V calling convention, a0-a7
    pub fn get_riscv_parameter_register_names() -> Vec<String> {
        // Generate "a0" to "a7"
        let a_range: Vec<String> = (0..=7).map(|i| format!("a{}", i)).collect();
        a_range
    }

    /// get float parameter register names according to RISC-V calling convention, fa0-fa7
    pub fn get_riscv_float_parameter_register_names() -> Vec<String> {
        // Generate "fa0" to "fa7"
        let fa_range: Vec<String> = (0..=7).map(|i| format!("fa{}", i)).collect();
        fa_range
    }

    /// return suitable register set for given type
    pub fn get_registerset_by_type(&self, ty:&CLangTypes) -> RegisterSet {
        match ty {
            _ if ty.is_floating_point_type() => self.get_float_registers(),
            _ if ty.is_pointer() || ty.is_struct() || ty.is_union() || ty.is_functionpointer() => 
                self.get_int_registers(),
            _ if ty.is_integer_type() => self.get_int_registers(),
            _ if ty.is_char() || ty.is_uchar() => self.get_int_registers(),
            _ if ty.is_enum() => self.get_int_registers(),
            _ if ty.is_typealias() => {
                if let Some(real_type) = ty.get_type_alias_real_type() {
                    self.get_registerset_by_type(real_type)
                }
                else {
                    let err_msg = format!("'{ty}' is type alias/typedef and should have real type defined.");
                    panic!("{}", err_msg)
                }
            }
            _ => { 
                panic!("{}", format!("get_registerset_by_type: unsupported type '{}', return empty register set", ty));
            }
        }
    }

    /// return register names as vector of strings
    pub fn to_name_vec(&self) -> Vec<String> {
        self.0.iter().map(|x| x.get_reg_name().to_string()).collect()
    }

    /// return register aliases as vector of strings
    pub fn to_alias_vec(&self) -> Vec<String> {
        self.0.iter().map(|x| x.get_reg_alias().to_string()).collect()
    }
}

// Implement the Add trait for union
impl Add for RegisterSet {
    type Output = RegisterSet;

    fn add(self, other: RegisterSet) -> RegisterSet {
        let mut r = self.0;
        for item in other.0 {
            if !r.contains(&item) {
                r.push(item);
            }
        }
        RegisterSet(r)
    }
}

// Implement the Sub trait for difference
impl Sub for RegisterSet {
    type Output = RegisterSet;

    fn sub(self, other: RegisterSet) -> RegisterSet {
        let mut r = Vec::new();
        for item in self.0 {
            if !other.0.contains(&item) {
                r.push(item);
            }
        }
        RegisterSet(r)
    }
}

// Implement the Mul trait for intersection
impl Mul for RegisterSet {
    type Output = RegisterSet;

    fn mul(self, other: RegisterSet) -> RegisterSet {
        let mut r = Vec::new();
        for item in self.0 {
            if other.0.contains(&item) {
                r.push(item);
            }
        }
        RegisterSet(r)
    }
}

// Implement AddAssign trait for RegisterSet
impl AddAssign for RegisterSet {
    fn add_assign(&mut self, other: RegisterSet) {
        for item in other.0 {
            if !self.0.contains(&item) {
                self.0.push(item);
            }
        }
    }
}

// Implement Index trait for RegisterSet
impl Index<usize> for RegisterSet {
    type Output = RegisterUsage;

    fn index(&self, index: usize) -> &Self::Output {
        // Get the element at the requested index
        &self.0[index]
    }
}

impl Default for RegisterSet {
    fn default() -> Self {
        Self(Vec::default())
    }
}

impl PartialEq for RegisterSet {
    fn eq(&self, other: &Self) -> bool {
        self.to_hashset() == other.to_hashset()
    }
}

impl Eq for RegisterSet {}

impl From<Vec<RegisterUsage>> for RegisterSet {
    fn from(v: Vec<RegisterUsage>) -> Self {
        Self(v)
    }
}

impl From<Vec<String>> for RegisterSet {
    fn from(v: Vec<String>) -> Self {
        (&v).into()
    }
}

impl From<[String; 0]> for RegisterSet {
    fn from(v: [String; 0]) -> Self {
        (&v.to_vec()).into()
    }
}

impl From<&Vec<String>> for RegisterSet {
    fn from(value: &Vec<String>) -> Self {
        Self::new_from_string_vec(value)
    }
}

impl From<&str> for RegisterSet {
    fn from(value: &str) -> Self {
        let v = value.split(',').map(|s| s.trim().to_string()).collect::<Vec<_>>();
        Self::new_from_string_vec(&v)
    }
}

/// test for register set
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_set_operations() {
        let reg1 = RegisterUsage::new("t0");
        let reg2 = RegisterUsage::new("t1");
        let reg3 = RegisterUsage::new("s0");

        let set_a = RegisterSet::from(vec![reg1.clone(), reg2.clone()]);
        let set_b = RegisterSet::from(vec![reg2.clone(), reg3.clone()]);

        // Test union
        let union_set = set_a.clone() + set_b.clone();
        assert_eq!(union_set.len(), 3);

        // Test difference
        let diff_set = set_a.clone() - set_b.clone();
        assert_eq!(diff_set.len(), 1);
        assert_eq!(diff_set[0], reg1);

        // Test intersection
        let inter_set = set_a.clone() * set_b.clone();
        assert_eq!(inter_set.len(), 1);
        assert_eq!(inter_set[0], reg2);
    }

    #[test]
    fn test_register_set_equality() {
        let reg1 = RegisterUsage::new("t0");
        let reg2 = RegisterUsage::new("t1");

        let set_a = RegisterSet::from(vec![reg1.clone(), reg2.clone()]);
        let set_b = RegisterSet::from(vec![reg2.clone(), reg1.clone()]);

        assert_eq!(set_a, set_b);
    }

    #[test]
    fn test_float_integer_split() {
        let reg1 = RegisterUsage::new("t0"); // integer
        let reg2 = RegisterUsage::new("ft0"); // float
        let reg3 = RegisterUsage::new("s0"); // integer
        let reg4 = RegisterUsage::new("fs1"); // float

        let register_set = RegisterSet::from(vec![reg1, reg2, reg3, reg4]);

        let int_set = register_set.get_int_registers();
        let float_set = register_set.get_float_registers();

        assert_eq!(int_set.len(), 2);
        assert_eq!(float_set.len(), 2);
    }

    #[test]
    fn move_away_from_does_not_use_protected_registers() {
        let self_regs = RegisterSet::from(vec!["t0".to_string(), "t1".to_string()]);
        let conflicted = RegisterSet::from(vec!["t0".to_string(), "t1".to_string()]);
        let available = RegisterSet::from(vec!["t0".to_string(), "t1".to_string(), "t2".to_string()]);
        let fixed = RegisterSet::default();
        let protected = RegisterSet::from(vec!["t0".to_string()]);

        let (replacements, reg_to_push) = self_regs.move_away_from(&conflicted, &available, &fixed, &protected);

        assert!(!reg_to_push.get_register_alias_name().contains(&"t0".to_string()), 
                "protected output register was selected for push/pop: {reg_to_push:?}");
        assert!(replacements.iter().all(| x | x.old_reg != "t0" && x.new_reg != "t0"), 
                "protected output register was used in replacements: {replacements:?}");
    }
}