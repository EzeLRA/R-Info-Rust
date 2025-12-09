use crate::lib::lexer::scanner::Lexer;
use std::fs;

mod lib;
mod tests;

fn main() {
    let source = fs::read_to_string("src/tests/codigo.txt")
        .expect("Failed to read source file");
    let mut lx = Lexer::new(&source);
    match lx.tokenize() {
        Ok(tokens) => {
            for token in tokens {
                println!("{:?}", token);
            }
        }
        Err(e) => {
            eprintln!("Lexing error: {}", e);
        }
    }
}
