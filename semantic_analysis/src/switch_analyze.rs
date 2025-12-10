use crate::{SemanticAnalyzer, SemanticError};
use ast::{
    Expression, Stmt,
    ast::{Annotation, BlockElement, Program, SwitchCase},
};

#[derive(Clone, Copy, PartialEq)]
pub(crate) enum BreakTarget {
    Loop,
    Switch,
}

pub(crate) trait SwitchAnalyzer {
    fn analyze_control_flow(&mut self, ast: &mut Program) -> Result<(), SemanticError>;
    fn collect_cases_recursive(
        &mut self,
        stmt: &Stmt,
        switch_label: &str,
        cases: &mut Vec<SwitchCase>,
        default_label: &mut Option<String>,
        seen_values: &mut std::collections::HashSet<i64>,
    ) -> Result<(), SemanticError>;
    fn collect_cases(
        &mut self,
        stmt: &Stmt,
        switch_label: &str,
    ) -> Result<(Vec<SwitchCase>, Option<String>), SemanticError>;
    fn process_control_flow(
        &mut self,
        stmt: &mut Stmt,
        current_switch: Option<String>,
        current_loop: Option<String>,
        break_target: Option<BreakTarget>,
    ) -> Result<(), SemanticError>;
    fn make_switch_label(&mut self) -> String;
    fn make_loop_label(&mut self) -> String;
}

impl SwitchAnalyzer for SemanticAnalyzer {
    fn analyze_control_flow(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        for func in ast.declarations.iter_mut() {
            match func {
                ast::Declaration::FuncDecl { decl }=> {
                    if let Some(body) = &mut decl.body {
                        for element in body.elements.iter_mut()  {
                            if let BlockElement::Stmt(stmt) = element {
                                self.process_control_flow(stmt, None, None, None)?;
                            }
                        }
                    }
                }
                _=>{}
            }
        }
        Ok(())
    }

    fn collect_cases_recursive(
        &mut self,
        stmt: &Stmt,
        switch_label: &str,
        cases: &mut Vec<SwitchCase>,
        default_label: &mut Option<String>,
        seen_values: &mut std::collections::HashSet<i64>,
    ) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Case { value, body } => {
                if let Expression::Constant(val) = value {
                    if !seen_values.insert(*val) {
                        return Err(SemanticError::DuplicateCase(*val));
                    }

                    let case_label = format!("{}_case_{}", switch_label, val);
                    cases.push(SwitchCase {
                        value: *val,
                        label: case_label,
                    });
                }

                self.collect_cases_recursive(body, switch_label, cases, default_label, seen_values)
            }

            Stmt::Default { body } => {
                if default_label.is_some() {
                    return Err(SemanticError::MultipleDefaults);
                }

                *default_label = Some(format!("{}_default", switch_label));
                self.collect_cases_recursive(body, switch_label, cases, default_label, seen_values)
            }

