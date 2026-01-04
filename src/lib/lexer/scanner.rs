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
    paren_stack: Vec<(char, usize, usize)>, // (tipo de paréntesis, línea, columna)
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
            paren_stack: Vec::new(),
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
            paren_stack: Vec::new(),
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
        self.paren_stack.clear();
        
        while self.position < self.chars.len() {
            let char = self.chars[self.position];
            
            match char {
                // Comentarios
                '{' => {
                    self.read_comment()?;
                    continue;
                }
                
                // Paréntesis que abre
                '(' => self.handle_open_parenthesis()?,
                
                // Paréntesis que cierra
                ')' => self.handle_close_parenthesis()?,
                
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
                        self.skip_whitespace_only();
                    }
                }
                
                // Dígitos
                c if c.is_ascii_digit() => self.read_number()?,
                
                // Letras (identificadores)
                c if c.is_alphabetic() || c == '_' => {self.read_identifier()?; self.at_line_start = false;},
                
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
        
        // Verificar paréntesis sin cerrar al final del archivo
        self.check_unclosed_parentheses()?;
        
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
    
    fn handle_open_parenthesis(&mut self) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        
        // Añadir a la pila de paréntesis
        self.paren_stack.push(('(', start_line, start_column));
        
        // Crear token de paréntesis que abre
        self.tokens.push(Token::new(
            TokenType::OpenedParenthesis,
            "(".to_string(),
            start_line,
            start_column
        ));
        
        self.position += 1;
        self.column += 1;
        self.at_line_start = false;
        
        Ok(())
    }
    
    fn handle_close_parenthesis(&mut self) -> Result<(), CompilerError> {
        let start_line = self.line;
        let start_column = self.column;
        
        // Verificar si hay paréntesis que abrir
        if self.paren_stack.is_empty() {
            return Err(CompilerError::new(
                "Paréntesis de cierre sin apertura correspondiente".to_string(),
                start_line,
                start_column
            ));
        }
        
        // Verificar que el paréntesis que cierra corresponda al que abre
        let last_paren = self.paren_stack.last().unwrap();
        if last_paren.0 != '(' {
            return Err(CompilerError::new(
                "Paréntesis de cierre no corresponde con la apertura".to_string(),
                start_line,
                start_column
            ));
        }
        
        // Remover de la pila
        self.paren_stack.pop();
        
        // Crear token de paréntesis que cierra
        self.tokens.push(Token::new(
            TokenType::ClosedParenthesis,
            ")".to_string(),
            start_line,
            start_column
        ));
        
        self.position += 1;
        self.column += 1;
        self.at_line_start = false;
        
        Ok(())
    }
    
    fn check_unclosed_parentheses(&self) -> Result<(), CompilerError> {
        for (paren_type, line, column) in &self.paren_stack {
            return Err(CompilerError::new(
                format!("Paréntesis '{}' sin cerrar", paren_type),
                *line,
                *column
            ));
        }
        Ok(())
    }
    
    fn handle_indentation(&mut self) -> Result<(), CompilerError> {
        let start_pos = self.position;
        let mut indent = 0;
        
        // Solo contar espacios/tabs al inicio de línea
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
        
        // IMPORTANTE: Solo procesar indentación si estamos realmente al inicio de línea
        // y después de espacios hay algo que no sea salto de línea
        if self.position >= self.chars.len() || self.chars[self.position] == '\n' {
            // Línea vacía o solo espacios, no generar tokens de indentación
            self.at_line_start = false;
            return Ok(());
        }
        
        let last_indent = *self.indent_stack.last().unwrap();
        
        // Solo generar tokens INDENT/DEDENT si hay cambio real de indentación
        if indent != self.current_indent {
            if indent > last_indent {
                self.tokens.push(Token::new(
                    TokenType::Indent,
                    "",
                    self.line,
                    1
                ));
                self.indent_stack.push(indent);
            } else if indent < last_indent {
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
        }
        
        self.at_line_start = false;
        self.current_indent = indent;
        
        Ok(())
    }
    
    fn is_operator(&self, c: char) -> bool {
        matches!(c, '+' | '-' | '*' | '/' | '=' | '<' | '>' | '&' | '|' | '~')
    }
    
    // Saltar espacios sin procesar indentación
    fn skip_whitespace_only(&mut self) {
        while self.position < self.chars.len() && 
            self.chars[self.position].is_whitespace() &&
            self.chars[self.position] != '\n' {
            self.position += 1;
            self.column += 1;
            self.at_line_start = false;
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
        
        // Verificar si hay suficientes caracteres para un operador de dos caracteres
        if self.position + 1 < self.chars.len() {
            let second_char = self.chars[self.position + 1];
            let two_char_op = format!("{}{}", first_char, second_char);
            
            // Lista de operadores de dos caracteres
            let (token_type, value, chars_to_consume) = match two_char_op.as_str() {
                ":=" => (TokenType::Assign, two_char_op, 2),
                "<>" => (TokenType::NotEquals, two_char_op, 2),
                "<=" => (TokenType::LessEqual, two_char_op, 2),
                ">=" => (TokenType::GreaterEqual, two_char_op, 2),
                "==" => (TokenType::Equals, two_char_op, 2),
                _ => {
                    // No es un operador de dos caracteres, usar un solo carácter
                    let (token_type, value) = match first_char {
                        ',' => (TokenType::Comma, first_char.to_string()),
                        ':' => (TokenType::Declaration, first_char.to_string()),
                        '&' => (TokenType::And, first_char.to_string()),
                        '|' => (TokenType::Or, first_char.to_string()),
                        '~' => (TokenType::Not, first_char.to_string()),
                        '+' => (TokenType::Plus, first_char.to_string()),
                        '-' => (TokenType::Minus, first_char.to_string()),
                        '*' => (TokenType::Multiply, first_char.to_string()),
                        '/' => (TokenType::Divide, first_char.to_string()),
                        '=' => (TokenType::Equals, first_char.to_string()),
                        '<' => (TokenType::Less, first_char.to_string()),
                        '>' => (TokenType::Greater, first_char.to_string()),
                        _ => {
                            return Err(CompilerError::new(
                                format!("Operador no reconocido: '{}'", first_char),
                                start_line,
                                start_column
                            ));
                        }
                    };
                    (token_type, value, 1)
                }
            };
            
            self.tokens.push(Token::new(
                token_type,
                value,
                start_line,
                start_column
            ));
            
            self.position += chars_to_consume;
            self.column += chars_to_consume;
            self.at_line_start = false;
            return Ok(());
        }
        
        // Solo queda un carácter, manejar operadores de un solo carácter
        let (token_type, value) = match first_char {
            ',' => (TokenType::Comma, first_char.to_string()),
            ':' => (TokenType::Declaration, first_char.to_string()),
            '&' => (TokenType::And, first_char.to_string()),
            '|' => (TokenType::Or, first_char.to_string()),
            '~' => (TokenType::Not, first_char.to_string()),
            '+' => (TokenType::Plus, first_char.to_string()),
            '-' => (TokenType::Minus, first_char.to_string()),
            '*' => (TokenType::Multiply, first_char.to_string()),
            '/' => (TokenType::Divide, first_char.to_string()),
            '=' => (TokenType::Equals, first_char.to_string()),
            '<' => (TokenType::Less, first_char.to_string()),
            '>' => (TokenType::Greater, first_char.to_string()),
            _ => {
                return Err(CompilerError::new(
                    format!("Operador no reconocido: '{}'", first_char),
                    start_line,
                    start_column
                ));
            }
        };
        
        self.tokens.push(Token::new(
            token_type,
            value,
            start_line,
            start_column
        ));
        
        self.position += 1;
        self.column += 1;
        self.at_line_start = false;
        
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
        
        // Mostrar estadísticas de paréntesis
        println!("\n=== Balance de paréntesis ===");
        if self.paren_stack.is_empty() {
            println!("Todos los paréntesis están balanceados");
        } else {
            println!("Paréntesis sin cerrar: {}", self.paren_stack.len());
            for (paren_type, line, column) in &self.paren_stack {
                println!("  '{}' en línea {}, columna {}", paren_type, line, column);
            }
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
    
    // Método para verificar el balance de paréntesis
    pub fn is_parentheses_balanced(&self) -> bool {
        self.paren_stack.is_empty()
    }
    
    // Método para obtener información sobre paréntesis no cerrados
    pub fn get_unclosed_parentheses(&self) -> Vec<(char, usize, usize)> {
        self.paren_stack.clone()
    }
}