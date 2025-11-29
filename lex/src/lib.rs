pub mod error;
pub mod lexer;
mod span;
pub mod token;
use crate::span::Span;
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::LexerError;
    use crate::lexer::Lexer;
    fn test(tokens: Vec<token::Token>, expected: Vec<String>) {
        for (i, token) in tokens.iter().enumerate() {
            assert_eq!(token.to_string(), expected[i]);
        }
    }
    #[test]
    fn lexer_setup() -> Result<(), error::LexerError> {
        Ok(())
    }
    #[test]
    fn main_fn_lex_test() -> Result<(), error::LexerError> {
        let mut lex = Lexer::new();
        let tokens = lex.tokenize(
            "int main(void) \
         //daddadad
         {return 100;}"
                .to_string(),
        )?;
        test(
            tokens,
            vec![
                "int".to_string(),
                "main".to_string(),
                "(".to_string(),
                "void".to_string(),
                ")".to_string(),
                "{".to_string(),
                "return".to_string(),
                "100".to_string(),
                ";".to_string(),
                "}".to_string(),
            ],
        );
        Ok(())
    }
    #[test]
    fn span_test() -> Result<(), error::LexerError> {
        let mut lex = Lexer::new();

        let tokens = lex.tokenize("int \nmain(void)".to_string())?;

        assert_eq!(tokens[0].span().line, 1);
        assert_eq!(tokens[1].span().column, 0);
        assert_eq!(tokens[2].span().column, 4);
        assert_eq!(tokens[3].span().column, 5);
        Ok(())
    }
    #[test]
    fn span_error_test() -> Result<(), error::LexerError> {
        let mut lex = Lexer::new();

        let tokens = lex.tokenize("void?".to_string());
        match tokens {
            Err(e) => match e {
                LexerError::InvalidToken(line, col, _) => {
                    assert_eq!(line, 1);
                    assert_eq!(col, col)
                }
                _ => {}
            },
            _ => {}
        }

        Ok(())
    }
    #[test]
    fn multi_comment_test() -> Result<(), error::LexerError> {
        let mut lex = Lexer::new();

        let tokens = lex.tokenize("void/* */".to_string())?;

        test(
            tokens,
            vec![
                "void".to_string(),
                "*".to_string(),
                "{".to_string(),
                "}".to_string(),
            ],
        );
        Ok(())
    }
    #[test]
    fn comment_test() -> Result<(), error::LexerError> {
        let mut lex = Lexer::new();

        let tokens = lex.tokenize(
            "void//AAA\n \
        }"
            .to_string(),
        )?;

        test(tokens, vec!["void".to_string(), "}".to_string()]);
        Ok(())
    }
    #[test]
    fn unary_tokens_test() -> Result<(), LexerError> {
        let mut lex = Lexer::new();
        let tokens = lex.tokenize("~ -- -".to_string())?;
        test(
            tokens,
            vec!["~".to_string(), "--".to_string(), "-".to_string()],
        );
        let tokens = lex.tokenize("--2".to_string())?;
        test(tokens, vec!["--".to_string(), "2".to_string()]);
        let tokens = lex.tokenize("~~2".to_string())?;
        test(
            tokens,
            vec!["~".to_string(), "~".to_string(), "2".to_string()],
        );

        Ok(())
    }
    #[test]
    fn goto_tokens_test() -> Result<(), LexerError> {
        let mut lex = Lexer::new();
        let tokens = lex.tokenize("goto".to_string())?;
        test(tokens, vec!["goto".to_string()]);

        Ok(())
    }
}
