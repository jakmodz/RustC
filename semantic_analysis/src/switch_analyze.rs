use ast::ast::{Annotation, BlockElement, Expression, Program, Stmt, SwitchCase};
use crate::{SemanticAnalyzer, SemanticError};

pub trait SwitchAnalyzer {
    fn analyze_switch_statements(&mut self, ast: &mut Program) -> Result<(), SemanticError>;
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
    fn process_switch(
        &mut self,
        stmt: &mut Stmt,
        current_switch: Option<String>,
    ) -> Result<(), SemanticError>;
    fn make_switch_label(&mut self) -> String;
}


impl SwitchAnalyzer for SemanticAnalyzer {
    fn analyze_switch_statements(&mut self, ast: &mut Program) -> Result<(), SemanticError> {
        for element in ast.function.body.elements.iter_mut() {
            if let BlockElement::Stmt(stmt) = element {
                self.process_switch(stmt, None)?;
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
                            s, switch_label, cases, default_label, seen_values
                        )?;
                    }
                }
                Ok(())
            }

            Stmt::If { then_branch, else_branch, .. } => {
                self.collect_cases_recursive(
                    then_branch, switch_label, cases, default_label, seen_values
                )?;
                if let Some(else_br) = else_branch {
                    self.collect_cases_recursive(
                        else_br, switch_label, cases, default_label, seen_values
                    )?;
                }
                Ok(())
            }

            // Don't recurse into nested switches or loops
            Stmt::Switch { .. } | Stmt::While { .. } |
            Stmt::DoWhile { .. } | Stmt::For { .. } => Ok(()),

            _ => Ok(()),
        }
    }
    fn collect_cases(
        &mut self,
        stmt: &Stmt,
        switch_label: &str,
    ) -> Result<(Vec<SwitchCase>, Option<String>), SemanticError>{
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
    fn process_switch(
        &mut self,
        stmt: &mut Stmt,
        current_switch: Option<String>,
    ) -> Result<(), SemanticError>{
        match stmt {
            Stmt::Switch { body, annotation, cases, default_label, .. } => {
                let switch_label = self.make_switch_label();
                *annotation = Annotation::SwitchLabel(switch_label.clone());


                let (collected_cases, collected_default) =
                    self.collect_cases(body, &switch_label)?;

                *cases = collected_cases;
                *default_label = collected_default;

                self.process_switch(body, Some(switch_label))
            }

            Stmt::Case { value, body } => {

                if !matches!(value, Expression::Constant(_)) {
                    return Err(SemanticError::NonConstantCase);
                }

                if current_switch.is_none() {
                    return Err(SemanticError::CaseOutsideSwitch);
                }

                self.process_switch(body, current_switch)
            }

            Stmt::Default { body } => {
                if current_switch.is_none() {
                    return Err(SemanticError::DefaultOutsideSwitch);
                }

                self.process_switch(body, current_switch)
            }

            Stmt::Break(label) => {
                // Break can exit either a loop or a switch
                if let Some(switch_label) = &current_switch {
                    *label = Annotation::SwitchLabel(switch_label.clone());
                    Ok(())
                } else {

                    Ok(())
                }
            }

            Stmt::While { body, .. } | Stmt::DoWhile { body, .. } => {
                self.process_switch(body, None)
            }

            Stmt::For { body, .. } => {
                self.process_switch(body, None)
            }

            Stmt::If { then_branch, else_branch, .. } => {
                self.process_switch(then_branch, current_switch.clone())?;
                if let Some(else_br) = else_branch {
                    self.process_switch(else_br, current_switch)?;
                }
                Ok(())
            }

            Stmt::Compound { block } => {
                for element in block.elements.iter_mut() {
                    if let BlockElement::Stmt(s) = element {
                        self.process_switch(s, current_switch.clone())?;
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
}