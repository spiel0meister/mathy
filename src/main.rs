use std::{
    env::args,
    fs::read_to_string,
    io::{Error, ErrorKind, Result},
    process::exit,
};

use mathy::{interpreter::Interpreter, lexer::Lexer, parser::Parser, util::error};

fn main() -> Result<()> {
    let args_: Vec<String> = args().collect();

    if args_.len() < 2 {
        return Err(error!(InvalidInput, "Missing filepath!"));
    }

    let file_path = &args_[1];
    let content = read_to_string(&file_path)?;

    let mut lexer = Lexer::new(file_path.to_string(), content);
    let out = lexer.tokenize();
    if let Err(err) = out {
        if let Some(msg) = err.into_inner() {
            eprintln!("{}", msg);
        }
        exit(1);
    }

    let out = out.unwrap();
    let mut parser = Parser::new(out);
    let out = parser.parse();
    if let Err(err) = out {
        if let Some(msg) = err.into_inner() {
            eprintln!("{}", msg);
        }
        exit(1);
    }

    let mut interpreter = Interpreter::new(out.unwrap());
    if let Err(err) = interpreter.interpret() {
        if let Some(msg) = err.into_inner() {
            eprintln!("{}", msg);
        }
        exit(1);
    }

    Ok(())
}
