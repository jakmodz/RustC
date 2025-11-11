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

    pub fn write(&mut self, program: AsmProgram, mut outputs: Vec<Box<dyn Write>>) -> Result<()> {
        for out in outputs.iter_mut() {
            self.write_function(out, &program.function)?;
            writeln!(out, ".section .note.GNU-stack,\"\",@progbits")?;
        }
        Ok(())
    }

    fn write_function(&mut self, out: &mut Box<dyn Write>, function: &AsmFunction) -> Result<()> {
        writeln!(out, "\t.globl {}", function.name)?;
        writeln!(out, "{}:", function.name)?;

        writeln!(out, "\tpushq\t%rbp")?;
        writeln!(out, "\tmovq\t%rsp, %rbp")?;

        for instruction in &function.instructions {
            self.write_instruction(out, instruction)?;
        }
        Ok(())
    }

    fn write_instruction(&mut self, out: &mut Box<dyn Write>, instruction: &Instruction) -> Result<()> {
        match instruction {
            Instruction::Mov { src, dst } => {
                self.write_mov(out, src, dst)?;
            }
            Instruction::Ret => {
                writeln!(out, "\tmovq\t%rbp, %rsp")?;
                writeln!(out, "\tpopq\t%rbp")?;
                writeln!(out, "\tret")?;
            }
            Instruction::Unary { op, operand } => {
                write!(out, "\t")?;
                self.write_unary_opcode(out, op)?;
                write!(out, "\t")?;
                self.write_operand(out, operand)?;
                writeln!(out)?;
            }
            Instruction::Allocate { size } => {
                writeln!(out, "\tsubq\t${}, %rsp", size)?;
                self.stack_offset += size.clone() as i64;
            }
            Instruction::Binary { binary_op, operand1, operand2 } => {
                self.write_binary(out, binary_op, operand1, operand2)?;
            }
            Instruction::IDiv { operand } => {
                let op = self.move_to_reg(out, operand, Register::R11)?;
                writeln!(out, "\tidivl\t{}", self.operand_str(&op))?;
            }
            Instruction::Cdq => {
                writeln!(out, "\tcdq")?;
            }
            Instruction::Cmp { operand1, operand2 } => {
                if !self.validate_operands(operand1, operand2) {
                    self.write_mov(out, operand1, &Operand::Reg(Register::R10))?;
                    writeln!(out, "\tcmpl\t{},%r10d", self.operand_str(operand2))?;
                } else {
                    writeln!(
                        out,
                        "\tcmpl\t{}, {}",
                        self.operand_str(operand1),
                        self.operand_str(operand2)
                    )?;
                }
            }
            Instruction::Jmp { identifier } => {
                writeln!(out, "\tjmp\t.L{}", identifier)?;
            }
            Instruction::JmpCc { identifier, cond_code } => {
                writeln!(out, "\tj{}\t.L{}",
                         convert_cond_code(cond_code),
                         identifier)?;
            }
            Instruction::SetCc { cond_code, operand } => {
                match operand {
                    Operand::Reg(r) => {
                        write!(out, "\tset{}\t", convert_cond_code(cond_code))?;
                        self.write_1byte_register(out, r)?;
                        writeln!(out)?;
                    }
                    _ => {
                        self.write_mov(out, operand, &Operand::Reg(Register::R10))?;
                        write!(out, "\tset{}\t", convert_cond_code(cond_code))?;
                        self.write_1byte_register(out, &Register::R10)?;
                        writeln!(out)?;
                        self.write_mov(out, &Operand::Reg(Register::R10), operand)?;
                    }
                }
            }
            Instruction::Label { identifier } => {
                writeln!(out, ".L{}:", identifier)?;
            }
        }
        Ok(())
    }

    fn write_mov(&mut self, out: &mut Box<dyn Write>, src: &Operand, dst: &Operand) -> Result<()> {
        if !self.validate_operands(src, dst) {
            writeln!(out, "\tmovl\t{}, %r10d", self.operand_str(src))?;
            writeln!(out, "\tmovl\t%r10d, {}", self.operand_str(dst))?;
        } else {
            writeln!(out, "\tmovl\t{}, {}", self.operand_str(src), self.operand_str(dst))?;
        }
        Ok(())
    }

    fn write_binary(
        &mut self,
        out: &mut Box<dyn Write>,
        op: &BinaryOpcode,
        operand1: &Operand,
        operand2: &Operand,
    ) -> Result<()> {
        match op {
            BinaryOpcode::Mul => {
                self.write_mul(out, operand1, operand2)?;
            }
            BinaryOpcode::Shl => {
                self.write_shift(out, "shll", operand1, operand2)?;
            }
            BinaryOpcode::Shr => {
                self.write_shift(out, "shrl", operand1, operand2)?;
            }
            _ => {
                let src = self.move_to_register_if_necessary(out, operand1, operand2)?;
                write!(out, "\t")?;
                self.write_binary_operator(out, op)?;
                write!(out, "\t")?;
                self.write_operand(out, &src)?;
                write!(out, ", ")?;
                self.write_operand(out, operand2)?;
                writeln!(out)?;
            }
        }
        Ok(())
    }

    fn write_mul(&mut self, out: &mut Box<dyn Write>, src: &Operand, dst: &Operand) -> Result<()> {
        let src_op = self.move_to_register_if_necessary(out, src, dst)?;

        match dst {
            Operand::Stack(_) | Operand::Pseudo(_) => {
                writeln!(out, "\tmovl\t{}, %r11d", self.operand_str(dst))?;
                writeln!(out, "\timull\t{}, %r11d", self.operand_str(&src_op))?;
                writeln!(out, "\tmovl\t%r11d, {}", self.operand_str(dst))?;
            }
            _ => {
                writeln!(out, "\timull\t{}, {}", self.operand_str(&src_op), self.operand_str(dst))?;
            }
        }
        Ok(())
    }

    fn write_shift(&mut self, out: &mut Box<dyn Write>, op: &str, src: &Operand, dst: &Operand) -> Result<()> {
        match src {
            Operand::Imn(_) => {
                writeln!(out, "\t{}\t{}, {}", op, self.operand_str(src), self.operand_str(dst))?;
            }
            _ => {
                writeln!(out, "\tmovl\t{}, %ecx", self.operand_str(src))?;
                writeln!(out, "\t{}\t%cl, {}", op, self.operand_str(dst))?;
            }
        }
        Ok(())
    }

    fn move_to_register_if_necessary(
        &mut self,
        out: &mut Box<dyn Write>,
        src: &Operand,
        dst: &Operand,
    ) -> Result<Operand> {
        if !self.validate_operands(src, dst) {
            writeln!(out, "\tmovl\t{}, %r10d", self.operand_str(src))?;
            Ok(Operand::Reg(Register::R10))
        } else {
            Ok(src.clone())
        }
    }

    fn move_to_reg(&mut self, out: &mut Box<dyn Write>, src: &Operand, reg: Register) -> Result<Operand> {
        match src {
            Operand::Reg(r) if r == &reg => Ok(src.clone()),
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
                let offset = if let Some(&off) = self.vars.get(name) {
                    off
                } else {
                    let new_offset = self.vars
                        .values()
                        .min()
                        .map(|&min_off| min_off - 4)
                        .unwrap_or(-4);
                    self.vars.insert(name.clone(), new_offset);
                    new_offset
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

    fn write_operand(&mut self, out: &mut Box<dyn Write>, operand: &Operand) -> Result<()> {
        write!(out, "{}", self.operand_str(operand))
    }

    fn write_unary_opcode(&self, out: &mut Box<dyn Write>, op: &UnaryOpcode) -> Result<()> {
        let opcode = match op {
            UnaryOpcode::Not => "notl",
            UnaryOpcode::Neg => "negl",
        };
        write!(out, "{}", opcode)
    }

    fn write_binary_operator(&self, out: &mut Box<dyn Write>, op: &BinaryOpcode) -> Result<()> {
        let opcode = match op {
            BinaryOpcode::Add => "addl",
            BinaryOpcode::Sub => "subl",
            BinaryOpcode::Mul => "imull",
            BinaryOpcode::Or => "orl",
            BinaryOpcode::And => "andl",
            BinaryOpcode::Xor => "xorl",
            BinaryOpcode::Shl => "shll",
            BinaryOpcode::Shr => "shrl",
        };
        write!(out, "{}", opcode)
    }

    fn write_1byte_register(&self, out: &mut Box<dyn Write>, reg: &Register) -> Result<()> {
        let reg = match reg {
            Register::AX => "%al",
            Register::DX => "%dl",
            Register::R10 => "%r10b",
            Register::R11 => "%r11b",
        };
        write!(out, "{}", reg)
    }
}

impl Default for AsmGenerator {
    fn default() -> Self {
        Self::new()
    }
}