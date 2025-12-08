use crate::asm_ast::*;
use std::collections::HashMap;

pub struct InstructionFixup {
    stack_sizes: HashMap<String, usize>,
}

impl InstructionFixup {
    pub fn new(stack_sizes: HashMap<String, usize>) -> Self {
        Self { stack_sizes }
    }

    pub fn fixup(self, program: AsmProgram) -> AsmProgram {
        let functions = program
            .functions
            .into_iter()
            .map(|func| self.fixup_function(func))
            .collect();

        AsmProgram { functions }
    }

    fn fixup_function(&self, mut function: AsmFunction) -> AsmFunction {
        let stack_size = self.stack_sizes.get(&function.name).copied().unwrap_or(0);
        let aligned_size = round_up_to_16(stack_size);

        if aligned_size > 0 {
            function
                .instructions
                .insert(0, Instruction::Allocate { size: aligned_size });
        }

        function
    }
}

fn round_up_to_16(size: usize) -> usize {
    if size == 0 {
        0
    } else {
        ((size + 15) / 16) * 16
    }
}
