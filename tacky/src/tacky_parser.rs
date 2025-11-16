use crate::tacky;
use crate::tacky::Val::Constant;
use crate::tacky::{BinaryOp, TackyFunction, TackyInstruction, Val};
use ast::ast::Program;
use lex::token::TokenType;

pub struct TackyParser {
    pub var_counter: usize,
    pub label_counter:usize
}

impl TackyParser {
    pub fn new(var_counter: usize) -> Self {
        Self {
            var_counter,
            label_counter:0
        }
    }
    fn make_temporary(&mut self) -> String {
        let s = format!("tmp{}", self.var_counter);
        self.var_counter += 1;
        s
    }
    fn generate_label(&mut self,name:&str,instructions:&mut Vec<tacky::TackyInstruction>){
        let str = format!("{}_{}", name,self.label_counter);
        self.label_counter += 1;
        instructions.push(TackyInstruction::Label(str));
    }
    fn get_label(&mut self,name:String)->String{
        format!("{}_{}", name,self.label_counter)
    }
    pub fn emit_tacky(&mut self,ast:Program)->tacky::Program {
        let mut body = Vec::new();

        for element in ast.function.body {
                self.convert_block_element(element, &mut body);
        }

        tacky::Program {
            function:TackyFunction{
                name:ast.function.name,
                body
            }
        }
    }
    fn convert_block_element(&mut self, element:ast::ast::BlockElement, body:&mut Vec<tacky::TackyInstruction>) {
        match element {
            ast::ast::BlockElement::Stmt(stmt) => {
                self.convert_stmt(stmt,body);
            }
            ast::ast::BlockElement::Declaration(_) => {

                todo!()
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
            _=>todo!()
        }
    }
    fn convert_expr(&mut self, expr:ast::ast::Expression,instructions:&mut Vec<TackyInstruction>)->tacky::Val {
        match expr {
            ast::ast::Expression::Assignment {expr_to,initializer} => {
                todo!()
            }
            ast::ast::Expression::Var(name) => {
                todo!()
            },

            ast::ast::Expression::Constant(c) => {
                tacky::Val::Constant(c)
            }
            ast::ast::Expression::UnaryOP {op,expr}=>{
                let src = self.convert_expr(*expr,instructions);
                let dst = Val::Var(self.make_temporary());
                let unary_op = match op.get_token_type() {
                    TokenType::Hypen => tacky::UnaryOp::Negate,
                    TokenType::Tilde => tacky::UnaryOp::Complement,
                    TokenType::Exclamation => tacky::UnaryOp::Not,
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

                match op.get_token_type() {
                    TokenType::AmpersandAmpersand => {
                        let result_var = self.make_temporary();

                        let false_label = format!("false_label_{}", self.label_counter);
                        self.label_counter += 1;
                        let end_label = format!("end_label_{}", self.label_counter);
                        self.label_counter += 1;

                        let v1 = self.convert_expr(*left, instructions);
                        instructions.push(TackyInstruction::JumpIfZero {
                            cond: v1,
                            target: false_label.clone()
                        });

                        let v2 = self.convert_expr(*right, instructions);
                        instructions.push(TackyInstruction::JumpIfZero {
                            cond: v2,
                            target: false_label.clone()
                        });

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(1),
                            dst: Val::Var(result_var.clone())
                        });

                        instructions.push(TackyInstruction::Jump {
                            target: end_label.clone()
                        });

                        instructions.push(TackyInstruction::Label(false_label));

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(0),
                            dst: Val::Var(result_var.clone())
                        });

                        instructions.push(TackyInstruction::Label(end_label));

                        return Val::Var(result_var);
                    }

                    TokenType::PipePipe => {
                        let result_var = self.make_temporary();

                        let true_label = format!("or_true_{}", self.label_counter);
                        self.label_counter += 1;
                        let end_label = format!("or_end_{}", self.label_counter);
                        self.label_counter += 1;

                        let v1 = self.convert_expr(*left, instructions);
                        instructions.push(TackyInstruction::JumpIfNotZero {
                            cond: v1,
                            target: true_label.clone()
                        });

                        let v2 = self.convert_expr(*right, instructions);
                        instructions.push(TackyInstruction::JumpIfNotZero {
                            cond: v2,
                            target: true_label.clone()
                        });

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(0),
                            dst: Val::Var(result_var.clone())
                        });

                        instructions.push(TackyInstruction::Jump {
                            target: end_label.clone()
                        });

                        instructions.push(TackyInstruction::Label(true_label));

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(1),
                            dst: Val::Var(result_var.clone())
                        });
                        instructions.push(TackyInstruction::Label(end_label));

                        return Val::Var(result_var);
                    }
                    _ => {}
                }
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
                    TokenType::LeftShift=>BinaryOp::LeftShift,
                    TokenType::RightShift=>BinaryOp::RightShift,
                    TokenType::Ampersand=>BinaryOp::And,
                    TokenType::Caret=>BinaryOp::Xor,
                    TokenType::Pipe=>BinaryOp::Or,
                    TokenType::Greater=>BinaryOp::GreaterThan,
                    TokenType::Less=>BinaryOp::LessThan,
                    TokenType::LessEqual=>BinaryOp::LeesOrEqual,
                    TokenType::GreaterEqual=>BinaryOp::GreaterOrEqual,
                    TokenType::EqualEqual=>BinaryOp::Equal,
                    TokenType::ExclamationEqual=>BinaryOp::NotEqual,
                    _ => panic!("Unsupported binary operator")
                };

                instructions.push(TackyInstruction::Binary {
                    binary_op,
                    src1: v1,
                    src2: v2,
                    dst: dst.clone(),
                });
                dst
            }
        }
    }
}