            Stmt::Compound { block } => {
                for element in &block.elements {
                    if let BlockElement::Stmt(s) = element {
                        self.collect_cases_recursive(
                            s,
                            switch_label,
                            cases,
                            default_label,
                            seen_values,
                        )?;
                    }
                }
                Ok(())
            }

            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.collect_cases_recursive(
                    then_branch,
                    switch_label,
                    cases,
                    default_label,
                    seen_values,
                )?;
                if let Some(else_br) = else_branch {
                    self.collect_cases_recursive(
                        else_br,
                        switch_label,
                        cases,
                        default_label,
                        seen_values,
                    )?;
                }
                Ok(())
            }

            Stmt::Switch { .. } => Ok(()),

            Stmt::While { body, .. } | Stmt::DoWhile { body, .. } | Stmt::For { body, .. } => {
                self.collect_cases_recursive(body, switch_label, cases, default_label, seen_values)
            }

            _ => Ok(()),
        }
    }

    fn collect_cases(
        &mut self,
        stmt: &Stmt,
        switch_label: &str,
    ) -> Result<(Vec<SwitchCase>, Option<String>), SemanticError> {
        let mut cases = Vec::new();
        let mut default_label = None;
        let mut seen_values = std::collections::HashSet::new();

        self.collect_cases_recursive(
            stmt,
            switch_label,
            &mut cases,
            &mut default_label,
            &mut seen_values,
        )?;

        Ok((cases, default_label))
    }

    fn process_control_flow(
        &mut self,
        stmt: &mut Stmt,
        current_switch: Option<String>,
        current_loop: Option<String>,
        break_target: Option<BreakTarget>,
    ) -> Result<(), SemanticError> {
        match stmt {
            Stmt::Switch {
                body,
                annotation,
                cases,
                default_label,
                ..
            } => {
                let switch_label = self.make_switch_label();
                *annotation = Annotation::SwitchLabel(switch_label.clone());

                let (collected_cases, collected_default) =
                    self.collect_cases(body, &switch_label)?;

                *cases = collected_cases;
                *default_label = collected_default;
                self.process_control_flow(
                    body,
                    Some(switch_label),
                    current_loop,
                    Some(BreakTarget::Switch),
                )
            }

            Stmt::Case { value, body } => {
                if !matches!(value, Expression::Constant(_)) {
                    return Err(SemanticError::NonConstantCase);
                }

                if current_switch.is_none() {
                    return Err(SemanticError::CaseOutsideSwitch);
                }

                self.process_control_flow(body, current_switch, current_loop, break_target)
            }

            Stmt::Default { body } => {
                if current_switch.is_none() {
                    return Err(SemanticError::DefaultOutsideSwitch);
                }

                self.process_control_flow(body, current_switch, current_loop, break_target)
            }

            Stmt::Continue(label) => {
                if let Some(loop_label) = current_loop {
                    *label = Annotation::LoopLabel(loop_label);
                    Ok(())
                } else {
                    Err(SemanticError::JumpStmtNotInLoop {
                        stmt: "Continue".to_string(),
                    })
                }
            }

            Stmt::Break(label) => match break_target {
                Some(BreakTarget::Switch) => {
                    if let Some(switch_label) = current_switch {
                        *label = Annotation::SwitchLabel(switch_label);
                        Ok(())
                    } else {
                        Err(SemanticError::JumpStmtNotInLoop {
                            stmt: "Break".to_string(),
                        })
                    }
                }
                Some(BreakTarget::Loop) => {
                    if let Some(loop_label) = current_loop {
                        *label = Annotation::LoopLabel(loop_label);
                        Ok(())
                    } else {
                        Err(SemanticError::JumpStmtNotInLoop {
                            stmt: "Break".to_string(),
                        })
                    }
                }
                None => Err(SemanticError::JumpStmtNotInLoop {
                    stmt: "Break".to_string(),
                }),
            },

            Stmt::While {
                body, annotation, ..
            } => {
                let loop_label = self.make_loop_label();
                *annotation = Annotation::LoopLabel(loop_label.clone());
                self.process_control_flow(
                    body,
                    current_switch,
                    Some(loop_label),
                    Some(BreakTarget::Loop),
                )
            }

            Stmt::DoWhile {
                body, annotation, ..
            } => {
                let loop_label = self.make_loop_label();
                *annotation = Annotation::LoopLabel(loop_label.clone());
                self.process_control_flow(
                    body,
                    current_switch,
                    Some(loop_label),
                    Some(BreakTarget::Loop),
                )
            }

            Stmt::For {
                body, annotation, ..
            } => {
                let loop_label = self.make_loop_label();
                *annotation = Annotation::LoopLabel(loop_label.clone());
                self.process_control_flow(
                    body,
                    current_switch,
                    Some(loop_label),
                    Some(BreakTarget::Loop),
                )
            }

            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                self.process_control_flow(
                    then_branch,
                    current_switch.clone(),
                    current_loop.clone(),
                    break_target,
                )?;
                if let Some(else_br) = else_branch {
                    self.process_control_flow(else_br, current_switch, current_loop, break_target)?;
                }
                Ok(())
            }

            Stmt::Compound { block } => {
                for element in block.elements.iter_mut() {
                    if let BlockElement::Stmt(s) = element {
                        self.process_control_flow(
                            s,
                            current_switch.clone(),
                            current_loop.clone(),
                            break_target,
                        )?;
                    }
                }
                Ok(())
            }

            _ => Ok(()),
        }
    }

    fn make_switch_label(&mut self) -> String {
        let label = format!("_switch_{}", self.switch_count);
        self.switch_count += 1;
        label
    }

    fn make_loop_label(&mut self) -> String {
        let label = format!("_loop_{}", self.loop_count);
        self.loop_count += 1;
        label
    }
}
