use std::{
    env::args,
    fs::read_to_string,
    io::{Error, ErrorKind, Result},
    process::exit,
};

mod interpreter;
mod lexer;
mod parser;
mod util;

fn get_file_path() -> Option<String> {
    let args_: Vec<String> = args().collect();

    if args_.len() < 2 {
        None
    } else {
        Some(args_[1].clone())
    }
}

fn main() -> Result<()> {
    let Some(file_path) = get_file_path() else {
        return Err(error!(InvalidInput, "Missing filepath!"));
    };
    let content = read_to_string(file_path)?;

    let mut lex = lexer::Lexer::new(content);
    let tokens = lex.tokenize()?;

    let mut pars = parser::Parser::new(tokens);
    let parsed = pars.parse()?;

    let mut inter = interpreter::Interpreter::new(parsed);
    if let Err(err) = inter.interpret() {
        if let Some(msg) = err.into_inner() {
            println!("{}", msg);
        }
        exit(1);
    }

    Ok(())
}
