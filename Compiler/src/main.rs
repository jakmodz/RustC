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
use std::path::Path;
use std::ffi::OsStr;
use semantic_analysis::SemanticError;
use semantic_analysis::SemanticAnalyzer;
#[derive(Error,Debug)]
enum CompilerError {
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Lexer Error: {0}")]
    LexerError(#[from]LexerError),
    #[error("Parser Error: {0}")]
    ParserError(#[from] ParserError),
    #[error("Semantic Error: {0}")]
    SemanticError(#[from] SemanticError),
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
    codegen: bool,
    #[arg(long)]
    tacky: bool,
    #[arg(long)]
    validate: bool,
}


fn main() {
    let mut code  = 0;
    if let Err(e) = run() {
           match e {
               CompilerError::IoError(_) => {code = 64;}
               CompilerError::LexerError(_) => {code = 65;}
               CompilerError::ParserError(_) => {code = 66;}
                CompilerError::SemanticError(_) => {code = 67;}
           }
          writeln!(stderr()," {}",e).unwrap();
       }

    exit(code);
}
fn run() -> Result<(), CompilerError> {
    let args = Args::parse();


    let mut file = File::open(&args.path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    /*
       lexing stage
     */
    let mut lex = Lexer::new();
    let tokens = lex.tokenize(content)?;
    if args.lex { return Ok(()) }

    /*
      parsing Stage
    */
    let mut parser = ast::parser::Parser::new(tokens);
    let mut ast = parser.parse()?;
    
    if args.parse { return Ok(()) }

    /*
        Semantic Validation Stage
    */
    let mut analyzer = SemanticAnalyzer::new();
    analyzer.semantic_analysis(&mut ast)?;

    if args.validate { return Ok(()) }
    /*
    Tacky Generation Stage
    */
    let mut tacky_parser = tacky::tacky_parser::TackyParser::new(analyzer.var_count);
    let tacky_program = tacky_parser.emit_tacky(ast.clone());
    
    if args.tacky { return Ok(()) }
    
    
    /*
     Assemble Stage
    */

    let mut asm_gen = asm_generator::AsmGenerator::new();
    let asm_ast = AsmParser::new().parse(tacky_program);

    if args.codegen { return Ok(()) }
    let input_path = Path::new(&args.path);
    let stem = input_path.file_stem().unwrap_or(OsStr::new("output"));
    let parent = input_path.parent().unwrap_or_else(|| Path::new("."));


    let asm_path = parent.join(format!("{}.s", stem.to_string_lossy()));


    let output_path = parent.join(stem);
    let file_out = File::create(&asm_path)?;
    let  out: Vec<Box<dyn Write>> = vec![
        Box::new(stdout()),
        Box::new(file_out),
    ];

    asm_gen.write(asm_ast, out)?;
    let status = std::process::Command::new("gcc")
        .arg("-o")
        .arg(&output_path)
        .arg(&asm_path)
        .status()
        .expect("failed to run gcc");

    if !status.success() {
        eprintln!("gcc linking failed for {:?}", asm_path);
        exit(1);
    }

    Ok(())
}

