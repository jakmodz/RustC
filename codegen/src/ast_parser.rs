use crate::asm_ast::*;
use lazy_static::lazy_static;
use tacky::tacky::{BinaryOp, Val};
use tacky::tacky::{TackyInstruction, UnaryOp};

lazy_static! {
    pub static ref REGISTER_FOR_CALLING: [Register; 6] = [
        Register::DI,
        Register::SI,
        Register::DX,
        Register::CX,
        Register::R8,
        Register::R9,
    ];
}

pub struct AsmParser {}

impl AsmParser {
    pub fn new() -> Self {
        Self {}
    }

    pub fn parse(&mut self, program: tacky::tacky::Program) -> AsmProgram {
        let mut functions = Vec::new();

        for func in program.functions.iter() {
            let mut instructions = Vec::new();

            for (i, param) in func.params.iter().enumerate() {
                if i < 6 {
                    let reg = REGISTER_FOR_CALLING[i].clone();
                    instructions.push(Instruction::Mov {
                        src: Operand::Reg(reg),
                        dst: self.convert_val(param.clone()),
                    });
                } else {
                    let stack_offset = 16 + ((i - 6) * 8) as i64;
                    instructions.push(Instruction::Mov {
                        src: Operand::Stack(stack_offset),
                        dst: self.convert_val(param.clone()),
                    });
                }
            }
            for ins in func.body.iter() {
                self.convert_instruction(ins.clone(), &mut instructions);
            }
            functions.push(AsmFunction {
                name: func.name.clone(),
                instructions,
            });
        }

        AsmProgram { functions }
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
            TackyInstruction::FnCall { fn_name, args, dst } => {
                let reg_args_count = args.len().min(6);
                let stack_args_count = if args.len() > 6 { args.len() - 6 } else { 0 };
                let stack_padding = if stack_args_count % 2 == 1 { 8 } else { 0 };
                if stack_padding != 0 {
                    instructions.push(Instruction::Allocate {
                        size: stack_padding,
                    });
                }
                for (i, item) in args.iter().take(reg_args_count).enumerate() {
                    let r = REGISTER_FOR_CALLING[i].clone();
                    let asm_arg = self.convert_val(item.clone());
                    instructions.push(Instruction::Mov {
                        src: asm_arg,
                        dst: Operand::Reg(r),
                    });
                }
                for item in args.iter().skip(6).rev() {
                    let asm_arg = self.convert_val(item.clone());

                    instructions.push(Instruction::Mov {
                        src: asm_arg,
                        dst: Operand::Reg(Register::R10),
                    });
                    instructions.push(Instruction::Push {
                        operand: Operand::Reg(Register::R10),
                    });
                }
                instructions.push(Instruction::Call { name: fn_name });
                let bytes_to_remove = 8 * stack_args_count + stack_padding;
                if bytes_to_remove != 0 {
                    instructions.push(Instruction::Deallocate {
                        size: bytes_to_remove,
                    });
                }
                let asm_dst = self.convert_val(dst);
                instructions.push(Instruction::Mov {
                    src: Operand::Reg(Register::AX),
                    dst: asm_dst,
                });
            }
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
