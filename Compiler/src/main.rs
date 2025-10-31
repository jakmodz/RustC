use std::fs::File;
use std::io::{stderr, Read};
use std::process::exit;
use clap::Parser;
use Lex::lexer::Lexer;

#[derive(Parser,Debug)]
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

    let args = Args::parse();

    let file = File::open(&args.path);
    if file.is_err() {
        eprintln!("file: {} cannot be opened.",& args.path);
        exit(64);
    }
    let mut content = String::new();

    file.unwrap().read_to_string(&mut content);

    let mut lex = Lexer::new();
    let res = lex.tokenize(content);
    if res.is_err(){
        eprintln!("Lexer Error: {}",res.err().unwrap());
        exit(65);
    }
    exit(0);
}
