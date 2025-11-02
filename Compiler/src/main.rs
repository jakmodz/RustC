use std::fs::File;
use std::io::{stderr, stdout, Read, Write};
use std::process::exit;
use clap::Parser;
use lex::lexer::Lexer;
use thiserror::Error;
use ast::parser_error::ParserError;
use codegen::asm_generator;
use lex::error::LexerError;
use codegen::ast_parser::AsmParser;
use codegen::asm_generator::*;
use std::path::Path;
use std::ffi::OsStr;
#[derive(Error,Debug)]
enum CompilerError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lexer Error: {0}")]
    LexerError(#[from]LexerError),
    #[error("Parser Error: {0}")]
    ParserError(#[from] ParserError)
}


#[derive(clap::Parser,Debug)]
#[command(version,about,long_about = None)]
struct Args{
    //path to file
    path:String,

    //to lexer stage
    #[arg(long)]
    lex: bool,
    //to parsing stage
    #[arg(long)]
    parse: bool,
    //to codegen stage
    #[arg(long)]
    codegen: bool
}


fn main() {
    let mut code  = 0;
    if let Err(e) = run() {
           match e {
               CompilerError::IoError(_) => {code = 64;}
               CompilerError::LexerError(_) => {code = 65;}
               CompilerError::ParserError(_) => {code = 66;}
           }
          writeln!(stderr()," {}",e).unwrap();
       }

    exit(code);
}
fn run() -> Result<(), CompilerError> {
    let args = Args::parse();

    // --- 1. Read input file ---
    let mut file = File::open(&args.path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    // --- 2. Lexing + parsing ---
    let mut lex = Lexer::new();
    let tokens = lex.tokenize(content)?;
    let mut parser = ast::parser::Parser::new(tokens);
    let ast = parser.parse()?;
    let asm_ast = AsmParser::new().parse(ast);

    // --- 3. Generate assembly ---
    let input_path = Path::new(&args.path);
    let stem = input_path.file_stem().unwrap_or(OsStr::new("output"));
    let parent = input_path.parent().unwrap_or_else(|| Path::new("."));

    // tmp assembly path, e.g. tests/chapter_1/valid/return_2.s
    let asm_path = parent.join(format!("{}.s", stem.to_string_lossy()));

    // output executable path, e.g. tests/chapter_1/valid/return_2
    let output_path = parent.join(stem);

    let file_out = File::create(&asm_path)?;
    let mut out: Vec<Box<dyn Write>> = vec![
        Box::new(stdout()),
        Box::new(file_out),
    ];

    let mut asm_gen = asm_generator::AsmGenerator::new();
    asm_gen.write(asm_ast, out)?;

    // --- 4. Assemble + link into executable ---
    let status = std::process::Command::new("gcc")
        .arg("-o")
        .arg(&output_path)
        .arg(&asm_path)
        .status()
        .expect("failed to run gcc");

    if !status.success() {
        eprintln!("gcc linking failed for {:?}", asm_path);
        std::process::exit(1);
    }

    Ok(())
}

