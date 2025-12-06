use lex::token::TokenType;

use crate::ast::{Annotation, ForInit};
use crate::{Stmt, ast::Block, parser::Parser, parser_error::ParserError};
use crate::ParseExpr;


pub trait ParseStmt {
    fn parse_stmt(&mut self) -> Result<Stmt, ParserError>;
}


impl ParseStmt for Parser{
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

}

