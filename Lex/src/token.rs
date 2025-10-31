#[derive(Debug,Clone)]
pub enum TokenType {
    Identifier,
    Constant,
    Int,
    Void,
    Return,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Semicolon
}


#[derive(Clone,Debug)]
#[derive(PartialEq)]
pub enum Token{

    Identifier(String),
    Constant(i64),

    //
    OpenParen,CloseParen,
    OpenBrace,CloseBrace,
    Semicolon,

    //Keywords
    Return,
        //types
        Int,Void,

}

impl Token{
    pub fn to_string(&self) -> String {
        match self {
            Token::Identifier(str) => str.clone(),
            Token::Constant(i) => i.to_string(),
            Token::OpenParen => "(".to_string(),
            Token::CloseParen => ")".to_string(),
            Token::OpenBrace => "{".to_string(),
            Token::CloseBrace => "}".to_string(),
            Token::Semicolon => ";".to_string(),
            Token::Return => "return".to_string(),
            Token::Int => "int".to_string(),
            Token::Void => "void".to_string(),
        }
    }
    pub fn from_string(s: &str) -> Option<Token> {
        match s {
            "(" => Some(Token::OpenParen),
            ")" => Some(Token::CloseParen),
            "{" => Some(Token::OpenBrace),
            "}" => Some(Token::CloseBrace),
            ";" => Some(Token::Semicolon),
            "return" => Some(Token::Return),
            "int" => Some(Token::Int),
            "void" => Some(Token::Void),
            _ => None,
        }
    }
    pub fn create_token(token_type: TokenType,s:&str)->Self{
        match token_type {
            TokenType::Identifier => {Token::Identifier(s.to_string()) }
            TokenType::Constant => {Token::Constant(s.parse::<i64>().unwrap())}
            TokenType::Int => {Token::Int}
            TokenType::Void => {Token::Void}
            TokenType::Return => {Token::Return}
            TokenType::OpenParen => {Token::OpenParen}
            TokenType::CloseParen => {Token::CloseParen}
            TokenType::OpenBrace => {Token::OpenBrace}
            TokenType::CloseBrace => {Token::CloseBrace}
            TokenType::Semicolon => {Token::Semicolon}
        }
    }
}