use lex::token::Token;

#[derive(Debug)]
#[derive(Clone)]
pub struct Program{
    pub function:Function,
}

#[derive(Debug,Clone)]
pub struct Function{
    pub name:String,
    pub body:Vec<Stmt>,
}
#[derive(Debug,Clone)]
#[derive(PartialEq)]
pub enum Stmt{
    Return{expr:Expression},
    If{condition:Expression,then_branch:Vec<Stmt>,else_branch:Option<Vec<Stmt>>},
}
#[derive(Debug,Clone)]
#[derive(PartialEq)]
pub enum  Expression{
    Constant(i64),
    UnaryOP{op:Token,expr:Box<Expression>},
    Grouping{expr:Box<Expression>},
}