use std::collections::HashMap;
use crate::lib::compilerError::CompilerError;
use super::super::lexer::token::{Token, TokenType, Keywords};
use super::ast::{ASTNode, Condition, Parameter};

pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
    current_token: Option<&'a Token>,
    indent_level: usize,
    keywords: Keywords,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token], keywords: Keywords) -> Self {
        let current_token = if !tokens.is_empty() { Some(&tokens[0]) } else { None };
        
        Self {
            tokens,
            position: 0,
            current_token,
            indent_level: 0,
            keywords,
        }
    }
    
    pub fn parse(&mut self) -> Result<ASTNode, CompilerError> {
        self.parse_program()
    }
    
    fn parse_program(&mut self) -> Result<ASTNode, CompilerError> {
        // Busca si el primer token obtenido por el lexer es "programa"
        self.consume(TokenType::Keyword, Some("programa"))?;
        
        // Almacenar el nombre del programa
        let program_name = self.consume(TokenType::Identifier, None)?.value.clone();
        
        let mut body = Vec::new();
        
        // Parsear secciones en el orden que aparecen
        if self.matches_value(TokenType::Keyword, "procesos") {
            body.push(self.parse_procesos()?);
        }
        
        body.push(self.parse_areas()?);
        body.push(self.parse_robots()?);
        body.push(self.parse_variables_section()?);
        body.push(self.parse_main_block()?);
        
        Ok(ASTNode::Program {
            name: program_name,
            body,
        })
    }
    
    fn parse_procesos(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::Keyword, Some("procesos"))?;
        let mut procesos = Vec::new();
        
        while !self.is_at_end() && !self.is_next_section() {
            if self.matches_value(TokenType::Keyword, "proceso") {
                procesos.push(self.parse_proceso()?);
            } else {
                self.advance()?;
            }
        }
        
        Ok(ASTNode::ProcesosSection { procesos })
    }
    
    fn parse_proceso(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::Keyword, Some("proceso"))?;
        let name = self.consume(TokenType::Identifier, None)?.value.clone();
        
        let mut parameters = Vec::new();
        let mut variables = None;
        
        // Parsear parámetros (ej: "E numAv: numero")
        while self.matches(TokenType::Parameter) {
            let param_token = self.consume(TokenType::Parameter, None)?;
            parameters.push(self.parse_parameter(&param_token.value)?);
        }
        
        if self.matches_value(TokenType::Keyword, "variables") {
            let var_section = self.parse_variables_section()?;
            variables = Some(Box::new(var_section));
        }
        
        self.consume(TokenType::Keyword, Some("comenzar"))?;
        let body = self.parse_block()?;
        self.consume(TokenType::Keyword, Some("fin"))?;
        
        Ok(ASTNode::Proceso {
            name,
            parameters,
            variables,
            body,
        })
    }
    
    fn parse_parameter(&self, param_string: &str) -> Result<Parameter, CompilerError> {
        // Ejemplo: "E numAv: numero" → {direction: 'E', name: 'numAv', type: 'numero'}
        let parts: Vec<&str> = param_string.split_whitespace().collect();
        
        if parts.len() < 2 {
            return Err(CompilerError::new(
                format!("Parámetro mal formado: '{}'", param_string),
                0, // Línea desconocida
                0, // Columna desconocida
            ));
        }
        
        let direction = parts[0].to_string();
        let name_type: Vec<&str> = parts[1].split(':').collect();
        
        let name = if !name_type.is_empty() {
            name_type[0].trim().to_string()
        } else {
            "".to_string()
        };
        
        let param_type = if name_type.len() > 1 {
            name_type[1].trim().to_string()
        } else {
            "numero".to_string()
        };
        
        Ok(Parameter {
            direction,
            name,
            param_type,
        })
    }
    
    fn parse_areas(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::Keyword, Some("areas"))?;
        let mut areas = Vec::new();
        
        while !self.is_at_end() && !self.is_next_section() {
            if self.matches(TokenType::Identifier) {
                areas.push(self.parse_area_definition()?);
            } else {
                self.advance()?;
            }
        }
        
        Ok(ASTNode::AreasSection { areas })
    }
    
    fn parse_area_definition(&mut self) -> Result<ASTNode, CompilerError> {
        let area_name = self.consume(TokenType::Identifier, None)?.value.clone();
        self.consume(TokenType::Declaration, None)?; // ':'
        
        // Buscar tipo de área (ElementalInstruction)
        let area_type = if self.matches_any_elemental_instruction() {
            self.consume(TokenType::ElementalInstruction, None)?.value.clone()
        } else {
            return Err(CompilerError::new(
                "Tipo de área esperado",
                self.current_token.map_or(0, |t| t.line),
                self.current_token.map_or(0, |t| t.column),
            ));
        };
        
        let dimensions = self.parse_parameter_list()?;
        
        Ok(ASTNode::AreaDefinition {
            name: area_name,
            area_type,
            dimensions,
        })
    }
    
    fn matches_any_elemental_instruction(&self) -> bool {
        if let Some(token) = self.current_token {
            token.token_type == TokenType::ElementalInstruction
        } else {
            false
        }
    }
    
    fn parse_robots(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::Keyword, Some("robots"))?;
        let mut robots = Vec::new();
        
        while !self.is_at_end() && !self.is_next_section() {
            if self.matches_value(TokenType::Keyword, "robot") {
                robots.push(self.parse_robot()?);
            } else {
                self.advance()?;
            }
        }
        
        Ok(ASTNode::RobotsSection { robots })
    }
    
    fn parse_robot(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::Keyword, Some("robot"))?;
        let name = self.consume(TokenType::Identifier, None)?.value.clone();
        let mut variables = None;
        
        if self.matches_value(TokenType::Keyword, "variables") {
            let var_section = self.parse_variables_section()?;
            // Extraer solo las declaraciones de VariablesSection
            if let ASTNode::VariablesSection { declarations } = var_section {
                variables = Some(declarations);
            }
        }
        
        self.consume(TokenType::Keyword, Some("comenzar"))?;
        let body = self.parse_block()?;
        self.consume(TokenType::Keyword, Some("fin"))?;
        
        Ok(ASTNode::Robot {
            name,
            variables,
            body,
        })
    }
    
    fn parse_variables_section(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::Keyword, Some("variables"))?;
        let mut declarations = Vec::new();
        
        while !self.is_at_end() && !self.is_next_section() {
            if self.matches(TokenType::Identifier) {
                declarations.push(self.parse_variable_declaration()?);
            } else {
                self.advance()?;
            }
        }
        
        Ok(ASTNode::VariablesSection { declarations })
    }
    
    fn parse_variable_declaration(&mut self) -> Result<ASTNode, CompilerError> {
        let name = self.consume(TokenType::Identifier, None)?.value.clone();
        self.consume(TokenType::Declaration, None)?; // ':'
        
        let var_type = if self.matches(TokenType::Identifier) {
            self.consume(TokenType::Identifier, None)?.value.clone()
        } else if self.matches(TokenType::Num) || self.matches(TokenType::Bool) {
            // Manejar tipos básicos
            let token = self.consume_token()?;
            token.value.clone()
        } else {
            return Err(CompilerError::new(
                "Tipo de variable esperado",
                self.current_token.map_or(0, |t| t.line),
                self.current_token.map_or(0, |t| t.column),
            ));
        };
        
        Ok(ASTNode::VariableDeclaration {
            name,
            variable_type: var_type,
        })
    }
    
    fn parse_main_block(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::Keyword, Some("comenzar"))?;
        let mut body = Vec::new();
        
        while !self.is_at_end() && !self.matches_value(TokenType::Keyword, "fin") {
            body.push(self.parse_statement()?);
        }
        
        self.consume(TokenType::Keyword, Some("fin"))?;
        
        Ok(ASTNode::MainBlock { body })
    }
    
    fn parse_block(&mut self) -> Result<Vec<ASTNode>, CompilerError> {
        let mut statements = Vec::new();
        
        // Esperar INDENT para bloques
        if self.matches(TokenType::Indent) {
            self.consume(TokenType::Indent, None)?;
            self.indent_level += 1;
            
            while !self.is_at_end() && !self.matches(TokenType::Dedent) {
                statements.push(self.parse_statement()?);
            }
            
            if self.matches(TokenType::Dedent) {
                self.consume(TokenType::Dedent, None)?;
            }
            self.indent_level -= 1;
        } else {
            // Bloque de una sola línea
            statements.push(self.parse_statement()?);
        }
        
        Ok(statements)
    }
    
    fn parse_statement(&mut self) -> Result<ASTNode, CompilerError> {
        if self.matches_value(TokenType::ControlSentence, "si") {
            self.parse_if_statement()
        } else if self.matches_value(TokenType::ControlSentence, "mientras") {
            self.parse_while_statement()
        } else if self.matches_value(TokenType::ControlSentence, "repetir") {
            self.parse_repeat_statement()
        } else if self.matches(TokenType::ElementalInstruction) {
            self.parse_elemental_instruction()
        } else if self.is_a_value() {
            // Puede ser una llamada a proceso o una asignación
            self.parse_identifier_statement()
        } else if self.is_mat_operator() {
            self.parse_operator()
        } else if self.matches(TokenType::Assign) {
            self.parse_assignment_statement()
        } else {
            Err(CompilerError::new(
                format!(
                    "Declaración no esperada: {:?} '{}'",
                    self.current_token.map(|t| t.token_type),
                    self.current_token.map_or("", |t| &t.value)
                ),
                self.current_token.map_or(0, |t| t.line),
                self.current_token.map_or(0, |t| t.column),
            ))
        }
    }
    fn is_a_value(&self) -> bool {
        if let Some(token) = self.current_token {
            matches!(token.token_type, TokenType::Num | TokenType::Identifier | TokenType::BoolValue | TokenType::Parameter)
        } else {
            false
        }
    }
    fn is_mat_operator(&self) -> bool {
        if let Some(token) = self.current_token {
            matches!(token.token_type, TokenType::Plus | TokenType::Minus | TokenType::Multiply | TokenType::Divide)||matches!(token.token_type, TokenType::And | TokenType::Or | TokenType::Not | TokenType::Equals | TokenType::NotEquals | TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual)
        } else {
            false
        }
    }
    fn parse_operator(&mut self) -> Result<ASTNode, CompilerError> {
        let operator = self.consume_token()?.value.clone();
        Ok(ASTNode::Operator { operator })
    }   
    fn parse_identifier_statement(&mut self) -> Result<ASTNode, CompilerError> {
        
        // Verificar si es una llamada a proceso (tiene parámetros)
        if self.matches(TokenType::Parameter) {
            let parameters = self.parse_parameter_list()?;
            let identifier = self.consume(TokenType::Identifier, None)?.value.clone();
            return Ok(ASTNode::ProcessCall {
                name: identifier,
                parameters,
            });
        
        }

        let valuePri = if self.matches(TokenType::Num) {
            self.consume(TokenType::Num, None)?.value.clone()
        } else if self.matches(TokenType::Identifier) {
            self.consume(TokenType::Identifier, None)?.value.clone()
        } else if self.matches(TokenType::BoolValue) {
            self.consume(TokenType::BoolValue, None)?.value.clone()
        } else {
            return Err(CompilerError::new(
                "Valor esperado para su uso",
                self.current_token.map_or(0, |t| t.line),
                self.current_token.map_or(0, |t| t.column),
            ));
        };

        Ok(ASTNode::Value {
            value: valuePri,
        })
    }
    
    fn parse_assignment_statement(&mut self) -> Result<ASTNode, CompilerError> {
        // Asignación simple: := valor
        self.consume(TokenType::Assign, None)?;
        let value = if self.matches(TokenType::Num) {
            self.consume(TokenType::Num, None)?.value.clone()
        }else if self.matches(TokenType::BoolValue) {
            self.consume(TokenType::BoolValue, None)?.value.clone()
        } else if self.matches(TokenType::Identifier) {
            self.consume(TokenType::Identifier, None)?.value.clone()
        } else{
            return Err(CompilerError::new(
                "Número esperado para asignación",
                self.current_token.map_or(0, |t| t.line),
                self.current_token.map_or(0, |t| t.column),
            ));
        };
        
        Ok(ASTNode::Assignment {
            target: None,
            operator: ":=".to_string(),
            value,
        })
    }
    
    fn parse_if_statement(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::ControlSentence, Some("si"))?;
        
        // Parsear condición
        let condition = self.parse_condition()?;
        
        // Parsear bloque THEN
        let consequent = self.parse_block()?;
        
        let mut alternate = None;
        
        // Verificar si hay un bloque SINO
        if self.matches_value(TokenType::ControlSentence, "sino") {
            self.consume(TokenType::ControlSentence, Some("sino"))?;
            alternate = Some(self.parse_block()?);
        }
        
        Ok(ASTNode::IfStatement {
            condition,
            consequent,
            alternate,
        })
    }
    
    fn parse_while_statement(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::ControlSentence, Some("mientras"))?;
        
        // Parsear condición
        let condition = self.parse_condition()?;
        
        // Parsear cuerpo del bucle
        let body = self.parse_block()?;
        
        Ok(ASTNode::WhileStatement {
            condition,
            body,
        })
    }
    
    fn parse_repeat_statement(&mut self) -> Result<ASTNode, CompilerError> {
        self.consume(TokenType::ControlSentence, Some("repetir"))?;
        let count = if self.matches(TokenType::Num) {
                self.consume(TokenType::Num, None)?.value.clone()
            } else if self.matches(TokenType::Identifier) {
                self.consume(TokenType::Identifier, None)?.value.clone()
            } else{
                return Err(CompilerError::new(
                    "Número esperado para asignación",
                    self.current_token.map_or(0, |t| t.line),
                    self.current_token.map_or(0, |t| t.column),
                ));
            };
        let body = self.parse_block()?;
        
        Ok(ASTNode::RepeatStatement {
            count,
            body,
        })
    }
    
    fn parse_condition(&mut self) -> Result<Condition, CompilerError> {
        let mut condition = String::new();
        
        // Leer la condición hasta encontrar un token que indique el fin
        while !self.is_at_end() && 
               !self.matches(TokenType::Indent) && 
               !self.matches(TokenType::ControlSentence)  
        {
            
            if let Some(token) = self.current_token {
                condition.push_str(&token.value);
                condition.push(' ');
                self.advance()?;
            } else {
                break;
            }
        }
        
        // Limpiar espacios extra
        let condition = condition.trim().to_string();
        
        // Si no hay condición, lanzar error
        if condition.is_empty() {
            return Err(CompilerError::new(
                "Condición esperada después de Si o Sino",
                self.current_token.map_or(0, |t| t.line),
                self.current_token.map_or(0, |t| t.column),
            ));
        }
        
        Ok(Condition { expression: condition })
    }
    
    fn parse_elemental_instruction(&mut self) -> Result<ASTNode, CompilerError> {
        let instruction = self.consume(TokenType::ElementalInstruction, None)?.value.clone();
        let parameters = self.parse_parameter_list()?;
        
        Ok(ASTNode::ElementalInstruction {
            instruction,
            parameters,
        })
    }
    
    fn parse_parameter_list(&mut self) -> Result<Vec<String>, CompilerError> {
        let mut parameters = Vec::new();
        
        if self.matches(TokenType::Parameter) {
            let param_token = self.consume(TokenType::Parameter, None)?;
            // Dividir parámetros por comas: "1,1,100,100" → ['1', '1', '100', '100']
            parameters.extend(
                param_token.value
                    .split(',')
                    .map(|p| p.trim().to_string())
                    .filter(|p| !p.is_empty())
            );
        }
        
        Ok(parameters)
    }
    
    // Métodos auxiliares
    fn consume(&mut self, expected_type: TokenType, expected_value: Option<&str>) -> Result<&'a Token, CompilerError> {
        self.expect(expected_type, expected_value)?;
        let token = self.current_token.ok_or_else(|| CompilerError::new(
            "Token esperado pero se alcanzó el final",
            0, 0
        ))?;
        
        self.advance()?;
        Ok(token)
    }
    
    fn consume_token(&mut self) -> Result<&'a Token, CompilerError> {
        let token = self.current_token.ok_or_else(|| CompilerError::new(
            "Token esperado pero se alcanzó el final",
            0, 0
        ))?;
        
        self.advance()?;
        Ok(token)
    }
    
    fn expect(&self, expected_type: TokenType, expected_value: Option<&str>) -> Result<(), CompilerError> {
        if self.is_at_end() {
            return Err(CompilerError::new(
                format!("Se esperaba {:?} pero se alcanzó el final", expected_type),
                0, 0
            ));
        }
        
        let token = self.current_token.ok_or_else(|| CompilerError::new(
            "Token actual no disponible",
            0, 0
        ))?;
        
        if token.token_type != expected_type {
            return Err(CompilerError::new(
                format!("Se esperaba {:?}, se obtuvo {:?} en línea {}", 
                    expected_type, token.token_type, token.line),
                token.line,
                token.column
            ));
        }
        
        if let Some(expected_val) = expected_value {
            if token.value != expected_val {
                return Err(CompilerError::new(
                    format!("Se esperaba '{}', se obtuvo '{}' en línea {}", 
                        expected_val, token.value, token.line),
                    token.line,
                    token.column
                ));
            }
        }
        
        Ok(())
    }
    
    // Método matches con solo tipo
    fn matches(&self, token_type: TokenType) -> bool {
        if let Some(token) = self.current_token {
            token.token_type == token_type
        } else {
            false
        }
    }
    
    // Método matches con tipo y valor
    fn matches_value(&self, token_type: TokenType, value: &str) -> bool {
        if let Some(token) = self.current_token {
            token.token_type == token_type && token.value == value
        } else {
            false
        }
    }
    
    fn is_next_section(&self) -> bool {
        let next_tokens = ["procesos", "areas", "robots", "variables", "comenzar"];
        
        if let Some(token) = self.current_token {
            token.token_type == TokenType::Keyword && 
            next_tokens.contains(&token.value.as_str())
        } else {
            false
        }
    }
    
    fn advance(&mut self) -> Result<(), CompilerError> {
        self.position += 1;
        if self.position < self.tokens.len() {
            self.current_token = Some(&self.tokens[self.position]);
        } else {
            self.current_token = None;
        }
        Ok(())
    }
    
    fn is_at_end(&self) -> bool {
        self.position >= self.tokens.len() || 
        self.current_token.map_or(true, |t| t.token_type == TokenType::EndFile)
    }
    
    // Método de utilidad para depuración
    pub fn debug_ast(node: &ASTNode, indent: usize) {
        let spaces = "  ".repeat(indent);
        
        match node {
            ASTNode::Program { name, body } => {
                println!("{}Programa: {}", spaces, name);
                for item in body {
                    Parser::debug_ast(item, indent + 1);
                }
            }
            ASTNode::ProcesosSection { procesos } => {
                println!("{}Sección Procesos:", spaces);
                for proceso in procesos {
                    Parser::debug_ast(proceso, indent + 1);
                }
            }
            ASTNode::Proceso { name, parameters, variables, body } => {
                println!("{}Proceso: {}", spaces, name);
                if !parameters.is_empty() {
                    println!("{}  Parámetros:", spaces);
                    for param in parameters {
                        println!("{}    {} {}: {}", spaces, param.direction, param.name, param.param_type);
                    }
                }
                if let Some(vars) = variables {
                    Parser::debug_ast(vars, indent + 1);
                }
                println!("{}  Cuerpo:", spaces);
                for stmt in body {
                    Parser::debug_ast(stmt, indent + 2);
                }
            }
            ASTNode::AreasSection { areas } => {
                println!("{}Sección Áreas:", spaces);
                for area in areas {
                    Parser::debug_ast(area, indent + 1);
                }
            }
            ASTNode::AreaDefinition { name, area_type, dimensions } => {
                println!("{}Área: {} ({}) [{}]", spaces, name, area_type, dimensions.join(", "));
            }
            ASTNode::RobotsSection { robots } => {
                println!("{}Sección Robots:", spaces);
                for robot in robots {
                    Parser::debug_ast(robot, indent + 1);
                }
            }
            ASTNode::Robot { name, variables, body } => {
                println!("{}Robot: {}", spaces, name);
                if let Some(vars) = variables {
                    println!("{}  Variables:", spaces);
                    for var in vars {
                        Parser::debug_ast(var, indent + 2);
                    }
                }
                println!("{}  Cuerpo:", spaces);
                for stmt in body {
                    Parser::debug_ast(stmt, indent + 2);
                }
            }
            ASTNode::VariablesSection { declarations } => {
                println!("{}Sección Variables:", spaces);
                for decl in declarations {
                    Parser::debug_ast(decl, indent + 1);
                }
            }
            ASTNode::VariableDeclaration { name, variable_type } => {
                println!("{}Variable: {}: {}", spaces, name, variable_type);
            }
            ASTNode::MainBlock { body } => {
                println!("{}Bloque Principal:", spaces);
                for stmt in body {
                    Parser::debug_ast(stmt, indent + 1);
                }
            }
            ASTNode::IfStatement { condition, consequent, alternate } => {
                println!("{}Si: {}", spaces, condition.expression);
                println!("{}  Entonces:", spaces);
                for stmt in consequent {
                    Parser::debug_ast(stmt, indent + 2);
                }
                if let Some(alt) = alternate {
                    println!("{}  Sino:", spaces);
                    for stmt in alt {
                        Parser::debug_ast(stmt, indent + 2);
                    }
                }
            }
            ASTNode::WhileStatement { condition, body } => {
                println!("{}Mientras: {}", spaces, condition.expression);
                for stmt in body {
                    Parser::debug_ast(stmt, indent + 1);
                }
            }
            ASTNode::RepeatStatement { count, body } => {
                println!("{}Repetir {} veces:", spaces, count);
                for stmt in body {
                    Parser::debug_ast(stmt, indent + 1);
                }
            }
            ASTNode::Assignment { target, operator, value } => {
                if let Some(tgt) = target {
                    println!("{}Asignación: {} {} {}", spaces, tgt, operator, value);
                } else {
                    println!("{}Asignación: {} {}", spaces, operator, value);
                }
            }
            ASTNode::ElementalInstruction { instruction, parameters } => {
                println!("{}Instrucción: {} ({})", spaces, instruction, parameters.join(", "));
            }
            ASTNode::ProcessCall { name, parameters } => {
                println!("{}Llamada a proceso: {} ({})", spaces, name, parameters.join(", "));
            }
            _ => println!("{}Nodo: {:?}", spaces, node),
        }
    }
}