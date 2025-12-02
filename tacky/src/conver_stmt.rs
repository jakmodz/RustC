use ast::ast::{Annotation, Stmt};
use crate::instruction_builder::InstructionBuilder;
use crate::tacky;
use crate::tacky::{TackyInstruction};
use crate::tacky_parser::TackyParser;
use crate::convert_expr::*;
pub trait StatementConverter {
    fn convert_stmt(&mut self, stmt: Stmt, instructions: &mut Vec<tacky::TackyInstruction>);
}
impl StatementConverter for TackyParser {
    fn convert_stmt(&mut self, stmt: Stmt, instructions: &mut Vec<TackyInstruction>) {
        match stmt {
            Stmt::Return { expr } => {
                let val = self.convert_expr(expr, instructions);
                InstructionBuilder::new(instructions).return_val(val);
            }
            Stmt::Expression { expr } => {
                self.convert_expr(expr, instructions);
            }
            Stmt::Null => {}
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let c = self.convert_expr(condition, instructions);
                if let Some(else_branch) = else_branch {
                    let else_label = self.label_generator.generate_label("else_label");
                    let end_label = self.label_generator.generate_label("end_label");

                    InstructionBuilder::new(instructions).jump_if_zero(c, else_label.clone());
                    self.convert_stmt(*then_branch, instructions);
                    InstructionBuilder::new(instructions).jump(end_label.clone());
                    InstructionBuilder::new(instructions).label(else_label);
                    self.convert_stmt(*else_branch, instructions);
                    InstructionBuilder::new(instructions).label(end_label);
                } else {
                    let end_label = self.label_generator.generate_label("end_label");

                    InstructionBuilder::new(instructions).jump_if_zero(c, end_label.clone());
                    self.convert_stmt(*then_branch, instructions);
                    InstructionBuilder::new(instructions).label(end_label);
                }
            }
            Stmt::Goto(label) => {
                InstructionBuilder::new(instructions).jump(label);
            }
            Stmt::Label(label) => {
                InstructionBuilder::new(instructions).label(label);
            }
            Stmt::Compound { block } => {
                self.convert_block(block, instructions);
            }
            Stmt::Continue(annotation) => {
                match annotation {
                    Annotation::LoopLabel(label) => {
                        let continue_label = format!("{}_continue", label);
                        InstructionBuilder::new(instructions).jump(continue_label);
                    }
                    _ => {}
                }
            }
            Stmt::Break(annotation) => {
                match annotation {
                    Annotation::LoopLabel(label) => {
                        let end_label = format!("{}_end", label);
                        InstructionBuilder::new(instructions).jump(end_label);
                    }
                    _ => {}
                }
            }
            Stmt::DoWhile {
                condition,
                body: loop_body,
                annotation,
                ..
            } => {

                let loop_label = match annotation {
                    Annotation::LoopLabel(label) => label,
                    _ => self.label_generator.generate_label("do"),
                };

                let start_label = format!("{}_start", loop_label);
                let continue_label = format!("{}_continue", loop_label);
                let end_label = format!("{}_end", loop_label);

                InstructionBuilder::new(instructions).label(start_label.clone());
                self.convert_stmt(*loop_body, instructions);
                InstructionBuilder::new(instructions).label(continue_label.clone());
                let v = self.convert_expr(condition, instructions);
                InstructionBuilder::new(instructions)
                    .jump_if_not_zero(v, start_label.clone())
                    .label(end_label);
            }
            Stmt::While { body, condition, annotation, .. } => {

                let loop_label = match annotation {
                    Annotation::LoopLabel(label) => label,
                    _ => self.label_generator.generate_label("while"),
                };

                let continue_label = format!("{}_continue", loop_label);
                let end_label = format!("{}_end", loop_label);

                InstructionBuilder::new(instructions).label(continue_label.clone());
                let c = self.convert_expr(condition, instructions);
                InstructionBuilder::new(instructions).jump_if_zero(c, end_label.clone());
                self.convert_stmt(*body, instructions);
                InstructionBuilder::new(instructions)
                    .jump(continue_label.clone())
                    .label(end_label);
            }
            Stmt::For {
                init,
                condition,
                increment,
                body,
                annotation,
            } => {
                let loop_label = match annotation {
                    Annotation::LoopLabel(label) => label,
                    _ => self.label_generator.generate_label("for"),
                };

                let start_label = format!("{}_start", loop_label);
                let continue_label = format!("{}_continue", loop_label);
                let end_label = format!("{}_end", loop_label);

                self.convert_for_init(init, instructions);

                InstructionBuilder::new(instructions).label(start_label.clone());

                if let Some(cond) = condition {
                    let condition_val = self.convert_expr(cond, instructions);
                    InstructionBuilder::new(instructions).jump_if_zero(condition_val, end_label.clone());
                }

                self.convert_stmt(*body, instructions);

                InstructionBuilder::new(instructions).label(continue_label.clone());

                if let Some(inc) = increment {
                    self.convert_expr(inc, instructions);
                }

                InstructionBuilder::new(instructions).jump(start_label.clone());

                InstructionBuilder::new(instructions).label(end_label);
            }
        }
    }
}
