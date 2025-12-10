use crate::tacky::{BinaryOp, UnaryOp, Val};

#[derive(Debug, Clone)]
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
    FnCall {
        fn_name: String,
        args: Vec<Val>,
        dst: Val,
    },
    Label(String),
}
