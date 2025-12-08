use ast::{
    Declaration, Expression, Stmt, VarType,
    ast::{Block, BlockElement, ForInit, FuncDecl, Program, VariableDecl},
};

use crate::{SemanticAnalyzer, SemanticError, symbol_entry::SymbolEntry};

pub trait TypeChecker {
    fn type_checking(&mut self, ast: &Program) -> Result<(), SemanticError>;
    fn check_types_var_decl(&mut self, decl: &VariableDecl) -> Result<(), SemanticError>;
    fn register_func_decl(&mut self, decl: &FuncDecl) -> Result<(), SemanticError>;
    fn check_func_body(&mut self, decl: &FuncDecl) -> Result<(), SemanticError>;
    fn check_types_func_decl(&mut self, decl: &FuncDecl) -> Result<(), SemanticError>;

    fn check_types_block(&mut self, block: &Block) -> Result<(), SemanticError>;
    fn check_types_expr(&mut self, expr: &Expression) -> Result<(), SemanticError>;
    fn check_types_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError>;
}

impl TypeChecker for SemanticAnalyzer {
    fn type_checking(&mut self, ast: &Program) -> Result<(), SemanticError> {
        for func in ast.functions.iter() {
            self.register_func_decl(func)?;
        }

        for func in ast.functions.iter() {
            self.check_func_body(func)?;
        }

        Ok(())
    }

    fn check_types_var_decl(&mut self, decl: &VariableDecl) -> Result<(), SemanticError> {
        if !self.symbol_table.contains_key(&decl.name) {
            self.symbol_table.insert(
                decl.name.clone(),
                SymbolEntry::Variable {
                    var_type: VarType::Int,
                },
            );
        }
        if let Some(init) = &decl.init {
            self.check_types_expr(init)?;
        }
        Ok(())
    }

