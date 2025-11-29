use lex::token::Token;

#[derive(Debug, Clone)]
pub struct Program {
    pub function: Function,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub body: Block,
}
#[derive(Debug, Clone, PartialEq)]
pub enum BlockElement {
    Stmt(Stmt),
    Declaration(Declaration),
}
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub elements: Vec<BlockElement>,
}
impl Block {
    pub fn new(elements: Vec<BlockElement>) -> Self {
        Self { elements }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Return {
        expr: Expression,
    },
    Expression {
        expr: Expression,
    },
    Null,
    If {
        condition: Expression,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    },
    Compound {
        block: Block,
    },
    Goto(String),
    Label(String),
}
#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    DefineVar {
        var_name: String,
        initializer: Option<Expression>,
    },
}
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Constant(i64),
    UnaryOP {
        op: Token,
        expr: Box<Expression>,
    },
    Var(String),
    Grouping {
        expr: Box<Expression>,
    },
    Binary {
        op: Token,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    Assignment {
        expr_to: Box<Expression>,
        initializer: Box<Expression>,
    },
    CompoundAssign {
        op: Token,
        var: Box<Expression>,
        expr: Box<Expression>,
    },
    Increment {
        expr: Box<Expression>,
        pre: bool,
    },
    Decrement {
        expr: Box<Expression>,
        pre: bool,
    },
    Conditional {
        cond: Box<Expression>,
        expr1: Box<Expression>,
        expr2: Box<Expression>,
    },
}
