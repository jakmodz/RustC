use tacky::tacky::{BinaryOp, UnaryOp};

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
    Binary { binary_op: BinaryOpcode, operand1: Operand, operand2: Operand },
    IDiv { operand: Operand },
    Cdq,
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
    DX,
    R10,
    R11
}
#[derive(Debug)]
pub enum UnaryOpcode{
    Not,
    Neg
}
#[derive(Debug)]
pub enum BinaryOpcode {
    Add,
    Sub,
    Mul,
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
pub(crate) fn convert_binary_op(op: BinaryOp) -> BinaryOpcode {
    match op {
        BinaryOp::Add => {
            BinaryOpcode::Add
        }
        BinaryOp::Subtract => {
            BinaryOpcode::Sub
        }
        BinaryOp::Multiply => {
            BinaryOpcode::Mul
        }
        _ => {
            panic!("unsuported")
        }
    }
}