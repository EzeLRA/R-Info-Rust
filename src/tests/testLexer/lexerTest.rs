use crate::Lexer;
use crate::lib::lexer::token::TokenType;
use std::fs;

#[cfg(test)]
mod testing_lexer{
    use crate::lib::lexer;

    use super::*;

    #[test]
    fn test_basic_keywords() {
        let source = fs::read_to_string("./src./tests/codigo.txt");
        match source {
            Ok(content) => {
                let mut lexer = Lexer::new(&content);
                
                let keywords = [
                    "proceso", "robot", "variables", "comenzar", 
                    "fin", "programa", "procesos", "areas", "robots"
                ];

                if let Ok(tokens) = lexer.tokenize(){
                    let keyword_tokens: Vec<_> = tokens
                    .iter()
                    .filter(|token| token.token_type == TokenType::Keyword)
                    .collect();

                    for token in keyword_tokens.iter(){
                        
                        assert!(keywords.contains(&token.value.as_str()), "Unexpected keyword: {}", token.value);
                    }    
                }else{
                    panic!("Failed to tokenize source code");
                }
                
            }
            Err(e) => {
                panic!("Failed to read source file: {}", e);
            }
        }
        
    }

    #[test]
    fn test_control_sentences() {
        let source = fs::read_to_string("./src./tests/codigo.txt");
        match source {
            Ok(content) => {
                let mut lexer = Lexer::new(&content);
                
                let control_sentences = [
                    "si", "sino", "mientras", "repetir"
                ];

                if let Ok(tokens) = lexer.tokenize(){
                    let control_sentence_tokens: Vec<_> = tokens
                    .iter()
                    .filter(|token| token.token_type == TokenType::ControlSentence)
                    .collect();

                    for token in control_sentence_tokens.iter(){
                        
                        assert!(control_sentences.contains(&token.value.as_str()), "Unexpected control sentence: {}", token.value);
                    }    
                }else{
                    panic!("Failed to tokenize source code");
                }
                
            }
            Err(e) => {
                panic!("Failed to read source file: {}", e);
            }
        }
        
    }

}