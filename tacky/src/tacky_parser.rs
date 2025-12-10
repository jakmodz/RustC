use std::collections::HashMap;

use crate::conver_stmt::StatementConverter;
use crate::convert_expr::ExpressionConverter;
use crate::instruction_builder::InstructionBuilder;
use crate::label_generator::LabelGenerator;
use crate::tacky::Val::Constant;
use crate::tacky::{self, StaticVar, TackyFunction, TopLevelConstruct};
use crate::tacky::{BinaryOp, Val};
use crate::tacky_instruction::TackyInstruction;
use ast::ast::{Annotation, Block, BlockElement, ForInit, Program};
use ast::decl::StorageClass;
use ast::{Declaration, Expression};
use semantic_analysis::symbol_entry::{IdentifierAttr, InitialValue};
use semantic_analysis::{SemanticAnalyzer, SymbolEntry};

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
    fn convert_symbols(
        &mut self,
        symbols: &HashMap<String, SymbolEntry>,
    ) -> Vec<TopLevelConstruct> {
        let mut defs = Vec::new();
        for (name, entry) in symbols {
            match entry.get_attr() {
                IdentifierAttr::StaticAttr { init_val, global } => match init_val {
                    InitialValue::Initial(val) => {
                        let var = StaticVar {
                            name: name.clone(),
                            global: global.clone(),
                            init: val.clone(),
                        };
                        defs.push(TopLevelConstruct::StaticVar(var))
                    }
                    InitialValue::Tentative => {
                        let var = StaticVar {
                            name: name.clone(),
                            global: global.clone(),
                            init: 0,
                        };
                        defs.push(TopLevelConstruct::StaticVar(var))
                    }
                    InitialValue::NoInit => {}
                },
                _ => {}
            }
        }
        defs
    }
    pub fn emit_tacky(
        &mut self,
        mut ast: Program,
        semantic_analyzer: &SemanticAnalyzer,
    ) -> tacky::Program {
        let mut constructs = Vec::new();
        constructs.append(&mut self.convert_symbols(&semantic_analyzer.symbol_table));

        for declaration in ast.declarations.iter_mut() {
            match declaration {
                Declaration::DefineVar(..) => {}
                Declaration::FuncDecl { decl } => {
                    let mut body = Vec::new();
                    let mut params = Vec::new();
                    if let Some(func_body) = decl.body.take() {
                        self.convert_block(func_body, &mut body);
                        for param in &decl.params {
                            params.push(Val::Var(param.clone()));
                        }
                        InstructionBuilder::new(&mut body).return_val(Constant(0));
                        constructs.push(TopLevelConstruct::Function(TackyFunction {
                            name: decl.name.clone(),
                            params,
                            body: body,
                            global: match semantic_analyzer.symbol_table.get(&decl.name) {
                                Some(entry) => entry.get_attr().is_global(),
                                None => false,
                            },
                        }));
                    }
                }
            }
        }
        tacky::Program { constructs }
    }

    fn convert_block_element(&mut self, element: BlockElement, body: &mut Vec<TackyInstruction>) {
        match element {
            BlockElement::Stmt(stmt) => {
                self.convert_stmt(stmt, body);
            }
            BlockElement::Declaration(decl) => {
                self.convert_declaration(decl, body);
            }
        }
    }

    pub(crate) fn convert_block(&mut self, block: Block, body: &mut Vec<TackyInstruction>) {
        for element in block.elements {
            self.convert_block_element(element, body);
        }
    }

    pub(crate) fn convert_declaration(
        &mut self,
        decl: Declaration,
        body: &mut Vec<TackyInstruction>,
    ) {
        match decl {
            Declaration::DefineVar(var) => {
                if var.storage_class == Some(StorageClass::Static)
                    || var.storage_class == Some(StorageClass::Extern)
                {
                    return;
                }
                if let Some(init_expr) = var.init {
                    let val = self.convert_expr(init_expr, body);
                    InstructionBuilder::new(body).copy(val, Val::Var(var.name));
                }
            }
            _ => {}
        }
    }

    pub(crate) fn convert_for_init(
        &mut self,
        init: ForInit,
        instructions: &mut Vec<TackyInstruction>,
    ) {
        match init {
            ForInit::Declaration(decl) => {
                self.convert_declaration(Declaration::DefineVar(decl), instructions);
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
                InstructionBuilder::new(body).jump(format!("{}_continue", label));
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
