use lex::token::{Token, TokenType};

use crate::{Expression, parser::Parser, parser_error::ParserError};

pub trait ParseExpr {
    fn parse_expression(&mut self, min_precedence: usize) -> Result<Expression, ParserError>;
    fn parse_factor(&mut self) -> Result<Expression, ParserError>;
    fn parse_middle(&mut self) -> Result<Expression, ParserError>;
}

impl ParseExpr for Parser {
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
            }else if next_token.get_token_type() == TokenType::QuestionMark {
                let middle = self.parse_middle()?;
                let right = self.parse_expression(next_token.get_precedence())?;
                left = Expression::Conditional {
                    cond: Box::new(left),
                    expr1: Box::new(middle),
                    expr2: Box::new(right),
                }
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
    fn parse_middle(&mut self) -> Result<Expression, ParserError> {
        self.eat()?;
        let expr = self.parse_expression(0)?;
        self.excepted_token(TokenType::Colon)?;
        Ok(expr)
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
                if self.peek()?.get_token_type() == TokenType::OpenParen {
                    self.eat()?;
                    let mut args = Vec::new();
                    if self.peek()?.get_token_type() != TokenType::CloseParen {
                        loop {
                            let arg = self.parse_expression(0)?;
                            args.push(arg);
                            if self.peek()?.get_token_type() == TokenType::Comma {
                                self.eat()?;
                            } else {
                                break;
                            }
                        }
                    }
                    self.excepted_token(TokenType::CloseParen)?;
                    return Ok(Expression::FunctionCall {
                        func_name: name,
                        args,
                    });
                }
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