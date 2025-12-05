// ============= Part 1: Fixed Statement Converter (conver_stmt.rs) =============
use crate::convert_expr::*;
use crate::instruction_builder::InstructionBuilder;
use crate::tacky;
use crate::tacky::Val::Constant;
use crate::tacky::{BinaryOp, TackyInstruction, Val};
use crate::tacky_parser::TackyParser;
use ast::ast::{Annotation, BlockElement, Expression, ForInit, Stmt, SwitchCase};

pub trait StatementConverter {
    fn convert_stmt(&mut self, stmt: Stmt, instructions: &mut Vec<tacky::TackyInstruction>);
    fn convert_stmt_with_case_labels(
        &mut self,
        stmt: Stmt,
        instructions: &mut Vec<TackyInstruction>,
        cases: &[SwitchCase],
        default_label: &Option<String>,
    );
    fn convert_do_while(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        loop_body: Box<Stmt>,
        condition: Expression,
        annotation: Annotation,
    );
    fn convert_switch(
        &mut self,
        expr: Box<Expression>,
        body: Box<Stmt>,
        cases: Vec<SwitchCase>,
        default_label: Option<String>,
        annotation: Annotation,
        instructions: &mut Vec<TackyInstruction>,
    );
    fn convert_while(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        body: Box<Stmt>,
        condition: Expression,
        annotation: Annotation,
    );
    fn convert_for(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        init: ForInit,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Box<Stmt>,
        annotation: Annotation,
    );
    fn convert_if(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        condition: Expression,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    );
}

impl StatementConverter for TackyParser {
    fn convert_stmt(&mut self, stmt: Stmt, instructions: &mut Vec<tacky::TackyInstruction>) {
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
                let condition_val = self.convert_expr(condition, instructions);
                if let Some(else_br) = else_branch {
                    let else_label = self.label_generator.generate_label("else_label");
                    let end_label = self.label_generator.generate_label("end_label");

                    InstructionBuilder::new(instructions)
                        .jump_if_zero(condition_val, else_label.clone());
                    self.convert_stmt(*then_branch, instructions);
                    InstructionBuilder::new(instructions).jump(end_label.clone());
                    InstructionBuilder::new(instructions).label(else_label);
                    self.convert_stmt(*else_br, instructions);
                    InstructionBuilder::new(instructions).label(end_label);
                } else {
                    let end_label = self.label_generator.generate_label("end_label");
                    InstructionBuilder::new(instructions)
                        .jump_if_zero(condition_val, end_label.clone());
                    self.convert_stmt(*then_branch, instructions);
                    InstructionBuilder::new(instructions).label(end_label);
                }
            }
            Stmt::Goto(label) => {
                InstructionBuilder::new(instructions).jump(label);
            }
            Stmt::Label(name) => {
                InstructionBuilder::new(instructions).label(name);
            }
            Stmt::Compound { block } => {
                self.convert_block(block, instructions);
            }
            Stmt::Break(annotation) => match annotation {
                Annotation::LoopLabel(label) => {
                    InstructionBuilder::new(instructions).jump(format!("{}_end", label));
                }
                Annotation::SwitchLabel(label) => {
                    InstructionBuilder::new(instructions).jump(format!("{}_end", label));
                }
                _ => {}
            },
            Stmt::Continue(annotation) => {
                if let Annotation::LoopLabel(label) = annotation {
                    InstructionBuilder::new(instructions).jump(format!("{}_continue", label));
                }
            }
            Stmt::While {
                condition,
                body,
                annotation,
            } => {
                let loop_label = match annotation {
                    Annotation::LoopLabel(label) => label,
                    _ => self.label_generator.generate_label("while"),
                };

                let continue_label = format!("{}_continue", loop_label);
                let end_label = format!("{}_end", loop_label);

                InstructionBuilder::new(instructions).label(continue_label.clone());
                let condition_val = self.convert_expr(condition, instructions);
                InstructionBuilder::new(instructions)
                    .jump_if_zero(condition_val, end_label.clone());

                self.convert_stmt(*body, instructions);
                InstructionBuilder::new(instructions)
                    .jump(continue_label)
                    .label(end_label);
            }
            Stmt::DoWhile {
                body,
                condition,
                annotation,
            } => {
                let loop_label = match annotation {
                    Annotation::LoopLabel(label) => label,
                    _ => self.label_generator.generate_label("do"),
                };

                let start_label = format!("{}_start", loop_label);
                let continue_label = format!("{}_continue", loop_label);
                let end_label = format!("{}_end", loop_label);

                InstructionBuilder::new(instructions).label(start_label.clone());
                self.convert_stmt(*body, instructions);
                InstructionBuilder::new(instructions).label(continue_label);
                let condition_val = self.convert_expr(condition, instructions);
                InstructionBuilder::new(instructions)
                    .jump_if_not_zero(condition_val, start_label)
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
                    InstructionBuilder::new(instructions)
                        .jump_if_zero(condition_val, end_label.clone());
                }

