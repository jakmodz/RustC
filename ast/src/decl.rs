use crate::Expression;

#[derive(Debug, Clone, PartialEq)]
pub enum Declaration {
    DefineVar {
        var_name: String,
        initializer: Option<Expression>,
    },
}
