use crate::tacky;
use crate::tacky::{BinaryOp, TackyFunction, TackyInstruction, Val};
use ast::ast::Program;
use lex::token::TokenType;
pub struct TackyParser {
    pub var_counter: usize
}

impl TackyParser {
    pub fn new() -> Self {
        Self {
            var_counter: 0
        }
    }
    fn make_temporary(&mut self) -> String {
        let str = format!("tmp{},", self.var_counter);
        self.var_counter += 1;
        str
    }
    pub fn emit_tacky(&mut self,ast:Program)->tacky::Program {

        let mut body = Vec::new();

        for stmt in ast.function.body {
                self.convert_stmt(stmt,&mut body);
        }


        tacky::Program {
            function:TackyFunction{
                name:ast.function.name,
                body
            }
        }
    }
    fn convert_stmt(&mut self, stmt:ast::ast::Stmt, body:&mut Vec<tacky::TackyInstruction>) {
        match stmt {
            ast::ast::Stmt::Return { expr } => {
                let val = self.convert_expr(expr,body);
                body.push(tacky::TackyInstruction::Return(val));
            }
            ast::ast::Stmt::If { .. } => {todo!()}
        }
    }
    fn convert_expr(&mut self, expr:ast::ast::Expression,instructions:&mut Vec<TackyInstruction>)->tacky::Val {
        match expr {
            ast::ast::Expression::Constant(c) => {
                tacky::Val::Constant(c)
            }
            ast::ast::Expression::UnaryOP {op,expr}=>{
                let src = self.convert_expr(*expr,instructions);
                let dst = tacky::Val::Var("tmp".to_string());
                let unary_op = match op.get_token_type() {
                    TokenType::Hypen => tacky::UnaryOp::Negate,
                    TokenType::Tilde => tacky::UnaryOp::Complement,
                    _ => panic!("Unsupported unary operator"),
                };

                instructions.push(TackyInstruction::Unary{
                    unary_op,
                    src,
                    dst:dst.clone()
                });
                dst
            }
            ast::ast::Expression::Grouping {expr}=>{
                self.convert_expr(*expr,instructions)
            }
            ast::ast::Expression::Binary { op, left, right } => {
                let v1 = self.convert_expr(*left, instructions);
                let v2 = self.convert_expr(*right, instructions);
                let dst_name = self.make_temporary();
                let dst = Val::Var(dst_name);
                let binary_op = match op.get_token_type() {
                    TokenType::Plus => BinaryOp::Add,
                    TokenType::Star => BinaryOp::Multiply,
                    TokenType::Hypen => BinaryOp::Subtract,
                    TokenType::Slash => BinaryOp::Divide,
                    TokenType::Percent => BinaryOp::Modulo,
                    _ => panic!("Unsupported binary operator")
                };
                instructions.push(TackyInstruction::Binary {
                    binary_op,
                    src1: v1,
                    src2: v2,
                    dst: dst.clone(),
                }
                );
                dst
            }
        }
    }
}