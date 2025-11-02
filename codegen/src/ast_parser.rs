use crate::asm_ast::*;
use ast::ast::*;

pub struct AsmParser{

}

impl AsmParser {
    pub fn new()->Self {
        Self{

        }
    }

    pub fn parse(&mut self, program: Program)->AsmProgram{
        let mut instructions = Vec::new();

        for stmt in program.function.body {
           self.convert_stmt(stmt,&mut instructions);
        }

        AsmProgram{
            function: AsmFunction{
                name: program.function.name,
                instructions
            }
        }
    }

    fn convert_stmt(&mut self, stmt:Stmt,instructions:&mut Vec<Instruction>){
        match stmt {
            Stmt::Return { expr } => {

                self.convert_expr(expr,instructions);
                // instructions.push(Instruction::Mov {
                //     src: Operand::Register,
                //     dst: Operand::Imn(c)
                // });
                instructions.push(Instruction::Ret);
            }
            Stmt::If { .. } => {todo!()}
        }
    }


    fn convert_expr(&mut self, expr: Expression,instructions:&mut Vec<Instruction>){
        match expr {
            Expression::Constant(c) => {
                instructions.push(Instruction::Mov {
                    dst: Operand::Register,
                    src: Operand::Imn(c)
                })
            }
        }
    }
}
