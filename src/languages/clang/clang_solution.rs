use crate::languages::clang::clang_program::CLangProgram;

#[derive(Debug, Clone)]
pub struct CLangSolution {
    programs : Vec<CLangProgram>,
}

impl CLangSolution {
    pub fn new(programs:Vec<CLangProgram>) -> Self {
        Self { programs }
    }

    pub fn get_programs(&self) -> &Vec<CLangProgram> {
        &self.programs
    }

    pub fn from_config_and_main_program(config:&mut crate::languages::clang::clang_asm_code_gen::code_gen_context::CodeGenContext, main_program:CLangProgram) -> Self {
        let mut programs = vec![main_program];
        programs.append(config.get_included_clang_programs_mut());
        Self::new(programs)
    }

    /// merge all CLangPrograms into the CLangProgram which has entry function 'main'
    /// if no or multiple 'main' functions exist, use the first CLangProgram as the merged result
    pub fn to_one_clang_program(mut self) -> CLangProgram {
        let mut main_program_index:Option<usize> = None;
        for (i, program) in self.get_programs().iter().enumerate() {
            if program.is_entry_point_exists() {
                main_program_index = Some(i);
                break;
            }
        }

        let mut merged_program = if let Some(index) = main_program_index {
            self.programs.remove(index)
        } else {
            self.programs.remove(0)
        };

        for n in self.programs.into_iter() {
            merged_program <<= n;        
        }

        merged_program
    }
}

impl Default for CLangSolution {
    fn default() -> Self {
        Self::new(Vec::default())
    }
}

impl From<Vec<CLangProgram>> for CLangSolution {
    fn from(programs:Vec<CLangProgram>) -> Self {
        Self::new(programs)
    }
}