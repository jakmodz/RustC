pub mod ast;
mod decl;
mod expr;
pub mod parser;
pub mod parser_error;
mod stmt;
pub use decl::Declaration;
pub use expr::Expression;
pub use stmt::Stmt;

#[cfg(test)]
mod tests {

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
        assert_eq!(ast.function.body.elements.len(), 3);
    }
    #[test]
    fn goto_label() {
        let source = String::from(
            "int main(void) {
 goto label1;
label1:
    return 0;
}",
        );
        let mut lexer = lex::lexer::Lexer::new();
        let tokens = lexer.tokenize(source).unwrap();
        let mut parser = crate::parser::Parser::new(tokens);
        let ast = parser.parse().unwrap();
        assert_eq!(ast.function.name, String::from("main"));
        assert_eq!(ast.function.body.elements.len(), 3);
    }
}
