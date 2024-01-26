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

fn main() -> Result<()> {
    let args_: Vec<String> = args().collect();

    if args_.len() < 2 {
        return Err(error!(InvalidInput, "Missing filepath!"));
    }

    let file_path = &args_[1];
    let content = read_to_string(&file_path)?;

    let mut lex = lexer::Lexer::new(file_path.to_string(), content);
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
