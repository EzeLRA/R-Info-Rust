use crate::lib::lexer::scanner::Lexer;
use crate::lib::lexer::token::Token;
use crate::lib::lexer::token::Keywords;
use crate::lib::optimizer::opti_bot::{Info, Optimizer};
use crate::lib::parser::processor::Parser;
use crate::lib::parser::processor::Program;
use crate::lib::semanticizer::analizer::SemanticAnalyzer;
use std::fs;

mod lib;
mod tests;

pub struct my_compiler {
    source: String,
    tokens: Vec<Token>,
    my_ast: Program,
    results: Vec<String>
}

impl my_compiler {
    fn new(source: String) -> Self {
        my_compiler {
            source,
            tokens: Vec::new(),
            my_ast: Program {
                nombre : "".to_string(),
                procesos: Vec::new(),
                areas: Vec::new(),
                robots_declarados: Vec::new(),
                robots_definidos: Vec::new(),
                robots_instanciados: Vec::new(),
                asignaciones_areas: Vec::new(),
                inicializaciones: Vec::new(),
            },
            results: Vec::new(),
        }
    }

    fn compile(&mut self) -> Vec<String> {
        let mut lx = Lexer::new(&self.source);
        match lx.tokenize() {
            Ok(tokens) => {
                self.tokens = tokens;
                let mut parser = Parser::new(&self.tokens);
                match parser.parse() {
                    Ok(ast) => {
                        self.my_ast = ast;
                        let mut analyzer = SemanticAnalyzer::new();
                        match analyzer.analizar(&self.my_ast) {
                            Ok(_) => {
                                // Capturar el string recibido y agregarlo a mi vec de string para salida
                                self.results.push("Semantic analysis completed successfully.".to_string());
                            }
                            Err(errores) => {
                                // Capturar el vec de structs de errores que se obtienen y pasarlo a string
                                let error_strings: Vec<String> = errores
                                    .iter()
                                    .map(|error| format!("{:?}", error))
                                    .collect();
                                self.results = error_strings;
                            }
                        }
                    }
                    // Capturar los structs de errores que se obtienen y pasarlo a string
                    Err(e) => {
                        let error_string = format!("Error al generar el AST: {:?}", e);
                        self.results = vec![error_string];
                    }
                }
            }
            // Capturar los structs de errores que se obtienen y pasarlo a string
            Err(e) => {
                let error_string = format!("Lexing error: {:?}", e);
                self.results = vec![error_string];
            }
        }
        self.results.clone() // Retornar una copia de los resultados
    }
}

fn main() {
    let source = fs::read_to_string("src/tests/codigo.txt")
        .expect("Failed to read source file");
    
    let mut compiler = my_compiler::new(source);
    let results = compiler.compile();
    
    // Imprimir los resultados
    for result in results {
        println!("{}", result);
    }
    
}