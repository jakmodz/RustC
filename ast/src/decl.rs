use crate::ast::{FuncDecl, VariableDecl};



#[derive(Debug, Clone, PartialEq)]
pub enum StorageClass{
    Extern,
    Static
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    DefineVar(VariableDecl),
    FuncDecl { decl: FuncDecl},
}
