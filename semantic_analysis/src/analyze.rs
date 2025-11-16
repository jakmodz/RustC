use ast::ast::Declaration;
use ast::ast::Program;
use crate::SemanticError;
use std::collections::HashMap;
use ast::ast::*;
use ast::ast::BlockElement;

pub struct SemanticAnalyzer {
    variables: HashMap<String,String>,
    pub var_count : usize,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            var_count: 0,
        }
    }

    pub fn semantic_analysis(&mut self,ast:&mut Program) ->Result<(),SemanticError> {

        for element in ast.function.body.iter_mut() {
            match element {
                BlockElement::Declaration(decl) => {
                    *element = BlockElement::Declaration(self.analyze_declaration(decl)?);
                }
                BlockElement::Stmt(stmt) => {
                    *element = BlockElement::Stmt(self.resolve_statement(stmt)?);
                }
            }
        }
        Ok(())
    }

    fn analyze_declaration(&mut self,decl:&Declaration)->Result<Declaration,SemanticError>{
        match decl {
            Declaration::DefineVar {var_name,initializer} => {
                if self.variables.contains_key(var_name) {
                    return Err(SemanticError::MultipleDeclaration {var_name:var_name.to_string()});
                }
                let unique_name = format!("{}.{}",var_name,self.var_count);
                self.var_count += 1;
                self.variables.insert(var_name.to_string(),unique_name.clone());
                let mut init = initializer.clone();
                if let Some(expr) = initializer {
                    init = Some(self.resolve_expression(expr)?);
                }
                Ok(Declaration::DefineVar {
                    var_name: unique_name,
                    initializer: init,
                })
            }
        }
    }
    fn resolve_statement(&mut self,stmt:&Stmt)->Result<Stmt,SemanticError>{
        match stmt {
            Stmt::Expression{expr}=>{
                let resolved_expr = self.resolve_expression(expr)?;
                Ok(Stmt::Expression{expr:resolved_expr})
            }
            Stmt::Return{expr}=>{
                let resolved_expr = self.resolve_expression(expr)?;
                Ok(Stmt::Return{expr:resolved_expr})
            }
            Stmt::Null=>{
                Ok(Stmt::Null)
            },
            Stmt::If { .. } => todo!()
        }
    }
    fn resolve_expression(&mut self,expression:&Expression)->Result<Expression,SemanticError>{
        match expression  {
            Expression::Assignment {expr_to,initializer}=>{
                match expr_to.as_ref()  {
                    Expression::Var(var_name)=>{
                         Ok(Expression::Assignment {
                             expr_to: Box::new(self.resolve_expression(expr_to)?),
                             initializer: Box::new(self.resolve_expression(initializer)?),
                         })
                    }
                    _=>{
                        Err(SemanticError::InvalidLeftValue {invalid:format!("{:?}",expr_to)} )
                    }
                }
            }
            Expression::UnaryOP {expr,op}=>{
                Ok(Expression::UnaryOP {op: op.clone(),expr:Box::new(self.resolve_expression(expr)?)})
            }
            Expression::Var(var_name)=>{
                if self.variables.contains_key(var_name) {
                     Ok(Expression::Var(self.variables.get(var_name).unwrap().to_string()))
                }else{
                    Err(SemanticError::UndeclaredVariable {var_name:var_name.to_string()})
                }
            }
            Expression::Grouping {expr  }=>{
                Ok(Expression::Grouping {expr:Box::new(self.resolve_expression(expr)?)})
            }
            Expression::Binary {op,left,right}=>{
                Ok(Expression::Binary {
                    op:op.clone(),
                    left:Box::new(self.resolve_expression(left)?),
                    right:Box::new(self.resolve_expression(right)?),
                })
            }
            _=>{
                Ok(expression.clone())
            }
        }
    }
}

