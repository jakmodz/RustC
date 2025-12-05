use lex::token::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    None,
    LoopLabel(String),
    SwitchLabel(String),
}

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
pub enum ForInit {
    Declaration(Declaration),
    Expression(Option<Expression>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub value: i64,
    pub label: String,
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
    Break(Annotation),
    Continue(Annotation),
    While {
        condition: Expression,
        body: Box<Stmt>,
        annotation: Annotation,
    },
    DoWhile {
        body: Box<Stmt>,
        condition: Expression,
        annotation: Annotation,
    },
    For {
        init: ForInit,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Box<Stmt>,
        annotation: Annotation,
    },
    Compound {
        block: Block,
    },
    Switch {
        expr: Expression,
        body: Box<Stmt>,
        annotation: Annotation,
        cases: Vec<SwitchCase>,
        default_label: Option<String>,
    },
    Case {
        value: Expression,
        body: Box<Stmt>,
    },
    Default {
        body: Box<Stmt>,
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
