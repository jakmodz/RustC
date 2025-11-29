use thiserror::Error;

#[derive(Debug, Error)]
pub enum SemanticError {
    #[error("Multiple declaration of variable '{var_name}'")]
    MultipleDeclaration { var_name: String },
    #[error("Undeclared variable '{var_name}'")]
    UndeclaredVariable { var_name: String },
    #[error("Invalid left value in assignment: '{invalid}'")]
    InvalidLeftValue { invalid: String },
    /*
     Labels Errors
    */
    #[error("Undeclared label '{label}'")]
    UndeclaredLabel { label: String },
    #[error("Duplicate label declaration '{label}'")]
    DuplicateLabel { label: String },
    #[error("No statement in label '{label}'")]
    EmptyLabel { label: String },
    #[error("Declaration in label '{label}'")]
    DeclarationInLabel { label: String },
}
