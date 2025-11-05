use crate::asm_ast::*;
use std::io::{ Write, Result};
use std::collections::{ HashMap};

pub struct AsmGenerator {
    pub vars: HashMap<String, i64>,
}

impl AsmGenerator {
    pub fn new() -> Self {

        Self {
            vars: HashMap::new(),
        }
    }
    fn validate_operands(&mut self,src: &Operand,dst:&Operand)->bool{
        match (src,dst) {
            (Operand::Imn(_),Operand::Imn(_)) =>{
                false
            }
            (Operand::Stack(_),Operand::Stack(_))=> {
                false
            }

            _ => {true}
        }
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
        writeln!(out, "pushq    %rbp")?;
        writeln!(out, "movq     %rsp, %rbp")?;
        for instruction in function.instructions.iter() {
            self.write_instruction(out, instruction)?;
        }
        Ok(())
    }

    fn write_instruction(&mut self, out: &mut Box<dyn Write>, instruction: &Instruction) -> Result<()> {
        match instruction {
            Instruction::Mov { src, dst } => {
                if  self.validate_operands(src,dst){
                    write!(out, "movl\t")?;
                    self.write_operand(out, src)?;
                    writeln!(out, ", %eax")?;
                    write!(out, "movl\t%eax,")?;
                    self.write_operand(out, dst)?;
                    writeln!(out)?;
                }else{
                    write!(out, "movl\t")?;
                    self.write_operand(out, src)?;
                    write!(out, ",")?;
                    self.write_operand(out, dst)?;
                    writeln!(out)?;
                }
            }
            Instruction::Ret => {
                writeln!(out,"movq\t%rbp,%rsp")?;
                writeln!(out,"popq\t%rbp")?;
                writeln!(out, "ret")?;
            },
            Instruction::Unary { op,operand }=>{
                self.write_unary_opcode(out, op)?;
                write!(out,"\t")?;
                self.write_operand(out, operand)?;
                writeln!(out)?;
            }
            Instruction::Allocate { size} =>{
                writeln!(out, "subq\t${},%rsp",size)?;
            }
        }
        Ok(())
    }

    fn write_operand(&mut self, out: &mut Box<dyn Write>, operand: &Operand) -> Result<()> {
        match operand {
            Operand::Imn(val) => {
                write!(out, "${}", val)?;
            }
            Operand::Reg(reg) => {
                match reg {
                    Register::AX => {
                        write!(out, "%eax")?;
                    }
                    Register::R10 => {
                        write!(out, "%r10d")?;
                    }
                }
            }
            Operand::Pseudo(name) => {
                let offset = if let Some(&off) = self.vars.get(name) {
                    off
                } else {
                    let new_offset = self.vars.values().min().map(|v| v - 4).unwrap_or(-4);
                    self.vars.insert(name.clone(), new_offset);
                    new_offset
                };
                write!(out, "{}(%rbp)", offset)?;
            }
            Operand::Stack(offset) => {
                write!(out, "{}(%rbp)", offset)?;
            }
        }
        Ok(())
    }
    fn write_unary_opcode(&self, out: &mut Box<dyn Write>, op: &UnaryOpcode) -> Result<()> {
        match op {
            UnaryOpcode::Not => {
                write!(out, "notl ")?;
            }
            UnaryOpcode::Neg => {
                write!(out, "negl ")?;
            }
        }
        Ok(())
    }
}