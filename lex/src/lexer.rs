use crate::Span;
use crate::error::LexerError;
use crate::patterns::*;
use crate::token::{Token, TokenType};

pub struct Lexer {
    line: usize,
    pos: usize,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer { line: 1, pos: 0 }
    }

    fn new_line(&mut self) {
        self.line += 1;
        self.pos = 0;
    }

    fn skip_comments(&mut self, remaining: &mut String) -> bool {
        for c in SKIP_PATTERNS.iter() {
            if let Some(mat) = c.find(remaining) {
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

        if let Some(mat) = MULIT_LINE_COMMENT.find(remaining) {
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

    pub fn tokenize(&mut self, input: String) -> Result<Vec<Token>, LexerError> {
        let mut tokens = Vec::new();
        let mut remaining = input.clone();

        while !remaining.is_empty() {
            if let Some(ch) = remaining.chars().next() {
                if ch.is_whitespace() {
                    if ch == '\n' {
                        self.new_line();
                    } else {
                        self.pos += 1;
                    }
                    remaining = remaining[ch.len_utf8()..].to_string();
                    continue;
                }
            }

            if self.skip_comments(&mut remaining) {
                continue;
            }

            let mut longest_match: Option<(TokenType, &str)> = None;
            for (regex, token_type) in PATTERNS.iter() {
                if let Some(mat) = regex.find(&remaining) {
                    let match_len = mat.end();
                    if longest_match.is_none()
                        || match_len > longest_match.as_ref().unwrap().1.len()
                    {
                        longest_match = Some((token_type.clone(), mat.as_str()))
                    }
                }
            }

            if longest_match.is_none() {
                return Err(LexerError::InvalidToken(
                    self.line,
                    self.pos + 1,
                    remaining.chars().next().unwrap(),
                ));
            }

            let matched = longest_match.unwrap();

            if matched.1.is_empty() {
                return Err(LexerError::InvalidToken(
                    self.line,
                    self.pos + 1,
                    remaining.chars().next().unwrap(),
                ));
            }

            if matches!(matched.0, TokenType::Constant)
                && let Some(next_char) = remaining[matched.1.len()..].chars().next()
            {
                if next_char.is_alphabetic() || next_char == '_' {
                    return Err(LexerError::InvalidToken(self.line, self.pos + 1, next_char));
                }
            }

            let start_pos = self.pos;
            self.pos += matched.1.len();
            tokens.push(Token::create_token(
                matched.0,
                matched.1,
                Span::new(start_pos, self.line),
            ));
            remaining = remaining[matched.1.len()..].to_string();
        }

        Ok(tokens)
    }
}

impl Default for Lexer {
    fn default() -> Self {
        Self::new()
    }
}
