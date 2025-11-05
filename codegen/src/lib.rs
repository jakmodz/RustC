pub mod asm_ast;
pub mod ast_parser;
pub mod asm_generator;

#[cfg(test)]
mod tests {
    use std::io::{stdout, Write};
    use crate::ast_parser::AsmParser;
    use super::*;
    use tacky::tacky_parser;

    #[test]
    fn test_asm_gen() -> std::io::Result<()> {
        let source = String::from(
            "int main(void) {
    return -(-4);
}"
        );
        let mut lexer = lex::lexer::Lexer::new();
        let tokens = lexer.tokenize(source).unwrap();
        let mut parser = ast::parser::Parser::new(tokens);
        let ast = parser.parse().unwrap();
        let tacky = tacky_parser::TackyParser::new().emit_tacky(ast);
        let asm_ast = AsmParser::new().parse(tacky);
        let mut asm_gen = asm_generator::AsmGenerator::new();
        let  out:Vec<Box<dyn Write>>    = vec![Box::new(stdout()),Box::new(std::fs::File::create("output.s")?)];
        asm_gen.write(asm_ast,out)?;
        Ok(())
    }

}
