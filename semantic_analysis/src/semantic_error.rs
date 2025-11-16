use thiserror::Error;

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Multiple declaration of variable '{var_name}'")]
    MultipleDeclaration { var_name: String },
    #[error("Undeclared variable '{var_name}'")]
    UndeclaredVariable { var_name: String },
    #[error("Invalid left value in assignment: '{invalid}'")]
    InvalidLeftValue { invalid: String },
}
