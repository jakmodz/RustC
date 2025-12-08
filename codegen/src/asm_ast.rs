use tacky::tacky::{BinaryOp, UnaryOp};

#[derive(Debug)]
pub struct AsmProgram {
    pub functions: Vec<AsmFunction>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct AsmFunction {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instruction {
    Mov {
        src: Operand,
        dst: Operand,
    },
    Unary {
        op: UnaryOpcode,
        operand: Operand,
    },
    Binary {
        binary_op: BinaryOpcode,
        operand1: Operand,
        operand2: Operand,
    },
    IDiv {
        operand: Operand,
    },
    Cmp {
        operand1: Operand,
        operand2: Operand,
    },
    Jmp {
        identifier: String,
    },
    JmpCc {
        cond_code: ConditionCode,
        identifier: String,
    },
    SetCc {
        cond_code: ConditionCode,
        operand: Operand,
    },
    Label {
        identifier: String,
    },
    Cdq,
    Allocate {
        size: usize,
    },
    Deallocate{
        size:usize
    },
    Push{
        operand: Operand
    },
    Call{
        name:String
    },
    Ret,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand {
    Imn(i64),
    Reg(Register),
    Pseudo(String),
    Stack(i64),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Register {
    AX,
    CX,
    DI,
    SI,
    DX,
    R8,
    R9,
    R10,
    R11,
}
#[derive(Debug, Clone, PartialEq)]
pub enum UnaryOpcode {
    Not,
    Neg,
}
#[derive(Debug, Clone, PartialEq)]
pub enum ConditionCode {
    Equal,
    NotEqual,
    LessThan,
    LessOrEqual,
    GreaterThan,
    GreaterOrEqual,
}
#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOpcode {
    Add,
    Sub,
    Mul,
    Or,
    And,
    Xor,
    Shl,
    Shr,
}
pub(crate) fn convert_unary_op(op: UnaryOp) -> UnaryOpcode {
    match op {
        UnaryOp::Complement => UnaryOpcode::Not,
        UnaryOp::Negate => UnaryOpcode::Neg,
        // UnaryOpcode::Not=>{
        //     todo!()
        // }
        _ => {
            todo!()
        }
    }
}
pub(crate) fn convert_binary_op(op: BinaryOp) -> BinaryOpcode {
    match op {
        BinaryOp::Add => BinaryOpcode::Add,
        BinaryOp::Subtract => BinaryOpcode::Sub,
        BinaryOp::Multiply => BinaryOpcode::Mul,
        BinaryOp::Or => BinaryOpcode::Or,
        BinaryOp::And => BinaryOpcode::And,
        BinaryOp::Xor => BinaryOpcode::Xor,
        BinaryOp::LeftShift => BinaryOpcode::Shl,
        BinaryOp::RightShift => BinaryOpcode::Shr,

        _ => {
            panic!("unsuported")
        }
    }
}
pub(crate) fn convert_condition_code(binary_op: BinaryOp) -> ConditionCode {
    match binary_op {
        BinaryOp::Equal => ConditionCode::Equal,
        BinaryOp::NotEqual => ConditionCode::NotEqual,
        BinaryOp::LessThan => ConditionCode::LessThan,
        BinaryOp::LeesOrEqual => ConditionCode::LessOrEqual,
        BinaryOp::GreaterThan => ConditionCode::GreaterThan,
        BinaryOp::GreaterOrEqual => ConditionCode::GreaterOrEqual,
        _ => {
            panic!("not a comparison operator")
        }
    }
}
pub(crate) fn convert_cond_code(code: &ConditionCode) -> String {
    match code {
        ConditionCode::Equal => "e",
        ConditionCode::NotEqual => "ne",
        ConditionCode::LessThan => "l",
        ConditionCode::LessOrEqual => "le",
        ConditionCode::GreaterThan => "g",
        ConditionCode::GreaterOrEqual => "ge",
    }
    .to_string()
}
