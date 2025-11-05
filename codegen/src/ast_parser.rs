use tacky::tacky::Val;
use crate::asm_ast::*;
use tacky::tacky::TackyInstruction;

pub struct AsmParser{

}

impl AsmParser {
    pub fn new()->Self {
        Self{

        }
    }

    pub fn parse(&mut self, program: tacky::tacky::Program)->AsmProgram{
        let mut instructions = Vec::new();

        for ins in program.function.body {
            self.convert_instruction(ins,&mut instructions);
        }

        AsmProgram{
            function: AsmFunction{
                name: program.function.name,
                instructions
            }
        }
    }

    fn convert_instruction(&mut self, ins:TackyInstruction,instructions:&mut Vec<Instruction>){
        match ins {
            TackyInstruction::Return(val) =>{
                let ins = Instruction::Mov{src:self.convert_val(val)
                    ,dst:Operand::Reg(Register::AX)};
                instructions.push(ins);
                instructions.push(Instruction::Ret);
            }
            TackyInstruction::Unary { unary_op,src,dst } => {
                instructions.push(Instruction::Mov{src:self.convert_val(src)
                    ,dst:self.convert_val(dst.clone())});
                instructions.push(Instruction::Unary{op:convert_unary_op(unary_op),operand:self.convert_val(dst)});
            }
        }
    }

    fn convert_val(&mut self, val:tacky::tacky::Val)->Operand{
        match val {
            Val::Constant(c)=>{
                Operand::Imn(c)
            }
            Val::Var(name)=>{
                Operand::Pseudo(name)
            }
        }
    }

}
