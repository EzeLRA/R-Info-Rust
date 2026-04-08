use std::collections::{HashMap, HashSet};
use crate::lib::compilerError::CompilerError;
use crate::lib::parser::ast::{Program, Proceso, Robot, Instruccion, Expresion};

pub struct SemanticAnalyzer {
    errores: Vec<CompilerError>,
    advertencias: Vec<String>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            errores: Vec::new(),
            advertencias: Vec::new(),
        }
    }
    
    pub fn analizar(&mut self, programa: &Program) -> Result<(), Vec<CompilerError>> {
        // 1. Analizar procesos (nombres únicos, parámetros únicos, variables únicas)
        let procesos_info = self.analizar_procesos(programa);
        
        // 2. Analizar robots (nombres únicos, variables únicas)
        self.analizar_robots(programa);
        
        // 3. Verificar invocaciones de procesos
        self.verificar_invocaciones_procesos(programa, &procesos_info);
        
        // 4. Verificar uso de variables locales
        self.verificar_variables_locales(programa);
        
        // 5. Verificar robots instanciados vs definidos
        self.verificar_instancias_robots(programa);
        
        // 6. Verificar asignaciones de área e inicializaciones
        self.verificar_configuracion_robots(programa);
        
        if self.errores.is_empty() {
            Ok(())
        } else {
            Err(self.errores.clone())
        }
    }
    
    fn analizar_procesos(&mut self, programa: &Program) -> HashMap<String, ProcesoInfo> {
        let mut procesos_info = HashMap::new();
        let mut nombres_procesos = HashSet::new();
        
        for proceso in &programa.procesos {
            // Verificar nombre único
            if nombres_procesos.contains(&proceso.nombre) {
                self.errores.push(CompilerError::new(
                    format!("Proceso '{}' declarado múltiples veces", proceso.nombre),
                    0, 0
                ));
                continue;
            }
            nombres_procesos.insert(proceso.nombre.clone());
            
            // Verificar parámetros únicos
            let mut nombres_parametros = HashSet::new();
            let mut parametros_info = Vec::new();
            
            for param in &proceso.parametros {
                if nombres_parametros.contains(&param.nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Parámetro '{}' duplicado en proceso '{}'", param.nombre, proceso.nombre),
                        0, 0
                    ));
                } else {
                    nombres_parametros.insert(param.nombre.clone());
                    parametros_info.push((param.nombre.clone(), param.tipo_dato.clone()));
                }
            }
            
            // Verificar variables locales únicas
            let mut nombres_variables = HashSet::new();
            for var in &proceso.variables {
                if nombres_variables.contains(&var.nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Variable '{}' declarada múltiples veces en proceso '{}'", 
                                var.nombre, proceso.nombre),
                        0, 0
                    ));
                } else {
                    nombres_variables.insert(var.nombre.clone());
                }
            }
            
            // Verificar que no haya conflicto entre parámetros y variables
            for param in &proceso.parametros {
                if nombres_variables.contains(&param.nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Conflicto: parámetro '{}' también declarado como variable en proceso '{}'",
                                param.nombre, proceso.nombre),
                        0, 0
                    ));
                }
            }
            
            procesos_info.insert(proceso.nombre.clone(), ProcesoInfo {
                parametros: parametros_info,
                variables: proceso.variables.clone(),
            });
        }
        
        procesos_info
    }
    
    fn analizar_robots(&mut self, programa: &Program) {
        let mut nombres_robots = HashSet::new();
        
        for robot in &programa.robots_definidos {
            // Verificar nombre único de robot
            if nombres_robots.contains(&robot.nombre) {
                self.errores.push(CompilerError::new(
                    format!("Robot '{}' definido múltiples veces", robot.nombre),
                    0, 0
                ));
            }
            nombres_robots.insert(robot.nombre.clone());
            
            // Verificar variables locales únicas en robot
            let mut nombres_variables = HashSet::new();
            for var in &robot.variables {
                if nombres_variables.contains(&var.nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Variable '{}' declarada múltiples veces en robot '{}'", 
                                var.nombre, robot.nombre),
                        0, 0
                    ));
                }
                nombres_variables.insert(var.nombre.clone());
            }
        }
    }
    
    fn verificar_invocaciones_procesos(&mut self, programa: &Program, 
                                      procesos_info: &HashMap<String, ProcesoInfo>) {
        let procesos_declarados: HashSet<String> = programa.procesos.iter()
            .map(|p| p.nombre.clone())
            .collect();
        
        // Verificar en robots
        for robot in &programa.robots_definidos {
            self.verificar_invocaciones_en_instrucciones(
                &robot.instrucciones, 
                &procesos_declarados, 
                &robot.nombre,
                procesos_info
            );
        }
    }
    
    fn verificar_invocaciones_en_instrucciones(&mut self, instrucciones: &[Instruccion], 
                                              procesos_declarados: &HashSet<String>, 
                                              contexto: &str,
                                              procesos_info: &HashMap<String, ProcesoInfo>) {
        for instruccion in instrucciones {
            match instruccion {
                Instruccion::LlamadaFuncion { nombre, argumentos } => {
                    // Verificar que el proceso llamado existe
                    if !procesos_declarados.contains(nombre) {
                        self.errores.push(CompilerError::new(
                            format!("Proceso '{}' no declarado (usado en '{}')", nombre, contexto),
                            0, 0
                        ));
                    } else {
                        // Verificar número de argumentos
                        if let Some(info) = procesos_info.get(nombre) {
                            if argumentos.len() != info.parametros.len() {
                                self.errores.push(CompilerError::new(
                                    format!("Proceso '{}' espera {} argumentos, se pasaron {} (en '{}')",
                                            nombre, info.parametros.len(), argumentos.len(), contexto),
                                    0, 0
                                ));
                            }
                        }
                        
                        // Verificar que no sea recursión directa
                        if nombre == contexto {
                            self.advertencias.push(format!(
                                "Recursión detectada: proceso '{}' se llama a sí mismo", nombre
                            ));
                        }
                    }
                }
                Instruccion::Si { entonces, sino, .. } => {
                    self.verificar_invocaciones_en_instrucciones(entonces, procesos_declarados, contexto, procesos_info);
                    self.verificar_invocaciones_en_instrucciones(sino, procesos_declarados, contexto, procesos_info);
                }
                Instruccion::Mientras { cuerpo, .. } => {
                    self.verificar_invocaciones_en_instrucciones(cuerpo, procesos_declarados, contexto, procesos_info);
                }
                Instruccion::Repetir { cuerpo, .. } => {
                    self.verificar_invocaciones_en_instrucciones(cuerpo, procesos_declarados, contexto, procesos_info);
                }
                _ => {}
            }
        }
    }
    
    fn verificar_variables_locales(&mut self, programa: &Program) {
        // Verificar variables en procesos
        for proceso in &programa.procesos {
            let mut ambito = HashMap::new();
            
            // Agregar parámetros al ámbito
            for param in &proceso.parametros {
                ambito.insert(param.nombre.clone(), param.tipo_dato.clone());
            }
            
            // Agregar variables locales al ámbito
            for var in &proceso.variables {
                ambito.insert(var.nombre.clone(), var.tipo_dato.clone());
            }
            
            // Verificar uso en instrucciones
            self.verificar_variables_en_instrucciones(&proceso.instrucciones, &ambito, &proceso.nombre);
        }
        
        // Verificar variables en robots
        for robot in &programa.robots_definidos {
            let mut ambito = HashMap::new();
            
            // Agregar variables del robot al ámbito
            for var in &robot.variables {
                ambito.insert(var.nombre.clone(), var.tipo_dato.clone());
            }
            
            // Verificar uso en instrucciones
            self.verificar_variables_en_instrucciones(&robot.instrucciones, &ambito, &robot.nombre);
        }
    }
    
    fn verificar_variables_en_instrucciones(&mut self, instrucciones: &[Instruccion], 
                                          ambito: &HashMap<String, String>, contexto: &str) {
        for instruccion in instrucciones {
            match instruccion {
                Instruccion::Asignacion { variable, expresion_texto } => {
                    // Verificar que la variable esté declarada
                    if !ambito.contains_key(variable) {
                        self.errores.push(CompilerError::new(
                            format!("Variable '{}' no declarada en '{}'", variable, contexto),
                            0, 0
                        ));
                    }
                    
                    // Verificar variables en la expresión texto
                    self.verificar_variables_en_texto(expresion_texto, ambito, contexto);
                }
                Instruccion::LlamadaFuncion { argumentos, .. } => {
                    for arg in argumentos {
                        self.verificar_variables_en_expresion(arg, ambito, contexto);
                    }
                }
                Instruccion::Si { condicion_texto, entonces, sino } => {
                    self.verificar_variables_en_texto(condicion_texto, ambito, contexto);
                    self.verificar_variables_en_instrucciones(entonces, ambito, contexto);
                    self.verificar_variables_en_instrucciones(sino, ambito, contexto);
                }
                Instruccion::Mientras { condicion_texto, cuerpo } => {
                    self.verificar_variables_en_texto(condicion_texto, ambito, contexto);
                    self.verificar_variables_en_instrucciones(cuerpo, ambito, contexto);
                }
                Instruccion::Repetir { condicion_texto, cuerpo } => {
                    self.verificar_variables_en_texto(condicion_texto, ambito, contexto);
                    self.verificar_variables_en_instrucciones(cuerpo, ambito, contexto);
                }
                Instruccion::Elemental { .. } => {
                    // Las instrucciones elementales no usan variables
                }
            }
        }
    }
    
    fn verificar_variables_en_texto(&mut self, texto: &str, ambito: &HashMap<String, String>, contexto: &str) {
        // Extraer posibles identificadores del texto
        let palabras: Vec<&str> = texto.split(|c: char| !c.is_alphanumeric() && c != '_' && c != '(' && c != ')')
                                       .filter(|s| !s.is_empty())
                                       .collect();
        
        for palabra in palabras {
            // Si parece un identificador y no es un número
            if palabra.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') {
                // Verificar si es un valor booleano literal (V o F)
                if self.es_valor_booleano_literal(palabra) {
                    continue; // No generar advertencia para V o F
                }
                
                if palabra.parse::<i32>().is_err() && 
                   !self.es_instruccion_elemental(palabra) &&
                   !ambito.contains_key(palabra) {
                    // Podría ser una variable no declarada
                    self.advertencias.push(format!(
                        "Posible variable '{}' no declarada usada en expresión en '{}'", 
                        palabra, contexto
                    ));
                }
            }
        }
    }
    
    fn verificar_variables_en_expresion(&mut self, expresion: &Expresion, 
                                       ambito: &HashMap<String, String>, contexto: &str) {
        match expresion {
            Expresion::Identificador(nombre) => {
                if !ambito.contains_key(nombre) && 
                   !self.es_instruccion_elemental(nombre) &&
                   !self.es_valor_booleano_literal(nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Variable '{}' no declarada en '{}'", nombre, contexto),
                        0, 0
                    ));
                }
            }
            Expresion::Elemental { nombre } => {
                // Verificar que la instrucción elemental existe
                if !self.es_instruccion_elemental(nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Instrucción elemental desconocida: '{}'", nombre),
                        0, 0
                    ));
                }
            }
            Expresion::Numero(_) => {} // Números son válidos
            Expresion::Booleano(_) => {} // Booleanos son válidos
        }
    }
    
    fn verificar_instancias_robots(&mut self, programa: &Program) {
        for instancia in &programa.robots_instanciados {
            // Verificar que el tipo de robot existe
            if !programa.robots_declarados.contains(&instancia.tipo) {
                self.errores.push(CompilerError::new(
                    format!("Robot instanciado '{}' con tipo no declarado '{}'", 
                            instancia.nombre, instancia.tipo),
                    0, 0
                ));
            }
            
            // Verificar que no haya nombres duplicados de instancias
            let instancias_con_mismo_nombre: Vec<_> = programa.robots_instanciados.iter()
                .filter(|r| r.nombre == instancia.nombre)
                .collect();
            
            if instancias_con_mismo_nombre.len() > 1 {
                self.errores.push(CompilerError::new(
                    format!("Múltiples instancias de robot con nombre '{}'", instancia.nombre),
                    0, 0
                ));
            }
        }
    }
    
    fn verificar_configuracion_robots(&mut self, programa: &Program) {
        for instancia in &programa.robots_instanciados {
            let nombre_instancia = Expresion::Identificador(instancia.nombre.clone());
            
            // Verificar asignación de área
            let tiene_area = programa.asignaciones_areas.iter()
                .any(|a| a.robot == nombre_instancia);
            
            if !tiene_area {
                self.advertencias.push(format!(
                    "Robot '{}' no tiene asignación de área", instancia.nombre
                ));
            }
            
            // Verificar inicialización
            let tiene_inicializacion = programa.inicializaciones.iter()
                .any(|i| i.robot == nombre_instancia);
            
            if !tiene_inicializacion {
                self.advertencias.push(format!(
                    "Robot '{}' no tiene inicialización de posición", instancia.nombre
                ));
            }
        }
        
        // Verificar que las áreas usadas existen
        for asignacion in &programa.asignaciones_areas {
            if let Expresion::Identificador(nombre_area) = &asignacion.area {
                let area_existe = programa.areas.iter()
                    .any(|a| a.nombre == *nombre_area);
                
                if !area_existe {
                    self.errores.push(CompilerError::new(
                        format!("Área '{}' no declarada", nombre_area),
                        0, 0
                    ));
                }
            }
        }
    }
    
    fn es_instruccion_elemental(&self, nombre: &str) -> bool {
        matches!(nombre,
            "HayFlorEnLaBolsa" |
            "HayPapelEnLaBolsa" |
            "HayFlorEnLaEsquina" |
            "HayPapelEnLaEsquina" |
            "mover" |
            "derecha" |
            "Pos" |
            "AsignarArea" |
            "Iniciar"
        )
    }
    
    fn es_valor_booleano_literal(&self, valor: &str) -> bool {
        matches!(valor, "V" | "F")
    }
    
    pub fn obtener_errores(&self) -> &[CompilerError] {
        &self.errores
    }
    
    pub fn obtener_advertencias(&self) -> &[String] {
        &self.advertencias
    }
    
    pub fn mostrar_resultados(&self) {
        if self.errores.is_empty() && self.advertencias.is_empty() {
            println!("✓ Análisis semántico completado sin errores ni advertencias.");
            return;
        }
        
        if !self.errores.is_empty() {
            println!("✗ Errores encontrados:");
            for error in &self.errores {
                println!("  - {}", error.message);
            }
        }
        
        if !self.advertencias.is_empty() {
            println!("⚠ Advertencias:");
            for advertencia in &self.advertencias {
                println!("  - {}", advertencia);
            }
        }
    }
}

// Estructura auxiliar para almacenar información de procesos
#[derive(Debug, Clone)]
struct ProcesoInfo {
    parametros: Vec<(String, String)>,
    variables: Vec<super::super::parser::ast::Variable>,
}