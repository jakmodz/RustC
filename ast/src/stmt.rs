use crate::{
    Expression,
    ast::{Annotation, Block, ForInit, SwitchCase},
};

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
