use thiserror::Error;

#[derive(Error,Debug)]
pub enum ParserError{
    #[error("Unexpected token '{2}' at line {0}, column {1}")]
    UnexpectedToken(usize,usize,String),
    #[error("Expected token '{2}' but found '{3}' at line {0}, column {1}")]
    ExpectedToken(usize,usize,String,String),
    #[error("Unexpected end of file")]
    UnexpectedEOF,
    #[error("Unknown parsing error")]
    UnknownError
}