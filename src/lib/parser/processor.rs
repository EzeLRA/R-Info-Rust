use std::collections::HashMap;
use crate::lib::compilerError::CompilerError;
use super::super::lexer::token::{Token, TokenType};

#[derive(Debug, Clone)]
pub struct RobotInstanciado {
    pub nombre: String,
    pub tipo: String,
}

#[derive(Debug, Clone)]
pub struct AsignacionArea {
    pub robot: Expresion,
    pub area: Expresion,
}

#[derive(Debug, Clone)]
pub struct InicializacionRobot {
    pub robot: Expresion,
    pub pos_x: Expresion,
    pub pos_y: Expresion,
}

// Estrutura principal del Ast
#[derive(Debug, Clone)]
pub struct Program {
    pub nombre: String,
    pub procesos: Vec<Proceso>,
    pub areas: Vec<Area>,
    pub robots_declarados: Vec<String>, // Nombres de tipos de robot
    pub robots_definidos: Vec<Robot>,   // Definiciones completas de robots
    pub robots_instanciados: Vec<RobotInstanciado>, // Robots declarados en sección variables
    pub asignaciones_areas: Vec<AsignacionArea>, // Asignaciones de área en el main
    pub inicializaciones: Vec<InicializacionRobot>, // Inicializaciones de posición
}

#[derive(Debug, Clone)]
pub struct Proceso {
    pub nombre: String,
    pub parametros: Vec<Parametro>,
    pub variables: Vec<Variable>,
    pub instrucciones: Vec<Instruccion>,
}

#[derive(Debug, Clone)]
pub struct Parametro {
    pub tipo: String, // "E", "S", "ES"
    pub nombre: String,
    pub tipo_dato: String,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub nombre: String,
    pub tipo_dato: String,
}

#[derive(Debug, Clone)]
pub struct Area {
    pub nombre: String,
    pub tipo: String,
    pub coordenadas: (i32, i32, i32, i32),
}

#[derive(Debug, Clone)]
pub struct Robot {
    pub nombre: String,
    pub variables: Vec<Variable>,
    pub instrucciones: Vec<Instruccion>,
}

#[derive(Debug, Clone)]
pub enum Instruccion {
    Asignacion { variable: String, valor: Expresion },
    LlamadaFuncion { nombre: String, argumentos: Vec<Expresion> },
    Si { condicion: Expresion, entonces: Vec<Instruccion>, sino: Vec<Instruccion> },
    Mientras { condicion: Expresion, cuerpo: Vec<Instruccion> },
    Repetir { condicion: Expresion, cuerpo: Vec<Instruccion> },
}

#[derive(Debug, Clone,PartialEq)]
pub enum Expresion {
    Identificador(String),
    Numero(i32),
    Booleano(bool),
    Binaria { izquierda: Box<Expresion>, operador: String, derecha: Box<Expresion> },
}

