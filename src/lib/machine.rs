use crate::lib::lexer::scanner::Lexer;
use crate::lib::parser::processor::Parser;
use crate::lib::semanticizer::analizer::SemanticAnalyzer;
use crate::lib::optimizer::opti_bot::Optimizer;


pub struct Machine {
    source : String,
}

impl Machine {
    pub fn new(source: String) -> Self {
        Self { source }
    }

    pub fn compile(&mut self) -> Vec<String> {
        // Aquí iría la lógica de compilación, pero por ahora solo devuelve un mensaje de ejemplo
        vec![format!("Compilando el código fuente:\n{}", self.source)]
    }

    pub fn print_compilation(&self){
        let mut lx = Lexer::new(&self.source);
        
        match lx.tokenize() {
            Ok(tokens) => {
                tokens.iter().for_each(|token| println!("{:?}", token));
                println!("");
                let mut parser = Parser::new(&tokens);
                match parser.parse() {
                    Ok(ast) => {
                        println!("AST: {:?}", ast);
                        println!("");
                        let mut analyzer = SemanticAnalyzer::new();
                        match analyzer.analizar(&ast) {
                            Ok(_) => {
                                let mut optimizer = Optimizer::new();
                                optimizer.process(&ast);
                                println!("Optimización completada.");
                                
                            }
                            Err(errores) => {
                                println!("Errores semánticos: {:?}", errores);
                                let error_strings: Vec<String> = errores
                                    .iter()
                                    .map(|error| format!("{:?}", error))
                                    .collect();
                                
                            }
                        }
                    }
                    Err(e) => {
                        println!("Error al generar el AST: {}", e);
                        let error_string = format!("Error al generar el AST: {:?}", e);
                       
                    }
                }
            }
            Err(e) => {
                println!("Lexing error: {}", e);
                let error_string = format!("Lexing error: {:?}", e);
                
            }
        }
        
    }

}