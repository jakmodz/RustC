use crate::label_generator::LabelGenerator;
use crate::tacky;
use crate::tacky::Val::Constant;
use crate::tacky::{BinaryOp, TackyFunction, TackyInstruction, Val};

use crate::conver_stmt::StatementConverter;
use crate::convert_expr::ExpressionConverter;
use crate::instruction_builder::InstructionBuilder;
use ast::ast::{Annotation, BlockElement, Declaration, Expression, ForInit, Program};

pub struct TackyParser {
    pub var_counter: usize,
    pub label_generator: LabelGenerator,
}

impl TackyParser {
    pub fn new(var_counter: usize) -> Self {
        Self {
            var_counter,
            label_generator: LabelGenerator::new(),
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

    fn convert_declaration(&mut self, decl: Declaration, body: &mut Vec<tacky::TackyInstruction>) {
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
}
