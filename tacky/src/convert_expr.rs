use crate::instruction_builder::InstructionBuilder;
use crate::tacky;
use crate::tacky::Val::Constant;
use crate::tacky::{BinaryOp, TackyInstruction, Val};
use crate::tacky_parser::TackyParser;

use ast::Expression;
use lex::token::TokenType;

pub trait ExpressionConverter {
    fn convert_expr(&mut self, expr: Expression, instructions: &mut Vec<TackyInstruction>) -> Val;
}

impl ExpressionConverter for TackyParser {
    fn convert_expr(&mut self, expr: Expression, instructions: &mut Vec<TackyInstruction>) -> Val {
        match expr {
            Expression::Assignment {
                expr_to,
                initializer,
            } => {
                let result = self.convert_expr(*initializer, instructions);
                match expr_to.as_ref() {
                    Expression::Var(name) => {
                        InstructionBuilder::new(instructions).copy(result, Val::Var(name.clone()));
                        Val::Var(name.clone())
                    }
                    _ => unreachable!(),
                }
            }
            Expression::CompoundAssign { op, var, expr } => {
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
                    Expression::Var(name) => name.clone(),
                    _ => panic!("Compound assignment target must be a variable"),
                };

                let lhs = Val::Var(name);
                InstructionBuilder::new(instructions).binary(
                    binary_op,
                    lhs.clone(),
                    rhs,
                    lhs.clone(),
                );

                lhs
            }
            Expression::Var(name) => Val::Var(name),
            Expression::Increment { expr, pre } => {
                self.convert_inc_dec(expr, instructions, pre, BinaryOp::Add)
            }
            Expression::Decrement { expr, pre } => {
                self.convert_inc_dec(expr, instructions, pre, BinaryOp::Subtract)
            }
            Expression::Constant(c) => Constant(c),
            Expression::UnaryOP { op, expr } => {
                let src = self.convert_expr(*expr, instructions);
                let dst = Val::Var(self.make_temporary());
                let unary_op = match op.get_token_type() {
                    TokenType::Hypen => tacky::UnaryOp::Negate,
                    TokenType::Tilde => tacky::UnaryOp::Complement,
                    TokenType::Exclamation => tacky::UnaryOp::Not,
                    _ => panic!("Unsupported unary operator"),
                };

                InstructionBuilder::new(instructions).unary(unary_op, src, dst.clone());
                dst
            }
            Expression::Grouping { expr } => self.convert_expr(*expr, instructions),
            Expression::Binary { op, left, right } => {
                match op.get_token_type() {
                    TokenType::AmpersandAmpersand => {
                        let result_var = self.make_temporary();

                        let false_label = self.label_generator.generate_label("false_label");
                        let end_label = self.label_generator.generate_label("end_label");

                        let v1 = self.convert_expr(*left, instructions);
                        InstructionBuilder::new(instructions).jump_if_zero(v1, false_label.clone());

                        let v2 = self.convert_expr(*right, instructions);
                        InstructionBuilder::new(instructions).jump_if_zero(v2, false_label.clone());

                        InstructionBuilder::new(instructions)
                            .copy(Constant(1), Val::Var(result_var.clone()))
                            .jump(end_label.clone())
                            .label(false_label)
                            .copy(Constant(0), Val::Var(result_var.clone()))
                            .label(end_label);

                        return Val::Var(result_var);
                    }

                    TokenType::PipePipe => {
                        let result_var = self.make_temporary();

                        let true_label = self.label_generator.generate_label("or_true");
                        let end_label = self.label_generator.generate_label("or_false");

                        let v2 = self.convert_expr(*left, instructions);
                        InstructionBuilder::new(instructions)
                            .jump_if_not_zero(v2, true_label.clone());

                        let v1 = self.convert_expr(*right, instructions);
                        InstructionBuilder::new(instructions)
                            .jump_if_not_zero(v1, true_label.clone());

                        InstructionBuilder::new(instructions)
                            .copy(Constant(0), Val::Var(result_var.clone()))
                            .jump(end_label.clone())
                            .label(true_label)
                            .copy(Constant(1), Val::Var(result_var.clone()))
                            .label(end_label);

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

                InstructionBuilder::new(instructions).binary(binary_op, v1, v2, dst.clone());
                dst
            }
            Expression::Conditional { cond, expr1, expr2 } => {
                let end_label = self.label_generator.generate_label("end_label");
                let e2 = self.label_generator.generate_label("e2_label");
                let c = self.convert_expr(*cond, instructions);
                InstructionBuilder::new(instructions).jump_if_zero(c, e2.clone());

                let v1 = self.convert_expr(*expr1, instructions);
                let result_var = self.make_temporary();

                InstructionBuilder::new(instructions)
                    .copy(v1, Val::Var(result_var.clone()))
                    .jump(end_label.clone())
                    .label(e2);

                let v2 = self.convert_expr(*expr2, instructions);

                InstructionBuilder::new(instructions)
                    .copy(v2, Val::Var(result_var.clone()))
                    .label(end_label);

                Val::Var(result_var)
            }
            Expression::FunctionCall { func_name: _, args: _ } => unreachable!(),
        }
    }
}
