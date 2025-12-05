pub mod asm_ast;
pub mod asm_generator;
pub mod ast_parser;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast_parser::AsmParser;
    use semantic_analysis::SemanticAnalyzer;
    use semantic_analysis::switch_analyze;
    use semantic_analysis::switch_analyze::SwitchAnalyzer;
    use std::io::{Write, stdout};
    use tacky::tacky_parser;

    #[test]
    fn test_asm_gen() -> std::io::Result<()> {
        let source = String::from(
            "int main(void) {
    int acc = 0;
    int ctr = 0;
    for (int i = 0; i < 10; i = i + 1)  {
        // make sure break statements here break out of switch but not loop
        switch(i) {
            case 0:
                acc = 2;
                break;
            case 1:
                acc = acc * 3;
                break;
            case 2:
                acc = acc * 4;
                break;
            default:
                acc = acc + 1;
        }
        ctr = ctr + 1;
    }

    return ctr == 10 && acc == 31;
}


",
        );
        let mut lexer = lex::lexer::Lexer::new();
        let tokens = lexer.tokenize(source).unwrap();
        let mut parser = ast::parser::Parser::new(tokens);
        let mut ast = parser.parse().unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&mut ast).unwrap();

        println!("{:#?}", ast);
        let tacky = tacky_parser::TackyParser::new(analyzer.var_count).emit_tacky(ast);

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
