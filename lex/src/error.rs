use thiserror::Error;

#[derive(Error,Debug)]
pub enum LexerError{
    #[error("Invalid token '{2}' at line {0}, column {1}")]
    InvalidToken(usize,usize,char),
    #[error("Unterminated comment starting at line {0}, column {1}")]
    UnterminatedComment(usize,usize),
    #[error("Unexpected end of file")]
    UnexpectedEOF,
    #[error("Invalid number at line {0} column {1}")]
    InvalidNumberFormat(char,char),
    #[error("Unknown error")]
    UnknownToken
    
}