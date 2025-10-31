use regex::Regex;
use crate::token::{Token,TokenType};
use lazy_static::lazy_static;
use crate::error::LexerError;

lazy_static! {
    static ref PATTERNS: Vec<(Regex, TokenType)> = {
        vec![
            // Keywords first (most specific)
            (Regex::new(r"^int\b").unwrap(), TokenType::Int),
            (Regex::new(r"^void\b").unwrap(), TokenType::Void),
            (Regex::new(r"^return\b").unwrap(), TokenType::Return),

            // Identifiers (after keywords)
            (Regex::new(r"^[a-zA-Z_]\w*").unwrap(), TokenType::Identifier),

            // Numbers
            (Regex::new(r"^[0-9]+").unwrap(), TokenType::Constant),

            // Punctuation
            (Regex::new(r"^\(").unwrap(), TokenType::OpenParen),
            (Regex::new(r"^\)").unwrap(), TokenType::CloseParen),
            (Regex::new(r"^\{").unwrap(), TokenType::OpenBrace),
            (Regex::new(r"^\}").unwrap(), TokenType::CloseBrace),
            (Regex::new(r"^\;").unwrap(), TokenType::Semicolon),
        ]
    };
    static ref SKIP_PATTERNS: Vec<Regex> = {
        vec![
            Regex::new(r"^//[^\n]*").unwrap(),
            Regex::new(r"^/\*[\s\S]*?\*/").unwrap(),
        ]
    };
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


    pub fn tokenize(&mut self,mut input:String)->Result<Vec<Token>,LexerError>{
        let mut tokens = Vec::new();
        let mut remaining = input.clone();

        while !remaining.is_empty() {
            if remaining.starts_with(char::is_whitespace){
                remaining = remaining.trim_start().to_string();
                if remaining.starts_with("//") {
                    if let Some(newline_pos) = remaining.find('\n') {
                        remaining = remaining[newline_pos..].to_string();
                    } else {
                        break;
                    }
                    continue;
                }
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
                return Err(LexerError::UnknownToken)
            }

            let matched = longest_match.unwrap();

            tokens.push(Token::create_token(matched.0,matched.1));
            remaining = remaining[matched.1.len()..].to_string();


        }

        Ok(tokens)
    }
}