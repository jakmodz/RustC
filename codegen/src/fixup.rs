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
        let constructs = program
            .constructs
            .into_iter()
            .map(|cons| match cons {
                AsmConstruct::Func(func) => AsmConstruct::Func(self.fixup_function(func)),
                other => other,
            })
            .collect();

        AsmProgram { constructs }
    }

    fn fixup_function(&self, mut function: AsmFunction) -> AsmFunction {
        let stack_size = self.stack_sizes.get(&function.name).copied().unwrap_or(0);
        let aligned_size = round_up_to_16(stack_size);

        if aligned_size > 0 {
            function
                .instructions
                .insert(0, Instruction::Allocate { size: aligned_size });
        }
        
        let mut new_instructions = Vec::new();
        for instr in function.instructions.into_iter() {
            new_instructions.extend(Self::fixup_instruction(instr));
        }
        function.instructions = new_instructions;

        function
    }

    fn is_memory_operand(operand: &Operand) -> bool {
        matches!(operand, Operand::Stack(_) | Operand::Data(_))
    }
    
    fn needs_rewrite(op1: &Operand, op2: &Operand) -> bool {
        let op1_is_mem = Self::is_memory_operand(op1);
        let op2_is_mem = Self::is_memory_operand(op2);
        
        if op1_is_mem && op2_is_mem {
            return true;
        }
        
        if matches!(op1, Operand::Imn(_)) && matches!(op2, Operand::Imn(_)) {
            return true;
        }
        
        false
    }

    fn fixup_instruction(instr: Instruction) -> Vec<Instruction> {
        let temp_reg = Operand::Reg(Register::R10);
        match instr {
            Instruction::Mov { src, dst } => {
                let is_reg_to_imm = matches!((&src, &dst), (Operand::Reg(_), Operand::Imn(_)));
                
                let needs_rewrite = Self::needs_rewrite(&src, &dst) || is_reg_to_imm;
                
                if needs_rewrite {
                    vec![
                        Instruction::Mov {
                            src,
                            dst: temp_reg.clone(),
                        },
                        Instruction::Mov {
                            src: temp_reg,
                            dst,
                        },
                    ]
                } else {
                    vec![Instruction::Mov { src, dst }]
                }
            }
            Instruction::Binary {
                binary_op,
                operand1,
                operand2,
            } => {
                if Self::needs_rewrite(&operand1, &operand2) {
                    vec![
                        Instruction::Mov {
                            src: operand1,
                            dst: temp_reg.clone(),
                        },
                        Instruction::Binary {
                            binary_op,
                            operand1: temp_reg,
                            operand2,
                        },
                    ]
                } else {
                    vec![Instruction::Binary {
                        binary_op,
                        operand1,
                        operand2,
                    }]
                }
            }
            Instruction::Cmp { operand1, operand2 } => {
                let needs_mem_rewrite = Self::needs_rewrite(&operand1, &operand2);

                if matches!(operand2, Operand::Imn(_)) {
                    vec![
                        Instruction::Mov {
                            src: operand2,
                            dst: temp_reg.clone(),
                        },
                        Instruction::Cmp {
                            operand1,
                            operand2: temp_reg,
                        },
                    ]
                } else if needs_mem_rewrite {
                    vec![
                        Instruction::Mov {
                            src: operand1,
                            dst: temp_reg.clone(),
                        },
                        Instruction::Cmp {
                            operand1: temp_reg,
                            operand2,
                        },
                    ]
                } else {
                    vec![Instruction::Cmp { operand1, operand2 }]
                }
            }
            other => vec![other],
        }
    }
}

fn round_up_to_16(size: usize) -> usize {
    if size == 0 {
        0
    } else {
        ((size + 15) / 16) * 16
    }
}
