use ast::VarType;

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum SymbolEntry {
    Variable {
        var_type: VarType,
    },
    Function {
        var_type: VarType,
        already_defined: bool,
    },
}

impl SymbolEntry {
    pub(crate) fn get_var_type(&self) -> VarType {
        match self {
            SymbolEntry::Variable { var_type } => var_type.clone(),
            SymbolEntry::Function { var_type, .. } => var_type.clone(),
        }
    }
}
