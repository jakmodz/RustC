use crate::tacky;
use crate::tacky::Val::Constant;
use crate::tacky::{BinaryOp, TackyFunction, TackyInstruction, Val};
use ast::ast::{Expression, Program};
use lex::token::TokenType;

pub struct TackyParser {
    pub var_counter: usize,
    pub label_counter: usize,
}

impl TackyParser {
    pub fn new(var_counter: usize) -> Self {
        Self {
            var_counter,
            label_counter: 0,
        }
    }
    fn make_temporary(&mut self) -> String {
        let s = format!("tmp.{}", self.var_counter);
        self.var_counter += 1;
        s
    }
    fn generate_label(&mut self, name: &str, instructions: &mut Vec<tacky::TackyInstruction>) {
        let str = format!("{}_{}", name, self.label_counter);
        self.label_counter += 1;
        instructions.push(TackyInstruction::Label(str));
    }
    fn get_label(&mut self, name: String) -> String {
        format!("{}_{}", name, self.label_counter)
    }
    pub fn emit_tacky(&mut self, ast: Program) -> tacky::Program {
        let mut body = Vec::new();

        for element in ast.function.body {
            self.convert_block_element(element, &mut body);
        }
        body.push(TackyInstruction::Return(Constant(0)));
        tacky::Program {
            function: TackyFunction {
                name: ast.function.name,
                body,
            },
        }
    }
    fn convert_block_element(
        &mut self,
        element: ast::ast::BlockElement,
        body: &mut Vec<tacky::TackyInstruction>,
    ) {
        match element {
            ast::ast::BlockElement::Stmt(stmt) => {
                self.convert_stmt(stmt, body);
            }
            ast::ast::BlockElement::Declaration(decl) => {
                self.convert_declaration(decl, body);
            }
        }
    }
    fn convert_declaration(
        &mut self,
        decl: ast::ast::Declaration,
        body: &mut Vec<tacky::TackyInstruction>,
    ) {
        match decl {
            ast::ast::Declaration::DefineVar {
                var_name,
                initializer,
            } => {
                if let Some(init_expr) = initializer {
                    let val = self.convert_expr(init_expr, body);
                    body.push(tacky::TackyInstruction::Copy {
                        src: val,
                        dst: Val::Var(var_name),
                    });
                }
            }
        }
    }
    fn convert_stmt(&mut self, stmt: ast::ast::Stmt, body: &mut Vec<tacky::TackyInstruction>) {
        match stmt {
            ast::ast::Stmt::Return { expr } => {
                let val = self.convert_expr(expr, body);
                body.push(tacky::TackyInstruction::Return(val));
            }
            ast::ast::Stmt::Expression { expr } => {
                self.convert_expr(expr, body);
            }
            ast::ast::Stmt::Null => {}
            ast::ast::Stmt::If { .. } => {
                todo!()
            }
        }
    }
    fn convert_expr(
        &mut self,
        expr: ast::ast::Expression,
        instructions: &mut Vec<TackyInstruction>,
    ) -> tacky::Val {
        match expr {
            ast::ast::Expression::Assignment {
                expr_to,
                initializer,
            } => {
                let result = self.convert_expr(*initializer, instructions);
                match expr_to.as_ref() {
                    ast::ast::Expression::Var(name) => {
                        instructions.push(TackyInstruction::Copy {
                            src: result,
                            dst: Val::Var(name.clone()),
                        });
                        Val::Var(name.clone())
                    }
                    _ => unreachable!(),
                }
            }
            ast::ast::Expression::CompoundAssign { op, var, expr } => {
                let rhs = self.convert_expr(*expr, instructions);
                let binary_op = match op.get_token_type() {
                    TokenType::PlusEqual => BinaryOp::Add,
                    TokenType::StarEqual => BinaryOp::Multiply,
                    TokenType::HypenEqual => BinaryOp::Subtract,
                    TokenType::SlashEqual => BinaryOp::Divide,
                    TokenType::PercentEqual => BinaryOp::Modulo,
                    TokenType::LeftShiftEqual => BinaryOp::LeftShift,
                    TokenType::RightShiftEqual => BinaryOp::RightShift,
                    TokenType::AmpersandEqual => BinaryOp::And,
                    TokenType::CaretEqual => BinaryOp::Xor,
                    TokenType::PipeEqual => BinaryOp::Or,
                    _ => panic!("Unsupported compound assignment"),
                };
                let name = match var.as_ref() {
                    ast::ast::Expression::Var(name) => name.clone(),
                    _ => panic!("Compound assignment target must be a variable"),
                };

                let lhs = Val::Var(name.clone());

                instructions.push(TackyInstruction::Binary {
                    binary_op,
                    src1: lhs.clone(),
                    src2: rhs,
                    dst: lhs.clone(),
                });

                lhs
            }
            ast::ast::Expression::Var(name) => Val::Var(name),
            ast::ast::Expression::Increment { expr, pre } => {
                self.convert_inc_dec(expr, instructions, pre, BinaryOp::Add)
            }
            ast::ast::Expression::Decrement { expr, pre } => {
                self.convert_inc_dec(expr, instructions, pre, BinaryOp::Subtract)
            }
            ast::ast::Expression::Constant(c) => tacky::Val::Constant(c),
            ast::ast::Expression::UnaryOP { op, expr } => {
                let src = self.convert_expr(*expr, instructions);
                let dst = Val::Var(self.make_temporary());
                let unary_op = match op.get_token_type() {
                    TokenType::Hypen => tacky::UnaryOp::Negate,
                    TokenType::Tilde => tacky::UnaryOp::Complement,
                    TokenType::Exclamation => tacky::UnaryOp::Not,
                    _ => panic!("Unsupported unary operator"),
                };

                instructions.push(TackyInstruction::Unary {
                    unary_op,
                    src,
                    dst: dst.clone(),
                });
                dst
            }
            ast::ast::Expression::Grouping { expr } => self.convert_expr(*expr, instructions),
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
                            target: false_label.clone(),
                        });

