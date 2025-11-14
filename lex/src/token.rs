use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref OPERATOR_PRECEDENCE: HashMap<TokenType, usize> = {
        let mut map = HashMap::new();

        map.insert(TokenType::Percent,50);
        map.insert(TokenType::Star,50);
        map.insert(TokenType::Slash,50);
        map.insert(TokenType::Plus,45);
        map.insert(TokenType::Hypen,45);
        map.insert(TokenType::LeftShift, 40);
        map.insert(TokenType::RightShift, 40);
        map.insert(TokenType::Less, 35);
        map.insert(TokenType::LessEqual, 35);
        map.insert(TokenType::Greater, 35);
        map.insert(TokenType::GreaterEqual, 35);
        map.insert(TokenType::EqualEqual, 30);
        map.insert(TokenType::ExclamationEqual, 30);
        map.insert(TokenType::Ampersand, 25);
        map.insert(TokenType::Caret, 20);
        map.insert(TokenType::Pipe, 15);
        map.insert(TokenType::AmpersandAmpersand, 10);
        map.insert(TokenType::PipePipe, 5);
        map

    };
}
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
#[derive(Hash)]
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
    Star,
    Plus,
    Slash,
    Percent,
    Pipe,
    Ampersand,
    Caret,
    LeftShift,
    RightShift,
    Exclamation,
    AmpersandAmpersand,
    PipePipe,
    EqualEqual,
    ExclamationEqual,
    Less,
    Greater,
    LessEqual,
    GreaterEqual,
    Equal,
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
    //Logical
    Exclamation(Span),
    AmpersandAmpersand(Span),
    PipePipe(Span),
    EqualEqual(Span),
    ExclamationEqual(Span),
    Less(Span),
    Greater(Span),
    LessEqual(Span),
    GreaterEqual(Span),

    //Binary tokens
    Star(Span),
    Plus(Span),
    Slash(Span),
    Percent(Span),
    Pipe      (Span),
    Ampersand (Span),
    Caret     (Span),
    LeftShift (Span),
    RightShift(Span),


    //Keywords
    Return(Span),
    //types
    Int(Span),Void(Span),
    Equal(Span),

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
            Token::Plus(_) => TokenType::Plus,
            Token::Star(_) => TokenType::Star,
            Token::Percent(_) => TokenType::Percent,
            Token::Slash(_) => TokenType::Slash,
            Token::Pipe      (_) => TokenType::Pipe,
            Token::Ampersand (_) => TokenType::Ampersand,
            Token::Caret     (_) => TokenType::Caret,
            Token::LeftShift (_) => TokenType::LeftShift,
            Token::RightShift(_) => TokenType::RightShift,
            Token::Exclamation(_)=>TokenType::Exclamation,
            Token::AmpersandAmpersand(_)=>TokenType::AmpersandAmpersand,
            Token::PipePipe(_)=>TokenType::PipePipe,
            Token::EqualEqual(_)=>TokenType::EqualEqual,
            Token::ExclamationEqual(_)=>TokenType::ExclamationEqual,
            Token::Less(_)=>TokenType::Less,
            Token::Greater(_)=>TokenType::Greater,
            Token::LessEqual(_)=>TokenType::LessEqual,
            Token::GreaterEqual(_)=>TokenType::GreaterEqual,
            Token::Equal(_)=>TokenType::Equal,
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
            Token::Plus(_) => "+".to_string(),
            Token::Star(_) => "*".to_string(),
            Token::Percent(_) => "%".to_string(),
            Token::Slash(_) => "/".to_string(),
            Token::Pipe      (_) => "|".to_string(),
            Token::Caret     (_) => "^".to_string(),
            Token::LeftShift (_) =>"<<".to_string(),
            Token::RightShift(_) => ">>".to_string(),
            Token::Ampersand(_) => "&".to_string(),
            Token::Exclamation(_) => "!".to_string(),
            Token::AmpersandAmpersand(_)=>"&&".to_string(),
            Token::PipePipe(_)=>"||".to_string(),
            Token::EqualEqual(_)=>"==".to_string(),
            Token::ExclamationEqual(_)=>"!=".to_string(),
            Token::Less(_)=>"<".to_string(),
            Token::Greater(_)=>">".to_string(),
            Token::LessEqual(_)=>"<=".to_string(),
            Token::GreaterEqual(_)=>">=".to_string(),
            Token::Equal(_)=>"=".to_string(),

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
            Token::Star(span) |
            Token::Plus(span) |
            Token::Slash(span) |
            Token::Percent(span) |
            Token::Pipe(span) |
            Token::Ampersand (span) |
            Token::Caret(span) |
            Token::LeftShift(span) |
            Token::RightShift(span) |
            Token::AmpersandAmpersand(span)|
            Token::PipePipe(span)|
            Token::EqualEqual(span)|
            Token::ExclamationEqual(span)|
            Token::Less(span)|
            Token::Greater(span)|
            Token::LessEqual(span)|
            Token::GreaterEqual(span)|
            Token::Exclamation(span) |
            Token::Void(span)=> span,
            Token::Equal(span)=> span,
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
            "+" => Some(TokenType::Plus),
            "%" => Some(TokenType::Percent),
            "*" => Some(TokenType::Star),
            "/" => Some(TokenType::Slash),
            "^"=>Some(TokenType::Caret),
            "&"=>Some(TokenType::Ampersand),
            "|" =>Some(TokenType::Pipe),
            "<<"=>Some(TokenType::LeftShift),
            ">>"=>Some(TokenType::RightShift),
            "!"=> Some(TokenType::Exclamation),
            "&&"=> Some(TokenType::AmpersandAmpersand),
            "||"=> Some(TokenType::PipePipe),
            "=="=> Some(TokenType::EqualEqual),
            "!="=> Some(TokenType::ExclamationEqual),
            "<"=> Some(TokenType::Less),
            ">"=> Some(TokenType::Greater),
            "<="=> Some(TokenType::LessEqual),
            ">="=> Some(TokenType::GreaterEqual),
            "="=> Some(TokenType::Equal),
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
            TokenType::Percent => { Token::Percent(span) }
            TokenType::Slash => { Token::Slash(span) }
            TokenType::Star => { Token::Star(span) }
            TokenType::Plus => { Token::Plus(span) }
            TokenType::Pipe=>{Token::Pipe(span) }
            TokenType::Ampersand=>{Token::Ampersand(span)}
            TokenType::Caret=>{Token::Caret(span)}
            TokenType::LeftShift=>{Token::LeftShift(span)}
            TokenType::RightShift=>{Token::RightShift(span)},
            TokenType::Exclamation=>{Token::Exclamation(span)}
            TokenType::AmpersandAmpersand=>{Token::AmpersandAmpersand(span)}
            TokenType::PipePipe=>{Token::PipePipe(span)}
            TokenType::EqualEqual =>{Token::EqualEqual(span)}
            TokenType::ExclamationEqual =>{Token::ExclamationEqual(span)}
            TokenType::Less =>{Token::Less(span)}
            TokenType::Greater =>{Token::Greater(span)}
            TokenType::LessEqual =>{Token::LessEqual(span)}
            TokenType::GreaterEqual=>{Token::GreaterEqual(span)}
            TokenType::Equal =>{Token::Equal(span)}
        }
    }
    pub fn is_binary_op(&self) -> bool {
        match self.get_token_type() {
            TokenType::Hypen | TokenType::Plus |
            TokenType::Slash | TokenType::Star | TokenType::Percent |
            TokenType::Pipe | TokenType::Ampersand |TokenType::Caret |
            TokenType::LeftShift| TokenType::RightShift| TokenType::AmpersandAmpersand| TokenType::PipePipe |
            TokenType::EqualEqual | TokenType::ExclamationEqual| TokenType::Less|
            TokenType::Greater| TokenType::LessEqual| TokenType::GreaterEqual=> true,
            _ => false
        }
    }

    pub fn get_precedence(&self) -> usize {
        if let Some(u) = OPERATOR_PRECEDENCE.get(&self.get_token_type()) {
            return u.clone();
        }
        0
    }
}