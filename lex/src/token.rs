use crate::Span;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
lazy_static! {
    static ref OPERATOR_PRECEDENCE: HashMap<TokenType, usize> = {
        let mut map = HashMap::new();

        map.insert(TokenType::Percent, 50);
        map.insert(TokenType::Star, 50);
        map.insert(TokenType::Slash, 50);
        map.insert(TokenType::Plus, 45);
        map.insert(TokenType::Hypen, 45);
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
        map.insert(TokenType::QuestionMark, 3);
        map.insert(TokenType::Equal, 1);
        map.insert(TokenType::PlusEqual, 1);
        map.insert(TokenType::HypenEqual, 1);
        map.insert(TokenType::StarEqual, 1);
        map.insert(TokenType::SlashEqual, 1);
        map.insert(TokenType::PercentEqual, 1);
        map.insert(TokenType::AmpersandEqual, 1);
        map.insert(TokenType::PipeEqual, 1);
        map.insert(TokenType::CaretEqual, 1);
        map.insert(TokenType::LeftShiftEqual, 1);
        map.insert(TokenType::RightShiftEqual, 1);
        map
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum TokenType {
    Identifier,
    Constant,
    Int,
    Void,
    Goto,
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
    PlusEqual,
    HypenEqual,
    StarEqual,
    SlashEqual,
    PercentEqual,
    AmpersandEqual,
    PipeEqual,
    CaretEqual,
    LeftShiftEqual,
    RightShiftEqual,
    PlusPlus,
    If,
    Else,
    QuestionMark,
    Colon,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Token {
    Identifier(String, Span),
    Constant(i64, Span),

    //
    OpenParen(Span),
    CloseParen(Span),
    OpenBrace(Span),
    CloseBrace(Span),
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
    Pipe(Span),
    Ampersand(Span),
    Caret(Span),
    LeftShift(Span),
    RightShift(Span),
    Equal(Span),
    PlusEqual(Span),
    HypenEqual(Span),
    StarEqual(Span),
    SlashEqual(Span),
    PercentEqual(Span),
    AmpersandEqual(Span),
    PipeEqual(Span),
    CaretEqual(Span),
    LeftShiftEqual(Span),
    RightShiftEqual(Span),
    PlusPlus(Span),

    //Keywords
    Return(Span),
    Goto(Span),
    //types
    Int(Span),
    Void(Span),
    If(Span),
    Else(Span),
    QuestionMark(Span),
    Colon(Span),
}

impl Token {
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
            Token::Pipe(_) => TokenType::Pipe,
            Token::Ampersand(_) => TokenType::Ampersand,
            Token::Caret(_) => TokenType::Caret,
            Token::LeftShift(_) => TokenType::LeftShift,
            Token::RightShift(_) => TokenType::RightShift,
            Token::Exclamation(_) => TokenType::Exclamation,
            Token::AmpersandAmpersand(_) => TokenType::AmpersandAmpersand,
            Token::PipePipe(_) => TokenType::PipePipe,
            Token::EqualEqual(_) => TokenType::EqualEqual,
            Token::ExclamationEqual(_) => TokenType::ExclamationEqual,
            Token::Less(_) => TokenType::Less,
            Token::Greater(_) => TokenType::Greater,
            Token::LessEqual(_) => TokenType::LessEqual,
            Token::GreaterEqual(_) => TokenType::GreaterEqual,
            Token::Equal(_) => TokenType::Equal,
            Token::PlusEqual(_) => TokenType::PlusEqual,
            Token::HypenEqual(_) => TokenType::HypenEqual,
            Token::StarEqual(_) => TokenType::StarEqual,
            Token::SlashEqual(_) => TokenType::SlashEqual,
            Token::PercentEqual(_) => TokenType::PercentEqual,
            Token::AmpersandEqual(_) => TokenType::AmpersandEqual,
            Token::PipeEqual(_) => TokenType::PipeEqual,
            Token::CaretEqual(_) => TokenType::CaretEqual,
            Token::LeftShiftEqual(_) => TokenType::LeftShiftEqual,
            Token::RightShiftEqual(_) => TokenType::RightShiftEqual,
            Token::PlusPlus(_) => TokenType::PlusPlus,
            Token::If(_) => TokenType::If,
            Token::Else(_) => TokenType::Else,
            Token::QuestionMark(_) => TokenType::QuestionMark,
            Token::Colon(_) => TokenType::Colon,
            Token::Goto(_) => TokenType::Goto,
        }
    }

    pub fn span(&self) -> &Span {
        match self {
            Token::Identifier(_, span)
            | Token::Constant(_, span)
            | Token::OpenParen(span)
            | Token::CloseParen(span)
            | Token::OpenBrace(span)
            | Token::CloseBrace(span)
            | Token::Semicolon(span)
            | Token::Return(span)
            | Token::Int(span)
            | Token::Tilde(span)
            | Token::Hypen(span)
            | Token::HypenHypen(span)
            | Token::Star(span)
            | Token::Plus(span)
            | Token::Slash(span)
            | Token::Percent(span)
            | Token::Pipe(span)
            | Token::Ampersand(span)
            | Token::Caret(span)
            | Token::LeftShift(span)
            | Token::RightShift(span)
            | Token::AmpersandAmpersand(span)
            | Token::PipePipe(span)
            | Token::EqualEqual(span)
            | Token::ExclamationEqual(span)
            | Token::Less(span)
            | Token::Greater(span)
            | Token::LessEqual(span)
            | Token::GreaterEqual(span)
            | Token::Exclamation(span)
            | Token::Void(span)
            | Token::Equal(span)
            | Token::PlusEqual(span)
            | Token::HypenEqual(span)
            | Token::StarEqual(span)
            | Token::SlashEqual(span)
            | Token::PercentEqual(span)
            | Token::AmpersandEqual(span)
            | Token::PipeEqual(span)
            | Token::CaretEqual(span)
            | Token::LeftShiftEqual(span)
            | Token::RightShiftEqual(span)
            | Token::PlusPlus(span)
            | Token::If(span)
            | Token::Else(span)
            | Token::QuestionMark(span)
            | Token::Colon(span)
            | Token::Goto(span) => span,
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
            "^" => Some(TokenType::Caret),
            "&" => Some(TokenType::Ampersand),
            "|" => Some(TokenType::Pipe),
            "<<" => Some(TokenType::LeftShift),
            ">>" => Some(TokenType::RightShift),
            "!" => Some(TokenType::Exclamation),
            "&&" => Some(TokenType::AmpersandAmpersand),
            "||" => Some(TokenType::PipePipe),
            "==" => Some(TokenType::EqualEqual),
            "!=" => Some(TokenType::ExclamationEqual),
            "<" => Some(TokenType::Less),
            ">" => Some(TokenType::Greater),
            "<=" => Some(TokenType::LessEqual),
            ">=" => Some(TokenType::GreaterEqual),
            "=" => Some(TokenType::Equal),
            "+=" => Some(TokenType::PlusEqual),
            "-=" => Some(TokenType::HypenEqual),
            "*=" => Some(TokenType::StarEqual),
            "/=" => Some(TokenType::SlashEqual),
            "%=" => Some(TokenType::PercentEqual),
            "&=" => Some(TokenType::AmpersandEqual),
            "|=" => Some(TokenType::PipeEqual),
            "^=" => Some(TokenType::CaretEqual),
            "<<=" => Some(TokenType::LeftShiftEqual),
            ">>=" => Some(TokenType::RightShiftEqual),
            "++" => Some(TokenType::PlusPlus),
            "if" => Some(TokenType::If),
            "else" => Some(TokenType::Else),
            "goto" => Some(TokenType::Goto),
            "?" => Some(TokenType::QuestionMark),
            ":" => Some(TokenType::Colon),
            _ => None,
        }
    }
    pub fn create_token(token_type: TokenType, s: &str, span: Span) -> Self {
        match token_type {
            TokenType::Identifier => Token::Identifier(s.to_string(), span),
            TokenType::Constant => Token::Constant(s.parse::<i64>().unwrap(), span),
            TokenType::Int => Token::Int(span),
            TokenType::Void => Token::Void(span),
            TokenType::Return => Token::Return(span),
            TokenType::OpenParen => Token::OpenParen(span),
            TokenType::CloseParen => Token::CloseParen(span),
            TokenType::OpenBrace => Token::OpenBrace(span),
            TokenType::CloseBrace => Token::CloseBrace(span),
            TokenType::Semicolon => Token::Semicolon(span),
            TokenType::Tilde => Token::Tilde(span),
            TokenType::Hypen => Token::Hypen(span),
            TokenType::HypenHypen => Token::HypenHypen(span),
            TokenType::Percent => Token::Percent(span),
            TokenType::Slash => Token::Slash(span),
            TokenType::Star => Token::Star(span),
            TokenType::Plus => Token::Plus(span),
            TokenType::Pipe => Token::Pipe(span),
            TokenType::Ampersand => Token::Ampersand(span),
            TokenType::Caret => Token::Caret(span),
            TokenType::LeftShift => Token::LeftShift(span),
            TokenType::RightShift => Token::RightShift(span),
            TokenType::Exclamation => Token::Exclamation(span),
            TokenType::AmpersandAmpersand => Token::AmpersandAmpersand(span),
            TokenType::PipePipe => Token::PipePipe(span),
            TokenType::EqualEqual => Token::EqualEqual(span),
            TokenType::ExclamationEqual => Token::ExclamationEqual(span),
            TokenType::Less => Token::Less(span),
            TokenType::Greater => Token::Greater(span),
            TokenType::LessEqual => Token::LessEqual(span),
            TokenType::GreaterEqual => Token::GreaterEqual(span),
            TokenType::Equal => Token::Equal(span),
            TokenType::PlusEqual => Token::PlusEqual(span),
            TokenType::HypenEqual => Token::HypenEqual(span),
            TokenType::StarEqual => Token::StarEqual(span),
            TokenType::SlashEqual => Token::SlashEqual(span),
            TokenType::PercentEqual => Token::PercentEqual(span),
            TokenType::AmpersandEqual => Token::AmpersandEqual(span),
            TokenType::PipeEqual => Token::PipeEqual(span),
            TokenType::CaretEqual => Token::CaretEqual(span),
            TokenType::LeftShiftEqual => Token::LeftShiftEqual(span),
            TokenType::RightShiftEqual => Token::RightShiftEqual(span),
            TokenType::PlusPlus => Token::PlusPlus(span),
            TokenType::If => Token::If(span),
            TokenType::Else => Token::Else(span),
            TokenType::QuestionMark => Token::QuestionMark(span),
            TokenType::Colon => Token::Colon(span),
            TokenType::Goto => Token::Goto(span),
        }
    }
    pub fn is_binary_op(&self) -> bool {
        matches!(
            self.get_token_type(),
            TokenType::Hypen
                | TokenType::Plus
                | TokenType::Slash
                | TokenType::Star
                | TokenType::Percent
                | TokenType::Pipe
                | TokenType::Ampersand
                | TokenType::Caret
                | TokenType::LeftShift
                | TokenType::RightShift
                | TokenType::AmpersandAmpersand
                | TokenType::PipePipe
                | TokenType::EqualEqual
                | TokenType::ExclamationEqual
                | TokenType::Less
                | TokenType::Greater
                | TokenType::LessEqual
                | TokenType::GreaterEqual
                | TokenType::Equal
                | TokenType::QuestionMark
        )
    }
    pub fn is_compound_assign(&self) -> bool {
        matches!(
            self.get_token_type(),
            |TokenType::PlusEqual| TokenType::HypenEqual
                | TokenType::StarEqual
                | TokenType::SlashEqual
                | TokenType::PercentEqual
                | TokenType::AmpersandEqual
                | TokenType::PipeEqual
                | TokenType::CaretEqual
                | TokenType::LeftShiftEqual
                | TokenType::RightShiftEqual
        )
    }

    pub fn get_precedence(&self) -> usize {
        if let Some(u) = OPERATOR_PRECEDENCE.get(&self.get_token_type()) {
            return *u;
        }
        0
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Token::Identifier(str, _) => write!(f, "{}", str),
            Token::Constant(i, _) => write!(f, "{}", i),
            Token::OpenParen(_) => write!(f, "("),
            Token::CloseParen(_) => write!(f, ")"),
            Token::OpenBrace(_) => write!(f, "{{"),
            Token::CloseBrace(_) => write!(f, "}}"),
            Token::Semicolon(_) => write!(f, ";"),
            Token::Return(_) => write!(f, "return"),
            Token::Int(_) => write!(f, "int"),
            Token::Void(_) => write!(f, "void"),
            Token::Tilde(_) => write!(f, "~"),
            Token::Hypen(_) => write!(f, "-"),
            Token::HypenHypen(_) => write!(f, "--"),
            Token::Plus(_) => write!(f, "+"),
            Token::Star(_) => write!(f, "*"),
            Token::Percent(_) => write!(f, "%"),
            Token::Slash(_) => write!(f, "/"),
            Token::Pipe(_) => write!(f, "|"),
            Token::Caret(_) => write!(f, "^"),
            Token::LeftShift(_) => write!(f, "<<"),
            Token::RightShift(_) => write!(f, ">>"),
            Token::Ampersand(_) => write!(f, "&"),
            Token::Exclamation(_) => write!(f, "!"),
            Token::AmpersandAmpersand(_) => write!(f, "&&"),
            Token::PipePipe(_) => write!(f, "||"),
            Token::EqualEqual(_) => write!(f, "=="),
            Token::ExclamationEqual(_) => write!(f, "!="),
            Token::Less(_) => write!(f, "<"),
            Token::Greater(_) => write!(f, ">"),
            Token::LessEqual(_) => write!(f, "<="),
            Token::GreaterEqual(_) => write!(f, ">="),
            Token::Equal(_) => write!(f, "="),
            Token::PlusEqual(_) => write!(f, "+="),
            Token::HypenEqual(_) => write!(f, "-="),
            Token::StarEqual(_) => write!(f, "*="),
            Token::SlashEqual(_) => write!(f, "/="),
            Token::PercentEqual(_) => write!(f, "%="),
            Token::AmpersandEqual(_) => write!(f, "&="),
            Token::PipeEqual(_) => write!(f, "|="),
            Token::CaretEqual(_) => write!(f, "^="),
            Token::LeftShiftEqual(_) => write!(f, "<<="),
            Token::RightShiftEqual(_) => write!(f, ">>="),
            Token::PlusPlus(_) => write!(f, "++"),
            Token::If(_) => write!(f, "if"),
            Token::Else(_) => write!(f, "else"),
            Token::QuestionMark(_) => write!(f, "?"),
            Token::Colon(_) => write!(f, ":"),
            Token::Goto(_) => write!(f, "goto"),
        }
    }
}
