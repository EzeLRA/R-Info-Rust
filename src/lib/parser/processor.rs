use std::collections::HashMap;
use crate::lib::compilerError::CompilerError;
use super::super::lexer::token::{Token, TokenType};

// Nodos del AST
#[derive(Debug, Clone)]
pub enum ASTNode {
    Program {
        name: String,
        procedures: Vec<ProcedureNode>,
        areas: Vec<AreaNode>,
        robots: Vec<RobotNode>,
        global_vars: Vec<VariableDeclaration>,
        main_block: BlockNode,
    },
    Procedure(ProcedureNode),
    Robot(RobotNode),
    Area(AreaNode),
    VariableDeclaration(VariableDeclaration),
    Block(BlockNode),
    Assignment {
        variable: String,
        value: Box<ASTNode>,
    },
    FunctionCall {
        name: String,
        args: Vec<ASTNode>,
    },
    Identifier(String),
    NumberLiteral(i32),
    BooleanLiteral(bool),
    BinaryOperation {
        left: Box<ASTNode>,
        operator: BinaryOperator,
        right: Box<ASTNode>,
    },
    IfStatement {
        condition: Box<ASTNode>,
        then_block: BlockNode,
        else_block: Option<BlockNode>,
    },
    WhileStatement {
        condition: Box<ASTNode>,
        body: BlockNode,
    },
    RepeatStatement {
        condition: Box<ASTNode>,
        body: BlockNode,
    },
    Parameter {
        param_type: ParameterType,
        name: String,
        data_type: String,
    },
}

#[derive(Debug, Clone)]
pub struct ProcedureNode {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub variables: Vec<VariableDeclaration>,
    pub body: BlockNode,
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub param_type: ParameterType,
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
    In,  // E
    Out, // S
    InOut, // ES
}

#[derive(Debug, Clone)]
pub struct VariableDeclaration {
    pub name: String,
    pub data_type: String,
}

#[derive(Debug, Clone)]
pub struct RobotNode {
    pub name: String,
    pub variables: Vec<VariableDeclaration>,
    pub body: BlockNode,
}

#[derive(Debug, Clone)]
pub struct AreaNode {
    pub name: String,
    pub area_type: String,
    pub coordinates: (i32, i32, i32, i32),
}

