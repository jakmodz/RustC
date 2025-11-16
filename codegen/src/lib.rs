pub mod asm_ast;
pub mod asm_generator;
pub mod ast_parser;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::AsmParser;
    use semantic_analysis::SemanticAnalyzer;
    use std::io::{Write, stdout};
    use tacky::tacky_parser;

    #[test]
    fn test_asm_gen() -> std::io::Result<()> {
        let source = String::from(
            "
            int main(void) {
            int a = 1;
            int b = !a++;
            return (a == 2 && b == 0);
        }",
        );
        let mut lexer = lex::lexer::Lexer::new();
        let tokens = lexer.tokenize(source).unwrap();
        let mut parser = ast::parser::Parser::new(tokens);
        let mut ast = parser.parse().unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.semantic_analysis(&mut ast).unwrap();
        println!("{:#?}", ast);
        let tacky = tacky_parser::TackyParser::new(analyzer.var_count).emit_tacky(ast);
        println!("{:#?}", tacky);
        let asm_ast = AsmParser::new().parse(tacky);
        let mut asm_gen = asm_generator::AsmGenerator::new();
        let out: Vec<Box<dyn Write>> = vec![
            Box::new(stdout()),
            Box::new(std::fs::File::create("output.s")?),
        ];
        asm_gen.write(asm_ast, out)?;
        Ok(())
    }
}
