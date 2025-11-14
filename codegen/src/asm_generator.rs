use crate::asm_ast::*;
use std::collections::HashMap;
use std::io::{Result, Write};

pub struct AsmGenerator {
    pub vars: HashMap<String, i64>,
    stack_offset: i64,
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            vars: HashMap::new(),
            stack_offset: 0,
        }
    }

    fn validate_operands(&self, src: &Operand, dst: &Operand) -> bool {
        !matches!(
            (src, dst),
            (Operand::Imn(_), Operand::Imn(_))
                | (Operand::Stack(_), Operand::Stack(_))
                | (Operand::Pseudo(_), Operand::Pseudo(_))
                | (Operand::Stack(_), Operand::Pseudo(_))
                | (Operand::Pseudo(_), Operand::Stack(_))
                | (Operand::Reg(_), Operand::Imn(_))
        )
    }

    pub fn write(&mut self, program: AsmProgram, mut outputs: Vec<Box< dyn Write>>) -> Result<()> {
        for out in outputs.iter_mut() {
            self.write_function(out, &program.function)?;
            writeln!(out, ".section .note.GNU-stack,\"\",@progbits")?;
        }
        Ok(())
    }

    fn write_function(&mut self, out: &mut dyn Write, function: &AsmFunction) -> Result<()> {
        writeln!(out, "\t.globl {}\n{}:", function.name, function.name)?;
        writeln!(out, "\tpushq\t%rbp\n\tmovq\t%rsp, %rbp")?;

        for instr in &function.instructions {
            self.write_instruction(out, instr)?;
        }
        Ok(())
    }

    fn write_instruction(&mut self, out: &mut dyn Write, instr: &Instruction) -> Result<()> {
        match instr {
            Instruction::Mov { src, dst } => self.write_mov(out, src, dst)?,
            Instruction::Ret => writeln!(out, "\tmovq\t%rbp, %rsp\n\tpopq\t%rbp\n\tret")?,
            Instruction::Unary { op, operand } => {
                let operand = self.operand_str(&operand.clone());
                writeln!(out, "\t{} {}", self.unary_opcode(op),operand )?;
            }
            Instruction::Allocate { size } => {
                writeln!(out, "\tsubq\t${}, %rsp", size)?;
                self.stack_offset += *size as i64;
            }
            Instruction::Binary { binary_op, operand1, operand2 } => {
                self.write_binary(out, binary_op, operand1, operand2)?
            }
            Instruction::IDiv { operand } => {
                let op = self.move_to_reg(out, operand, Register::R11)?;
                writeln!(out, "\tidivl\t{}", self.operand_str(&op))?;
            }
            Instruction::Cdq => writeln!(out, "\tcdq")?,
            Instruction::Cmp { operand1, operand2 } => self.write_cmp(out, operand1, operand2)?,
            Instruction::Jmp { identifier } => writeln!(out, "\tjmp\t.L{}", identifier)?,
            Instruction::JmpCc { identifier, cond_code } => {
                writeln!(out, "\tj{}\t.L{}", convert_cond_code(cond_code), identifier)?
            }
            Instruction::SetCc { cond_code, operand } => self.write_setcc(out,cond_code,operand.clone() )?,
            Instruction::Label { identifier } => writeln!(out, ".L{}:", identifier)?,
        }
        Ok(())
    }

    fn write_mov(&mut self, out: &mut dyn Write, src: &Operand, dst: &Operand) -> Result<()> {
        if self.validate_operands(src, dst) {
            writeln!(out, "\tmovl\t{}, {}", self.operand_str(src), self.operand_str(dst))
        } else {
            writeln!(
                out,
                "\tmovl\t{}, %r10d\n\tmovl\t%r10d, {}",
                self.operand_str(src),
                self.operand_str(dst)
            )
        }
    }

    fn write_binary(
        &mut self,
        out: &mut dyn Write,
        op: &BinaryOpcode,
        operand1: &Operand,
        operand2: &Operand,
    ) -> Result<()> {
        match op {
            BinaryOpcode::Mul => self.write_mul(out, operand1, operand2),
            BinaryOpcode::Shl => self.write_shift(out, "shll", operand1, operand2),
            BinaryOpcode::Shr => self.write_shift(out, "shrl", operand1, operand2),
            _ => {
                let src = self.move_to_register_if_necessary(out, operand1, operand2)?;
                let opr_1 = self.operand_str(&src);
                let opr_2 = self.operand_str(operand2);
                writeln!(
                    out,
                    "\t{} {}, {}",
                    self.binary_opcode(op),
                    opr_1,
                    opr_2
                )
            }
        }
    }

    fn write_mul(&mut self, out: &mut dyn Write, src: &Operand, dst: &Operand) -> Result<()> {
        let src_op = self.move_to_register_if_necessary(out, src, dst)?;
        match dst {
            Operand::Stack(_) | Operand::Pseudo(_) => writeln!(
                out,
                "\tmovl\t{}, %r11d\n\timull\t{}, %r11d\n\tmovl\t%r11d, {}",
                self.operand_str(dst),
                self.operand_str(&src_op),
                self.operand_str(dst)
            ),
            _ => writeln!(out, "\timull\t{}, {}", self.operand_str(&src_op), self.operand_str(dst)),
        }
    }

    fn write_shift(&mut self, out: &mut dyn Write, op: &str, src: &Operand, dst: &Operand) -> Result<()> {
        match src {
            Operand::Imn(_) => writeln!(out, "\t{}\t{}, {}", op, self.operand_str(src), self.operand_str(dst)),
            _ => writeln!(out, "\tmovl\t{}, %ecx\n\t{}\t%cl, {}", self.operand_str(src), op, self.operand_str(dst)),
        }
    }

    fn write_cmp(&mut self, out: &mut dyn Write, operand1: &Operand, operand2: &Operand) -> Result<()> {
        if !self.validate_operands(operand1, operand2) {
            self.write_mov(out, operand1, &Operand::Reg(Register::R10))?;
            if matches!(operand2, Operand::Stack(_) | Operand::Pseudo(_)) {
                writeln!(out, "\tcmpl\t%r10d, {}", self.operand_str(operand2))?;
            } else {
                writeln!(out, "\tcmpl\t{}, %r10d", self.operand_str(operand2))?;
            }
        } else {
            writeln!(out, "\tcmpl\t{}, {}", self.operand_str(operand1), self.operand_str(operand2))?;
        }
        Ok(())
    }

    fn write_setcc(&mut self, out: &mut dyn Write, cond_code:&ConditionCode, operand: Operand) -> Result<()> {
        match operand {
            Operand::Reg(r) => writeln!(out, "\tset{}\t{}", convert_cond_code(&cond_code), self.byte_reg(&r)),
            _ => {
                self.write_mov(out, &operand, &Operand::Reg(Register::R10))?;
                writeln!(out, "\tset{}\t{}", convert_cond_code(&cond_code), self.byte_reg(&Register::R10))?;
                self.write_mov(out, &Operand::Reg(Register::R10), &operand)
            }
        }
    }

    fn move_to_register_if_necessary(&mut self, out: &mut dyn Write, src: &Operand, dst: &Operand) -> Result<Operand> {
        if self.validate_operands(src, dst) {
            Ok(src.clone())
        } else {
            writeln!(out, "\tmovl\t{}, %r10d", self.operand_str(src))?;
            Ok(Operand::Reg(Register::R10))
        }
    }

    fn move_to_reg(&mut self, out: &mut dyn Write, src: &Operand, reg: Register) -> Result<Operand> {
        match src {
            Operand::Reg(r) if *r == reg => Ok(src.clone()),
            _ => {
                writeln!(out, "\tmovl\t{}, {}", self.operand_str(src), self.reg_str(&reg))?;
                Ok(Operand::Reg(reg))
            }
        }
    }

    fn operand_str(&mut self, operand: &Operand) -> String {
        match operand {
            Operand::Imn(val) => format!("${}", val),
            Operand::Reg(reg) => self.reg_str(reg),
            Operand::Pseudo(name) => {
                let offset = match self.vars.get(name) {
                    Some(&off) => off,
                    None => {
                        let new_off = self.vars.values().min().map_or(-4, |&min| min - 4);
                        self.vars.insert(name.clone(), new_off);
                        new_off
                    }
                };
                format!("{}(%rbp)", offset)
            }
            Operand::Stack(offset) => format!("{}(%rbp)", offset),
        }
    }

    fn reg_str(&self, reg: &Register) -> String {
        match reg {
            Register::AX => "%eax".to_string(),
            Register::DX => "%edx".to_string(),
            Register::R10 => "%r10d".to_string(),
            Register::R11 => "%r11d".to_string(),
        }
    }

    fn byte_reg(&self, reg: &Register) -> &str {
        match reg {
            Register::AX => "%al",
            Register::DX => "%dl",
            Register::R10 => "%r10b",
            Register::R11 => "%r11b",
        }
    }

    fn unary_opcode(&self, op: &UnaryOpcode) -> &str {
        match op {
            UnaryOpcode::Not => "notl",
            UnaryOpcode::Neg => "negl",
        }
    }

    fn binary_opcode(&self, op: &BinaryOpcode) -> &str {
        match op {
            BinaryOpcode::Add => "addl",
            BinaryOpcode::Sub => "subl",
            BinaryOpcode::Mul => "imull",
            BinaryOpcode::Or => "orl",
            BinaryOpcode::And => "andl",
            BinaryOpcode::Xor => "xorl",
            BinaryOpcode::Shl => "shll",
            BinaryOpcode::Shr => "shrl",
        }
    }
}

impl Default for AsmGenerator {
    fn default() -> Self { Self::new() }
}
