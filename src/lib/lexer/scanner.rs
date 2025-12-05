use std::collections::HashMap;

use super::token::{Token, TokenType, Keywords};
use crate::lib::compilerError::{CompilerError};

pub struct Lexer<'a> {
    source: &'a str,
    chars: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
    tokens: Vec<Token>,
    indent_stack: Vec<usize>,
    at_line_start: bool,
    current_indent: usize,
    keywords: Keywords,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let chars: Vec<char> = source.chars().collect();
        
        Self {
            source,
            chars,
            position: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            indent_stack: vec![0],
            at_line_start: true,
            current_indent: 0,
            keywords: Keywords::new(),
        }
    }
    
    pub fn with_keywords(source: &'a str, keywords: Keywords) -> Self {
        let chars: Vec<char> = source.chars().collect();
        
        Self {
            source,
            chars,
            position: 0,
            line: 1,
            column: 1,
            tokens: Vec::new(),
            indent_stack: vec![0],
            at_line_start: true,
            current_indent: 0,
            keywords,
        }
    }
    
    pub fn tokenize(&mut self) -> Result<Vec<Token>, CompilerError> {
        self.tokens.clear();
        self.position = 0;
        self.line = 1;
        self.column = 1;
        self.at_line_start = true;
        self.indent_stack = vec![0];
        self.current_indent = 0;
        
        while self.position < self.chars.len() {
            let char = self.chars[self.position];
            
            match char {
                // Comentarios
                '{' => self.read_comment()?,
                
                // Parámetros
                '(' => self.read_parameter()?,
                
                // Nueva línea
                '\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.position += 1;
                    self.at_line_start = true;
                }
                
                // Espacios en blanco
                c if c.is_whitespace() && c != '\n' => {
                    if self.at_line_start {
                        self.handle_indentation()?;
                    } else {
                        self.skip_whitespace();
                    }
                }
                
                // Dígitos
                c if c.is_ascii_digit() => self.read_number()?,
                
                // Letras (identificadores)
                c if c.is_alphabetic() || c == '_' => self.read_identifier()?,
                
                // Strings
                '"' | '\'' => self.read_string(char)?,
                
                // Operadores
                c if self.is_operator(c) || c == ',' || c == ':' => self.read_operator()?,
                
                // Carácter inesperado
                _ => {
                    return Err(CompilerError::new(
                        format!("Carácter inesperado: < {} >", char),
                        self.line,
                        self.column
                    ));
                }
            }
        }
        
        // Añadir tokens DEDENT finales
        while self.indent_stack.len() > 1 {
            self.tokens.push(Token::new(
                TokenType::Dedent,
                "",
                self.line,
                1
            ));
            self.indent_stack.pop();
        }
        
        // Añadir token de fin de archivo
        self.tokens.push(Token::new(
            TokenType::EndFile,
            "",
            self.line,
            self.column
        ));
        
        Ok(self.tokens.clone())
    }
    
    fn handle_indentation(&mut self) -> Result<(), CompilerError> {
        let start_pos = self.position;
        let mut indent = 0;
        
        // Contar espacios y tabs
        while self.position < self.chars.len() {
            match self.chars[self.position] {
                ' ' => {
                    indent += 1;
                    self.position += 1;
                    self.column += 1;
                }
                '\t' => {
                    indent += 4; // Tabs como 4 espacios
                    self.position += 1;
                    self.column += 1;
                }
                _ => break,
            }
        }
        
        // Si la línea está vacía o es solo espacios/tabs
        if self.position >= self.chars.len() || self.chars[self.position] == '\n' {
            self.at_line_start = false;
            return Ok(());
        }
        
        let last_indent = *self.indent_stack.last().unwrap();
        
        // Nueva indentación mayor
        if indent > last_indent {
            self.tokens.push(Token::new(
                TokenType::Indent,
                "",
                self.line,
                1
            ));
            self.indent_stack.push(indent);
        }
        // Nueva indentación menor
        else if indent < last_indent {
            // Encontrar el nivel de indentación correspondiente
            while let Some(&stack_indent) = self.indent_stack.last() {
                if stack_indent <= indent {
                    if stack_indent < indent {
                        return Err(CompilerError::new(
                            "Indentación inconsistente",
                            self.line,
                            1
                        ));
                    }
                    break;
                }
                
                self.tokens.push(Token::new(
                    TokenType::Dedent,
                    "",
                    self.line,
                    1
                ));
                self.indent_stack.pop();
            }
        }
        
        self.at_line_start = false;
        self.current_indent = indent;
        
        Ok(())
    }
    
    fn is_operator(&self, c: char) -> bool {
        matches!(c, '+' | '-' | '*' | '/' | '=' | '<' | '>' | '!' | '&' | '|' | '~')
    }
    
    fn skip_whitespace(&mut self) {
        while self.position < self.chars.len() && 
            self.chars[self.position].is_whitespace() &&
            self.chars[self.position] != '\n' {
            self.position += 1;
            self.column += 1;
        }
    }
    
    fn read_number(&mut self) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.position;
        
        // Leer parte entera
        while self.position < self.chars.len() && self.chars[self.position].is_ascii_digit() {
            self.position += 1;
            self.column += 1;
        }
        
        let value: String = self.chars[start_pos..self.position].iter().collect();
        
        self.tokens.push(Token::new(
            TokenType::Num,
            value,
            start_line,
            start_column
        ));
        
        Ok(())
    }
    
    fn read_identifier(&mut self) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        let start_pos = self.position;
        
        while self.position < self.chars.len() {
            let c = self.chars[self.position];
            if c.is_alphanumeric() || c == '_' {
                self.position += 1;
                self.column += 1;
            } else {
                break;
            }
        }
        
        let value: String = self.chars[start_pos..self.position].iter().collect();
        
        // Determinar el tipo de token
        let token_type = self.determine_identifier_type(&value);
        
        self.tokens.push(Token::new(
            token_type,
            value.clone(),
            start_line,
            start_column
        ));
        
        Ok(())
    }
    
    fn determine_identifier_type(&self, value: &str) -> TokenType {
        // Primero verificar en keyword_map
        if let Some(&token_type) = self.keywords.keyword_map.get(value) {
            return token_type;
        }
        
        // Luego verificar en types_defined
        if let Some(&token_type) = self.keywords.types_defined.get(value) {
            return token_type;
        }
        
        // Verificar si es un valor booleano literal
        if self.is_boolean_literal(value) {
            return TokenType::Bool;
        }
        
        // Por defecto, es un identificador
        TokenType::Identifier
    }
    
    fn is_boolean_literal(&self, value: &str) -> bool {
        matches!(
            value.to_lowercase().as_str(),
            "true" | "false" | "verdadero" | "falso" | "v" | "f"
        )
    }
    
    fn read_string(&mut self, quote: char) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        
        self.position += 1; // Saltar comilla inicial
        self.column += 1;
        
        let start_pos = self.position;
        let mut value = String::new();
        
        while self.position < self.chars.len() && self.chars[self.position] != quote {
            let c = self.chars[self.position];
            
            // Manejar secuencias de escape
            if c == '\\' {
                self.position += 1;
                self.column += 1;
                
                if self.position >= self.chars.len() {
                    return Err(CompilerError::new(
                        "Secuencia de escape incompleta",
                        self.line,
                        self.column
                    ));
                }
                
                let escaped = match self.chars[self.position] {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    '\\' => '\\',
                    '\'' => '\'',
                    '"' => '"',
                    _ => return Err(CompilerError::new(
                        format!("Secuencia de escape desconocida: \\{}", self.chars[self.position]),
                        self.line,
                        self.column
                    )),
                };
                
                value.push(escaped);
            } else {
                value.push(c);
            }
            
            self.position += 1;
            self.column += 1;
        }
        
        if self.position >= self.chars.len() {
            return Err(CompilerError::new(
                "Cadena sin cerrar",
                start_line,
                start_column
            ));
        }
        
        self.position += 1; // Saltar comilla final
        self.column += 1;
        
        self.tokens.push(Token::new(
            TokenType::Str, 
            value,
            start_line,
            start_column
        ));
        
        Ok(())
    }
    
    fn read_operator(&mut self) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        let first_char = self.chars[self.position];
        
        // Operadores de un carácter específicos
        if first_char == ',' || first_char == ':' {
            self.position += 1;
            self.column += 1;
            
            let token_type = match first_char {
                ',' => TokenType::Comma,
                ':' => TokenType::Colon,
                _ => unreachable!(),
            };
            
            self.tokens.push(Token::new(
                token_type,
                first_char.to_string(),
                start_line,
                start_column
            ));
            
            return Ok(());
        }
        
        // Operadores de uno o dos caracteres
        let mut value = String::from(first_char);
        self.position += 1;
        self.column += 1;
        
        // Verificar si es un operador de dos caracteres
        if self.position < self.chars.len() {
            let second_char = self.chars[self.position];
            let two_char_op = format!("{}{}", first_char, second_char);
            
            match two_char_op.as_str() {
                ":=" | "==" | "<=" | ">=" => {
                    value = two_char_op;
                    self.position += 1;
                    self.column += 1;
                }
                _ => {}
            }
        }
        
        // Determinar el tipo específico del operador
        let token_type = match value.as_str() {
            ":=" => TokenType::Assign,
            "==" => TokenType::Equals,
            "<=" => TokenType::LessEqual,
            ">=" => TokenType::GreaterEqual,
            "&" => TokenType::And,
            "|" => TokenType::Or,
            "~" => TokenType::Not,
            _ => TokenType::Operator,
        };
        
        self.tokens.push(Token::new(
            token_type,
            value,
            start_line,
            start_column
        ));
        
        Ok(())
    }
    
    fn read_comment(&mut self) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        
        self.position += 1; // Saltar '{'
        self.column += 1;
        
        while self.position < self.chars.len() && self.chars[self.position] != '}' {
            if self.chars[self.position] == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.position += 1;
        }
        
        if self.position >= self.chars.len() {
            return Err(CompilerError::new(
                "Comentario sin cerrar",
                start_line,
                start_column
            ));
        }
        
        self.position += 1; // Saltar '}'
        self.column += 1;
        
        Ok(())
    }
    
    fn read_parameter(&mut self) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        
        self.position += 1; // Saltar '('
        self.column += 1;
        
        let mut value = String::new();
        
        while self.position < self.chars.len() && self.chars[self.position] != ')' {
            let c = self.chars[self.position];
            value.push(c);
            
            if c == '\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            
            self.position += 1;
        }
        
        if self.position >= self.chars.len() {
            return Err(CompilerError::new(
                "Parámetro sin cerrar",
                start_line,
                start_column
            ));
        }
        
        self.position += 1; // Saltar ')'
        self.column += 1;
        
        self.tokens.push(Token::new(
            TokenType::Parameter,
            value,
            start_line,
            start_column
        ));
        
        Ok(())
    }
    
    // Método de utilidad para depuración
    pub fn debug_tokens(&self) {
        println!("=== Tokens generados ===");
        for token in &self.tokens {
            println!("{:20} '{}' (línea {}, columna {})",
                token.token_type.as_str(),
                token.value,
                token.line,
                token.column
            );
        }
    }
    
    // Método para obtener estadísticas
    pub fn get_statistics(&self) -> HashMap<TokenType, usize> {
        let mut stats = HashMap::new();
        
        for token in &self.tokens {
            *stats.entry(token.token_type).or_insert(0) += 1;
        }
        
        stats
    }
}