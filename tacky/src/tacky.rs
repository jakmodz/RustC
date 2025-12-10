use crate::tacky_instruction::TackyInstruction;


#[derive(Debug,Clone)]
pub enum TopLevelConstruct{
    Function(TackyFunction),
    StaticVar(StaticVar)
}


#[derive(Debug)]
pub struct Program {
    pub constructs: Vec<TopLevelConstruct>,
}
#[derive(Debug,Clone)]
pub struct TackyFunction {
    pub name: String,
    pub params: Vec<Val>,
    pub body: Vec<TackyInstruction>,
    pub global:bool
}
#[derive(Debug,Clone)]
pub struct StaticVar{
    pub name:String,
    pub global:bool,
    pub init: i64
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
