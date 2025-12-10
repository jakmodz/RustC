use crate::asm_ast::*;
use std::io::{Result, Write};

pub struct AsmGenerator {
    stack_offset: i64,
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self { stack_offset: 0 }
    }

    pub fn write(&mut self, program: AsmProgram, mut outputs: Vec<Box<dyn Write>>) -> Result<()> {
        for out in outputs.iter_mut() {
            for construct in &program.constructs {
                match construct {
                    AsmConstruct::Func(function) => {
                        self.stack_offset = 0;
                        self.write_function(out, function)?;
                    },
                    AsmConstruct::StaticVar(static_var) => {
                        self.write_static_var(out, static_var)?;
                    }
                }
            }
            if cfg!(target_os = "linux") {
                writeln!(out, "\t.section\t.note.GNU-stack,\"\",@progbits")?;
            }
        }
        Ok(())
    }

    fn write_static_var(&mut self, out: &mut dyn Write, static_var: &AsmStaticVar) -> Result<()> {
        let name = if static_var.global && cfg!(target_os = "macos") {
            format!("_{}", static_var.name)
        } else if !static_var.global {
            format!("L_{}", static_var.name) 
        }
        else {
            static_var.name.clone()
        };
        
        if static_var.global {
            writeln!(out, "\t.globl {}", name)?;
        }
        
        if !static_var.global && cfg!(target_os = "linux") {
            writeln!(out, "\t.local {}", name)?; 
        }

        if static_var.init == 0 {
            writeln!(out, "\t.bss")?;
        }else{
             writeln!(out, "\t.data")?;
        }
       writeln!(out, "\t.balign 4")?;
        writeln!(out, "{}:", name)?; 
        writeln!(out, "\t.long {}", static_var.init)?; 

        Ok(())
    }

    fn write_function(&mut self, out: &mut dyn Write, function: &AsmFunction) -> Result<()> {
        let mut fn_name = function.name.clone();
        if cfg!(target_os = "macos") {
            fn_name = format!("_{}", fn_name);
        }
        writeln!(out,"\t.text")?; 
        if function.global {
            writeln!(out,"\t.globl {}", fn_name)?; 
        } 
        writeln!(out, "{}:", fn_name)?;
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
                let operand = self.operand_str(operand);
                writeln!(out, "\t{} {}", self.unary_opcode(op), operand)?;
            }
            Instruction::Allocate { size } => {
                writeln!(out, "\tsubq\t${}, %rsp", size)?;
                self.stack_offset += *size as i64;
            }
            Instruction::Binary {
                binary_op,
                operand1,
                operand2,
            } => self.write_binary(out, binary_op, operand1, operand2)?,
            Instruction::IDiv { operand } => {
                let op = self.move_to_reg(out, operand, Register::R11)?;
                writeln!(out, "\tidivl\t{}", self.operand_str(&op))?;
            }
            Instruction::Cdq => writeln!(out, "\tcdq")?,
            Instruction::Cmp { operand1, operand2 } => self.write_cmp(out, operand1, operand2)?,
            Instruction::Jmp { identifier } => {
                writeln!(out, "\tjmp\t{}", self.format_label(identifier))?
            }
            Instruction::JmpCc {
                identifier,
                cond_code,
            } => writeln!(
                out,
                "\tj{}\t{}",
                convert_cond_code(cond_code),
                self.format_label(identifier)
            )?,
            Instruction::SetCc { cond_code, operand } => {
                self.write_setcc(out, cond_code, operand.clone())?
            }
            Instruction::Label { identifier } => {
                writeln!(out, "{}:", self.format_label(identifier))?
            }
            Instruction::Deallocate { size } => {
                writeln!(out, "\taddq\t${}, %rsp", size)?;
                self.stack_offset -= *size as i64;
            }
            Instruction::Push { operand } => {
                let operand_str = match operand {
                    Operand::Reg(r) => self.reg_str_64(r),
                    _ => self.operand_str(operand),
                };
                writeln!(out, "\tpushq\t{}", operand_str)?;
                self.stack_offset += 8;
            }
            Instruction::Call { name } => {
                let call_name = if cfg!(target_os = "macos") {
                    format!("_{}", name)
                } else {
                    name.clone()
                };
                writeln!(out, "\tcall\t{}", call_name)?;
            }
        }
        Ok(())
    }

    fn write_mov(&mut self, out: &mut dyn Write, src: &Operand, dst: &Operand) -> Result<()> {
        writeln!(
            out,
            "\tmovl\t{}, {}",
            self.operand_str(src),
            self.operand_str(dst)
        )
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
            BinaryOpcode::Shr => self.write_shift(out, "sarl", operand1, operand2),
            _ => {
                let opr_1 = self.operand_str(operand1);
                let opr_2 = self.operand_str(operand2);
                writeln!(out, "\t{} {}, {}", self.binary_opcode(op), opr_1, opr_2)
            }
        }
    }

    fn write_mul(&mut self, out: &mut dyn Write, src: &Operand, dst: &Operand) -> Result<()> {
        match dst {
            Operand::Stack(_) | Operand::Data(_) => writeln!(
                out,
                "\tmovl\t{}, %r11d\n\timull\t{}, %r11d\n\tmovl\t%r11d, {}",
                self.operand_str(dst),
                self.operand_str(src),
                self.operand_str(dst)
            ),
            _ => writeln!(
                out,
                "\timull\t{}, {}",
                self.operand_str(src),
                self.operand_str(dst)
            ),
        }
    }

    fn write_shift(
        &mut self,
        out: &mut dyn Write,
        op: &str,
        src: &Operand,
        dst: &Operand,
    ) -> Result<()> {
        match src {
            Operand::Imn(_) => writeln!(
                out,
                "\t{}\t{}, {}",
                op,
                self.operand_str(src),
                self.operand_str(dst)
            ),
            _ => writeln!(
                out,
                "\tmovl\t{}, %ecx\n\t{}\t%cl, {}",
                self.operand_str(src),
                op,
                self.operand_str(dst)
            ),
        }
    }

    fn write_cmp(
        &mut self,
        out: &mut dyn Write,
        operand1: &Operand,
        operand2: &Operand,
    ) -> Result<()> {
        writeln!(
            out,
            "\tcmpl\t{}, {}",
            self.operand_str(operand1),
            self.operand_str(operand2)
        )
    }

    fn write_setcc(
        &mut self,
        out: &mut dyn Write,
        cond_code: &ConditionCode,
        operand: Operand,
    ) -> Result<()> {
        match operand {
            Operand::Reg(r) => writeln!(
                out,
                "\tset{}\t{}",
                convert_cond_code(cond_code),
                self.byte_reg(&r)
            ),
            _ => {
                self.write_mov(out, &operand, &Operand::Reg(Register::R10))?;
                writeln!(
                    out,
                    "\tset{}\t{}",
                    convert_cond_code(cond_code),
                    self.byte_reg(&Register::R10)
                )?;
                self.write_mov(out, &Operand::Reg(Register::R10), &operand)
            }
        }
    }

    fn move_to_reg(
        &mut self,
        out: &mut dyn Write,
        src: &Operand,
        reg: Register,
    ) -> Result<Operand> {
        match src {
            Operand::Reg(r) if *r == reg => Ok(src.clone()),
            _ => {
                writeln!(
                    out,
                    "\tmovl\t{}, {}",
                    self.operand_str(src),
                    self.reg_str(&reg)
                )?;
                Ok(Operand::Reg(reg))
            }
        }
    }

    fn operand_str(&self, operand: &Operand) -> String {
        match operand {
            Operand::Imn(val) => format!("${}", val),
            Operand::Reg(reg) => self.reg_str(reg),
            Operand::Stack(offset) => format!("{}(%rbp)", offset),
            Operand::Pseudo(_) => {
                panic!("Pseudoregisters should have been replaced before code generation")
            }
            Operand::Data(iden) =>{
                format!("{}(%rip)",iden) 
            },
        }
    }
    fn reg_str_64(&self, reg: &Register) -> String {
        match reg {
            Register::AX => "%rax".to_string(),
            Register::DX => "%rdx".to_string(),
            Register::CX => "%rcx".to_string(),
            Register::DI => "%rdi".to_string(),
            Register::SI => "%rsi".to_string(),
            Register::R8 => "%r8".to_string(),
            Register::R9 => "%r9".to_string(),
            Register::R10 => "%r10".to_string(),
            Register::R11 => "%r11".to_string(),
        }
    }
    fn reg_str(&self, reg: &Register) -> String {
        match reg {
            Register::AX => "%eax".to_string(),
            Register::DX => "%edx".to_string(),
            Register::CX => "%ecx".to_string(),
            Register::DI => "%edi".to_string(),
            Register::SI => "%esi".to_string(),
            Register::R8 => "%r8d".to_string(),
            Register::R9 => "%r9d".to_string(),
            Register::R10 => "%r10d".to_string(),
            Register::R11 => "%r11d".to_string(),
        }
    }

    fn byte_reg(&self, reg: &Register) -> &str {
        match reg {
            Register::AX => "%al",
            Register::DX => "%dl",
            Register::CX => "%cl",
            Register::DI => "%dil",
            Register::SI => "%sil",
            Register::R8 => "%r8b",
            Register::R9 => "%r9b",
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

    fn format_label(&self, identifier: &str) -> String {
        format!("L{}", identifier)
    }
}

impl Default for AsmGenerator {
    fn default() -> Self {
        Self::new()
    }
}