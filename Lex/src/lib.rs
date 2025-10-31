pub mod lexer;
pub mod token;
pub mod error;

#[cfg(test)]
mod tests {
    use crate::lexer::Lexer;
    use super::*;
    fn test(tokens:Vec<token::Token>, expected:Vec<String>) {
        for (i,token) in tokens.iter().enumerate() {
            assert_eq!(token.to_string(),expected[i]);
        }
    }
    #[test]
    fn lexer_setup() -> Result<(),error::LexerError> {

        Ok(())
    }
    #[test]
    fn main_fn_lex_test() -> Result<(),error::LexerError> {
        let mut lex = Lexer::new();
        //let tokens = lex.tokenize("int main(){return 42;}".to_string())?;
        let tokens = lex.tokenize("int main(void) \
         //daddadad
         {return 100;}".to_string())?;
        test(tokens, vec![
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
        ]);
        Ok(())
    }

}