                self.convert_stmt(*body, instructions);
                InstructionBuilder::new(instructions).label(continue_label);

                if let Some(inc) = increment {
                    self.convert_expr(inc, instructions);
                }

                InstructionBuilder::new(instructions)
                    .jump(start_label)
                    .label(end_label);
            }
            Stmt::Switch {
                expr,
                body,
                annotation,
                cases,
                default_label,
            } => {
                let switch_label = match annotation {
                    Annotation::SwitchLabel(label) => label,
                    _ => panic!("Switch statement must have a label"),
                };
                let end_label = self.label_generator.generate_label("switch_end");
                let prev_label = self.current_switch_label.replace(end_label.clone());
                let switch_val = self.convert_expr(expr, instructions);

                for case in cases.iter() {
                    let tmp = Val::Var(self.make_temporary());
                    InstructionBuilder::new(instructions).binary(
                        BinaryOp::Equal,
                        switch_val.clone(),
                        Constant(case.value),
                        tmp.clone(),
                    );
                    InstructionBuilder::new(instructions).jump_if_not_zero(tmp, case.label.clone());
                }

                if let Some(default) = default_label.clone() {
                    InstructionBuilder::new(instructions).jump(default.clone());
                } else {
                    InstructionBuilder::new(instructions).jump(format!("{}_end", switch_label));
                }

                self.emit_switch_body(*body, instructions, cases.as_slice(), &default_label);

                InstructionBuilder::new(instructions).label(format!("{}_end", switch_label));
            }
            Stmt::Case { body, .. } | Stmt::Default { body } => {
                self.convert_stmt(*body, instructions);
            }
        }
    }

    fn convert_stmt_with_case_labels(
        &mut self,
        stmt: Stmt,
        instructions: &mut Vec<TackyInstruction>,
        cases: &[SwitchCase],
        default_label: &Option<String>,
    ) {
        match stmt {
            Stmt::Case { value, body } => {
                if let Expression::Constant(case_value) = value {
                    if let Some(case) = cases.iter().find(|c| c.value == case_value) {
                        InstructionBuilder::new(instructions).label(case.label.clone());
                    }
                }
                self.convert_stmt_with_case_labels(*body, instructions, cases, default_label);
            }
            Stmt::Default { body } => {
                if let Some(label) = default_label {
                    InstructionBuilder::new(instructions).label(label.clone());
                }
                self.convert_stmt_with_case_labels(*body, instructions, cases, default_label);
            }
            Stmt::Compound { block } => {
                for element in block.elements {
                    match element {
                        BlockElement::Stmt(s) => {
                            self.convert_stmt_with_case_labels(
                                s,
                                instructions,
                                cases,
                                default_label,
                            );
                        }
                        BlockElement::Declaration(d) => {
                            self.convert_declaration(d, instructions);
                        }
                    }
                }
            }
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
                    self.convert_stmt_with_case_labels(
                        *then_branch,
                        instructions,
                        cases,
                        default_label,
                    );
                    InstructionBuilder::new(instructions).jump(end_label.clone());
                    InstructionBuilder::new(instructions).label(else_label);
                    self.convert_stmt_with_case_labels(
                        *else_branch,
                        instructions,
                        cases,
                        default_label,
                    );
                    InstructionBuilder::new(instructions).label(end_label);
                } else {
                    let end_label = self.label_generator.generate_label("end_label");

                    InstructionBuilder::new(instructions).jump_if_zero(c, end_label.clone());
                    self.convert_stmt_with_case_labels(
                        *then_branch,
                        instructions,
                        cases,
                        default_label,
                    );
                    InstructionBuilder::new(instructions).label(end_label);
                }
            }
            Stmt::While {
                body,
                condition,
                annotation,
            } => {
                let loop_label = match annotation {
                    Annotation::LoopLabel(label) => label,
                    _ => self.label_generator.generate_label("while"),
                };

                let continue_label = format!("{}_continue", loop_label);
                let end_label = format!("{}_end", loop_label);

                InstructionBuilder::new(instructions).label(continue_label.clone());
                let c = self.convert_expr(condition, instructions);
                InstructionBuilder::new(instructions).jump_if_zero(c, end_label.clone());
                self.convert_stmt_with_case_labels(*body, instructions, cases, default_label);
                InstructionBuilder::new(instructions)
                    .jump(continue_label)
                    .label(end_label);
            }
            Stmt::DoWhile {
                body,
                condition,
                annotation,
            } => {
                let loop_label = match annotation {
                    Annotation::LoopLabel(label) => label,
                    _ => self.label_generator.generate_label("do"),
                };

                let start_label = format!("{}_start", loop_label);
                let continue_label = format!("{}_continue", loop_label);
                let end_label = format!("{}_end", loop_label);

                InstructionBuilder::new(instructions).label(start_label.clone());
                self.convert_stmt_with_case_labels(*body, instructions, cases, default_label);
                InstructionBuilder::new(instructions).label(continue_label);
                let v = self.convert_expr(condition, instructions);
                InstructionBuilder::new(instructions)
                    .jump_if_not_zero(v, start_label)
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
                    InstructionBuilder::new(instructions)
                        .jump_if_zero(condition_val, end_label.clone());
                }

                self.convert_stmt_with_case_labels(*body, instructions, cases, default_label);
                InstructionBuilder::new(instructions).label(continue_label);

                if let Some(inc) = increment {
                    self.convert_expr(inc, instructions);
                }

                InstructionBuilder::new(instructions)
                    .jump(start_label)
                    .label(end_label);
            }
            // For nested switches, use regular convert_stmt
            Stmt::Switch { .. } => {
                self.convert_stmt(stmt, instructions);
            }
            // All other statements use regular conversion
            _ => {
                self.convert_stmt(stmt, instructions);
            }
        }
    }

    fn convert_do_while(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        loop_body: Box<Stmt>,
        condition: Expression,
        annotation: Annotation,
    ) {
        let loop_label = match annotation {
            Annotation::LoopLabel(label) => label,
            _ => self.label_generator.generate_label("do"),
        };

        let start_label = format!("{}_start", loop_label);
        let continue_label = format!("{}_continue", loop_label);
        let end_label = format!("{}_end", loop_label);

        InstructionBuilder::new(instructions).label(start_label.clone());
        self.convert_stmt(*loop_body, instructions);
        InstructionBuilder::new(instructions).label(continue_label);
        let v = self.convert_expr(condition, instructions);
        InstructionBuilder::new(instructions)
            .jump_if_not_zero(v, start_label)
            .label(end_label);
    }
    fn convert_switch(
        &mut self,
        expr: Box<Expression>,
        body: Box<Stmt>,
        cases: Vec<SwitchCase>,
        default_label: Option<String>,
        annotation: Annotation,
        instructions: &mut Vec<TackyInstruction>,
    ) {
        let switch_label = match annotation {
            Annotation::SwitchLabel(label) => label,
            _ => panic!("Switch must have SwitchLabel annotation"),
        };

        let expr_val = self.convert_expr(*expr, instructions);
        let end_label = format!("{}_end", switch_label);

        // Generate jump table
        for case in &cases {
            let cond_tmp = Val::Var(self.make_temporary());
            InstructionBuilder::new(instructions)
                .binary(
                    BinaryOp::Equal,
                    expr_val.clone(),
                    Val::Constant(case.value),
                    cond_tmp.clone(),
                )
                .jump_if_not_zero(cond_tmp, case.label.clone());
        }

        // Jump to default or end
        if let Some(default) = &default_label {
            InstructionBuilder::new(instructions).jump(default.clone());
        } else {
            InstructionBuilder::new(instructions).jump(end_label.clone());
        }

        // THIS IS THE KEY LINE - call the helper instead of convert_stmt
        self.convert_stmt_with_case_labels(*body, instructions, &cases, &default_label);

        InstructionBuilder::new(instructions).label(end_label);
    }

    fn convert_while(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        body: Box<Stmt>,
        condition: Expression,
        annotation: Annotation,
    ) {
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
            .jump(continue_label)
            .label(end_label);
    }

    fn convert_for(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        init: ForInit,
        condition: Option<Expression>,
        increment: Option<Expression>,
        body: Box<Stmt>,
        annotation: Annotation,
    ) {
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

        InstructionBuilder::new(instructions).label(continue_label);

        if let Some(inc) = increment {
            self.convert_expr(inc, instructions);
        }

        InstructionBuilder::new(instructions)
            .jump(start_label)
            .label(end_label);
    }

    fn convert_if(
        &mut self,
        instructions: &mut Vec<TackyInstruction>,
        condition: Expression,
        then_branch: Box<Stmt>,
        else_branch: Option<Box<Stmt>>,
    ) {
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
}