                        let v2 = self.convert_expr(*right, instructions);
                        instructions.push(TackyInstruction::JumpIfZero {
                            cond: v2,
                            target: false_label.clone(),
                        });

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(1),
                            dst: Val::Var(result_var.clone()),
                        });

                        instructions.push(TackyInstruction::Jump {
                            target: end_label.clone(),
                        });

                        instructions.push(TackyInstruction::Label(false_label));

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(0),
                            dst: Val::Var(result_var.clone()),
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

                        let v2 = self.convert_expr(*left, instructions);
                        instructions.push(TackyInstruction::JumpIfNotZero {
                            cond: v2,
                            target: true_label.clone(),
                        });

                        let v1 = self.convert_expr(*right, instructions);
                        instructions.push(TackyInstruction::JumpIfNotZero {
                            cond: v1,
                            target: true_label.clone(),
                        });

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(0),
                            dst: Val::Var(result_var.clone()),
                        });

                        instructions.push(TackyInstruction::Jump {
                            target: end_label.clone(),
                        });

                        instructions.push(TackyInstruction::Label(true_label));

                        instructions.push(TackyInstruction::Copy {
                            src: Constant(1),
                            dst: Val::Var(result_var.clone()),
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
                    TokenType::LeftShift => BinaryOp::LeftShift,
                    TokenType::RightShift => BinaryOp::RightShift,
                    TokenType::Ampersand => BinaryOp::And,
                    TokenType::Caret => BinaryOp::Xor,
                    TokenType::Pipe => BinaryOp::Or,
                    TokenType::Greater => BinaryOp::GreaterThan,
                    TokenType::Less => BinaryOp::LessThan,
                    TokenType::LessEqual => BinaryOp::LeesOrEqual,
                    TokenType::GreaterEqual => BinaryOp::GreaterOrEqual,
                    TokenType::EqualEqual => BinaryOp::Equal,
                    TokenType::ExclamationEqual => BinaryOp::NotEqual,
                    _ => panic!("Unsupported binary operator"),
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
    fn convert_inc_dec(
        &mut self,
        expr: Box<Expression>,
        instructions: &mut Vec<TackyInstruction>,
        pre: bool,
        binary_op: BinaryOp,
    ) -> Val {
        let var_name = self
            .get_var_name(&expr)
            .unwrap_or_else(|| panic!("Increment/decrement operator requires an lvalue"));
        let var = Val::Var(var_name);
        if pre {
            instructions.push(TackyInstruction::Binary {
                binary_op,
                src1: var.clone(),
                src2: Constant(1),
                dst: var.clone(),
            });
            var
        } else {
            let tmp = Val::Var(self.make_temporary());

            instructions.push(TackyInstruction::Copy {
                src: var.clone(),
                dst: tmp.clone(),
            });

            instructions.push(TackyInstruction::Binary {
                binary_op,
                src1: var.clone(),
                src2: Constant(1),
                dst: var.clone(),
            });

            tmp
        }
    }
    fn get_var_name(&mut self, expr: &Expression) -> Option<String> {
        match expr {
            ast::ast::Expression::Grouping { expr } => self.get_var_name(expr),
            ast::ast::Expression::Var(name) => Some(name.clone()),
            _ => None,
        }
    }
}