#[derive(Debug, Clone)]
pub struct BlockNode {
    pub statements: Vec<ASTNode>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BinaryOperator {
    Plus,
    Minus,
    Multiply,
    Divide,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    Equals,
    NotEquals,
    And,
    Or,
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    position: usize,
    current_token: Option<&'a Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let mut parser = Self {
            tokens,
            position: 0,
            current_token: None,
        };
        parser.advance();
        parser
    }
    
    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.current_token = Some(&self.tokens[self.position]);
            self.position += 1;
        } else {
            self.current_token = None;
        }
    }
    
    fn peek(&self) -> Option<&Token> {
        if self.position < self.tokens.len() {
            Some(&self.tokens[self.position])
        } else {
            None
        }
    }
    
    fn consume(&mut self, expected_type: TokenType, error_msg: &str) -> Result<(), CompilerError> {
        match self.current_token {
            Some(token) if token.token_type == expected_type => {
                self.advance();
                Ok(())
            }
            Some(token) => Err(CompilerError::new(
                format!("{}: esperado {:?}, encontrado {:?}",
                    error_msg, expected_type, token.token_type),
                token.line,
                token.column
            )),
            None => Err(CompilerError::new(
                format!("{}: esperado {:?}, pero no hay más tokens", error_msg, expected_type),
                0, 0
            )),
        }
    }
    
    fn expect_identifier(&mut self) -> Result<String, CompilerError> {
        match self.current_token {
            Some(token) if token.token_type == TokenType::Identifier => {
                let name = token.value.clone();
                self.advance();
                Ok(name)
            }
            Some(token) => Err(CompilerError::new(
                format!("Esperado identificador, encontrado {:?}", token.token_type),
                token.line,
                token.column
            )),
            None => Err(CompilerError::new("Esperado identificador, pero no hay más tokens", 0, 0)),
        }
    }
    
    pub fn parse(&mut self) -> Result<ASTNode, CompilerError> {
        self.parse_program()
    }
    
    fn parse_program(&mut self) -> Result<ASTNode, CompilerError> {
        // programa nombre
        self.consume(TokenType::Keyword, "Esperado 'programa'")?;
        let program_name = self.expect_identifier()?;
        
        let mut procedures = Vec::new();
        let mut areas = Vec::new();
        let mut robots = Vec::new();
        let mut global_vars = Vec::new();
        
        // Parsear secciones opcionales
        while self.current_token.is_some() {
            match self.current_token {
                Some(token) if token.token_type == TokenType::Keyword => {
                    match token.value.as_str() {
                        "procesos" => {
                            self.advance(); // Consumir "procesos"
                            procedures = self.parse_procedures()?;
                        }
                        "areas" => {
                            self.advance(); // Consumir "areas"
                            areas = self.parse_areas()?;
                        }
                        "robots" => {
                            self.advance(); // Consumir "robots"
                            robots = self.parse_robots()?;
                        }
                        "variables" => {
                            self.advance(); // Consumir "variables"
                            global_vars = self.parse_variable_declarations()?;
                        }
                        "comenzar" => break, // Salir para parsear el bloque principal
                        _ => return Err(CompilerError::new(
                            format!("Sección inesperada: {}", token.value),
                            token.line,
                            token.column
                        )),
                    }
                }
                _ => break,
            }
        }
        
        // Parsear bloque principal
        self.consume(TokenType::Keyword, "Esperado 'comenzar'")?;
        let main_block = self.parse_block()?;
        self.consume(TokenType::Keyword, "Esperado 'fin'")?;
        
        Ok(ASTNode::Program {
            name: program_name,
            procedures,
            areas,
            robots,
            global_vars,
            main_block,
        })
    }
    
    fn parse_procedures(&mut self) -> Result<Vec<ProcedureNode>, CompilerError> {
        let mut procedures = Vec::new();
        
        while self.current_token.is_some() {
            if let Some(token) = self.current_token {
                if token.token_type == TokenType::Keyword && token.value == "proceso" {
                    procedures.push(self.parse_procedure()?);
                } else if token.token_type == TokenType::Indent {
                    self.advance(); // Saltar indentación
                } else {
                    break; // Fin de las secciones de procedimientos
                }
            }
        }
        
        Ok(procedures)
    }
    
    fn parse_procedure(&mut self) -> Result<ProcedureNode, CompilerError> {
        self.consume(TokenType::Keyword, "Esperado 'proceso'")?;
        let name = self.expect_identifier()?;
        
        // Parsear parámetros
        let parameters = if self.current_token.is_some() && 
            self.current_token.unwrap().token_type == TokenType::OpenedParenthesis {
            self.advance(); // Consumir '('
            self.parse_parameters()?
        } else {
            Vec::new()
        };
        
        // Parsear sección de variables
        let variables = if self.current_token.is_some() &&
            self.current_token.unwrap().token_type == TokenType::Keyword &&
            self.current_token.unwrap().value == "variables" {
            self.advance(); // Consumir "variables"
            self.parse_variable_declarations()?
        } else {
            Vec::new()
        };
        
        // Parsear cuerpo
        self.consume(TokenType::Keyword, "Esperado 'comenzar'")?;
        let body = self.parse_block()?;
        self.consume(TokenType::Keyword, "Esperado 'fin'")?;
        
        Ok(ProcedureNode {
            name,
            parameters,
            variables,
            body,
        })
    }
    
    fn parse_parameters(&mut self) -> Result<Vec<Parameter>, CompilerError> {
        let mut parameters = Vec::new();
        
        while self.current_token.is_some() && 
            self.current_token.unwrap().token_type != TokenType::ClosedParenthesis {
            
            // Parsear tipo de parámetro (E, S, ES)
            let param_type = match self.current_token {
                Some(token) if token.token_type == TokenType::ParameterType => {
                    let pt = match token.value.as_str() {
                        "E" => ParameterType::In,
                        "S" => ParameterType::Out,
                        "ES" => ParameterType::InOut,
                        _ => return Err(CompilerError::new(
                            format!("Tipo de parámetro desconocido: {}", token.value),
                            token.line,
                            token.column
                        )),
                    };
                    self.advance();
                    pt
                }
                _ => return Err(CompilerError::new(
                    "Esperado tipo de parámetro (E, S, ES)",
                    self.current_token.unwrap().line,
                    self.current_token.unwrap().column
                )),
            };
            
            let name = self.expect_identifier()?;
            self.consume(TokenType::Declaration, "Esperado ':'")?;
            
            // Parsear tipo de dato
            let data_type = match self.current_token {
                Some(token) if token.token_type == TokenType::Num || 
                               token.token_type == TokenType::Bool => {
                    let dt = token.value.clone();
                    self.advance();
                    dt
                }
                _ => return Err(CompilerError::new(
                    "Esperado tipo de dato (numero, booleano)",
                    self.current_token.unwrap().line,
                    self.current_token.unwrap().column
                )),
            };
            
            parameters.push(Parameter {
                param_type,
                name,
                data_type,
            });
            
            // Verificar si hay más parámetros (separados por coma)
            if self.current_token.is_some() && 
                self.current_token.unwrap().token_type == TokenType::Comma {
                self.advance();
            }
        }
        
        self.consume(TokenType::ClosedParenthesis, "Esperado ')'")?;
        Ok(parameters)
    }
    
    fn parse_variable_declarations(&mut self) -> Result<Vec<VariableDeclaration>, CompilerError> {
        let mut declarations = Vec::new();
        
        while self.current_token.is_some() {
            if self.current_token.unwrap().token_type == TokenType::Indent {
                self.advance(); // Saltar indentación
            } else if self.current_token.unwrap().token_type == TokenType::Identifier {
                let name = self.expect_identifier()?;
                self.consume(TokenType::Declaration, "Esperado ':'")?;
                
                let data_type = match self.current_token {
                    Some(token) if token.token_type == TokenType::Num || 
                                   token.token_type == TokenType::Bool => {
                        let dt = token.value.clone();
                        self.advance();
                        dt
                    }
                    _ => return Err(CompilerError::new(
                        "Esperado tipo de dato",
                        self.current_token.unwrap().line,
                        self.current_token.unwrap().column
                    )),
                };
                
                declarations.push(VariableDeclaration { name, data_type });
            } else {
                break;
            }
        }
        
        Ok(declarations)
    }
    
    fn parse_areas(&mut self) -> Result<Vec<AreaNode>, CompilerError> {
        let mut areas = Vec::new();
        
        while self.current_token.is_some() {
            if self.current_token.unwrap().token_type == TokenType::Identifier {
                let name = self.expect_identifier()?;
                self.consume(TokenType::Declaration, "Esperado ':'")?;
                
                let area_type = match self.current_token {
                    Some(token) if token.token_type == TokenType::ElementalInstruction => {
                        let at = token.value.clone();
                        self.advance();
                        at
                    }
                    _ => return Err(CompilerError::new(
                        "Esperado tipo de área",
                        self.current_token.unwrap().line,
                        self.current_token.unwrap().column
                    )),
                };
                
                self.consume(TokenType::OpenedParenthesis, "Esperado '('")?;
                
                let mut coordinates = Vec::new();
                for _ in 0..4 {
                    if let Some(token) = self.current_token {
                        if token.token_type == TokenType::Num {
                            let num = token.value.parse::<i32>().map_err(|_| 
                                CompilerError::new(
                                    format!("Número inválido: {}", token.value),
                                    token.line,
                                    token.column
                                )
                            )?;
                            coordinates.push(num);
                            self.advance();
                        }
                        
                        if coordinates.len() < 4 && self.current_token.is_some() &&
                            self.current_token.unwrap().token_type == TokenType::Comma {
                            self.advance();
                        }
                    }
                }
                
                if coordinates.len() != 4 {
                    return Err(CompilerError::new(
                        "Se esperaban 4 coordenadas para el área",
                        self.current_token.unwrap().line,
                        self.current_token.unwrap().column
                    ));
                }
                
                self.consume(TokenType::ClosedParenthesis, "Esperado ')'")?;
                
                areas.push(AreaNode {
                    name,
                    area_type,
                    coordinates: (coordinates[0], coordinates[1], coordinates[2], coordinates[3]),
                });
            } else {
                break;
            }
        }
        
        Ok(areas)
    }
    
    fn parse_robots(&mut self) -> Result<Vec<RobotNode>, CompilerError> {
        let mut robots = Vec::new();
        
        while self.current_token.is_some() {
            if self.current_token.unwrap().token_type == TokenType::Keyword &&
                self.current_token.unwrap().value == "robot" {
                self.advance(); // Consumir "robot"
                let name = self.expect_identifier()?;
                
                // Parsear variables del robot
                let variables = if self.current_token.is_some() &&
                    self.current_token.unwrap().token_type == TokenType::Keyword &&
                    self.current_token.unwrap().value == "variables" {
                    self.advance(); // Consumir "variables"
                    self.parse_variable_declarations()?
                } else {
                    Vec::new()
                };
                
                // Parsear cuerpo del robot
                self.consume(TokenType::Keyword, "Esperado 'comenzar'")?;
                let body = self.parse_block()?;
                self.consume(TokenType::Keyword, "Esperado 'fin'")?;
                
                robots.push(RobotNode {
                    name,
                    variables,
                    body,
                });
            } else {
                break;
            }
        }
        
        Ok(robots)
    }
    
    fn parse_block(&mut self) -> Result<BlockNode, CompilerError> {
        let mut statements = Vec::new();
        
        while self.current_token.is_some() {
            match self.current_token {
                Some(token) if token.token_type == TokenType::Indent => {
                    self.advance(); // Saltar indentación
                }
                Some(token) if token.token_type == TokenType::Dedent => {
                    self.advance(); // Saltar dedentación
                    break; // Fin del bloque
                }
                Some(token) if token.token_type == TokenType::Keyword && token.value == "fin" => {
                    break; // Fin del bloque
                }
                _ => {
                    statements.push(self.parse_statement()?);
                }
            }
        }
        
        Ok(BlockNode { statements })
    }
    
    fn parse_statement(&mut self) -> Result<ASTNode, CompilerError> {
        match self.current_token {
            Some(token) if token.token_type == TokenType::Identifier => {
                let identifier = self.expect_identifier()?;
                
                // Verificar si es una asignación
                if self.current_token.is_some() && 
                    self.current_token.unwrap().token_type == TokenType::Assign {
                    self.advance(); // Consumir ":="
                    let value = self.parse_expression()?;
                    Ok(ASTNode::Assignment {
                        variable: identifier,
                        value: Box::new(value),
                    })
                } else {
                    // Es una llamada a función
                    self.parse_function_call_with_name(identifier)
                }
            }
            Some(token) if token.token_type == TokenType::ElementalInstruction => {
                let func_name = token.value.clone();
                self.advance();
                self.parse_function_call_with_name(func_name)
            }
            Some(token) if token.token_type == TokenType::ControlSentence => {
                match token.value.as_str() {
                    "si" => self.parse_if_statement(),
                    "mientras" => self.parse_while_statement(),
                    "repetir" => self.parse_repeat_statement(),
                    _ => Err(CompilerError::new(
                        format!("Sentencia de control desconocida: {}", token.value),
                        token.line,
                        token.column
                    )),
                }
            }
            _ => Err(CompilerError::new(
                "Sentencia no reconocida",
                self.current_token.unwrap().line,
                self.current_token.unwrap().column
            )),
        }
    }
    
    fn parse_function_call_with_name(&mut self, name: String) -> Result<ASTNode, CompilerError> {
        let args = if self.current_token.is_some() && 
            self.current_token.unwrap().token_type == TokenType::OpenedParenthesis {
            self.advance(); // Consumir '('
            let args = self.parse_argument_list()?;
            self.consume(TokenType::ClosedParenthesis, "Esperado ')'")?;
            args
        } else {
            Vec::new()
        };
        
        Ok(ASTNode::FunctionCall { name, args })
    }
    
    fn parse_argument_list(&mut self) -> Result<Vec<ASTNode>, CompilerError> {
        let mut args = Vec::new();
        
        while self.current_token.is_some() && 
            self.current_token.unwrap().token_type != TokenType::ClosedParenthesis {
            args.push(self.parse_expression()?);
            
            if self.current_token.is_some() && 
                self.current_token.unwrap().token_type == TokenType::Comma {
                self.advance();
            }
        }
        
        Ok(args)
    }
    
    fn parse_if_statement(&mut self) -> Result<ASTNode, CompilerError> {
        self.advance(); // Consumir "si"
        
        // Parsear condición
        let condition = if self.current_token.is_some() && 
            self.current_token.unwrap().token_type == TokenType::OpenedParenthesis {
            self.advance(); // Consumir '('
            let cond = self.parse_expression()?;
            self.consume(TokenType::ClosedParenthesis, "Esperado ')'")?;
            cond
        } else {
            // Condición sin paréntesis (ej: si HayFlorEnLaEsquina)
            self.parse_expression()?
        };
        
        // Parsear bloque THEN
        let then_block = self.parse_block()?;
        
        // Parsear ELSE opcional
        let else_block = if self.current_token.is_some() && 
            self.current_token.unwrap().token_type == TokenType::ControlSentence &&
            self.current_token.unwrap().value == "sino" {
            self.advance(); // Consumir "sino"
            Some(self.parse_block()?)
        } else {
            None
        };
        
        Ok(ASTNode::IfStatement {
            condition: Box::new(condition),
            then_block,
            else_block,
        })
    }
    
    fn parse_while_statement(&mut self) -> Result<ASTNode, CompilerError> {
        self.advance(); // Consumir "mientras"
        
        self.consume(TokenType::OpenedParenthesis, "Esperado '(' después de 'mientras'")?;
        let condition = self.parse_expression()?;
        self.consume(TokenType::ClosedParenthesis, "Esperado ')'")?;
        
        let body = self.parse_block()?;
        
        Ok(ASTNode::WhileStatement {
            condition: Box::new(condition),
            body,
        })
    }
    
    fn parse_repeat_statement(&mut self) -> Result<ASTNode, CompilerError> {
        self.advance(); // Consumir "repetir"
        
        let condition = self.parse_expression()?;
        let body = self.parse_block()?;
        
        Ok(ASTNode::RepeatStatement {
            condition: Box::new(condition),
            body,
        })
    }
    
    fn parse_expression(&mut self) -> Result<ASTNode, CompilerError> {
        self.parse_comparison()
    }
    
    fn parse_comparison(&mut self) -> Result<ASTNode, CompilerError> {
        let mut left = self.parse_addition()?;
        
        while self.current_token.is_some() {
            let operator = match self.current_token.unwrap().token_type {
                TokenType::Less => BinaryOperator::Less,
                TokenType::LessEqual => BinaryOperator::LessEqual,
                TokenType::Greater => BinaryOperator::Greater,
                TokenType::GreaterEqual => BinaryOperator::GreaterEqual,
                TokenType::Equals => BinaryOperator::Equals,
                TokenType::NotEquals => BinaryOperator::NotEquals,
                _ => break,
            };
            
            self.advance(); // Consumir operador
            let right = self.parse_addition()?;
            
            left = ASTNode::BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_addition(&mut self) -> Result<ASTNode, CompilerError> {
        let mut left = self.parse_multiplication()?;
        
        while self.current_token.is_some() {
            let operator = match self.current_token.unwrap().token_type {
                TokenType::Plus => BinaryOperator::Plus,
                TokenType::Minus => BinaryOperator::Minus,
                TokenType::And => BinaryOperator::And,
                TokenType::Or => BinaryOperator::Or,
                _ => break,
            };
            
            self.advance(); // Consumir operador
            let right = self.parse_multiplication()?;
            
            left = ASTNode::BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_multiplication(&mut self) -> Result<ASTNode, CompilerError> {
        let mut left = self.parse_primary()?;
        
        while self.current_token.is_some() {
            let operator = match self.current_token.unwrap().token_type {
                TokenType::Multiply => BinaryOperator::Multiply,
                TokenType::Divide => BinaryOperator::Divide,
                _ => break,
            };
            
            self.advance(); // Consumir operador
            let right = self.parse_primary()?;
            
            left = ASTNode::BinaryOperation {
                left: Box::new(left),
                operator,
                right: Box::new(right),
            };
        }
        
        Ok(left)
    }
    
    fn parse_primary(&mut self) -> Result<ASTNode, CompilerError> {
        match self.current_token {
            Some(token) => match token.token_type {
                TokenType::Identifier => {
                    let name = token.value.clone();
                    self.advance();
                    Ok(ASTNode::Identifier(name))
                }
                TokenType::Num => {
                    let value = token.value.parse::<i32>().map_err(|_| 
                        CompilerError::new(
                            format!("Número inválido: {}", token.value),
                            token.line,
                            token.column
                        )
                    )?;
                    self.advance();
                    Ok(ASTNode::NumberLiteral(value))
                }
                TokenType::BoolValue => {
                    let value = match token.value.as_str() {
                        "V" | "true" | "verdadero" => true,
                        "F" | "false" | "falso" => false,
                        _ => return Err(CompilerError::new(
                            format!("Valor booleano inválido: {}", token.value),
                            token.line,
                            token.column
                        )),
                    };
                    self.advance();
                    Ok(ASTNode::BooleanLiteral(value))
                }
                TokenType::ElementalInstruction => {
                    let func_name = token.value.clone();
                    self.advance();
                    self.parse_function_call_with_name(func_name)
                }
                TokenType::OpenedParenthesis => {
                    self.advance(); // Consumir '('
                    let expr = self.parse_expression()?;
                    self.consume(TokenType::ClosedParenthesis, "Esperado ')'")?;
                    Ok(expr)
                }
                TokenType::Not => {
                    self.advance(); // Consumir '~'
                    let expr = self.parse_primary()?;
                    // NOT como operador binario especial (0 == expr)
                    Ok(ASTNode::BinaryOperation {
                        left: Box::new(ASTNode::NumberLiteral(0)),
                        operator: BinaryOperator::Equals,
                        right: Box::new(expr),
                    })
                }
                _ => Err(CompilerError::new(
                    format!("Expresión primaria inesperada: {:?}", token.token_type),
                    token.line,
                    token.column
                )),
            },
            None => Err(CompilerError::new(
                "Se esperaba una expresión primaria",
                0, 0
            )),
        }
    }
}

// Funciones de utilidad para imprimir/debuggear el AST
impl ASTNode {
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        match self {
            ASTNode::Program { name, procedures, areas, robots, global_vars, main_block } => {
                let mut result = format!("{}Programa: {}\n", indent_str, name);
                
                if !procedures.is_empty() {
                    result.push_str(&format!("{}Procedimientos:\n", indent_str));
                    for proc in procedures {
                        result.push_str(&proc.pretty_print(indent + 1));
                    }
                }
                
                if !areas.is_empty() {
                    result.push_str(&format!("{}Áreas:\n", indent_str));
                    for area in areas {
                        result.push_str(&area.pretty_print(indent + 1));
                    }
                }
                
                if !robots.is_empty() {
                    result.push_str(&format!("{}Robots:\n", indent_str));
                    for robot in robots {
                        result.push_str(&robot.pretty_print(indent + 1));
                    }
                }
                
                if !global_vars.is_empty() {
                    result.push_str(&format!("{}Variables globales:\n", indent_str));
                    for var in global_vars {
                        result.push_str(&var.pretty_print(indent + 1));
                    }
                }
                
                result.push_str(&format!("{}Bloque principal:\n", indent_str));
                result.push_str(&main_block.pretty_print(indent + 1));
                
                result
            }
            ASTNode::Assignment { variable, value } => {
                format!("{}{} := {}\n", indent_str, variable, value.pretty_print(0))
            }
            ASTNode::FunctionCall { name, args } => {
                let args_str = args.iter()
                    .map(|arg| arg.pretty_print(0))
                    .collect::<Vec<String>>()
                    .join(", ");
                format!("{}{}({})\n", indent_str, name, args_str)
            }
            ASTNode::Identifier(name) => format!("{}", name),
            ASTNode::NumberLiteral(value) => format!("{}", value),
            ASTNode::BooleanLiteral(value) => format!("{}", value),
            ASTNode::IfStatement { condition, then_block, else_block } => {
                let mut result = format!("{}SI {} ENTONCES\n", 
                    indent_str, condition.pretty_print(0));
                result.push_str(&then_block.pretty_print(indent + 1));
                
                if let Some(else_block) = else_block {
                    result.push_str(&format!("{}SINO\n", indent_str));
                    result.push_str(&else_block.pretty_print(indent + 1));
                }
                
                result
            }
            ASTNode::WhileStatement { condition, body } => {
                format!("{}MIENTRAS {} HACER\n{}",
                    indent_str, condition.pretty_print(0), body.pretty_print(indent + 1))
            }
            ASTNode::RepeatStatement { condition, body } => {
                format!("{}REPETIR {} HACER\n{}",
                    indent_str, condition.pretty_print(0), body.pretty_print(indent + 1))
            }
            ASTNode::BinaryOperation { left, operator, right } => {
                let op_str = match operator {
                    BinaryOperator::Plus => "+",
                    BinaryOperator::Minus => "-",
                    BinaryOperator::Multiply => "*",
                    BinaryOperator::Divide => "/",
                    BinaryOperator::Less => "<",
                    BinaryOperator::LessEqual => "<=",
                    BinaryOperator::Greater => ">",
                    BinaryOperator::GreaterEqual => ">=",
                    BinaryOperator::Equals => "==",
                    BinaryOperator::NotEquals => "<>",
                    BinaryOperator::And => "&",
                    BinaryOperator::Or => "|",
                };
                format!("({} {} {})", left.pretty_print(0), op_str, right.pretty_print(0))
            }
            _ => format!("{}[Otro nodo]\n", indent_str),
        }
    }
}

impl ProcedureNode {
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut result = format!("{}Proceso {}(", indent_str, self.name);
        
        let params_str = self.parameters.iter()
            .map(|p| p.pretty_print())
            .collect::<Vec<String>>()
            .join(", ");
        result.push_str(&format!("{})\n", params_str));
        
        if !self.variables.is_empty() {
            result.push_str(&format!("{}  Variables:\n", indent_str));
            for var in &self.variables {
                result.push_str(&var.pretty_print(indent + 2));
            }
        }
        
        result.push_str(&format!("{}  Cuerpo:\n", indent_str));
        result.push_str(&self.body.pretty_print(indent + 2));
        
        result
    }
}

impl Parameter {
    pub fn pretty_print(&self) -> String {
        let type_str = match self.param_type {
            ParameterType::In => "E",
            ParameterType::Out => "S",
            ParameterType::InOut => "ES",
        };
        format!("{} {}: {}", type_str, self.name, self.data_type)
    }
}

impl VariableDeclaration {
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        format!("{}{}: {}\n", indent_str, self.name, self.data_type)
    }
}

impl RobotNode {
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        let mut result = format!("{}Robot {}\n", indent_str, self.name);
        
        if !self.variables.is_empty() {
            result.push_str(&format!("{}  Variables:\n", indent_str));
            for var in &self.variables {
                result.push_str(&var.pretty_print(indent + 2));
            }
        }
        
        result.push_str(&format!("{}  Cuerpo:\n", indent_str));
        result.push_str(&self.body.pretty_print(indent + 2));
        
        result
    }
}

impl AreaNode {
    pub fn pretty_print(&self, indent: usize) -> String {
        let indent_str = "  ".repeat(indent);
        format!("{}{}: {}({}, {}, {}, {})\n", 
            indent_str, self.name, self.area_type,
            self.coordinates.0, self.coordinates.1,
            self.coordinates.2, self.coordinates.3)
    }
}

impl BlockNode {
    pub fn pretty_print(&self, indent: usize) -> String {
        let mut result = String::new();
        for stmt in &self.statements {
            result.push_str(&stmt.pretty_print(indent));
        }
        result
    }
}