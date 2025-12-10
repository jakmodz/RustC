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
    /*
    Syntax Errors
    */
    #[error("{stmt} statement not within a loop")]
    JumpStmtNotInLoop { stmt: String },
    #[error("Case value  is not a constant expression")]
    NonConstantCase,
    #[error("Case statement outside of switch")]
    CaseOutsideSwitch,
    #[error("Default statement outside of switch")]
    DefaultOutsideSwitch,
    #[error("Duplicate case value '{0}' in switch statement")]
    DuplicateCase(i64),
    #[error("Multiple default labels in switch statement")]
    MultipleDefaults,

    #[error("Undeclared function '{func_name}'")]
    UndeclaredFunction { func_name: String },

    #[error("Function '{func_name}' is defined more than once")]
    DuplicateFunctionDefinition { func_name: String },
    #[error("Incomplete function declaration for '{func_name}'")]
    IncompleteFunctionDeclaration { func_name: String },

    #[error("Variable '{var_name}' used as function name")]
    VariableAsFunctionName { var_name: String },
    #[error(
        "Function '{func_name}' called with wrong number of arguments. Excepted: {excepted} got: {got}"
    )]
    WrongNumberOfArguments {
        func_name: String,
        excepted: usize,
        got: usize,
    },
    #[error("Function '{func_name}' used as var name")]
    FunctionAsVariableName { func_name: String },
    #[error("Static function '{func_name}' declaration follows non-static")]
    StaticFunctionAfterNonStatic { func_name: String },
    #[error("Non const initializer for static variable '{var_name}'")]
    NonConstStaticInitializer { var_name: String },
    #[error("Conflicting variable linkage for '{var_name}'")]
    ConflictingVariableLinkage { var_name: String },
    #[error("Conflicting file scope variable definitions for '{var_name}'")]
    ConflictingFileVariableDefinitions { var_name: String },
    #[error("Initializer on local extern variable declaration '{var_name}'")]
    InitializerOnLocalExtern { var_name: String },
    #[error("Conflicting function linkage for '{func_name}'")]
    ConflictingFunctionLinkage { func_name: String },
    #[error("Invalid storage class for variable in for loop initializer '{var_name}'")]
    InvalidStorageClassInForInit { var_name: String },
}
