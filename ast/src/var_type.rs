
#[derive(Debug, Clone, PartialEq)]
pub enum VarType {
    Int,
    FunType{
        param_count: usize,
    },
}

impl VarType{
    pub fn get_param_count(&self)->usize{
        match self {
            VarType::Int => unreachable!(),
            VarType::FunType { param_count } => *param_count,
        }
    }
    
}