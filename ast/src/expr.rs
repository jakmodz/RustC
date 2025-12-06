use lex::token::Token;



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
    FunctionCall {
        func_name: String,
        args: Vec<Expression>,
    },
}
