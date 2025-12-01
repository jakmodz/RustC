use ast::ast::Stmt;
use crate::instruction_builder::InstructionBuilder;
use crate::tacky;
use crate::tacky::TackyInstruction;
use crate::tacky_parser::TackyParser;
use crate::convert_expr::*;
pub trait StatementConverter {
    fn convert_stmt(&mut self, stmt: Stmt, body: &mut Vec<tacky::TackyInstruction>);
}
impl StatementConverter for TackyParser {
    fn convert_stmt(&mut self, stmt: Stmt, body: &mut Vec<TackyInstruction>) {
        match stmt {
            Stmt::Return { expr } => {
                let val = self.convert_expr(expr, body);
                InstructionBuilder::new(body).return_val(val);
            }
            Stmt::Expression { expr } => {
                self.convert_expr(expr, body);
            }
            Stmt::Null => {}
            Stmt::If {
                condition,
                then_branch,
                else_branch,
            } => {
                let c = self.convert_expr(condition, body);
                if let Some(else_branch) = else_branch {
                    let else_label = self.label_generator.generate_label("else_label");
                    let end_label = self.label_generator.generate_label("end_label");

                    InstructionBuilder::new(body).jump_if_zero(c, else_label.clone());
                    self.convert_stmt(*then_branch, body);
                    InstructionBuilder::new(body).jump(end_label.clone());
                    InstructionBuilder::new(body).label(else_label);
                    self.convert_stmt(*else_branch, body);
                    InstructionBuilder::new(body).label(end_label);
                } else {
                    let end_label = self.label_generator.generate_label("end_label");

                    InstructionBuilder::new(body).jump_if_zero(c, end_label.clone());
                    self.convert_stmt(*then_branch, body);
                    InstructionBuilder::new(body).label(end_label);
                }
            }
            Stmt::Goto(label) => {
                InstructionBuilder::new(body).jump(label);
            }
            Stmt::Label(label) => {
                InstructionBuilder::new(body).label(label);
            }
            Stmt::Compound { block } => {
                self.convert_block(block, body);
            }
            Stmt::Continue(annotation) | Stmt::Break(annotation) => {
                self.convert_annotation(body, annotation);
            }
            Stmt::DoWhile {
                condition,
                body: loop_body,
                annotation,
            } => {
                let start_label = self.label_generator.generate_label("do_start");
                let end_label = self.label_generator.generate_label("do_end");

                InstructionBuilder::new(body).label(start_label.clone());
                self.convert_stmt(*loop_body, body);
                let c = self.convert_expr(condition, body);
                InstructionBuilder::new(body).jump_if_zero(c, end_label.clone());
                InstructionBuilder::new(body).jump(start_label.clone());
                InstructionBuilder::new(body).label(end_label);
                // propagate any annotation (e.g. loop labels)
                self.convert_annotation(body, annotation);
            }
            _ => todo!(),
        }
    }
}
