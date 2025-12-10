use ast::VarType;


#[derive(Debug, Clone, PartialEq)]
pub enum InitialValue{
    Tentative,
    Initial(i64),
    NoInit
}
#[derive(Debug, Clone, PartialEq)]
pub enum IdentifierAttr{
    FunAttr{defined:bool,global:bool},
    StaticAttr{init_val: InitialValue,global:bool},
    LocalAttr
}
impl IdentifierAttr {
    pub fn get_defined(&self)->bool{
        match self {
            IdentifierAttr::FunAttr { defined, .. } => defined.clone(),
            _=>{
                panic!("Expected Function Attribute")
            }
        }
    }
    pub fn is_global(&self)->bool{
        match self {
            IdentifierAttr::FunAttr { global, .. } => global.clone(),
            IdentifierAttr::StaticAttr { global, .. } => global.clone(),
            IdentifierAttr::LocalAttr => false,
        }
    }
    pub fn get_initial_value(&self) -> &InitialValue {
           match self {
               IdentifierAttr::StaticAttr { init_val, .. } => init_val,
               _ => {
                   panic!("Expected Static Attribute")
               }
           }
    }
}
#[derive(Debug, Clone, PartialEq)]
pub enum SymbolEntry {
    Variable {
        var_type: VarType,
        attr: IdentifierAttr
    },
    Function {
        var_type: VarType,
        attr: IdentifierAttr
    },
}

impl SymbolEntry {
    pub(crate) fn get_var_type(&self) -> VarType {
        match self {
            SymbolEntry::Variable { var_type ,..} => var_type.clone(),
            SymbolEntry::Function { var_type, .. } => var_type.clone(),
        }
    }
    pub fn get_attr(&self) -> IdentifierAttr {
        match self {
            SymbolEntry::Variable { attr ,..} => attr.clone(),
            SymbolEntry::Function { attr, .. } => attr.clone(),
        }
    }
}
