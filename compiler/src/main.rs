use ast::parser_error::ParserError;
use clap::Parser;
use codegen::asm_generator;
use codegen::ast_parser::AsmParser;
use codegen::fixup::InstructionFixup;
use codegen::pseudoreg_replacement::PseudoregReplacement;
use lex::error::LexerError;
use lex::lexer::Lexer;
use semantic_analysis::SemanticAnalyzer;
use semantic_analysis::SemanticError;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, exit};
use thiserror::Error;

#[derive(Error, Debug)]
enum CompilerError {
    #[error("IO Error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Lexer Error: {0}")]
    Lexer(#[from] LexerError),
    #[error("Parser Error: {0}")]
    Parser(#[from] ParserError),
    #[error("Semantic Error: {0}")]
    Semantic(#[from] SemanticError),
}

#[derive(clap::Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short)]
    c: bool,
    paths: Vec<String>,
    #[arg(long)]
    lex: bool,
    #[arg(long)]
    parse: bool,
    #[arg(long)]
    validate: bool,
    #[arg(long)]
    tacky: bool,
    #[arg(long)]
    codegen: bool,
}

fn main() {
    let mut code = 0;
    if let Err(e) = run() {
        match e {
            CompilerError::Io(_) => {
                code = 64;
            }
            CompilerError::Lexer(_) => {
                code = 65;
            }
            CompilerError::Parser(_) => {
                code = 66;
            }
            CompilerError::Semantic(_) => {
                code = 67;
            }
        }
        eprintln!("{}", e)
    }
    exit(code);
}

fn run() -> Result<(), CompilerError> {
    let args = Args::parse();
    let mut asm_files = Vec::new();

    for path in args.paths.iter() {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        let mut lex = Lexer::new();
        let tokens = lex.tokenize(content)?;
        if args.lex {
            continue;
        }

        let mut parser = ast::parser::Parser::new(tokens);
        let mut ast = parser.parse()?;
        if args.parse {
            continue;
        }
        let mut analyzer = SemanticAnalyzer::new();
        analyzer.analyze(&mut ast)?;
        if args.validate {
            continue;
        }

        let mut tacky_parser = tacky::tacky_parser::TackyParser::new(analyzer.var_count);
        let tacky_program = tacky_parser.emit_tacky(ast);
        if args.tacky {
            continue;
        }

        let asm_with_pseudos = AsmParser::new().parse(tacky_program);
        let mut replacer = PseudoregReplacement::new();
        let (asm_with_stack, stack_sizes) = replacer.replace(asm_with_pseudos);
        let fixup = InstructionFixup::new(stack_sizes);
        let final_asm = fixup.fixup(asm_with_stack);
        if args.codegen {
            continue;
        }

        let input_path = Path::new(path);
        let stem = input_path.file_stem().unwrap_or(OsStr::new("output"));
        let parent = input_path.parent().unwrap_or_else(|| Path::new("."));
        let asm_path = parent.join(format!("{}.s", stem.to_string_lossy()));

        let file_out = File::create(&asm_path)?;
        let out: Vec<Box<dyn Write>> = vec![Box::new(file_out)];
        let mut asm_gen = asm_generator::AsmGenerator::new();
        asm_gen.write(final_asm, out)?;

        asm_files.push(asm_path);
    }

    if args.lex || args.parse || args.validate || args.tacky || args.codegen {
        return Ok(());
    }

    if !asm_files.is_empty() {
        let mut command = Command::new("gcc");

        if args.c {
            for asm_file in &asm_files {
                let stem = asm_file.file_stem().unwrap();
                let parent = asm_file.parent().unwrap_or_else(|| Path::new("."));
                let obj_file = parent.join(format!("{}.o", stem.to_string_lossy()));

                let status = Command::new("gcc")
                    .arg("-c")
                    .arg(asm_file)
                    .arg("-o")
                    .arg(&obj_file)
                    .status()
                    .expect("failed to run gcc");

                if !status.success() {
                    eprintln!("gcc compilation failed: {}", status);
                    exit(1);
                }
            }
        } else {
            let first_path = Path::new(&args.paths[0]);
            let stem = first_path.file_stem().unwrap_or(OsStr::new("a.out"));
            let parent = first_path.parent().unwrap_or_else(|| Path::new("."));
            let output_path = parent.join(stem);

            command.arg("-o").arg(&output_path);
            for asm_file in &asm_files {
                command.arg(asm_file);
            }

            let status = command.status().expect("failed to run gcc");
            if !status.success() {
                eprintln!("gcc linking failed: {}", status);
                exit(1);
            }
        }
    }

    Ok(())
}
