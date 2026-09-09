use std::{fmt::{Display, Formatter}, ops::Index};

use core_utils::number::get_u64_from_str;
use crate::languages::clang::CLangItem;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayDimension (Vec<usize>);

impl ArrayDimension {
    pub fn new(dimensions:Vec<usize>) -> Self {
        Self (dimensions)
    }

    pub fn get_dimensions_values(&self) -> &Vec<usize> {
        &self.0
    }

    pub fn get_dimension_count(&self) -> usize {
        self.0.len()
    }

    pub fn get_total_elements(&self) -> usize {
        self.0.iter().product()
    }

    /// Convert a flat index into an N-dimensional index tuple
    fn index_at(&self, mut flat: usize) -> Vec<usize> {
        let mut idx = vec![0; self.0.len()];
        for i in (0..self.0.len()).rev() {
            idx[i] = flat % self.0[i];
            flat /= self.0[i];
        }
        idx
    }

    /// Get an iterator of all possible N-dimensional index tuples for the array dimensions
    pub fn clang_indices(&self) -> impl Iterator<Item = Vec<usize>> + '_ {
        (0..self.get_total_elements())
            .map(|flat| self.index_at(flat))
    }

    /// Get an iterator of all possible N-dimensional index tuples as strings for the array dimensions
    pub fn clang_indices_string(&self) -> Vec<Vec<String>> {
        self.clang_indices()
            .map(|idx| idx.iter().map(|x| x.to_string()).collect::<Vec<_>>())
            .collect()
    }
}

impl Index<usize> for ArrayDimension {
    type Output = usize;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl Display for ArrayDimension {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let dimensions_str = self.0.iter().map(|d| d.to_string()).collect::<Vec<_>>().join(", ");
        write!(f, "[{}]", dimensions_str)
    }
}

impl From<Vec<usize>> for ArrayDimension {
    fn from(v:Vec<usize>) -> Self {
        Self::new(v)
    }
}

impl From<Vec<String>> for ArrayDimension {
    fn from(value: Vec<String>) -> Self {
        let dimensions = value.into_iter()
            .map(|s| get_u64_from_str(&s).unwrap() as usize)
            .collect::<Vec<_>>();
        dimensions.into()
    }
}

impl From<Vec<(CLangItem,)>> for ArrayDimension {
    fn from(value: Vec<(CLangItem,)>) -> Self {
        let mut dimension = vec![];
        for (item,) in value.into_iter() {
            if let Some(atom) = item.get_atom_item() {
                if let Some(val) = atom.get_string_value() {
                    dimension.push(get_u64_from_str(&val).unwrap() as usize);
                }
            }
        }
        
        dimension.into()
    }
}