use crate::lib::compilerError::CompilerError;
use crate::lib::lexer::token::{Token, TokenType};
use crate::lib::parser::ast::*;

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
        let mut robots_instanciados = Vec::new();
        
        while let Some(token) = self.current {
            match token.token_type {
                TokenType::Keyword => match token.value.as_str() {
                    "procesos" => {
                        self.avanzar();
                        procesos = self.parse_procesos()?;
                    }
                    "areas" => {
                        self.avanzar();
                        areas = self.parse_areas()?;
                    }
                    "robots" => {
                        self.avanzar();
                        let (declarados, definidos) = self.parse_robots()?;
                        robots_declarados = declarados;
                        robots_definidos = definidos;
                    }
                    "variables" => {
                        self.avanzar();
                        
                        while let Some(t) = self.current {
                            if t.token_type == TokenType::Indent || t.token_type == TokenType::Dedent {
                                self.avanzar();
                                continue;
                            }
                            
                            if t.token_type == TokenType::Keyword && t.value == "comenzar" {
                                break;
                            }
                            
                            if t.token_type == TokenType::Identifier {
                                let nombre_instancia = t.value.clone();
                                self.avanzar();
                                
                                if let Some(next_token) = self.current {
                                    if next_token.token_type == TokenType::Declaration {
                                        self.avanzar();
                                        
                                        if let Some(tipo_token) = self.current {
                                            if tipo_token.token_type == TokenType::Identifier {
                                                let tipo_robot = tipo_token.value.clone();
                                                self.avanzar();
                                                
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
                                self.avanzar();
                            }
                        }
                    }
                    "comenzar" => break,
                    _ => self.avanzar(),
                }
                TokenType::Indent | TokenType::Dedent => {
                    self.avanzar();
                }
                _ => break,
            }
        }
        
        let mut instrucciones_principales = Vec::new();
        let mut asignaciones_areas = Vec::new();
        let mut inicializaciones = Vec::new();
        
        if let Some(token) = self.current {
            if token.token_type == TokenType::Keyword && token.value == "comenzar" {
                self.avanzar();
                while let Some(token) = self.current {
                    if token.token_type == TokenType::Keyword && token.value == "fin" {
                        self.avanzar();
                        break;
                    } else if token.token_type == TokenType::Indent || 
                              token.token_type == TokenType::Dedent {
                        self.avanzar();
                    } else {
                        if let Ok(instr) = self.parse_instruccion() {
                            match &instr {
                                Instruccion::LlamadaFuncion { nombre, argumentos } => {
                                    if nombre == "AsignarArea" && argumentos.len() == 2 {
                                        asignaciones_areas.push(AsignacionArea {
                                            robot: argumentos[0].clone(),
                                            area: argumentos[1].clone(),
                                        });
                                    } else if nombre == "Iniciar" && argumentos.len() == 3 {
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
                            self.avanzar();
                        }
                    }
                }
            }
        }
        
        for robot in &robots_instanciados {
            let nombre_robot_exp = Expresion::Identificador(robot.nombre.clone());
            
            let tiene_asignacion_area = asignaciones_areas.iter()
                .any(|asig| asig.robot == nombre_robot_exp);
            
            if !tiene_asignacion_area {
                println!("Advertencia: Robot '{}' no tiene asignación de área", robot.nombre);
            }
            
            let tiene_inicializacion = inicializaciones.iter()
                .any(|init| init.robot == nombre_robot_exp);
            
            if !tiene_inicializacion {
                println!("Advertencia: Robot '{}' no tiene inicialización", robot.nombre);
            }
        }
        
        Ok(Program {
            nombre,
            procesos,
            areas,
            robots_declarados,
            robots_definidos,
            robots_instanciados,
            asignaciones_areas,
            inicializaciones,
        })
    }
    
    fn parse_procesos(&mut self) -> Result<Vec<Proceso>, CompilerError> {
        let mut procesos = Vec::new();
        
        while let Some(token) = self.current {
            if token.token_type == TokenType::Indent || token.token_type == TokenType::Dedent {
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
        
        let mut parametros = Vec::new();
        if self.coincidir(TokenType::OpenedParenthesis) {
            self.avanzar();
            
            while let Some(token) = self.current {
                if token.token_type == TokenType::ClosedParenthesis {
                    self.avanzar();
                    break;
                }
                
                let tipo_param = if token.token_type == TokenType::ParameterType {
                    let tipo = token.value.clone();
                    self.avanzar();
                    tipo
                } else {
                    "E".to_string()
                };
                
                let nombre_param = if let Some(t) = self.current {
                    let nombre = t.value.clone();
                    self.avanzar();
                    nombre
                } else {
                    return Err(CompilerError::new("Esperado nombre del parámetro", 0, 0));
                };
                
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
                
                if let Some(t) = self.current {
                    if t.token_type == TokenType::Comma {
                        self.avanzar();
                    }
                }
            }
        }
        
        let mut variables = Vec::new();
        if let Some(token) = self.current {
            if token.token_type == TokenType::Keyword && token.value == "variables" {
                self.avanzar();
                
                while let Some(token) = self.current {
                    if token.token_type == TokenType::Keyword && token.value == "comenzar" {
                        break;
                    } else if token.token_type == TokenType::Indent || 
                              token.token_type == TokenType::Dedent {
                        self.avanzar();
                    } else if token.token_type == TokenType::Identifier {
                        variables.push(self.parse_variable()?);
                    } else {
                        self.avanzar();
                    }
                }
            }
        }
        
        let mut instrucciones = Vec::new();
        if let Some(token) = self.current {
            if token.token_type == TokenType::Keyword && token.value == "comenzar" {
                self.avanzar();
                
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
                            self.avanzar();
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
                
                let mut nums = Vec::new();
                for _ in 0..4 {
                    if let Some(t) = self.current {
                        if t.token_type == TokenType::Num {
                            let num = t.value.parse::<i32>().unwrap_or(0);
                            nums.push(num);
                            self.avanzar();
                            
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
                self.avanzar();
                
                let nombre = if let Some(t) = self.current {
                    let nombre = t.value.clone();
                    self.avanzar();
                    nombre
                } else {
                    return Err(CompilerError::new("Esperado nombre del robot", 0, 0));
                };
                
                declarados.push(nombre.clone());
                
                let mut variables = Vec::new();
                if let Some(t) = self.current {
                    if t.token_type == TokenType::Keyword && t.value == "variables" {
                        self.avanzar();
                        
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
                
                let mut instrucciones = Vec::new();
                if let Some(t) = self.current {
                    if t.token_type == TokenType::Keyword && t.value == "comenzar" {
                        self.avanzar();
                        
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
            let start_line = token.line;
            
            match token.token_type {
                TokenType::Identifier => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    
                    if let Some(t) = self.current {
                        if t.token_type == TokenType::Assign {
                            self.avanzar();
                            
                            let expresion_texto = self.capturar_expresion_texto(start_line)?;
                            
                            Ok(Instruccion::Asignacion {
                                variable: nombre,
                                expresion_texto,
                            })
                        } else {
                            let argumentos = if self.coincidir(TokenType::OpenedParenthesis) {
                                self.avanzar();
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
                        Ok(Instruccion::LlamadaFuncion {
                            nombre,
                            argumentos: Vec::new(),
                        })
                    }
                }
                TokenType::ElementalInstruction => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    
                    if self.es_instruccion_elemental(&nombre) {
                        Ok(Instruccion::Elemental { nombre })
                    } else {
                        let argumentos = if self.coincidir(TokenType::OpenedParenthesis) {
                            self.avanzar();
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

    fn capturar_expresion_texto(&mut self, start_line: usize) -> Result<String, CompilerError> {
        let mut expresion_parts = Vec::new();
        let mut parentesis_count = 0;
        
        while let Some(token) = self.current {
            if token.line != start_line {
                break;
            }
            
            if token.token_type == TokenType::Indent || 
               token.token_type == TokenType::Dedent ||
               token.token_type == TokenType::Keyword ||
               (token.token_type == TokenType::ControlSentence && 
                (token.value == "si" || token.value == "mientras" || token.value == "repetir" || token.value == "sino")) {
                break;
            }
            
            if token.token_type == TokenType::OpenedParenthesis {
                parentesis_count += 1;
            } else if token.token_type == TokenType::ClosedParenthesis {
                if parentesis_count > 0 {
                    parentesis_count -= 1;
                }
            }
            
            expresion_parts.push(token.value.clone());
            self.avanzar();
            
            if parentesis_count > 0 {
                continue;
            }
        }
        
        let expresion_completa = expresion_parts.join(" ").trim().to_string();
        
        if expresion_completa.is_empty() {
            Err(CompilerError::new("Expresión vacía después de ':='", 0, 0))
        } else {
            Ok(expresion_completa)
        }
    }

    fn capturar_condicion_texto(&mut self, start_line: usize) -> Result<String, CompilerError> {
        let mut condicion_parts = Vec::new();
        let mut parentesis_count = 0;
        
        while let Some(token) = self.current {
            if token.token_type == TokenType::OpenedParenthesis && parentesis_count == 0 && !condicion_parts.is_empty() {
                parentesis_count += 1;
            } else if token.token_type == TokenType::ClosedParenthesis && parentesis_count > 0 {
                parentesis_count -= 1;
            }
            
            if token.token_type == TokenType::Indent || 
               token.token_type == TokenType::Dedent ||
               token.token_type == TokenType::Keyword ||
               (token.token_type == TokenType::ControlSentence && 
                token.value != "si" && token.value != "mientras" && token.value != "repetir") {
                break;
            }
            
            condicion_parts.push(token.value.clone());
            self.avanzar();
            
            if parentesis_count > 0 {
                continue;
            }
            
            if token.line != start_line && parentesis_count == 0 {
                break;
            }
        }
        
        let condicion_completa = condicion_parts.join(" ").trim().to_string();
        
        if condicion_completa.is_empty() {
            Err(CompilerError::new("Condición vacía", 0, 0))
        } else {
            Ok(condicion_completa)
        }
    }

    fn es_instruccion_elemental(&self, nombre: &str) -> bool {
        matches!(nombre,
            "HayFlorEnLaBolsa" |
            "HayPapelEnLaBolsa" |
            "HayFlorEnLaEsquina" |
            "HayPapelEnLaEsquina"
        )
    }

    fn parse_si(&mut self) -> Result<Instruccion, CompilerError> {
        self.avanzar();
        
        let condicion_texto = self.capturar_condicion_texto(self.current.map_or(0, |t| t.line))?;
        
        let mut entonces = Vec::new();
        while let Some(token) = self.current {
            if token.token_type == TokenType::ControlSentence && token.value == "sino" {
                self.avanzar();
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
            condicion_texto,
            entonces,
            sino,
        })
    }
    
    fn parse_mientras(&mut self) -> Result<Instruccion, CompilerError> {
        self.avanzar();
        
        let condicion_texto = self.capturar_condicion_texto(self.current.map_or(0, |t| t.line))?;
        
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
        
        Ok(Instruccion::Mientras { condicion_texto, cuerpo })
    }
    
    fn parse_repetir(&mut self) -> Result<Instruccion, CompilerError> {
        self.avanzar();
        
        let condicion_texto = self.capturar_condicion_texto(self.current.map_or(0, |t| t.line))?;
        
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
        
        Ok(Instruccion::Repetir { condicion_texto, cuerpo })
    }

    // Método simplificado para parsear expresiones simples (para argumentos de funciones)
    fn parse_expresion_simple(&mut self) -> Result<Expresion, CompilerError> {
        if let Some(token) = self.current {
            match token.token_type {
                TokenType::ElementalInstruction => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    
                    if self.es_instruccion_elemental(&nombre) {
                        Ok(Expresion::Elemental { nombre: nombre.clone() })
                    } else {
                        let argumentos = if self.coincidir(TokenType::OpenedParenthesis) {
                            self.avanzar();
                            let args = self.parse_lista_argumentos()?;
                            self.consumir(TokenType::ClosedParenthesis, "Esperado ')'")?;
                            args
                        } else {
                            Vec::new()
                        };
                        
                        if argumentos.is_empty() {
                            Ok(Expresion::Identificador(nombre))
                        } else {
                            Ok(Expresion::Identificador(format!("{}(...)", nombre)))
                        }
                    }
                },
                TokenType::Identifier => {
                    let nombre = token.value.clone();
                    self.avanzar();
                    Ok(Expresion::Identificador(nombre))
                },
                TokenType::Num => {
                    let valor = token.value.parse::<i32>().unwrap_or(0);
                    self.avanzar();
                    Ok(Expresion::Numero(valor))
                },
                TokenType::BoolValue => {
                    let valor = token.value == "V";
                    self.avanzar();
                    Ok(Expresion::Booleano(valor))
                },
                TokenType::OpenedParenthesis => {
                    self.avanzar();
                    // Para expresiones entre paréntesis, parsear como identificador compuesto
                    let mut contenido = Vec::new();
                    let mut parentesis_count = 1;
                    
                    while let Some(t) = self.current {
                        if t.token_type == TokenType::OpenedParenthesis {
                            parentesis_count += 1;
                        } else if t.token_type == TokenType::ClosedParenthesis {
                            parentesis_count -= 1;
                            if parentesis_count == 0 {
                                self.avanzar();
                                break;
                            }
                        }
                        
                        contenido.push(t.value.clone());
                        self.avanzar();
                    }
                    
                    let contenido_str = contenido.join(" ");
                    Ok(Expresion::Identificador(format!("({})", contenido_str)))
                },
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

    // Método parse_expresion que ahora simplemente delega a parse_expresion_simple
    fn parse_expresion(&mut self) -> Result<Expresion, CompilerError> {
        self.parse_expresion_simple()
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