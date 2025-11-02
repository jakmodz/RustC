use crate::asm_ast::*;
use std::io::{ Write, Result};

pub struct AsmGenerator {

}

impl AsmGenerator {
    pub fn new() -> Self {

        Self {

        }
    }

    pub fn write(&mut self, program: AsmProgram, mut outputs: Vec<Box<dyn Write>>) -> Result<()> {
        for out in outputs.iter_mut() {
            self.write_function(out, &program.function)?;
            writeln!(out, ".section .note.GNU-stack,\"\",@progbits")?;
        }
        Ok(())
    }

    fn write_function(&self, out: &mut Box<dyn Write>, function: &AsmFunction) -> Result<()> {
        writeln!(out, "\t.globl {}", function.name)?;
        writeln!(out, "{}:", function.name)?;
        for instruction in function.instructions.iter() {
            self.write_instruction(out, instruction)?;
        }
        Ok(())
    }

    fn write_instruction(&self, out: &mut Box<dyn Write>, instruction: &Instruction) -> Result<()> {
        write!(out, "\t")?;
        match instruction {
            Instruction::Mov { src, dst } => {
                write!(out, "movl ")?;
                self.write_operand(out, src)?;
                write!(out, ",")?;
                self.write_operand(out, dst)?;
            }
            Instruction::Ret => {
                write!(out, "ret")?;
            }
        }
        writeln!(out)?;
        Ok(())
    }

    fn write_operand(&self, out: &mut Box<dyn Write>, operand: &Operand) -> Result<()> {
        match operand {
            Operand::Imn(c) => {
                write!(out, "${}", c)?;
            }
            Operand::Register => {
                write!(out, "%eax")?;
            }
        }
        Ok(())
    }
}