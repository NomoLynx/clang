#[derive(Debug, Clone, PartialEq)]
pub struct MarkdownData {
    data : String,
    var_name: String,

    /// this file sequence is to set the order when multiple markdown files are used
    sequence_id : usize,    
}

impl MarkdownData {
    pub fn new(data:String, var_name:String) -> MarkdownData {
        MarkdownData {
            data,
            var_name,
            sequence_id: 0,
        }
    }

    pub fn get_var_name(&self) -> &str {
        &self.var_name
    }

    pub fn get_data(&self) -> &str {
        &self.data
    }

    pub fn get_sequence_id(&self) -> usize {
        self.sequence_id
    }
}

impl Default for MarkdownData {
    fn default() -> Self {
        MarkdownData {
            data: String::new(),
            var_name: String::new(),
            sequence_id: 0,
        }
    }
}