    fn check_types_expr(&mut self, expr: &Expression) -> Result<(), SemanticError> {
        match expr {
            Expression::FunctionCall { func_name, args } => {
                let f_type = self.symbol_table.get(func_name);
                if f_type.is_none() {
                    return Err(SemanticError::UndeclaredFunction {
                        func_name: func_name.clone(),
                    });
                }
                match f_type.unwrap() {
                    SymbolEntry::Function { var_type, .. } => {
                        if *var_type == VarType::Int {
                            return Err(SemanticError::VariableAsFunctionName {
                                var_name: func_name.clone(),
                            });
                        }
                        if var_type.get_param_count() != args.len() {
                            return Err(SemanticError::WrongNumberOfArguments {
                                func_name: func_name.clone(),
                                excepted: var_type.get_param_count(),
                                got: args.len(),
                            });
                        }
                        for arg in args.iter() {
                            self.check_types_expr(arg)?;
                        }
                    }
                    _ => {
                        return Err(SemanticError::VariableAsFunctionName {
                            var_name: func_name.clone(),
                        });
                    }
                }
            }
            Expression::Assignment {
                expr_to,
                initializer,
            } => {
                self.check_types_expr(expr_to)?;
                self.check_types_expr(initializer)?;
            }
            Expression::Var(name) => {
                if self.symbol_table.contains_key(name) {
                    if self.symbol_table.get(name).unwrap().get_var_type() != VarType::Int {
                        return Err(SemanticError::FunctionAsVariableName {
                            func_name: name.clone(),
                        });
                    }
                } else {
                    return Err(SemanticError::UndeclaredVariable {
                        var_name: name.clone(),
                    });
                }
            }
            Expression::UnaryOP { expr, .. } => {
                self.check_types_expr(expr)?;
            }
            Expression::Grouping { expr } => {
                self.check_types_expr(expr)?;
            }
            Expression::Binary { left, right, .. } => {
                self.check_types_expr(left)?;
                self.check_types_expr(right)?;
            }
            Expression::CompoundAssign { var, expr, .. } => {
                self.check_types_expr(var)?;
                self.check_types_expr(expr)?;
            }
            Expression::Increment { expr, .. } => {
                self.check_types_expr(expr)?;
            }
            Expression::Decrement { expr, .. } => {
                self.check_types_expr(expr)?;
            }
            Expression::Conditional { cond, expr1, expr2 } => {
                self.check_types_expr(cond)?;
                self.check_types_expr(expr1)?;
                self.check_types_expr(expr2)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn register_func_decl(&mut self, decl: &FuncDecl) -> Result<(), SemanticError> {
        let fun_type = VarType::FunType {
            param_count: decl.params.len(),
        };
        let has_body = decl.body.is_some();
        let mut aleready_defined = false;
        if self.symbol_table.contains_key(&decl.name) {
            let old_entry = self.symbol_table.get(&decl.name).unwrap();
            match old_entry {
                SymbolEntry::Function {
                    already_defined: defined,
                    var_type: old_type,
                } => {
                    aleready_defined = *defined;
                    let params = old_type.get_param_count();
                    let decl_param_count = decl.params.len();
                    if params != decl_param_count {
                        return Err(SemanticError::WrongNumberOfArguments {
                            func_name: decl.name.clone(),
                            excepted: old_type.get_param_count(),
                            got: decl.params.len(),
                        });
                    }

                    if aleready_defined && has_body {
                        return Err(SemanticError::DuplicateFunctionDefinition {
                            func_name: decl.name.clone(),
                        });
                    }
                }
                SymbolEntry::Variable { .. } => {
                    return Err(SemanticError::VariableAsFunctionName {
                        var_name: decl.name.clone(),
                    });
                }
            }
        }
        self.symbol_table.insert(
            decl.name.clone(),
            SymbolEntry::Function {
                var_type: fun_type,
                already_defined: aleready_defined || has_body,
            },
        );
        Ok(())
    }

    fn check_func_body(&mut self, decl: &FuncDecl) -> Result<(), SemanticError> {
        if decl.body.is_some() {
            let mut global_scope = std::mem::take(&mut self.symbol_table);

            let mut function_scope = global_scope.clone();

            for param in decl.params.iter() {
                function_scope.insert(
                    param.clone(),
                    SymbolEntry::Variable {
                        var_type: VarType::Int,
                    },
                );
            }

            self.symbol_table = function_scope;
            if let Some(body) = &decl.body {
                self.check_types_block(body)?;
            }

            for (name, entry) in self.symbol_table.iter() {
                if matches!(entry, SymbolEntry::Function { .. }) {
                    global_scope.insert(name.clone(), entry.clone());
                }
            }

            self.symbol_table = global_scope;
        }
        Ok(())
    }

    fn check_types_func_decl(&mut self, decl: &FuncDecl) -> Result<(), SemanticError> {
        self.register_func_decl(decl)
    }

    fn check_types_block(&mut self, block: &Block) -> Result<(), SemanticError> {
        let mut outer_scope = std::mem::take(&mut self.symbol_table);
        let inner_scope = outer_scope.clone();
        self.symbol_table = inner_scope;

        for element in block.elements.iter() {
            match element {
                BlockElement::Declaration(decl) => match decl {
                    Declaration::DefineVar(var_decl) => {
                        self.check_types_var_decl(var_decl)?;
                    }
                    Declaration::FuncDecl { decl } => {
                        if decl.body.is_some() {
                            return Err(SemanticError::DuplicateFunctionDefinition {
                                func_name: decl.name.clone(),
                            });
                        }
                        self.check_types_func_decl(decl)?;
                    }
                },
                BlockElement::Stmt(stmt) => {
                    self.check_types_stmt(stmt)?;
                }
            }
        }

        for (name, entry) in self.symbol_table.iter() {
            if matches!(entry, SymbolEntry::Function { .. }) {
                outer_scope.insert(name.clone(), entry.clone());
            }
        }

        self.symbol_table = outer_scope;
        Ok(())
    }

    fn check_types_stmt(&mut self, stmt: &Stmt) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Expression { expr } => {
                self.check_types_expr(expr)?;
            }
            Stmt::Return { expr } => {
                self.check_types_expr(expr)?;
            }
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.check_types_expr(condition)?;
                self.check_types_stmt(then_branch)?;
                if let Some(else_branch) = else_branch {
                    self.check_types_stmt(else_branch)?;
                }
            }
            Stmt::While {
                condition, body, ..
            } => {
                self.check_types_expr(condition)?;
                self.check_types_stmt(body)?;
            }
            Stmt::For {
                init,
                condition,
                increment,
                body,
                ..
            } => {
                let outer_scope = std::mem::take(&mut self.symbol_table);
                let inner_scope = outer_scope.clone();
                self.symbol_table = inner_scope;

                match init {
                    ForInit::Declaration(variable_decl) => {
                        self.check_types_var_decl(variable_decl)?;
                    }
                    ForInit::Expression(expression) => {
                        if let Some(expr) = expression {
                            self.check_types_expr(expr)?;
                        }
                    }
                }
                if let Some(condition) = condition {
                    self.check_types_expr(condition)?;
                }
                if let Some(increment) = increment {
                    self.check_types_expr(increment)?;
                }
                self.check_types_stmt(body)?;

                self.symbol_table = outer_scope;
            }
            Stmt::DoWhile {
                body, condition, ..
            } => {
                self.check_types_stmt(body)?;
                self.check_types_expr(condition)?;
            }
            Stmt::Compound { block } => self.check_types_block(block)?,
            Stmt::Switch { expr, body, .. } => {
                self.check_types_expr(expr)?;
                self.check_types_stmt(body)?;
            }
            Stmt::Case { value, body } => {
                self.check_types_expr(value)?;
                self.check_types_stmt(body)?;
            }
            Stmt::Default { body } => {
                self.check_types_stmt(body)?;
            }
            _ => {}
        }
        Ok(())
    }
}
