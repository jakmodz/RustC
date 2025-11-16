pub mod ast;
pub mod parser;
pub mod parser_error;

#[cfg(test)]
mod tests {
    use crate::ast::*;

    #[test]
    fn ast_parser_test() {
        let source = String::from(
            "int main(void) {
                return 42;
            }",
        );
        let mut lexer = lex::lexer::Lexer::new();
        let tokens = lexer.tokenize(source).unwrap();
        let mut parser = crate::parser::Parser::new(tokens);
        let ast = parser.parse().unwrap();
        println!("{:#?}", ast);
    }
    #[test]
    fn parser_test() {
        let source = String::from(
            "int main(void) {
    int a = 1;
    int b = 2;
    return a = b = 4;
}",
        );
        let mut lexer = lex::lexer::Lexer::new();
        let tokens = lexer.tokenize(source).unwrap();
        let mut parser = crate::parser::Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.function.name, String::from("main"));
        assert_eq!(ast.function.body.len(), 3);
    }
}
