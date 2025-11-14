use crate::error::LexerError;
use crate::token::{Span, Token, TokenType};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref PATTERNS: Vec<(Regex, TokenType)> = {
        vec![
            // Multi-character operators MUST come first!
            (Regex::new(r"^<<").unwrap(), TokenType::LeftShift),
            (Regex::new(r"^>>").unwrap(), TokenType::RightShift),
            (Regex::new(r"^--").unwrap(), TokenType::HypenHypen),
            (Regex::new(r"^&&").unwrap(), TokenType::AmpersandAmpersand),
            (Regex::new(r"^\|\|").unwrap(), TokenType::PipePipe),
            (Regex::new(r"^==").unwrap(), TokenType::EqualEqual),
            (Regex::new(r"^!=").unwrap(), TokenType::ExclamationEqual),
            (Regex::new(r"^<=").unwrap(), TokenType::LessEqual),
            (Regex::new(r"^>=").unwrap(), TokenType::GreaterEqual),

            // Keywords
            (Regex::new(r"^int\b").unwrap(), TokenType::Int),
            (Regex::new(r"^void\b").unwrap(), TokenType::Void),
            (Regex::new(r"^return\b").unwrap(), TokenType::Return),

            // Identifiers (after keywords)
            (Regex::new(r"^[a-zA-Z_]\w*").unwrap(), TokenType::Identifier),

            // Numbers
            (Regex::new(r"^[0-9]+").unwrap(), TokenType::Constant),

            // Punctuation and single-character operators
            (Regex::new(r"^\(").unwrap(), TokenType::OpenParen),
            (Regex::new(r"^\)").unwrap(), TokenType::CloseParen),
            (Regex::new(r"^\{").unwrap(), TokenType::OpenBrace),
            (Regex::new(r"^\}").unwrap(), TokenType::CloseBrace),
            (Regex::new(r"^;").unwrap(), TokenType::Semicolon),
            (Regex::new(r"^~").unwrap(), TokenType::Tilde),
            (Regex::new(r"^-").unwrap(), TokenType::Hypen),
            (Regex::new(r"^=").unwrap(), TokenType::Equal),
            (Regex::new(r"^\+").unwrap(), TokenType::Plus),
            (Regex::new(r"^\*").unwrap(), TokenType::Star),
            (Regex::new(r"^%").unwrap(), TokenType::Percent),
            (Regex::new(r"^/").unwrap(), TokenType::Slash),
            (Regex::new(r"^\|").unwrap(), TokenType::Pipe),
            (Regex::new(r"^&").unwrap(), TokenType::Ampersand),
            (Regex::new(r"^\^").unwrap(), TokenType::Caret),
            (Regex::new(r"^!").unwrap(), TokenType::Exclamation),
            (Regex::new(r"^<").unwrap(), TokenType::Less),
            (Regex::new(r"^>").unwrap(), TokenType::Greater),

        ]
    };

    static ref SKIP_PATTERNS: Vec<Regex> = {
        vec![
           Regex::new(r"^//[^\n]*\n?").unwrap(),
           Regex::new(r"^#(?:[^\n\\]|\\\n)*\n?").unwrap()
        ]
    };

    static ref MULIT_LINE_COMMENT: Regex = Regex::new(r"^/\*[\s\S]*?\*/").unwrap();
}

pub struct Lexer{
    line:usize,
    pos:usize
}

impl Lexer{
    pub fn new() -> Self{
        Lexer{
            line:1,
            pos:0
        }
    }

    fn new_line(&mut self){
        self.line+=1;
        self.pos = 0;
    }

    fn skip_comments(&mut self, remaining: &mut String) -> bool{
        for c in SKIP_PATTERNS.iter() {
            if let Some(mat) = c.find(&remaining) {
                let matched_text = mat.as_str();
                if matched_text.ends_with('\n') {
                    self.new_line();
                } else {
                    self.pos += matched_text.len();
                }
                *remaining = remaining[mat.len()..].to_string();
                return true;
            }
        }

        if let Some(mat) = MULIT_LINE_COMMENT.find(&remaining) {
            let mat_text = mat.as_str();
            for c in mat_text.chars() {
                if c == '\n' {
                    self.new_line();
                } else {
                    self.pos += 1;
                }
            }
            *remaining = remaining[mat.end()..].to_string();
            return true;
        }

        false
    }

    pub fn tokenize(&mut self, input:String) -> Result<Vec<Token>,LexerError>{
        let mut tokens = Vec::new();
        let mut remaining = input.clone();

        while !remaining.is_empty() {

            if remaining.starts_with(char::is_whitespace){
                let ch = remaining.chars().next().unwrap();
                if ch == '\n' {
                    self.new_line();
                } else {
                    self.pos += 1;
                }
                remaining = remaining[ch.len_utf8()..].to_string();
                continue
            }


            if self.skip_comments(&mut remaining) {
                continue
            }


            let mut longest_match: Option<(TokenType,&str)> = None;
            for (regex,token_type) in PATTERNS.iter(){
                if let Some(mat) = regex.find(&remaining){
                    let match_len = mat.end();
                    if longest_match.is_none() || match_len > longest_match.as_ref().unwrap().1.len() {
                        longest_match = Some((token_type.clone(),mat.as_str()))
                    }
                }
            }

            if longest_match.is_none(){
                return Err(LexerError::InvalidToken(self.line,self.pos+1,remaining.chars().next().unwrap()))
            }

            let matched = longest_match.unwrap();

            if matches!(matched.0, TokenType::Constant) {
                if let Some(next_char) = remaining[matched.1.len()..].chars().next() {
                    if next_char.is_alphabetic() || next_char == '_' {
                        return Err(LexerError::InvalidToken(self.line, self.pos + 1, next_char));
                    }
                }
            }

            self.pos += matched.1.len();
            tokens.push(Token::create_token(matched.0,matched.1,Span::new(self.pos,self.line)));
            remaining = remaining[matched.1.len()..].to_string();
        }

        Ok(tokens)
    }
}