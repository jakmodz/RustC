use crate::ast::*;
use crate::parser_error::ParserError;
use lex::token;
use lex::token::{Token, TokenType};

pub struct Parser {
    tokens: Vec<token::Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<token::Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn excepted_token(&mut self, expected: TokenType) -> Result<(), ParserError> {
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
        Ok(())
    }
    fn except_token_optional(&mut self, expected: TokenType) -> Result<(), ParserError> {
        let res = self.excepted_token(expected);
        if res.is_err() {
            return res;
        }
        Ok(())
    }

    fn peek(&mut self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        Ok(current_token.clone())
    }
    fn eat(&mut self) -> Result<Token, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        self.pos += 1;
        Ok(current_token.clone())
    }

    pub fn parse(&mut self) -> Result<Program, ParserError> {
        let mut function_body = Vec::new();
        self.excepted_token(TokenType::Int)?;
        self.excepted_token(TokenType::Identifier)?;
        self.excepted_token(TokenType::OpenParen)?;
        self.except_token_optional(TokenType::Void)?;
        self.excepted_token(TokenType::CloseParen)?;
        self.excepted_token(TokenType::OpenBrace)?;
        while self.peek()?.get_token_type() != TokenType::CloseBrace {
            let block_item = self.parse_block_element()?;

            function_body.push(block_item);
        }
        self.eat()?;
        if self.pos < self.tokens.len() {
            return Err(ParserError::UnexpectedToken(
                self.tokens[self.pos].span().line,
                self.tokens[self.pos].span().column,
                format!("{:?}", self.tokens[self.pos].get_token_type()),
            ));
        }
        Ok(Program {
            function: Function {
                name: "main".to_string(),
                body: function_body,
            },
        })
    }
    fn parse_block_element(&mut self) -> Result<BlockElement, ParserError> {
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

    /*
    Parse Declaration
    */

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
        let var_name = match var_name_token {
            Token::Identifier(name, _span) => name,
            _ => unreachable!(),
        };
        let mut initializer: Option<Expression> = None;
        let next_token = self.peek()?;
        if next_token.get_token_type() == TokenType::Equal {
            self.eat()?;
            let expr = self.parse_expression(0)?;
            initializer = Some(expr);
        }
        self.excepted_token(TokenType::Semicolon)?;
        Ok(Declaration::DefineVar {
            var_name,
            initializer,
        })
    }

    /*
    Parse Statement
     */
    fn parse_stmt(&mut self) -> Result<Stmt, ParserError> {
        let next_token = self.peek()?;
        match next_token.get_token_type() {
            TokenType::Return => {
                self.eat()?;
                let return_val = self.parse_expression(0)?;
                self.excepted_token(TokenType::Semicolon)?;
                Ok(Stmt::Return { expr: return_val })
            }
            TokenType::Semicolon => {
                self.eat()?;
                Ok(Stmt::Null)
            }
            _ => {
                let expr = self.parse_expression(0)?;
                self.excepted_token(TokenType::Semicolon)?;
                Ok(Stmt::Expression { expr })
            }
        }
    }

    /*
    Parse Expression
    */
    fn parse_expression(&mut self, min_precedence: usize) -> Result<Expression, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let mut left = self.parse_factor()?;
        let mut next_token = self.peek()?;
        if next_token.get_token_type() == TokenType::PlusPlus {
            if !matches!(left, Expression::Var(_)) {
                return Err(ParserError::InvalidLValue(
                    next_token.span().line,
                    next_token.span().column,
                    "Postfix ++ requires an lvalue".to_string(),
                ));
            }
            self.eat()?;
            left = Expression::Increment {
                expr: Box::new(left),
                pre: false,
            };
        } else if next_token.get_token_type() == TokenType::HypenHypen {
            if !matches!(left, Expression::Var(_)) {
                return Err(ParserError::InvalidLValue(
                    next_token.span().line,
                    next_token.span().column,
                    "Postfix -- requires an lvalue".to_string(),
                ));
            }
            self.eat()?;
            left = Expression::Decrement {
                expr: Box::new(left),
                pre: false,
            };
        }
        next_token = self.peek()?;
        while (next_token.is_binary_op() || next_token.is_compound_assign())
            && next_token.get_precedence() >= min_precedence
        {
            if next_token.get_token_type() == TokenType::Equal {
                let op = self.eat()?;
                let op_precedence = op.get_precedence();
                let right = self.parse_expression(op_precedence)?;
                left = Expression::Assignment {
                    expr_to: Box::new(left),
                    initializer: Box::new(right),
                };
            } else if next_token.is_compound_assign() {
                let op = self.eat()?;
                let op_precedence = op.get_precedence();
                left = Expression::CompoundAssign {
                    op,
                    var: Box::new(left),
                    expr: Box::new(self.parse_expression(op_precedence)?),
                };
            } else {
                let operator = self.eat()?;
                let op_precedence = operator.get_precedence();
                let right = self.parse_expression(op_precedence + 1)?;
                left = Expression::Binary {
                    op: operator,
                    left: Box::new(left),
                    right: Box::new(right),
                };
            }
            next_token = self.peek()?
        }

        Ok(left)
    }
    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let next_token = self.peek()?;
        let mut result = match next_token {
            Token::Constant(value, _span) => {
                self.eat()?;
                Expression::Constant(value)
            }
            Token::OpenParen(_) => {
                self.eat()?;
                let expr = self.parse_expression(0)?;
                self.excepted_token(TokenType::CloseParen)?;
                expr
            }
            Token::Hypen(_) | Token::Tilde(_) | Token::Exclamation(_) => {
                let op = self.eat()?;
                let expr = self.parse_factor()?;
                Expression::UnaryOP {
                    op,
                    expr: Box::new(expr),
                }
            }
            Token::Identifier(name, _span) => {
                self.eat()?;
                Expression::Var(name)
            }
            Token::PlusPlus(_) => {
                self.eat()?;
                let expr = self.parse_factor()?;
                Expression::Increment {
                    expr: Box::new(expr),
                    pre: true,
                }
            }
            Token::HypenHypen(_) => {
                self.eat()?;
                let expr = self.parse_factor()?;
                Expression::Decrement {
                    expr: Box::new(expr),
                    pre: true,
                }
            }
            _ => {
                let current_token = self.peek()?;
                return Err(ParserError::UnexpectedToken(
                    current_token.span().line,
                    current_token.span().column,
                    format!("{:?}", current_token.get_token_type()),
                ));
            }
        };
        loop {
            let next = self.peek()?;
            if next.get_token_type() == TokenType::PlusPlus {
                if !matches!(result, Expression::Var(_)) {
                    return Err(ParserError::InvalidLValue(
                        next.span().line,
                        next.span().column,
                        "Postfix ++ requires an lvalue".to_string(),
                    ));
                }
                self.eat()?;
                result = Expression::Increment {
                    expr: Box::new(result),
                    pre: false,
                };
            } else if next.get_token_type() == TokenType::HypenHypen {
                if !matches!(result, Expression::Var(_)) {
                    return Err(ParserError::InvalidLValue(
                        next.span().line,
                        next.span().column,
                        "Postfix -- requires an lvalue".to_string(),
                    ));
                }
                self.eat()?;
                result = Expression::Decrement {
                    expr: Box::new(result),
                    pre: false,
                };
            } else {
                break;
            }
        }
        Ok(result)
    }
}
