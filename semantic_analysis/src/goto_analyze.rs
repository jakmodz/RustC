use std::{iter::Peekable, slice::Iter};

use ast::{
    Stmt,
    ast::{BlockElement, Program},
};

use crate::{SemanticAnalyzer, SemanticError};

pub trait GotoAnalyze {
    fn analyze_goto_statements(&mut self, ast: &mut Program) -> Result<(), SemanticError>;
    fn resolve_label(
        &mut self,
        stmt: &Stmt,
        iter: &mut Peekable<Iter<BlockElement>>,
    ) -> Result<(), SemanticError>;
    fn resolve_goto(&self, stmt: &Stmt) -> Result<Stmt, SemanticError>;
}

impl GotoAnalyze for SemanticAnalyzer {
    fn analyze_goto_statements(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        for func in ast.declarations.iter_mut() {
            match func {
                ast::Declaration::FuncDecl { decl }=> {
                    if let Some(body) = &mut decl.body {
                        let mut iter = body.elements.iter().peekable();
                        while let Some(element) = iter.next() {
                            if let BlockElement::Stmt(stmt) = element {
                                self.resolve_label(stmt, &mut iter)?;
                            }
                        }
                        for element in body.elements.iter_mut() {
                            if let BlockElement::Stmt(stmt) = element {
                                self.resolve_goto(stmt)?;
                            }
                        }
                    }
                }
                _=>{}
            }
        }
        Ok(())
    }
    fn resolve_label(
        &mut self,
        stmt: &Stmt,
        iter: &mut Peekable<Iter<BlockElement>>,
    ) -> Result<(), SemanticError> {
        match stmt {
            Stmt::If {
                else_branch,
                then_branch,
                ..
            } => {
                self.resolve_label(then_branch, iter)?;
                if let Some(else_branch) = else_branch {
                    self.resolve_label(else_branch, iter)?;
                }
                Ok(())
                
            }
            Stmt::Label(label) => {
                if self.labels.contains(label) {
                    return Err(SemanticError::DuplicateLabel {
                        label: label.to_string(),
                    });
                }
                if let Some(c) = iter.peek() {
                    if matches!(c, &BlockElement::Declaration(_)) {
                        return Err(SemanticError::DeclarationInLabel {
                            label: label.to_string(),
                        });
                    }
                } else {
                    return Err(SemanticError::EmptyLabel {
                        label: label.to_string(),
                    });
                }
                self.labels.insert(label.to_string());
                Ok(())
            }
            _ => Ok(()),
        }
    }
    fn resolve_goto(&self, stmt: &Stmt) -> Result<Stmt, SemanticError> {
        match stmt {
            Stmt::Goto(label) => {
                if !self.labels.contains(label) {
                    return Err(SemanticError::UndeclaredLabel {
                        label: label.to_string(),
                    });
                }
                Ok(stmt.clone())
            }
            _ => Ok(stmt.clone()),
        }
    }
}
