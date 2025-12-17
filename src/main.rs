use crate::lib::lexer::scanner::Lexer;
use crate::lib::lexer::token::Keywords;
use crate::lib::parser::processor::Parser;
use std::fs;

mod lib;
mod tests;

fn main() {
    let source = fs::read_to_string("src/tests/codigo.txt")
        .expect("Failed to read source file");
    let mut lx = Lexer::new(&source);
    match lx.tokenize() {
        Ok(tokens) => {
            //Lexer
            for token in &tokens {
                println!("{:?}", token);
            }
            //Parser
            let mut prser = Parser::new(&tokens,Keywords::new());
            match prser.parse() {
                Ok(ast) => {
                    println!("{:#?}", ast);
                }
                Err(e) => {
                    eprintln!("Parsing error: {}", e);
                }
            }
            
        }
        Err(e) => {
            eprintln!("Lexing error: {}", e);
        }
    }
}
