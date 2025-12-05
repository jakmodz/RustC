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
    fn peek_next(&mut self) -> Result<Token, ParserError> {
        if self.pos + 1 >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let next_token = &self.tokens[self.pos + 1];
        Ok(next_token.clone())
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
                body: Block::new(function_body),
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
            TokenType::If => {
                self.eat()?;
                self.excepted_token(TokenType::OpenParen)?;
                let condition = self.parse_expression(0)?;
                self.excepted_token(TokenType::CloseParen)?;

                let then_branch = self.parse_stmt()?;
                let mut else_branch = None;
                let next_token = self.peek()?;
                if next_token.get_token_type() == TokenType::Else {
                    self.eat()?;
                    let else_stmts = self.parse_stmt()?;
                    else_branch = Some(else_stmts);
                }
                Ok(Stmt::If {
                    condition,
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                })
            }
            TokenType::Switch => {
                self.eat()?;
                self.excepted_token(TokenType::OpenParen)?;
                let expr = self.parse_expression(0)?;
                self.excepted_token(TokenType::CloseParen)?;
                let body = self.parse_stmt()?;
                Ok(Stmt::Switch {
                    expr,
                    body: Box::new(body),
                    annotation: Annotation::None,
                    cases: Vec::new(),
                    default_label: None,
                })
            }
            TokenType::Case => {
                self.eat()?;
                let value = self.parse_expression(0)?;
                self.excepted_token(TokenType::Colon)?;
                let body = self.parse_stmt()?;
                Ok(Stmt::Case {
                    value,
                    body: Box::new(body),
                })
            }
            TokenType::Default => {
                self.eat()?;
                self.excepted_token(TokenType::Colon)?;
                let body = self.parse_stmt()?;
                Ok(Stmt::Default {
                    body: Box::new(body),
                })
            }
            TokenType::Goto => {
                self.eat()?;
                let name = self.eat()?.to_string();
                self.excepted_token(TokenType::Semicolon)?;
                Ok(Stmt::Goto(name))
            }
            TokenType::Identifier => {
                if self.peek_next()?.get_token_type() == TokenType::Colon {
                    let name = self.eat()?.to_string();
                    self.eat()?;
                    Ok(Stmt::Label(name))
                } else {
                    let expr = self.parse_expression(0)?;
                    self.excepted_token(TokenType::Semicolon)?;
                    Ok(Stmt::Expression { expr })
                }
            }
            TokenType::OpenBrace => {
                self.eat()?;
                let mut block_elements = Vec::new();
                while self.peek()?.get_token_type() != TokenType::CloseBrace {
                    let block_item = self.parse_block_element()?;
                    block_elements.push(block_item);
                }
                self.excepted_token(TokenType::CloseBrace)?;
                Ok(Stmt::Compound {
                    block: Block::new(block_elements),
                })
            }
            TokenType::Break => {
                self.eat()?;
                self.excepted_token(TokenType::Semicolon)?;
                Ok(Stmt::Break(Annotation::None))
            }
            TokenType::Continue => {
                self.eat()?;
                self.excepted_token(TokenType::Semicolon)?;
                Ok(Stmt::Continue(Annotation::None))
            }
            TokenType::While => {
                self.eat()?;
                self.excepted_token(TokenType::OpenParen)?;
                let condition = self.parse_expression(0)?;
                self.excepted_token(TokenType::CloseParen)?;
                let body = self.parse_stmt()?;
                Ok(Stmt::While {
                    condition,
                    body: Box::new(body),
                    annotation: Annotation::None,
                })
            }
            TokenType::Do => {
                self.eat()?;
                let body = self.parse_stmt()?;
                self.excepted_token(TokenType::While)?;
                self.excepted_token(TokenType::OpenParen)?;
                let condition = self.parse_expression(0)?;
                self.excepted_token(TokenType::CloseParen)?;
                self.excepted_token(TokenType::Semicolon)?;
                Ok(Stmt::DoWhile {
                    body: Box::new(body),
                    condition,
                    annotation: Annotation::None,
                })
            }
            TokenType::For => {
                self.eat()?;
                self.excepted_token(TokenType::OpenParen)?;

                let init = if self.peek()?.get_token_type() == TokenType::Semicolon {
                    self.eat()?;
                    ForInit::Expression(None)
                } else {
                    self.parse_for_init()?
                };
                let cond = if self.peek()?.get_token_type() == TokenType::Semicolon {
                    None
                } else {
                    Some(self.parse_expression(0)?)
                };
                self.excepted_token(TokenType::Semicolon)?;
                let increment = if self.peek()?.get_token_type() == TokenType::CloseParen {
                    None
                } else {
                    Some(self.parse_expression(0)?)
                };
                self.excepted_token(TokenType::CloseParen)?;
                let body = self.parse_stmt()?;
                Ok(Stmt::For {
                    init,
                    condition: cond,
                    increment,
                    body: Box::new(body),
                    annotation: Annotation::None,
                })
            }
            _ => {
                let expr = self.parse_expression(0)?;
                self.excepted_token(TokenType::Semicolon)?;
                Ok(Stmt::Expression { expr })
            }
        }
    }
    fn parse_for_init(&mut self) -> Result<ForInit, ParserError> {
        let next_token = self.peek()?;
        match next_token.get_token_type() {
            TokenType::Int => {
                let decl = self.parse_declaration()?;
                Ok(ForInit::Declaration(decl))
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
            } else if next_token.get_token_type() == TokenType::QuestionMark {
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
