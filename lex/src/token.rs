#[derive(PartialEq,Debug,Clone)]
pub struct Span{
    pub column:usize,
    pub line:usize
}

impl Span{
    pub fn new(column:usize,line:usize)->Self{
        Self{
            column,
            line
        }
    }
}

#[derive(Debug,Clone,PartialEq,Eq)]
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
    Semicolon,
    Tilde,
    Hypen,
    HypenHypen,
}


#[derive(Clone,Debug)]
#[derive(PartialEq)]
pub enum Token{

    Identifier(String,Span),
    Constant(i64,Span),

    //
    OpenParen(Span),CloseParen(Span),
    OpenBrace(Span),CloseBrace(Span),
    Semicolon(Span),
    Tilde(Span),
    Hypen(Span),
    HypenHypen(Span),

    //Keywords
    Return(Span),
        //types
        Int(Span),Void(Span),

}

impl Token{
    pub fn get_token_type(&self) -> TokenType {
        match self {
            Token::Identifier(_, _) => TokenType::Identifier,
            Token::Constant(_, _) => TokenType::Constant,
            Token::OpenParen(_) => TokenType::OpenParen,
            Token::CloseParen(_) => TokenType::CloseParen,
            Token::OpenBrace(_) => TokenType::OpenBrace,
            Token::CloseBrace(_) => TokenType::CloseBrace,
            Token::Semicolon(_) => TokenType::Semicolon,
            Token::Return(_) => TokenType::Return,
            Token::Int(_) => TokenType::Int,
            Token::Void(_) => TokenType::Void,
            Token::Tilde(_) => TokenType::Tilde,
            Token::Hypen(_) => TokenType::Hypen,
            Token::HypenHypen(_) => TokenType::HypenHypen,
        }
    }

    pub fn to_string(&self) -> String {
        match self {
            Token::Identifier(str,_) => str.clone(),
            Token::Constant(i,_) => i.to_string(),
            Token::OpenParen(_) => "(".to_string(),
            Token::CloseParen(_) => ")".to_string(),
            Token::OpenBrace (_)=> "{".to_string(),
            Token::CloseBrace (_)=> "}".to_string(),
            Token::Semicolon(_) => ";".to_string(),
            Token::Return(_) => "return".to_string(),
            Token::Int (_)=> "int".to_string(),
            Token::Void (_)=> "void".to_string(),
            Token::Tilde(_) => "~".to_string(),
            Token::Hypen(_) => "-".to_string(),
            Token::HypenHypen(_) => "--".to_string(),
        }
    }
    pub fn span(&self) -> &Span {
        match self {
            Token::Identifier(_, span) |
            Token::Constant(_, span) |
            Token::OpenParen(span) |
            Token::CloseParen(span) |
            Token::OpenBrace(span) |
            Token::CloseBrace(span) |
            Token::Semicolon(span) |
            Token::Return(span) |
            Token::Int(span) |
            Token::Tilde(span) |
            Token::Hypen(span) |
            Token::HypenHypen(span) |
            Token::Void(span) => span,
            
        }
    }
    pub fn from_string(s: &str) -> Option<TokenType> {
        match s {
            "(" => Some(TokenType::OpenParen),
            ")" => Some(TokenType::CloseParen),
            "{" => Some(TokenType::OpenBrace),
            "}" => Some(TokenType::CloseBrace),
            ";" => Some(TokenType::Semicolon),
            "return" => Some(TokenType::Return),
            "int" => Some(TokenType::Int),
            "void" => Some(TokenType::Void),
            "~" => Some(TokenType::Tilde),
            "-" => Some(TokenType::Hypen),
            "--" => Some(TokenType::HypenHypen),
            _ => None,
        }
    }
    pub fn create_token(token_type: TokenType,s:&str,span: Span)->Self{
        match token_type {
            TokenType::Identifier => {Token::Identifier(s.to_string(), span)}
            TokenType::Constant => {Token::Constant(s.parse::<i64>().unwrap(),span) }
            TokenType::Int => {Token::Int(span)}
            TokenType::Void => {Token::Void(span)}
            TokenType::Return => {Token::Return(span)}
            TokenType::OpenParen => {Token::OpenParen(span)}
            TokenType::CloseParen => {Token::CloseParen(span)}
            TokenType::OpenBrace => {Token::OpenBrace(span)}
            TokenType::CloseBrace => {Token::CloseBrace(span)}
            TokenType::Semicolon => {Token::Semicolon(span)}
            TokenType::Tilde => {Token::Tilde(span)}
            TokenType::Hypen => {Token::Hypen(span)}
            TokenType::HypenHypen => {Token::HypenHypen(span)}
        }
    }

}