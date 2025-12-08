mod analyze;
mod goto_analyze;
mod map_entry;
mod semantic_error;
pub mod switch_analyze;
mod symbol_entry;
mod type_checker;

pub use analyze::SemanticAnalyzer;
pub(crate) use goto_analyze::GotoAnalyze;
pub use semantic_error::SemanticError;
#[cfg(test)]
mod tests {
    fn init(src: &str) -> Result<(), SemanticError> {
        let mut lexer = lex::lexer::Lexer::new();
        let tokens = lexer.tokenize(src.to_string()).unwrap();
        let mut parser = ast::parser::Parser::new(tokens);
        let mut ast = parser.parse().unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.variable_resolution(&mut ast)
    }

    use super::*;
    #[test]
    fn test_multiple_declaration() {
        let source = String::from(
            "
        int main(void) {
            int a = 1;
            int a = 2;
            return a;
        }",
        );
        let result = init(&source);
        assert_eq!(result.is_ok(), false);
    }
    #[test]
    fn test_undeclared() {
        let source = String::from(
            "
        int main(void) {
            int b = 1;
            return a;
        }",
        );
        let result = init(&source);
        assert_eq!(result.is_ok(), false);
    }
    #[test]
    fn test_wrong_left() {
        let source = String::from(
            "
        int main(void) {
            int b = 1;
            2 = b;
            return a;
        }",
        );
        let result = init(&source);
        assert_eq!(result.is_ok(), false);
    }
}
