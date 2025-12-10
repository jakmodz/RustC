use crate::ParseExpr;
use crate::ParseStmt;
use crate::decl::StorageClass;
use crate::parser_error::ParserError;
use crate::{Declaration, Expression, ast::*};
use lex::token;
use lex::token::{Token, TokenType};

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
        let mut declarations = Vec::new();

        while self.pos < self.tokens.len() {
            let decl = self.parse_declaration()?;
            declarations.push(decl);
        }

        Ok(Program { declarations })
    }

    pub(crate) fn parse_block_element(&mut self) -> Result<BlockElement, ParserError> {
        if self.pos < self.tokens.len() {
            let next_token = self.peek()?;
            match next_token.get_token_type() {
                TokenType::Int | TokenType::Static | TokenType::Extern => {
                    let decl = self.parse_declaration()?;
                    return Ok(BlockElement::Declaration(decl));
                }
                _ => {}
            }
        }
        
        let stmt = self.parse_stmt()?;
        Ok(BlockElement::Stmt(stmt))
    }

    fn parse_type_and_storage_class(&mut self) -> Result<Option<StorageClass>, ParserError> {
        let mut types = Vec::new();
        let mut storage_classes = Vec::new();

        loop {
            if self.pos >= self.tokens.len() {
                break;
            }
            let next_token = self.peek()?;
            match next_token.get_token_type() {
                TokenType::Int => {
                    self.eat()?;
                    types.push(TokenType::Int);
                }
                TokenType::Static => {
                    self.eat()?;
                    storage_classes.push(StorageClass::Static);
                }
                TokenType::Extern => {
                    self.eat()?;
                    storage_classes.push(StorageClass::Extern);
                }
                _ => break,
            }
        }

        if types.len() != 1 {
            if self.pos >= self.tokens.len() {
                return Err(ParserError::UnexpectedEOF);
            }
            let token = self.peek()?;
            return Err(ParserError::UnexpectedToken(
                token.span().line,
                token.span().column,
                "Expected exactly one type specifier".to_string(),
            ));
        }

        if storage_classes.len() > 1 {
            let token = self.peek()?;
            return Err(ParserError::UnexpectedToken(
                token.span().line,
                token.span().column,
                "Multiple storage-class specifiers".to_string(),
            ));
        }

        let storage_class = if storage_classes.is_empty() {
            None
        } else {
            Some(storage_classes[0].clone())
        };

        Ok(storage_class)
    }

    fn parse_declaration(&mut self) -> Result<Declaration, ParserError> {
        let storage_class = self.parse_type_and_storage_class()?;
        
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
                        storage_class,
                    },
                })
            } else {
                self.excepted_token(TokenType::Semicolon)?;

                Ok(Declaration::FuncDecl {
                    decl: FuncDecl {
                        name,
                        params,
                        body: None,
                        storage_class,
                    },
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
                storage_class,
            }))
        }
    }

    pub(crate) fn parse_for_init(&mut self) -> Result<ForInit, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        
        let next_token = self.peek()?;
        match next_token.get_token_type() {
            TokenType::Int | TokenType::Static | TokenType::Extern => {
                let decl = self.parse_declaration()?;
                match decl {
                    Declaration::DefineVar(var_decl) => Ok(ForInit::Declaration(var_decl)),
                    _ => Err(ParserError::UnexpectedToken(
                        next_token.span().line,
                        next_token.span().column,
                        "Function declaration not allowed in for loop init".to_string(),
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