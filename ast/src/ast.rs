
#[derive(Debug)]
pub struct Program{
    pub function:Function,
}

#[derive(Debug)]
pub struct Function{
    pub name:String,
    pub body:Vec<Stmt>,
}
#[derive(Debug)]
#[derive(PartialEq)]
pub enum Stmt{
    Return{expr:Expression},
    If{condition:Expression,then_branch:Vec<Stmt>,else_branch:Option<Vec<Stmt>>},
}
#[derive(Debug)]
#[derive(PartialEq)]
pub enum  Expression{
    Constant(i64),
}