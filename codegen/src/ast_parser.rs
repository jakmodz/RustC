use crate::asm_ast::*;
use tacky::tacky::{BinaryOp, Val};
use tacky::tacky::{TackyInstruction, UnaryOp};

pub struct AsmParser {}

impl AsmParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&mut self, program: tacky::tacky::Program) -> AsmProgram {
        todo!();
        let mut instructions = Vec::new();
        for func in program.functions.iter_mut() {
            for ins in func.body {
                self.convert_instruction(ins, &mut instructions);
            }
        }

        AsmProgram {
            function: AsmFunction {
                name:  "main".to_string(),
                instructions,
            },
        }
    }

    fn convert_instruction(&mut self, ins: TackyInstruction, instructions: &mut Vec<Instruction>) {
        match ins {
            TackyInstruction::Return(val) => {
                let ins = Instruction::Mov {
                    src: self.convert_val(val),
                    dst: Operand::Reg(Register::AX),
                };
                instructions.push(ins);
                instructions.push(Instruction::Ret);
            }
            TackyInstruction::Unary { unary_op, src, dst } => match unary_op {
                UnaryOp::Negate | UnaryOp::Complement => {
                    instructions.push(Instruction::Mov {
                        src: self.convert_val(src),
                        dst: self.convert_val(dst.clone()),
                    });
                    instructions.push(Instruction::Unary {
                        op: convert_unary_op(unary_op),
                        operand: self.convert_val(dst),
                    });
                }
                UnaryOp::Not => {
                    instructions.push(Instruction::Cmp {
                        operand1: Operand::Imn(0),
                        operand2: self.convert_val(src.clone()),
                    });
                    instructions.push(Instruction::Mov {
                        src: Operand::Imn(0),
                        dst: self.convert_val(dst.clone()),
                    });
                    instructions.push(Instruction::SetCc {
                        cond_code: ConditionCode::Equal,
                        operand: self.convert_val(dst.clone()),
                    });
                }
            },
            TackyInstruction::Binary {
                binary_op,
                dst,
                src2,
                src1,
            } => {
                if src1 != dst {
                    instructions.push(Instruction::Mov {
                        src: self.convert_val(src1.clone()),
                        dst: self.convert_val(dst.clone()),
                    });
                }

                if binary_op.is_comparison() {
                    instructions.push(Instruction::Cmp {
                        operand1: self.convert_val(src2.clone()),
                        operand2: self.convert_val(src1.clone()),
                    });
                    instructions.push(Instruction::Mov {
                        src: Operand::Imn(0),
                        dst: self.convert_val(dst.clone()),
                    });
                    instructions.push(Instruction::SetCc {
                        cond_code: convert_condition_code(binary_op),
                        operand: self.convert_val(dst.clone()),
                    });
                    return;
                }

                match binary_op {
                    BinaryOp::Divide => {
                        instructions.push(Instruction::Mov {
                            src: self.convert_val(src1.clone()),
                            dst: Operand::Reg(Register::AX),
                        });
                        instructions.push(Instruction::Cdq);
                        instructions.push(Instruction::IDiv {
                            operand: self.convert_val(src2),
                        });
                        instructions.push(Instruction::Mov {
                            src: Operand::Reg(Register::AX),
                            dst: self.convert_val(dst),
                        });
                    }
                    BinaryOp::Modulo => {
                        instructions.push(Instruction::Mov {
                            src: self.convert_val(src1.clone()),
                            dst: Operand::Reg(Register::AX),
                        });
                        instructions.push(Instruction::Cdq);
                        instructions.push(Instruction::IDiv {
                            operand: self.convert_val(src2),
                        });
                        instructions.push(Instruction::Mov {
                            src: Operand::Reg(Register::DX),
                            dst: self.convert_val(dst),
                        });
                    }
                    _ => instructions.push(Instruction::Binary {
                        binary_op: convert_binary_op(binary_op),
                        operand1: self.convert_val(src2),
                        operand2: self.convert_val(dst),
                    }),
                }
            }
            TackyInstruction::JumpIfZero { target, cond } => {
                instructions.push(Instruction::Cmp {
                    operand1: Operand::Imn(0),
                    operand2: self.convert_val(cond),
                });
                instructions.push(Instruction::JmpCc {
                    cond_code: ConditionCode::Equal,
                    identifier: target,
                });
            }
            TackyInstruction::JumpIfNotZero { target, cond } => {
                instructions.push(Instruction::Cmp {
                    operand1: Operand::Imn(0),
                    operand2: self.convert_val(cond),
                });
                instructions.push(Instruction::JmpCc {
                    cond_code: ConditionCode::NotEqual,
                    identifier: target,
                });
            }
            TackyInstruction::Label(label) => {
                instructions.push(Instruction::Label { identifier: label });
            }

            TackyInstruction::Jump { target } => {
                instructions.push(Instruction::Jmp { identifier: target });
            }
            TackyInstruction::Copy { dst, src } => instructions.push(Instruction::Mov {
                src: self.convert_val(src),
                dst: self.convert_val(dst),
            }),
        }
    }

    fn convert_val(&mut self, val: Val) -> Operand {
        match val {
            Val::Constant(c) => Operand::Imn(c),
            Val::Var(name) => Operand::Pseudo(name),
        }
    }
}
impl Default for AsmParser {
    fn default() -> Self {
        Self::new()
    }
}
