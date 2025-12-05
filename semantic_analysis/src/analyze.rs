use crate::SemanticError;
use crate::map_entry::VariableEntry;
use ast::ast::BlockElement;
use ast::ast::Declaration;
use ast::ast::Program;
use ast::ast::*;
use std::collections::{HashMap, HashSet};
use std::iter::Peekable;
use std::slice::Iter;
use crate::switch_analyze::SwitchAnalyzer;

pub struct SemanticAnalyzer {
    variables: HashMap<String, VariableEntry>,
    labels: HashSet<String>,
    pub var_count: usize,
    pub loop_count: usize,
    pub switch_count: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            labels: HashSet::new(),
            var_count: 0,
            loop_count: 0,
            switch_count: 0,
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
        self.analyze_goto_statements(ast)?;
        self.analyze_control_flow(ast)?;
        Ok(())
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
            Stmt::While {
                body, condition, ..
            } => {
                let resolved_condition = self.resolve_expression(condition)?;
                let resolved_body = Box::new(self.resolve_statement(body)?);
                Ok(Stmt::While {
                    condition: resolved_condition,
                    body: resolved_body,
                    annotation: Annotation::None,
                })
            }
            Stmt::DoWhile {
                body, condition, ..
            } => {
                let resolved_condition = self.resolve_expression(condition)?;
                let resolved_body = Box::new(self.resolve_statement(body)?);
                Ok(Stmt::DoWhile {
                    body: resolved_body,
                    condition: resolved_condition,
                    annotation: Annotation::None,
                })
            }
            Stmt::For {
                condition,
                body,
                init,
                increment,
                ..
            } => {
                let saved_variables = self.variables.clone();

                for entry in self.variables.values_mut() {
                    entry.from_current_block = false;
                }

                let resolver_init = self.resolve_for_init(init)?;
                let resolved_condition = self.resolve_optional_expr(condition)?;
                let resolved_increment = self.resolve_optional_expr(increment)?;
                let resolved_body = Box::new(self.resolve_statement(body)?);

                self.variables = saved_variables;

                Ok(Stmt::For {
                    init: resolver_init,
                    condition: resolved_condition,
                    increment: resolved_increment,
                    body: resolved_body,
                    annotation: Annotation::None,
                })
            }
            Stmt::Switch { expr, body, annotation, cases, default_label } => {
                let resolved_expr = self.resolve_expression(expr)?;
                let resolved_body = Box::new(self.resolve_statement(body)?);
                Ok(Stmt::Switch {
                    expr: resolved_expr,
                    body: resolved_body,
                    annotation: annotation.clone(),
                    cases: cases.clone(),
                    default_label: default_label.clone(),
                })
            }

            Stmt::Case { value, body } => {
                let resolved_value = self.resolve_expression(value)?;
                let resolved_body = Box::new(self.resolve_statement(body)?);
                Ok(Stmt::Case {
                    value: resolved_value,
                    body: resolved_body,
                })
            }

            Stmt::Default { body } => {
                let resolved_body = Box::new(self.resolve_statement(body)?);
                Ok(Stmt::Default {
                    body: resolved_body,
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

    fn resolve_for_init(&mut self, init: &ForInit) -> Result<ForInit, SemanticError> {
        match init {
            ForInit::Declaration(decl) => Ok(ForInit::Declaration(self.resolve_declaration(decl)?)),
            ForInit::Expression(expr_opt) => {
                let resolved_expr_opt = match expr_opt {
                    Some(expr) => Some(self.resolve_expression(expr)?),
                    None => None,
                };
                Ok(ForInit::Expression(resolved_expr_opt))
            }
        }
    }

    fn resolve_optional_expr(
        &mut self,
        expr: &Option<Expression>,
    ) -> Result<Option<Expression>, SemanticError> {
        match expr {
            Some(expr) => Ok(Some(self.resolve_expression(expr)?)),
            None => Ok(None),
        }
    }

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

    fn make_loop_label(&mut self) -> String {
        let label = format!("_loop_{}", self.loop_count);
        self.loop_count += 1;
        label
    }

    fn analyze_loop_labels(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        for element in ast.function.body.elements.iter_mut() {
            if let BlockElement::Stmt(stmt) = element {
                self.label_loops(stmt, None)?;
            }
        }
        Ok(())
    }

    fn label_loops(
        &mut self,
        stmt: &mut Stmt,
        current_label: Option<String>,
    ) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Break(label) => {
                if let Some(loop_label) = current_label {
                    // We're inside a loop - break should exit the loop
                    *label = Annotation::LoopLabel(loop_label);
                    Ok(())
                } else if matches!(label, Annotation::SwitchLabel(_)) {
                    // Already has a switch label and no loop context - keep it
                    Ok(())
                } else if matches!(label, Annotation::None) {
                    Err(SemanticError::JumpStmtNotInLoop {
                        stmt: "Break".to_string(),
                    })
                } else {
                    Ok(())
                }
            }

            Stmt::Continue(label) => {
                if let Some(loop_label) = current_label {
                    *label = Annotation::LoopLabel(loop_label);
                    Ok(())
                } else {
                    Err(SemanticError::JumpStmtNotInLoop {
                        stmt: "Continue".to_string(),
                    })
                }
            }
            Stmt::While {
                body, annotation, ..
            } => {
                let loop_label = self.make_loop_label();
                *annotation = Annotation::LoopLabel(loop_label.clone());
                self.label_loops(body, Some(loop_label))
            }
            Stmt::DoWhile {
                body, annotation, ..
            } => {
                let loop_label = self.make_loop_label();
                *annotation = Annotation::LoopLabel(loop_label.clone());
                self.label_loops(body, Some(loop_label))
            }
            Stmt::For {
                body, annotation, ..
            } => {
                let loop_label = self.make_loop_label();
                *annotation = Annotation::LoopLabel(loop_label.clone());
                self.label_loops(body, Some(loop_label))
            }
            Stmt::If {
                else_branch,
                then_branch,
                ..
            } => {
                self.label_loops(then_branch, current_label.clone())?;
                if let Some(else_branch) = else_branch {
                    self.label_loops(else_branch, current_label)?;
                }
                Ok(())
            }
            Stmt::Compound { block } => {
                for element in block.elements.iter_mut() {
                    if let BlockElement::Stmt(stmt) = element {
                        self.label_loops(stmt, current_label.clone())?;
                    }
                }
                Ok(())
            }

            // NEW: descend into switch to label nested loops and jump statements
            Stmt::Switch { body, .. } => {
                self.label_loops(body, current_label.clone())?;
                Ok(())
            }

            // NEW: descend into case/default bodies
            Stmt::Case { body, .. } => {
                self.label_loops(body, current_label.clone())?;
                Ok(())
            }
            Stmt::Default { body } => {
                self.label_loops(body, current_label.clone())?;
                Ok(())
            }

            _ => Ok(()),
        }
    }
}