pub struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
    current: Option<&'a Token>,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        let mut parser = Self {
            tokens,
            pos: 0,
            current: None,
        };
        parser.avanzar();
        parser
    }
    
    fn avanzar(&mut self) {
        if self.pos < self.tokens.len() {
            self.current = Some(&self.tokens[self.pos]);
            self.pos += 1;
        } else {
            self.current = None;
        }
    }
    
    fn coincidir(&mut self, tipo: TokenType) -> bool {
        if let Some(token) = self.current {
            token.token_type == tipo
        } else {
            false
        }
    }
    
    fn consumir(&mut self, tipo: TokenType, mensaje: &str) -> Result<(), CompilerError> {
        if self.coincidir(tipo) {
            self.avanzar();
            Ok(())
        } else {
            let token = self.current.unwrap();
            Err(CompilerError::new(
                format!("{}: esperado {:?}", mensaje, tipo),
                token.line,
                token.column
            ))
        }
    }
    
    pub fn parse(&mut self) -> Result<Program, CompilerError> {
        self.parse_programa()
    }
    
    fn parse_programa(&mut self) -> Result<Program, CompilerError> {
        // programa nombre
        self.consumir(TokenType::Keyword, "Esperado 'programa'")?;
        let nombre = if let Some(token) = self.current {
            let nombre = token.value.clone();
            self.avanzar();
            nombre
        } else {
            return Err(CompilerError::new("Esperado nombre del programa", 0, 0));
        };
        
        let mut procesos = Vec::new();
        let mut areas = Vec::new();
        let mut robots_declarados = Vec::new();
        let mut robots_definidos = Vec::new();
        let mut robots_instanciados: Vec<RobotInstanciado> = Vec::new(); // Nuevo: robots declarados en sección variables
        
        // Parsear secciones
        while let Some(token) = self.current {
            match token.token_type {
                TokenType::Keyword => match token.value.as_str() {
                    "procesos" => {
                        self.avanzar(); // consumir "procesos"
                        procesos = self.parse_procesos()?;
                    }
                    "areas" => {
                        self.avanzar(); // consumir "areas"
                        areas = self.parse_areas()?;
                    }
                    "robots" => {
                        self.avanzar(); // consumir "robots"
                        let (declarados, definidos) = self.parse_robots()?;
                        robots_declarados = declarados;
                        robots_definidos = definidos;
                    }
                    "variables" => {
                        self.avanzar(); // consumir "variables"
                        
                        // Parsear declaraciones de variables globales (instanciación de robots)
                        while let Some(t) = self.current {
                            // Saltar indentación
                            if t.token_type == TokenType::Indent || t.token_type == TokenType::Dedent {
                                self.avanzar();
                                continue;
                            }
                            
                            // Si encontramos "comenzar", terminamos la sección de variables
                            if t.token_type == TokenType::Keyword && t.value == "comenzar" {
                                break;
                            }
                            
                            // Parsear declaración de robot: nombre_instancia : tipo_robot
                            if t.token_type == TokenType::Identifier {
                                let nombre_instancia = t.value.clone();
                                self.avanzar();
                                
                                // Verificar que siga el operador de declaración
                                if let Some(next_token) = self.current {
                                    if next_token.token_type == TokenType::Declaration {
                                        self.avanzar(); // consumir ":"
                                        
                                        // Obtener el tipo de robot
                                        if let Some(tipo_token) = self.current {
                                            if tipo_token.token_type == TokenType::Identifier {
                                                let tipo_robot = tipo_token.value.clone();
                                                self.avanzar();
                                                
                                                // Verificar que el tipo de robot esté definido
                                                if !robots_declarados.contains(&tipo_robot) {
                                                    return Err(CompilerError::new(
                                                        format!("Tipo de robot no definido: {}", tipo_robot),
                                                        tipo_token.line,
                                                        tipo_token.column
                                                    ));
                                                }
                                                
                                                robots_instanciados.push(RobotInstanciado {
                                                    nombre: nombre_instancia,
                                                    tipo: tipo_robot,
                                                });
                                            } else {
                                                return Err(CompilerError::new(
                                                    "Esperado tipo de robot después de ':'",
                                                    tipo_token.line,
                                                    tipo_token.column
                                                ));
                                            }
                                        } else {
                                            return Err(CompilerError::new(
                                                "Declaración de robot incompleta",
                                                t.line,
                                                t.column
                                            ));
                                        }
                                    } else {
                                        return Err(CompilerError::new(
                                            "Esperado ':' en declaración de robot",
                                            next_token.line,
                                            next_token.column
                                        ));
                                    }
                                } else {
                                    return Err(CompilerError::new(
                                        "Declaración de robot incompleta",
                                        t.line,
                                        t.column
                                    ));
                                }
                            } else {
                                self.avanzar(); // saltar otros tokens
                            }
                        }
                    }
                    "comenzar" => break, // Salir para parsear instrucciones principales
                    _ => self.avanzar(),
                }
                TokenType::Indent | TokenType::Dedent => {
                    self.avanzar(); // ignorar indentación
                }
                _ => break,
            }
        }
        
        // Parsear bloque principal (instrucciones después de "comenzar")
        let mut instrucciones_principales = Vec::new();
        let mut asignaciones_areas = Vec::new();
        let mut inicializaciones = Vec::new();
        
        if let Some(token) = self.current {
            if token.token_type == TokenType::Keyword && token.value == "comenzar" {
                self.avanzar(); // consumir "comenzar"
                while let Some(token) = self.current {
                    if token.token_type == TokenType::Keyword && token.value == "fin" {
                        self.avanzar();
                        break;
                    } else if token.token_type == TokenType::Indent || 
                            token.token_type == TokenType::Dedent {
                        self.avanzar();
                    } else {
                        if let Ok(instr) = self.parse_instruccion() {
                            // Clasificar las instrucciones principales
                            match &instr {
                                Instruccion::LlamadaFuncion { nombre, argumentos } => {
                                    if nombre == "AsignarArea" && argumentos.len() == 2 {
                                        // Capturar asignación de área
                                        asignaciones_areas.push(AsignacionArea {
                                            robot: argumentos[0].clone(),
                                            area: argumentos[1].clone(),
                                        });
                                    } else if nombre == "Iniciar" && argumentos.len() == 3 {
                                        // Capturar inicialización de robot
                                        inicializaciones.push(InicializacionRobot {
                                            robot: argumentos[0].clone(),
                                            pos_x: argumentos[1].clone(),
                                            pos_y: argumentos[2].clone(),
                                        });
                                    }
                                    instrucciones_principales.push(instr);
                                }
                                _ => {
                                    instrucciones_principales.push(instr);
                                }
                            }
                        } else {
                            self.avanzar(); // saltar si hay error
                        }
                    }
                }
            }
        }
        
        // Validar que todos los robots instanciados tengan asignación de área e inicialización
        for robot in &robots_instanciados {
            let nombre_robot_exp = Expresion::Identificador(robot.nombre.clone());
            
            // Verificar asignación de área
            let tiene_asignacion_area = asignaciones_areas.iter()
                .any(|asig| asig.robot == nombre_robot_exp);
            
            if !tiene_asignacion_area {
                println!("Advertencia: Robot '{}' no tiene asignación de área", robot.nombre);
            }
            
            // Verificar inicialización
            let tiene_inicializacion = inicializaciones.iter()
                .any(|init| init.robot == nombre_robot_exp);
            
            if !tiene_inicializacion {
                println!("Advertencia: Robot '{}' no tiene inicialización", robot.nombre);
            }
        }
        
        if !instrucciones_principales.is_empty() {
            robots_definidos.push(Robot {
                nombre: "main".to_string(),
                variables: Vec::new(),
                instrucciones: instrucciones_principales,
            });
        }
        
        Ok(Program {
            nombre,
            procesos,
            areas,
            robots_declarados,
            robots_definidos,
            robots_instanciados: robots_instanciados,
            asignaciones_areas: asignaciones_areas,
            inicializaciones: inicializaciones,
        })
    }
    
    fn parse_procesos(&mut self) -> Result<Vec<Proceso>, CompilerError> {
        let mut procesos = Vec::new();
        
        while let Some(token) = self.current {
            if (token.token_type == TokenType::Indent) || (token.token_type == TokenType::Dedent){
                self.avanzar();
            } else if token.token_type == TokenType::Keyword && token.value == "proceso" {
                procesos.push(self.parse_proceso()?);
            } else {
                break;
            }
        }
        
        Ok(procesos)
    }
    
    fn parse_proceso(&mut self) -> Result<Proceso, CompilerError> {
        self.consumir(TokenType::Keyword, "Esperado 'proceso'")?;
        
        let nombre = if let Some(token) = self.current {
            let nombre = token.value.clone();
            self.avanzar();
            nombre
        } else {
            return Err(CompilerError::new("Esperado nombre del proceso", 0, 0));
        };
        
        // Parámetros
        let mut parametros = Vec::new();
        if self.coincidir(TokenType::OpenedParenthesis) {
            self.avanzar(); // consumir '('
            
            while let Some(token) = self.current {
                if token.token_type == TokenType::ClosedParenthesis {
                    self.avanzar();
                    break;
                }
                
                // Tipo de parámetro (E, S, ES)
                let tipo_param = if token.token_type == TokenType::ParameterType {
                    let tipo = token.value.clone();
                    self.avanzar();
                    tipo
                } else {
                    "E".to_string() // Por defecto
                };
                
                // Nombre del parámetro
                let nombre_param = if let Some(t) = self.current {
                    let nombre = t.value.clone();
                    self.avanzar();
                    nombre
                } else {
                    return Err(CompilerError::new("Esperado nombre del parámetro", 0, 0));
                };
                
                // Tipo de dato
                self.consumir(TokenType::Declaration, "Esperado ':'")?;
                let tipo_dato = if let Some(t) = self.current {
                    let tipo = t.value.clone();
                    self.avanzar();
                    tipo
                } else {
                    return Err(CompilerError::new("Esperado tipo de dato", 0, 0));
                };
                
                parametros.push(Parametro {
                    tipo: tipo_param,
                    nombre: nombre_param,
                    tipo_dato,
                });
                
                // Verificar si hay más parámetros
                if let Some(t) = self.current {
                    if t.token_type == TokenType::Comma {
                        self.avanzar();
                    }
                }
            }
        }
        
        // Variables
        let mut variables = Vec::new();
        if let Some(token) = self.current {
            if token.token_type == TokenType::Keyword && token.value == "variables" {
                self.avanzar(); // consumir "variables"
                
                while let Some(token) = self.current {
                    if token.token_type == TokenType::Keyword && token.value == "comenzar" {
                        break;
                    } else if token.token_type == TokenType::Indent || 
                              token.token_type == TokenType::Dedent {
                        self.avanzar();
                    } else if token.token_type == TokenType::Identifier {
                        variables.push(self.parse_variable()?);
                    } else {
                        self.avanzar(); // saltar otros tokens
                    }
                }
            }
        }
        
        // Instrucciones
        let mut instrucciones = Vec::new();
        if let Some(token) = self.current {
            if token.token_type == TokenType::Keyword && token.value == "comenzar" {
                self.avanzar(); // consumir "comenzar"
                
                while let Some(token) = self.current {
                    if token.token_type == TokenType::Keyword && token.value == "fin" {
                        self.avanzar();
                        break;
                    } else if token.token_type == TokenType::Indent || 
                              token.token_type == TokenType::Dedent {
                        self.avanzar();
                    } else {
                        if let Ok(instr) = self.parse_instruccion() {
                            instrucciones.push(instr);
                        } else {
                            self.avanzar(); // saltar si hay error
                        }
                    }
                }
            }
        }
        
        Ok(Proceso {
            nombre,
            parametros,
            variables,
            instrucciones,
        })
    }
    
    fn parse_variable(&mut self) -> Result<Variable, CompilerError> {
        let nombre = if let Some(token) = self.current {
            let nombre = token.value.clone();
            self.avanzar();
            nombre
        } else {
            return Err(CompilerError::new("Esperado nombre de variable", 0, 0));
        };
        
        self.consumir(TokenType::Declaration, "Esperado ':'")?;
        
        let tipo_dato = if let Some(token) = self.current {
            let tipo = token.value.clone();
            self.avanzar();
            tipo
        } else {
            return Err(CompilerError::new("Esperado tipo de dato", 0, 0));
        };
        
        Ok(Variable { nombre, tipo_dato })
    }
    
    fn parse_areas(&mut self) -> Result<Vec<Area>, CompilerError> {
        let mut areas = Vec::new();
        
        while let Some(token) = self.current {
            if token.token_type == TokenType::Identifier {
                let nombre = token.value.clone();
                self.avanzar();
                
                self.consumir(TokenType::Declaration, "Esperado ':'")?;
                
                let tipo = if let Some(t) = self.current {
                    let tipo = t.value.clone();
                    self.avanzar();
                    tipo
                } else {
                    return Err(CompilerError::new("Esperado tipo de área", 0, 0));
                };
                
                self.consumir(TokenType::OpenedParenthesis, "Esperado '('")?;
                
                // Leer 4 números
                let mut nums = Vec::new();
                for _ in 0..4 {
                    if let Some(t) = self.current {
                        if t.token_type == TokenType::Num {
                            let num = t.value.parse::<i32>().unwrap_or(0);
                            nums.push(num);
                            self.avanzar();
                            
                            // Consumir coma si no es el último
                            if nums.len() < 4 {
                                if let Some(next) = self.current {
                                    if next.token_type == TokenType::Comma {
                                        self.avanzar();
                                    }
                                }
                            }
                        } else {
                            break;
                        }
                    }
                }
                
                self.consumir(TokenType::ClosedParenthesis, "Esperado ')'")?;
                
                if nums.len() == 4 {
                    areas.push(Area {
                        nombre,
                        tipo,
                        coordenadas: (nums[0], nums[1], nums[2], nums[3]),
                    });
                }
            } else if token.token_type == TokenType::Indent || 
                      token.token_type == TokenType::Dedent {
                self.avanzar();
            } else {
                break;
            }
        }
        
        Ok(areas)
    }
    
    fn parse_robots(&mut self) -> Result<(Vec<String>, Vec<Robot>), CompilerError> {
        let mut declarados = Vec::new();
        let mut definidos = Vec::new();
        
        while let Some(token) = self.current {
            if token.token_type == TokenType::Keyword && token.value == "robot" {
                self.avanzar(); // consumir "robot"
                
                // Nombre del robot
                let nombre = if let Some(t) = self.current {
                    let nombre = t.value.clone();
                    self.avanzar();
                    nombre
                } else {
                    return Err(CompilerError::new("Esperado nombre del robot", 0, 0));
                };
                
                declarados.push(nombre.clone());
                
                // Variables del robot
                let mut variables = Vec::new();
                if let Some(t) = self.current {
                    if t.token_type == TokenType::Keyword && t.value == "variables" {
                        self.avanzar(); // consumir "variables"
                        
                        while let Some(t) = self.current {
                            if t.token_type == TokenType::Keyword && t.value == "comenzar" {
                                break;
                            } else if t.token_type == TokenType::Indent || 
                                      t.token_type == TokenType::Dedent {
                                self.avanzar();
                            } else if t.token_type == TokenType::Identifier {
                                variables.push(self.parse_variable()?);
                            } else {
                                self.avanzar();
                            }
                        }
                    }
                }
                
                // Instrucciones del robot
                let mut instrucciones = Vec::new();
                if let Some(t) = self.current {
                    if t.token_type == TokenType::Keyword && t.value == "comenzar" {
                        self.avanzar(); // consumir "comenzar"
                        
                        while let Some(t) = self.current {
                            if t.token_type == TokenType::Keyword && t.value == "fin" {
                                self.avanzar();
                                break;
                            } else if t.token_type == TokenType::Indent || 
                                      t.token_type == TokenType::Dedent {
                                self.avanzar();
                            } else {
                                if let Ok(instr) = self.parse_instruccion() {
                                    instrucciones.push(instr);
                                } else {
                                    self.avanzar();
                                }
                            }
                        }
                    }
                }
                
                definidos.push(Robot {
                    nombre,
                    variables,
                    instrucciones,
                });
            } else if token.token_type == TokenType::Indent || 
                      token.token_type == TokenType::Dedent {
                self.avanzar();
            } else {
                break;
            }
        }
        
        Ok((declarados, definidos))
    }
    
    fn parse_instruccion(&mut self) -> Result<Instruccion, CompilerError> {
        if let Some(token) = self.current {
            let start_line = token.line; // Guardar línea inicial
            
            match token.token_type {
                TokenType::Identifier => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    
                    // Verificar si es asignación
                    if let Some(t) = self.current {
                        if t.token_type == TokenType::Assign {
                            self.avanzar(); // consumir ":="
                            
                            // Parsear expresión completa hasta cambio de línea o indentación
                            let valor = self.parse_expresion_linea_completa(start_line)?;
                            
                            Ok(Instruccion::Asignacion {
                                variable: nombre,
                                valor,
                            })
                        } else {
                            // Llamada a función
                            let argumentos = if self.coincidir(TokenType::OpenedParenthesis) {
                                self.avanzar(); // consumir '('
                                let args = self.parse_lista_argumentos()?;
                                self.consumir(TokenType::ClosedParenthesis, "Esperado ')'")?;
                                args
                            } else {
                                Vec::new()
                            };
                            
                            Ok(Instruccion::LlamadaFuncion {
                                nombre,
                                argumentos,
                            })
                        }
                    } else {
                        // Solo identificador sin operador (instrucción simple)
                        Ok(Instruccion::LlamadaFuncion {
                            nombre,
                            argumentos: Vec::new(),
                        })
                    }
                }
                TokenType::ElementalInstruction => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    
                    let argumentos = if self.coincidir(TokenType::OpenedParenthesis) {
                        self.avanzar(); // consumir '('
                        let args = self.parse_lista_argumentos()?;
                        self.consumir(TokenType::ClosedParenthesis, "Esperado ')'")?;
                        args
                    } else {
                        Vec::new()
                    };
                    
                    Ok(Instruccion::LlamadaFuncion {
                        nombre,
                        argumentos,
                    })
                }
                TokenType::ControlSentence => match token.value.as_str() {
                    "si" => self.parse_si(),
                    "mientras" => self.parse_mientras(),
                    "repetir" => self.parse_repetir(),
                    _ => Err(CompilerError::new(
                        format!("Instrucción de control desconocida: {}", token.value),
                        token.line,
                        token.column
                    )),
                }
                _ => Err(CompilerError::new(
                    format!("Instrucción no reconocida: {:?}", token.token_type),
                    token.line,
                    token.column
                )),
            }
        } else {
            Err(CompilerError::new("Se esperaba una instrucción", 0, 0))
        }
    }

    // Nueva función para parsear expresión completa en una línea
    fn parse_expresion_linea_completa(&mut self, start_line: usize) -> Result<Expresion, CompilerError> {
        // Parsear la primera parte de la expresión
        let mut expr = self.parse_expresion_simple()?;
        
        // Continuar parseando mientras estemos en la misma línea
        while let Some(token) = self.current {
            // Verificar si seguimos en la misma línea
            if token.line != start_line {
                break;
            }
            
            // Verificar si es fin de instrucción
            if token.token_type == TokenType::Indent || 
            token.token_type == TokenType::Dedent ||
            token.token_type == TokenType::Keyword ||
            (token.token_type == TokenType::ControlSentence && 
                (token.value == "si" || token.value == "mientras" || token.value == "repetir" || token.value == "sino")) {
                break;
            }
            
            // Si encontramos un operador, parsear como expresión binaria
            if self.es_operador_binario(token) {
                let operador = self.parse_operador_binario()?;
                let derecha = self.parse_expresion_simple()?;
                
                expr = Expresion::Binaria {
                    izquierda: Box::new(expr),
                    operador,
                    derecha: Box::new(derecha),
                };
            } else {
                // Si no es operador, terminamos la expresión
                break;
            }
        }
        
        Ok(expr)
    }

    // Método para parsear expresión simple (sin operadores binarios)
    fn parse_expresion_simple(&mut self) -> Result<Expresion, CompilerError> {
        if let Some(token) = self.current {
            match token.token_type {
                TokenType::Identifier => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    Ok(Expresion::Identificador(nombre))
                }
                TokenType::Num => {
                    let valor = token.value.parse::<i32>().unwrap_or(0);
                    self.avanzar();
                    Ok(Expresion::Numero(valor))
                }
                TokenType::BoolValue => {
                    let valor = token.value == "V" ;
                    self.avanzar();
                    Ok(Expresion::Booleano(valor))
                }
                TokenType::ElementalInstruction => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    
                    let argumentos = if self.coincidir(TokenType::OpenedParenthesis) {
                        self.avanzar(); // consumir '('
                        let args = self.parse_lista_argumentos()?;
                        self.consumir(TokenType::ClosedParenthesis, "Esperado ')'")?;
                        args
                    } else {
                        Vec::new()
                    };
                    
                    // Convertir llamada a función a expresión
                    if argumentos.is_empty() {
                        Ok(Expresion::Identificador(nombre))
                    } else {
                        // Para simplificar, tratamos funciones con argumentos como identificadores
                        // En una implementación completa, necesitarías un nodo FunctionCall en Expresion
                        Ok(Expresion::Identificador(format!("{}(...)", nombre)))
                    }
                }
                TokenType::OpenedParenthesis => {
                    self.avanzar(); // consumir '('
                    let expr = self.parse_expresion_linea_completa(token.line)?;
                    self.consumir(TokenType::ClosedParenthesis, "Esperado ')'")?;
                    Ok(expr)
                }
                _ => Err(CompilerError::new(
                    format!("Expresión simple no válida: {:?}", token.token_type),
                    token.line,
                    token.column
                )),
            }
        } else {
            Err(CompilerError::new("Se esperaba una expresión simple", 0, 0))
        }
    }

    // Verificar si el token es un operador binario
    fn es_operador_binario(&self, token: &Token) -> bool {
        matches!(token.token_type,
            TokenType::Plus |
            TokenType::Minus |
            TokenType::Multiply |
            TokenType::Divide |
            TokenType::Less |
            TokenType::LessEqual |
            TokenType::Greater |
            TokenType::GreaterEqual |
            TokenType::Equals |
            TokenType::NotEquals |
            TokenType::And |
            TokenType::Or
        )
    }

    // Parsear operador binario
    fn parse_operador_binario(&mut self) -> Result<String, CompilerError> {
        if let Some(token) = self.current {
            if self.es_operador_binario(token) {
                let operador = match token.token_type {
                    TokenType::Plus => "+",
                    TokenType::Minus => "-",
                    TokenType::Multiply => "*",
                    TokenType::Divide => "/",
                    TokenType::Less => "<",
                    TokenType::LessEqual => "<=",
                    TokenType::Greater => ">",
                    TokenType::GreaterEqual => ">=",
                    TokenType::Equals => "==",
                    TokenType::NotEquals => "<>",
                    TokenType::And => "&",
                    TokenType::Or => "|",
                    _ => return Err(CompilerError::new(
                        "Operador no reconocido",
                        token.line,
                        token.column
                    )),
                };
                
                self.avanzar();
                Ok(operador.to_string())
            } else {
                Err(CompilerError::new(
                    format!("Se esperaba operador binario, encontrado: {:?}", token.token_type),
                    token.line,
                    token.column
                ))
            }
        } else {
            Err(CompilerError::new("Se esperaba operador binario", 0, 0))
        }
    }
    
    fn parse_si(&mut self) -> Result<Instruccion, CompilerError> {
        self.avanzar(); // consumir "si"
        
        let condicion = self.parse_expresion()?;
        
        let mut entonces = Vec::new();
        while let Some(token) = self.current {
            if token.token_type == TokenType::ControlSentence && token.value == "sino" {
                self.avanzar(); // consumir "sino"
                break;
            } else if token.token_type == TokenType::Dedent {
                break;
            } else if token.token_type == TokenType::Indent || 
                      token.token_type == TokenType::Dedent {
                self.avanzar();
            } else {
                if let Ok(instr) = self.parse_instruccion() {
                    entonces.push(instr);
                } else {
                    self.avanzar();
                }
            }
        }
        
        let mut sino = Vec::new();
        while let Some(token) = self.current {
            if token.token_type == TokenType::Dedent {
                break;
            } else if token.token_type == TokenType::Indent || 
                      token.token_type == TokenType::Dedent {
                self.avanzar();
            } else {
                if let Ok(instr) = self.parse_instruccion() {
                    sino.push(instr);
                } else {
                    self.avanzar();
                }
            }
        }
        
        Ok(Instruccion::Si {
            condicion,
            entonces,
            sino,
        })
    }
    
    fn parse_mientras(&mut self) -> Result<Instruccion, CompilerError> {
        self.avanzar(); // consumir "mientras"
        
        let condicion = if self.coincidir(TokenType::OpenedParenthesis) {
            self.avanzar(); // consumir '('
            let cond = self.parse_expresion()?;
            self.consumir(TokenType::ClosedParenthesis, "Esperado ')'")?;
            cond
        } else {
            self.parse_expresion()?
        };
        
        let mut cuerpo = Vec::new();
        while let Some(token) = self.current {
            if token.token_type == TokenType::Dedent {
                break;
            } else if token.token_type == TokenType::Indent || 
                      token.token_type == TokenType::Dedent {
                self.avanzar();
            } else {
                if let Ok(instr) = self.parse_instruccion() {
                    cuerpo.push(instr);
                } else {
                    self.avanzar();
                }
            }
        }
        
        Ok(Instruccion::Mientras { condicion, cuerpo })
    }
    
    fn parse_repetir(&mut self) -> Result<Instruccion, CompilerError> {
        self.avanzar(); // consumir "repetir"
        
        let condicion = self.parse_expresion()?;
        
        let mut cuerpo = Vec::new();
        while let Some(token) = self.current {
            if token.token_type == TokenType::Dedent {
                break;
            } else if token.token_type == TokenType::Indent || 
                      token.token_type == TokenType::Dedent {
                self.avanzar();
            } else {
                if let Ok(instr) = self.parse_instruccion() {
                    cuerpo.push(instr);
                } else {
                    self.avanzar();
                }
            }
        }
        
        Ok(Instruccion::Repetir { condicion, cuerpo })
    }
    
    // Método parse_expresion original modificado para usar la nueva implementación
    fn parse_expresion(&mut self) -> Result<Expresion, CompilerError> {
        if let Some(token) = self.current {
            self.parse_expresion_linea_completa(token.line)
        } else {
            Err(CompilerError::new("Se esperaba una expresión", 0, 0))
        }
    }
    
    fn parse_lista_argumentos(&mut self) -> Result<Vec<Expresion>, CompilerError> {
        let mut argumentos = Vec::new();
        
        while let Some(token) = self.current {
            if token.token_type == TokenType::ClosedParenthesis {
                break;
            }
            
            argumentos.push(self.parse_expresion()?);
            
            if let Some(t) = self.current {
                if t.token_type == TokenType::Comma {
                    self.avanzar();
                }
            }
        }
        
        Ok(argumentos)
    }
}