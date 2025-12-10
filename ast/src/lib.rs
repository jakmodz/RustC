pub mod ast;
pub mod decl;
mod expr;
mod parse_expr;
mod parse_stmt;
pub mod parser;
pub mod parser_error;
mod stmt;
mod var_type;
pub use decl::Declaration;
pub use expr::Expression;
pub(crate) use parse_expr::ParseExpr;
pub(crate) use parse_stmt::ParseStmt;
pub use stmt::Stmt;
pub use var_type::VarType;

#[cfg(test)]
mod tests {
    use crate::Declaration;
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

        assert_eq!(ast.declarations.len(), 1);
        
        match &ast.declarations[0] {
            Declaration::FuncDecl { decl } => {
                assert_eq!(decl.name, "main");
                assert!(decl.body.is_some());
                assert_eq!(decl.storage_class, None);
            }
            _ => panic!("Expected function declaration"),
        }
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
        
        assert_eq!(ast.declarations.len(), 1);
        
        match &ast.declarations[0] {
            Declaration::FuncDecl { decl } => {
                assert_eq!(decl.name, "main");
                assert!(decl.body.is_some());
                
                let body = decl.body.as_ref().unwrap();
                assert_eq!(body.elements.len(), 3);
            }
            _ => panic!("Expected function declaration"),
        }
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
        
        match &ast.declarations[0] {
            Declaration::FuncDecl { decl } => {
                assert_eq!(decl.name, "main");
                assert!(decl.body.is_some());
                
                let body = decl.body.as_ref().unwrap();
                assert_eq!(body.elements.len(), 3);
            }
            _ => panic!("Expected function declaration"),
        }
    }
}