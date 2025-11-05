use tacky::tacky::UnaryOp;

#[derive(Debug)]
pub struct AsmProgram{
    pub function: AsmFunction,
}
#[derive(Debug)]
pub struct AsmFunction{
    pub name: String,
    pub instructions: Vec<Instruction>
}

#[derive(Debug)]
pub enum Instruction{
    Mov{src: Operand,dst:Operand},
    Unary{op:UnaryOpcode,operand:Operand},
    Allocate{size:usize},
    Ret
}

#[derive(Debug)]
pub enum Operand{
    Imn(i64),
    Reg(Register),
    Pseudo(String),
    Stack(i64)
}

#[derive(Debug)]
pub enum Register{
    AX,
    R10
}
#[derive(Debug)]
pub enum UnaryOpcode{
    Not,
    Neg
}
pub(crate )fn convert_unary_op(op:UnaryOp)->UnaryOpcode{
    match op {
        UnaryOp::Complement=>{
            UnaryOpcode::Not
        }
        UnaryOp::Negate=>{
            UnaryOpcode::Neg
        }
    }
}