#[derive(Debug)]
pub struct Program {
    pub function: TackyFunction,
}
#[derive(Debug)]
pub struct TackyFunction {
    pub name: String,
    pub body: Vec<TackyInstruction>,
}
#[derive(Debug)]
pub enum TackyInstruction {
    Return(Val),
    Unary {
        unary_op: UnaryOp,
        src: Val,
        dst: Val,
    },
    Binary {
        binary_op: BinaryOp,
        src1: Val,
        src2: Val,
        dst: Val,
    },
    Copy {
        src: Val,
        dst: Val,
    },
    Jump {
        target: String,
    },
    JumpIfZero {
        cond: Val,
        target: String,
    },
    JumpIfNotZero {
        cond: Val,
        target: String,
    },
    Label(String),
    
}

#[derive(Clone, Debug, PartialEq)]
pub enum Val {
    Var(String),
    Constant(i64),
}
#[derive(Clone, Debug)]
pub enum UnaryOp {
    Complement,
    Negate,
    Not,
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
    Equal,
    NotEqual,
    LessThan,
    LeesOrEqual,
    GreaterThan,
    GreaterOrEqual,
}

impl BinaryOp {
    pub fn is_comparison(&self) -> bool {
        matches!(
            self,
            BinaryOp::Equal
                | BinaryOp::NotEqual
                | BinaryOp::LessThan
                | BinaryOp::LeesOrEqual
                | BinaryOp::GreaterThan
                | BinaryOp::GreaterOrEqual
        )
    }
}
