use crate::{Declaration, Expression, ast::*};
use crate::parser_error::ParserError;
use lex::token;
use lex::token::{Token, TokenType};
use crate::ParseExpr;
use crate::ParseStmt;

pub struct Parser {
    pub(crate) tokens: Vec<token::Token>,
    pub(crate) pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<token::Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    pub(crate) fn excepted_token(&mut self, expected: TokenType) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        if current_token.get_token_type() != expected {
            return Err(ParserError::ExpectedToken(
                current_token.span().line,
                current_token.span().column,
                format!("{:?}", expected),
                format!("{:?}", current_token.get_token_type()),
            ));
        }
        self.pos += 1;
        Ok(current_token.clone())
    }
    
    pub(crate) fn except_token_optional(&mut self, expected: TokenType) -> Result<(), ParserError> {
        let res = self.excepted_token(expected);
        match res {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    pub(crate) fn peek(&mut self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        Ok(current_token.clone())
    }
    
    pub(crate) fn peek_next(&mut self) -> Result<Token, ParserError> {
        if self.pos + 1 >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let next_token = &self.tokens[self.pos + 1];
        Ok(next_token.clone())
    }
    
    pub(crate) fn eat(&mut self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        self.pos += 1;
        Ok(current_token.clone())
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut functions = Vec::new();
        
        while self.pos < self.tokens.len() {
            let func_decl = self.parse_function()?;
            functions.push(func_decl);
        }
        
        Ok(Program { functions })
    }

    fn parse_function(&mut self) -> Result<FuncDecl, ParserError> {
        self.excepted_token(TokenType::Int)?;
        
        let token = self.excepted_token(TokenType::Identifier)?;
        let name = token.to_string();
        
        self.excepted_token(TokenType::OpenParen)?;
        let mut params = Vec::new();
        
        if self.peek()?.get_token_type() != TokenType::CloseParen {
            if self.peek()?.get_token_type() == TokenType::Void {
                self.eat()?;
            } else {
                loop {
                    self.excepted_token(TokenType::Int)?;
                    let param_name_token = self.excepted_token(TokenType::Identifier)?;
                    let param_name = param_name_token.to_string();
                    params.push(param_name);
                    
                    if self.peek()?.get_token_type() == TokenType::Comma {
                        self.eat()?;
                    } else {
                        break;
                    }
                }
            }
        }
        
        self.excepted_token(TokenType::CloseParen)?;
        
        let next_token = self.peek()?;
        if next_token.get_token_type() == TokenType::Semicolon {
            self.eat()?;
            Ok(FuncDecl {
                name,
                params,
                body: None,
            })
        } else if next_token.get_token_type() == TokenType::OpenBrace {
            
            self.eat()?;
            let mut function_body = Vec::new();
            
            while self.peek()?.get_token_type() != TokenType::CloseBrace {
                let block_item = self.parse_block_element()?;
                function_body.push(block_item);
            }
            
            self.excepted_token(TokenType::CloseBrace)?;
            
            Ok(FuncDecl {
                name,
                params,
                body: Some(Block::new(function_body)),
            })
        } else {
            return Err(ParserError::ExpectedToken(
                next_token.span().line,
                next_token.span().column,
                "OpenBrace or Semicolon".to_string(),
                format!("{:?}", next_token.get_token_type()),
            ));
        }
    }

    pub(crate) fn parse_block_element(&mut self) -> Result<BlockElement, ParserError> {
        let next_token = self.peek()?;
        match next_token.get_token_type() {
            TokenType::Int => {
                let decl = self.parse_declaration()?;
                Ok(BlockElement::Declaration(decl))
            }
            _ => {
                let stmt = self.parse_stmt()?;
                Ok(BlockElement::Stmt(stmt))
            }
        }
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParserError> {
        self.excepted_token(TokenType::Int)?;
        let var_name_token = self.eat()?;
        
        if var_name_token.get_token_type() != TokenType::Identifier {
            return Err(ParserError::ExpectedToken(
                var_name_token.span().line,
                var_name_token.span().column,
                format!("{:?}", TokenType::Identifier),
                format!("{:?}", var_name_token.get_token_type()),
            ));
        }
        
        let name = match var_name_token {
            Token::Identifier(name, _span) => name,
            _ => unreachable!(),
        };
        
        if self.peek()?.get_token_type() == TokenType::OpenParen {
            self.eat()?;
            let mut params = Vec::new();
            
            if self.peek()?.get_token_type() != TokenType::CloseParen {
                if self.peek()?.get_token_type() == TokenType::Void {
                    self.eat()?;
                } else {
                    loop {
                        self.excepted_token(TokenType::Int)?;
                        let param_name_token = self.excepted_token(TokenType::Identifier)?;
                        let param_name = param_name_token.to_string();
                        params.push(param_name);
                        
                        if self.peek()?.get_token_type() == TokenType::Comma {
                            self.eat()?;
                        } else {
                            break;
                        }
                    }
                }
            }
            
            self.excepted_token(TokenType::CloseParen)?;
            
            if self.peek()?.get_token_type() == TokenType::OpenBrace {
                self.eat()?;
                let mut function_body = Vec::new();
                
                while self.peek()?.get_token_type() != TokenType::CloseBrace {
                    let block_item = self.parse_block_element()?;
                    function_body.push(block_item);
                }
                
                self.excepted_token(TokenType::CloseBrace)?;
                
                Ok(Declaration::FuncDecl {
                    decl: FuncDecl {
                        name,
                        params,
                        body: Some(Block::new(function_body)),
                    }
                })
            } else {
                self.excepted_token(TokenType::Semicolon)?;
                
                Ok(Declaration::FuncDecl {
                    decl: FuncDecl {
                        name,
                        params,
                        body: None,
                    }
                })
            }
        } else {
            
            let mut initializer: Option<Expression> = None;
            let next_token = self.peek()?;
            
            if next_token.get_token_type() == TokenType::Equal {
                self.eat()?;
                let expr = self.parse_expression(0)?;
                initializer = Some(expr);
            }
            
            self.excepted_token(TokenType::Semicolon)?;
            
            Ok(Declaration::DefineVar(VariableDecl {
                name,
                init: initializer,
            }))
        }
    }
    
    pub(crate) fn parse_for_init(&mut self) -> Result<ForInit, ParserError> {
        let next_token = self.peek()?;
        match next_token.get_token_type() {
            TokenType::Int => {
                let decl = self.parse_declaration()?;
                match decl {
                    Declaration::DefineVar(var_decl) => Ok(ForInit::Declaration(var_decl)),
                    _ => Err(ParserError::UnexpectedToken(
                        next_token.span().line,
                        next_token.span().column,
                        format!("{:?}", next_token.get_token_type()),
                    )),
                }
            }
            _ => {
                let expr = if next_token.get_token_type() == TokenType::Semicolon {
                    None
                } else {
                    Some(self.parse_expression(0)?)
                };
                self.excepted_token(TokenType::Semicolon)?;
                Ok(ForInit::Expression(expr))
            }
        }
    }
}