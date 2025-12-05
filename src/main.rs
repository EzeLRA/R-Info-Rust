use crate::lib::lexer::scanner::Lexer;
use std::fs;

mod lib;
mod tests;

fn main() {
    let source_code = fs::read_to_string("./src/codigo.txt");
    print!("{}", source_code.unwrap());
    /* 
    println!("\nArchivos en el directorio:");
    if let Ok(entries) = fs::read_dir("./src") {
        for entry in entries {
            if let Ok(entry) = entry {
                println!("  {:?}", entry.path());
            }
        }
    }*/
}
