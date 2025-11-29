use crate::SemanticError;
use crate::map_entry::VariableEntry;
use ast::ast::BlockElement;
use ast::ast::Declaration;
use ast::ast::Program;
use ast::ast::*;
use std::collections::{HashMap, HashSet};
use std::iter::Peekable;
use std::slice::Iter;
pub struct SemanticAnalyzer {
    variables: HashMap<String, VariableEntry>,
    labels: HashSet<String>,
    pub var_count: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            labels: HashSet::new(),
            var_count: 0,
        }
    }

    fn resolve_block(
        &mut self,
        elements: &[BlockElement],
    ) -> Result<Vec<BlockElement>, SemanticError> {
        let mut resolved_elements = Vec::new();

        for element in elements.iter() {
            match element {
                BlockElement::Declaration(decl) => {
                    let resolved_decl = self.resolve_declaration(decl)?;
                    resolved_elements.push(BlockElement::Declaration(resolved_decl));
                }
                BlockElement::Stmt(stmt) => {
                    let resolved_stmt = self.resolve_statement(stmt)?;
                    resolved_elements.push(BlockElement::Stmt(resolved_stmt));
                }
            }
        }

        Ok(resolved_elements)
    }
    pub fn analyze(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        self.variable_resolution(ast)?;
        self.analyze_goto_statements(ast)
    }
    pub fn variable_resolution(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        ast.function.body.elements = self.resolve_block(&ast.function.body.elements)?;
        Ok(())
    }

    fn resolve_declaration(&mut self, decl: &Declaration) -> Result<Declaration, SemanticError> {
        match decl {
            Declaration::DefineVar {
                var_name,
                initializer,
            } => {
                if let Some(entry) = self.variables.get(var_name) {
                    if entry.from_current_block {
                        return Err(SemanticError::MultipleDeclaration {
                            var_name: var_name.to_string(),
                        });
                    }
                }

                let unique_name = format!("{}.{}", var_name, self.var_count);
                self.var_count += 1;

                self.variables.insert(
                    var_name.to_string(),
                    VariableEntry::new(unique_name.clone(), true),
                );

                let resolved_init = match initializer {
                    Some(expr) => Some(self.resolve_expression(expr)?),
                    None => None,
                };

                Ok(Declaration::DefineVar {
                    var_name: unique_name,
                    initializer: resolved_init,
                })
            }
        }
    }
    fn resolve_statement(&mut self, stmt: &Stmt) -> Result<Stmt, SemanticError> {
        match stmt {
            Stmt::Expression { expr } => {
                let resolved_expr = self.resolve_expression(expr)?;
                Ok(Stmt::Expression {
                    expr: resolved_expr,
                })
            }
            Stmt::Return { expr } => {
                let resolved_expr = self.resolve_expression(expr)?;
                Ok(Stmt::Return {
                    expr: resolved_expr,
                })
            }
            Stmt::Null => Ok(Stmt::Null),
            Stmt::If {
                else_branch,
                condition,
                then_branch,
            } => {
                let resolved_condition = self.resolve_expression(condition)?;
                let resolved_then = Box::new(self.resolve_statement(then_branch)?);
                let resolved_else = match else_branch {
                    Some(else_stmt) => Some(Box::new(self.resolve_statement(else_stmt)?)),
                    None => None,
                };
                Ok(Stmt::If {
                    condition: resolved_condition,
                    then_branch: resolved_then,
                    else_branch: resolved_else,
                })
            }
            Stmt::Compound { block } => {
                let saved_variables = self.variables.clone();

                for entry in self.variables.values_mut() {
                    entry.from_current_block = false;
                }

                let resolved_block = self.resolve_block(&block.elements)?;

                self.variables = saved_variables;

                Ok(Stmt::Compound {
                    block: Block::new(resolved_block),
                })
            }

            _ => Ok(stmt.clone()),
        }
    }

    fn resolve_expression(&mut self, expression: &Expression) -> Result<Expression, SemanticError> {
        match expression {
            Expression::Assignment {
                expr_to,
                initializer,
            } => match expr_to.as_ref() {
                Expression::Var(_var_name) => Ok(Expression::Assignment {
                    expr_to: Box::new(self.resolve_expression(expr_to)?),
                    initializer: Box::new(self.resolve_expression(initializer)?),
                }),
                _ => Err(SemanticError::InvalidLeftValue {
                    invalid: format!("{:?}", expr_to),
                }),
            },
            Expression::Decrement { pre, expr } => Ok(Expression::Decrement {
                pre: *pre,
                expr: Box::new(self.resolve_expression(expr)?),
            }),
            Expression::Increment { expr, pre } => Ok(Expression::Increment {
                pre: *pre,
                expr: Box::new(self.resolve_expression(expr)?),
            }),
            Expression::UnaryOP { expr, op } => Ok(Expression::UnaryOP {
                op: op.clone(),
                expr: Box::new(self.resolve_expression(expr)?),
            }),
            Expression::Var(var_name) => {
                if self.variables.contains_key(var_name) {
                    Ok(Expression::Var(
                        self.variables.get(var_name).unwrap().name.to_string(),
                    ))
                } else {
                    Err(SemanticError::UndeclaredVariable {
                        var_name: var_name.to_string(),
                    })
                }
            }
            Expression::Grouping { expr } => Ok(Expression::Grouping {
                expr: Box::new(self.resolve_expression(expr)?),
            }),
            Expression::Binary { op, left, right } => Ok(Expression::Binary {
                op: op.clone(),
                left: Box::new(self.resolve_expression(left)?),
                right: Box::new(self.resolve_expression(right)?),
            }),
            Expression::CompoundAssign { op, var, expr } => Ok(Expression::CompoundAssign {
                op: op.clone(),
                var: Box::new(self.resolve_expression(var)?),
                expr: Box::new(self.resolve_expression(expr)?),
            }),
            Expression::Conditional { cond, expr1, expr2 } => {
                let resolved_cond = self.resolve_expression(cond)?;
                let resolved_expr1 = self.resolve_expression(expr1)?;
                let resolved_expr2 = self.resolve_expression(expr2)?;
                Ok(Expression::Conditional {
                    cond: Box::new(resolved_cond),
                    expr1: Box::new(resolved_expr1),
                    expr2: Box::new(resolved_expr2),
                })
            }
            _ => Ok(expression.clone()),
        }
    }
    /*
    resolving gotos statemtents by checking if the label exists in the current scope
    */
    fn analyze_goto_statements(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        let mut iter = ast.function.body.elements.iter().peekable();
        while let Some(element) = iter.next() {
            if let BlockElement::Stmt(stmt) = element {
                self.resolve_label(stmt, &mut iter)?;
            }
        }
        for element in ast.function.body.elements.iter_mut() {
            if let BlockElement::Stmt(stmt) = element {
                self.resolve_goto(stmt)?;
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
