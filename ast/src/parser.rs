use crate::ast::*;
use crate::parser_error::ParserError;
use lex::token;
use lex::token::{Token, TokenType};


pub struct Parser{
    tokens: Vec<token::Token>,
    pos:usize,
}

impl Parser{
    pub fn new(tokens:Vec<token::Token>)->Self{
        Parser{
            tokens,
            pos:0
        }
    }


    fn excepted_token(&mut self,expected:TokenType)->Result<(),ParserError>{
        if self.pos >= self.tokens.len(){
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        if current_token.get_token_type() != expected{
            return Err(ParserError::ExpectedToken(current_token.span().line, current_token.span().column,
                format!("{:?}",expected), format!("{:?}",current_token.get_token_type())));
        }
        self.pos+=1;
        Ok(())
    }
    fn except_token_optional(&mut self,expected:TokenType) ->Result<(),ParserError>{
        let res = self.excepted_token(expected);
        if res.is_err() {
            return res;
        }
        Ok(())
    }
   
    fn peek(&mut self,)->Result<Token,ParserError>{
        if self.pos >= self.tokens.len(){
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        Ok(current_token.clone())
    }
    fn eat(&mut self)->Result<Token,ParserError>{
        if self.pos >= self.tokens.len(){
            return Err(ParserError::UnexpectedEOF);
        }
        let current_token = &self.tokens[self.pos];
        self.pos+=1;
        Ok(current_token.clone())
    }

    pub fn parse(&mut self,)->Result<Program,ParserError>{
        self.excepted_token(TokenType::Int)?;
        self.excepted_token(TokenType::Identifier)?;
        self.excepted_token(TokenType::OpenParen)?;
        self.except_token_optional(TokenType::Void)?;
        self.excepted_token(TokenType::CloseParen)?;
        self.excepted_token(TokenType::OpenBrace)?;
        let body = self.parse_stmt()?;
        self.excepted_token(TokenType::CloseBrace)?;

        if self.pos < self.tokens.len(){
            return Err(ParserError::UnexpectedToken(
                self.tokens[self.pos].span().line,
                self.tokens[self.pos].span().column,
                format!("{:?}",self.tokens[self.pos].get_token_type())
            ));
        }
        Ok(Program{function:Function{
            name:"main".to_string(),
            body:vec![body]
        }})
    }


    /*
    Parse Statement
     */
    fn parse_stmt(&mut self)->Result<Stmt,ParserError>{
        self.excepted_token(TokenType::Return)?;
        let next_token = self.peek()?;
        let return_val = self.parse_expression(next_token.get_precedence())?;
        self.excepted_token(TokenType::Semicolon)?;
        Ok(Stmt::Return{expr:return_val})
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
        while next_token.is_binary_op() && next_token.get_precedence() >= min_precedence {
            let operator = self.eat()?;
            let right = self.parse_expression(next_token.get_precedence() + 1)?;
            left = Expression::Binary { op: operator, left: Box::new(left), right: Box::new(right) };
            next_token = self.peek()?
        }

        Ok(left)
    }
    fn parse_factor(&mut self) -> Result<Expression, ParserError> {
        if self.pos >= self.tokens.len() {
            return Err(ParserError::UnexpectedEOF);
        }
        let next_token =self.peek()?;
        match next_token {
            Token::Constant(value, _span)=>{
                self.eat()?;
                Ok(Expression::Constant(value))
            },
            Token::OpenParen(_) => {
                self.eat()?;
                let expr = self.parse_expression(next_token.get_precedence())?;
                self.excepted_token(TokenType::CloseParen)?;
                Ok(Expression::Grouping{
                    expr:Box::new(expr)
                })
            }
            Token::Hypen(_)| Token::Tilde(_)=>{
                let op = self.eat()?;
                let expr = self.parse_factor()?;
                Ok(Expression::UnaryOP{
                    op,
                    expr:Box::new(expr)
                })
            }
            _=>{
                let current_token = self.peek()?;
                Err(ParserError::UnexpectedToken(current_token.span().line,current_token.span().column,
                                                 format!("{:?}", current_token.get_token_type())))
            }
        }
    }
    

}


