
pub struct Program {
    pub function: TackyFunction,

}

pub struct TackyFunction {
    pub name: String,
    pub body: Vec<TackyInstruction>,
}

pub enum TackyInstruction {
    Return(Val),
    Unary{unary_op: UnaryOp, src: Val,dst: Val},
    Binary { binary_op: BinaryOp, src1: Val, src2: Val, dst: Val }

}

#[derive(Clone)]
pub enum Val{
    Var(String),
    Constant(i64),
}
#[derive(Clone,Debug)]
pub enum UnaryOp{
    Complement,
    Negate,
}
#[derive(Clone, Debug)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,
    Or,
    And,
    Xor,
    LeftShift,
    RightShift,
}


