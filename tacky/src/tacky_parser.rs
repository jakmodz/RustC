use crate::label_generator::LabelGenerator;
use crate::tacky;
use crate::tacky::Val::Constant;
use crate::tacky::{BinaryOp, TackyFunction, TackyInstruction, Val};

use crate::conver_stmt::StatementConverter;
use crate::convert_expr::ExpressionConverter;
use crate::instruction_builder::InstructionBuilder;
use ast::ast::{
    Annotation, BlockElement, Declaration, Expression, ForInit, Program, Stmt, SwitchCase,
};

pub struct TackyParser {
    pub var_counter: usize,
    pub label_generator: LabelGenerator,
    pub(crate) current_switch_label: Option<String>,
}

impl TackyParser {
    pub fn new(var_counter: usize) -> Self {
        Self {
            var_counter,
            label_generator: LabelGenerator::new(),
            current_switch_label: None,
        }
    }

    pub(crate) fn make_temporary(&mut self) -> String {
        let s = format!("tmp.{}", self.var_counter);
        self.var_counter += 1;
        s
    }

    pub fn emit_tacky(&mut self, ast: Program) -> tacky::Program {
        let mut body = Vec::new();

        self.convert_block(ast.function.body, &mut body);
        InstructionBuilder::new(&mut body).return_val(Constant(0));

        tacky::Program {
            function: TackyFunction {
                name: ast.function.name,
                body,
            },
        }
    }

    fn convert_block_element(
        &mut self,
        element: BlockElement,
        body: &mut Vec<tacky::TackyInstruction>,
    ) {
        match element {
            BlockElement::Stmt(stmt) => {
                self.convert_stmt(stmt, body);
            }
            BlockElement::Declaration(decl) => {
                self.convert_declaration(decl, body);
            }
        }
    }

    pub(crate) fn convert_block(
        &mut self,
        block: ast::ast::Block,
        body: &mut Vec<tacky::TackyInstruction>,
    ) {
        for element in block.elements {
            self.convert_block_element(element, body);
        }
    }

    pub(crate) fn convert_declaration(
        &mut self,
        decl: Declaration,
        body: &mut Vec<tacky::TackyInstruction>,
    ) {
        match decl {
            Declaration::DefineVar {
                var_name,
                initializer,
            } => {
                if let Some(init_expr) = initializer {
                    let val = self.convert_expr(init_expr, body);
                    InstructionBuilder::new(body).copy(val, Val::Var(var_name));
                }
            }
        }
    }

    pub(crate) fn convert_for_init(
        &mut self,
        init: ForInit,
        instructions: &mut Vec<TackyInstruction>,
    ) {
        match init {
            ForInit::Declaration(decl) => {
                self.convert_declaration(decl, instructions);
            }
            ForInit::Expression(expr) => {
                if let Some(expr) = expr {
                    self.convert_expr(expr, instructions);
                }
            }
        }
    }

    pub(crate) fn convert_inc_dec(
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
            InstructionBuilder::new(instructions).binary(
                binary_op,
                var.clone(),
                Constant(1),
                var.clone(),
            );
            var
        } else {
            let tmp = Val::Var(self.make_temporary());

            InstructionBuilder::new(instructions)
                .copy(var.clone(), tmp.clone())
                .binary(binary_op, var.clone(), Constant(1), var.clone());

            tmp
        }
    }

    pub fn convert_annotation(&mut self, body: &mut Vec<TackyInstruction>, annotation: Annotation) {
        match annotation {
            Annotation::LoopLabel(label) => {
                InstructionBuilder::new(body).jump(label);
            }
            _ => {}
        }
    }

    fn get_var_name(&mut self, expr: &Expression) -> Option<String> {
        match expr {
            Expression::Grouping { expr } => self.get_var_name(expr),
            Expression::Var(name) => Some(name.clone()),
            _ => None,
        }
    }
    pub(crate) fn emit_switch_body(
        &mut self,
        stmt: Stmt,
        instructions: &mut Vec<TackyInstruction>,
        cases: &[SwitchCase],
        default_label: &Option<String>,
    ) {
        match stmt {
            Stmt::Case { value, body } => {
                if let Expression::Constant(val) = value {
                    if let Some(case) = cases.iter().find(|c| c.value == val) {
                        InstructionBuilder::new(instructions).label(case.label.clone());
                    }
                }
                self.emit_switch_body(*body, instructions, cases, default_label);
            }

            Stmt::Default { body } => {
                if let Some(label) = default_label {
                    InstructionBuilder::new(instructions).label(label.clone());
                }
                self.emit_switch_body(*body, instructions, cases, default_label);
            }

            Stmt::Break(Annotation::None) => {
                if let Some(end_label) = &self.current_switch_label {
                    InstructionBuilder::new(instructions).jump(end_label.clone());
                }
            }
            Stmt::Continue(_) | Stmt::Break(_) => {
                self.convert_stmt(stmt, instructions);
            }

            Stmt::Compound { block } => {
                for element in block.elements {
                    match element {
                        BlockElement::Stmt(s) => {
                            self.emit_switch_body(s, instructions, cases, default_label);
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
                if let Some(else_br) = else_branch {
                    let else_label = self.label_generator.generate_label("else_label");
                    let end_label = self.label_generator.generate_label("end_label");

                    InstructionBuilder::new(instructions).jump_if_zero(c, else_label.clone());
                    self.emit_switch_body(*then_branch, instructions, cases, default_label);
                    InstructionBuilder::new(instructions).jump(end_label.clone());
                    InstructionBuilder::new(instructions).label(else_label);
                    self.emit_switch_body(*else_br, instructions, cases, default_label);
                    InstructionBuilder::new(instructions).label(end_label);
                } else {
                    let end_label = self.label_generator.generate_label("end_label");
                    InstructionBuilder::new(instructions).jump_if_zero(c, end_label.clone());
                    self.emit_switch_body(*then_branch, instructions, cases, default_label);
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

                self.emit_switch_body(*body, instructions, cases, default_label);
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
                self.emit_switch_body(*body, instructions, cases, default_label);
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

                self.emit_switch_body(*body, instructions, cases, default_label);
                InstructionBuilder::new(instructions).label(continue_label);

                if let Some(inc) = increment {
                    self.convert_expr(inc, instructions);
                }

                InstructionBuilder::new(instructions)
                    .jump(start_label)
                    .label(end_label);
            }

            Stmt::Switch { .. } => {
                self.convert_stmt(stmt, instructions);
            }

            _ => {
                self.convert_stmt(stmt, instructions);
            }
        }
    }
}
