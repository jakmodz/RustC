use crate::{Declaration, Expression, Stmt};

#[derive(Debug, Clone, PartialEq)]
pub enum Annotation {
    None,
    LoopLabel(String),
    SwitchLabel(String),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub functions: Vec<FuncDecl>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FuncDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Option<Block>,
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
    Declaration(VariableDecl),
    Expression(Option<Expression>),
}
#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub name: String,
    pub init: Option<Expression>,
}
#[derive(Debug, Clone, PartialEq)]
pub struct SwitchCase {
    pub value: i64,
    pub label: String,
}
