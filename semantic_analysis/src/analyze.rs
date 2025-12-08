use crate::GotoAnalyze;
use crate::SemanticError;
use crate::map_entry::SymbolKind;
use crate::map_entry::VariableEntry;
use crate::switch_analyze::SwitchAnalyzer;
use crate::symbol_entry::SymbolEntry;
use crate::type_checker::TypeChecker;
use ast::Declaration;
use ast::Expression;
use ast::Stmt;
use ast::ast::BlockElement;
use ast::ast::Program;
use ast::ast::*;
use std::collections::{HashMap, HashSet};

pub struct SemanticAnalyzer {
    pub(crate) variables: HashMap<String, VariableEntry>,
    pub(crate) labels: HashSet<String>,
    pub(crate) symbol_table: HashMap<String, SymbolEntry>,
    pub var_count: usize,
    pub(crate) loop_count: usize,
    pub(crate) switch_count: usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            labels: HashSet::new(),
            symbol_table: HashMap::new(),
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
        self.type_checking(ast)?;
        self.analyze_goto_statements(ast)?;
        self.analyze_control_flow(ast)?;
        Ok(())
    }

    pub fn variable_resolution(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        for func in ast.functions.iter_mut() {
            let resolved = self.resolve_function_declaration(&func.clone())?;
            if let Declaration::FuncDecl { decl: new_func } = resolved {
                *func = new_func;
            }
        }
        Ok(())
    }
    fn resolve_declaration(&mut self, decl: &Declaration) -> Result<Declaration, SemanticError> {
        match decl {
            Declaration::DefineVar(var) => {
                let mut self_variables = self.variables.clone();
                let unique_name = self.insert_variable(&var.name, &mut self_variables)?;

                self.variables = self_variables;
                let resolved_init = match var.init.as_ref() {
                    Some(expr) => Some(self.resolve_expression(&expr)?),
                    None => None,
                };

                Ok(Declaration::DefineVar(VariableDecl {
                    name: unique_name,
                    init: resolved_init,
                }))
            }
            Declaration::FuncDecl { decl: declaration } => {
                Ok(self.resolve_function_declaration(declaration)?)
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
            Stmt::Switch {
                expr,
                body,
                annotation,
                cases,
                default_label,
            } => {
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
            Expression::FunctionCall { func_name, args } => {
                if self.variables.contains_key(func_name) {
                    let entry = self.variables.get(func_name).unwrap();
                    if !matches!(entry.kind, crate::map_entry::SymbolKind::Fun) {
                        return Err(SemanticError::VariableAsFunctionName {
                            var_name: func_name.clone(),
                        });
                    }
                    let new_name = self.get_resolved_function_name(func_name)?;
                    let mut resolved_args = Vec::new();
                    for arg in args.iter() {
                        resolved_args.push(self.resolve_expression(arg)?);
                    }

                    return Ok(Expression::FunctionCall {
                        func_name: new_name,
                        args: resolved_args,
                    });
                }
                Err(SemanticError::UndeclaredFunction {
                    func_name: func_name.clone(),
                })
            }
            _ => Ok(expression.clone()),
        }
    }

    fn resolve_for_init(&mut self, init: &ForInit) -> Result<ForInit, SemanticError> {
        match init {
            ForInit::Declaration(decl) => {
                let decl = Declaration::DefineVar(decl.clone());
                let res = self.resolve_declaration(&decl)?;
                Ok(ForInit::Declaration(
                    if let Declaration::DefineVar(var_decl) = res {
                        var_decl
                    } else {
                        unreachable!()
                    },
                ))
            }
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
    fn resolve_function_declaration(
        &mut self,
        decl: &FuncDecl,
    ) -> Result<Declaration, SemanticError> {
        if let Some(prev) = self.variables.get(&decl.name) {
            if prev.from_current_block && !prev.has_linkage {
                return Err(SemanticError::MultipleDeclaration {
                    var_name: decl.name.clone(),
                });
            }
        }
        self.variables.insert(
            decl.name.clone(),
            VariableEntry::new(decl.name.clone(), true, SymbolKind::Fun, true),
        );

        let mut inner_map = self.variables.clone();

        for entry in inner_map.values_mut() {
            entry.from_current_block = false;
        }

        let mut resolved_params = Vec::new();
        for param in decl.params.iter() {
            let unique = self.insert_variable(param, &mut inner_map)?;
            resolved_params.push(unique);
        }

        let resolved_body = if let Some(body) = &decl.body {
            let saved_scope = std::mem::replace(&mut self.variables, inner_map);

            let resolved_elements = self.resolve_block(&body.elements)?;

            let _ = std::mem::replace(&mut self.variables, saved_scope);

            Some(Block::new(resolved_elements))
        } else {
            None
        };

        Ok(Declaration::FuncDecl {
            decl: FuncDecl {
                name: decl.name.clone(),
                params: resolved_params,
                body: resolved_body,
            },
        })
    }
    fn get_resolved_function_name(&mut self, name: &String) -> Result<String, SemanticError> {
        Ok(name.clone())
    }
    fn insert_variable(
        &mut self,
        var: &String,
        map: &mut HashMap<String, VariableEntry>,
    ) -> Result<String, SemanticError> {
        if let Some(entry) = map.get(var) {
            if entry.from_current_block {
                return Err(SemanticError::MultipleDeclaration {
                    var_name: var.clone(),
                });
            }
        }

        let unique_name = format!("{}.{}", var, self.var_count);
        self.var_count += 1;

        map.insert(
            var.clone(),
            VariableEntry::new(unique_name.clone(), true, SymbolKind::Var, false),
        );
        Ok(unique_name)
    }
}
