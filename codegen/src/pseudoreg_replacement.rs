use crate::asm_ast::*;
use std::collections::HashMap;

pub struct PseudoregReplacement {
    var_map: HashMap<String, i64>,
    current_offset: i64,
}

impl PseudoregReplacement {
    pub fn new() -> Self {
        Self {
            var_map: HashMap::new(),
            current_offset: 0,
        }
    }

    pub fn replace(&mut self, program: AsmProgram) -> (AsmProgram, HashMap<String, usize>) {
        let mut stack_sizes = HashMap::new();
        let mut processed_functions = Vec::new();

        for function in program.functions {
            self.var_map.clear();
            self.current_offset = 0;

            let instructions = function
                .instructions
                .into_iter()
                .map(|instr| self.replace_in_instruction(instr))
                .collect();

            let stack_size = self.current_offset.abs() as usize;
            stack_sizes.insert(function.name.clone(), stack_size);

            processed_functions.push(AsmFunction {
                name: function.name,
                instructions,
            });
        }

        (
            AsmProgram {
                functions: processed_functions,
            },
            stack_sizes,
        )
    }

    fn replace_in_instruction(&mut self, instruction: Instruction) -> Instruction {
        match instruction {
            Instruction::Mov { src, dst } => Instruction::Mov {
                src: self.replace_in_operand(src),
                dst: self.replace_in_operand(dst),
            },
            Instruction::Unary { op, operand } => Instruction::Unary {
                op,
                operand: self.replace_in_operand(operand),
            },
            Instruction::Binary {
                binary_op,
                operand1,
                operand2,
            } => Instruction::Binary {
                binary_op,
                operand1: self.replace_in_operand(operand1),
                operand2: self.replace_in_operand(operand2),
            },
            Instruction::IDiv { operand } => Instruction::IDiv {
                operand: self.replace_in_operand(operand),
            },
            Instruction::Cmp { operand1, operand2 } => Instruction::Cmp {
                operand1: self.replace_in_operand(operand1),
                operand2: self.replace_in_operand(operand2),
            },
            Instruction::SetCc { cond_code, operand } => Instruction::SetCc {
                cond_code,
                operand: self.replace_in_operand(operand),
            },
            Instruction::Push { operand } => Instruction::Push {
                operand: self.replace_in_operand(operand),
            },
            Instruction::Ret
            | Instruction::Cdq
            | Instruction::Jmp { .. }
            | Instruction::JmpCc { .. }
            | Instruction::Label { .. }
            | Instruction::Allocate { .. }
            | Instruction::Deallocate { .. }
            | Instruction::Call { .. } => instruction,
        }
    }

    fn replace_in_operand(&mut self, operand: Operand) -> Operand {
        match operand {
            Operand::Pseudo(name) => {
                let offset = if let Some(&existing_offset) = self.var_map.get(&name) {
                    existing_offset
                } else {
                    self.current_offset -= 4;
                    self.var_map.insert(name.clone(), self.current_offset);
                    self.current_offset
                };
                Operand::Stack(offset)
            }
            other => other,
        }
    }
}

impl Default for PseudoregReplacement {
    fn default() -> Self {
        Self::new()
    }
}
