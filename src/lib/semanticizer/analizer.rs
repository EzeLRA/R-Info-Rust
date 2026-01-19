use std::collections::{HashMap, HashSet};
use crate::lib::compilerError::CompilerError;
use super::super::parser::processor::{Program, Proceso, Robot, Instruccion, Expresion};

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
        // 1. Analizar procesos
        let procesos_validos = self.analizar_procesos(programa);
        
        // 2. Analizar robots (que pueden usar procesos)
        self.analizar_robots(programa, &procesos_validos);
        
        // 3. Verificar invocaciones de procesos
        self.verificar_invocaciones_procesos(programa, &procesos_validos);
        
        // 4. Verificar uso de variables locales
        self.verificar_variables_locales(programa);
        
        if self.errores.is_empty() {
            Ok(())
        } else {
            Err(self.errores.clone())
        }
    }
    
    fn analizar_procesos(&mut self, programa: &Program) -> HashMap<String, (Vec<(String, String)>, String)> {
        let mut procesos_validos = HashMap::new();
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
            for param in &proceso.parametros {
                if nombres_parametros.contains(&param.nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Parámetro '{}' duplicado en proceso '{}'", param.nombre, proceso.nombre),
                        0, 0
                    ));
                }
                nombres_parametros.insert(param.nombre.clone());
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
                }
                nombres_variables.insert(var.nombre.clone());
            }
            
            // Almacenar información del proceso para verificaciones posteriores
            let parametros_info: Vec<(String, String)> = proceso.parametros
                .iter()
                .map(|p| (p.nombre.clone(), p.tipo_dato.clone()))
                .collect();
            
            procesos_validos.insert(proceso.nombre.clone(), (parametros_info, "void".to_string()));
        }
        
        procesos_validos
    }
    
    fn analizar_robots(&mut self, programa: &Program, procesos_validos: &HashMap<String, (Vec<(String, String)>, String)>) {
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
                                      procesos_validos: &HashMap<String, (Vec<(String, String)>, String)>) {
        // Verificar que los procesos solo se usen después de ser declarados
        
        // Primero, crear lista de procesos declarados
        let mut procesos_declarados = HashSet::new();
        for proceso in &programa.procesos {
            procesos_declarados.insert(proceso.nombre.clone());
        }
        
        // Verificar en robots
        for robot in &programa.robots_definidos {
            self.verificar_invocaciones_en_instrucciones(&robot.instrucciones, &procesos_declarados, &robot.nombre);
        }
        
    }
    
    fn verificar_invocaciones_en_instrucciones(&mut self, instrucciones: &[Instruccion], 
                                              procesos_declarados: &HashSet<String>, contexto: &str) {
        for instruccion in instrucciones {
            match instruccion {
                Instruccion::LlamadaFuncion { nombre, .. } => {
                    if procesos_declarados.contains(nombre) {
                        // Verificar que el proceso no se llame a sí mismo (recursión simple no permitida)
                        if nombre == contexto {
                            self.errores.push(CompilerError::new(
                                format!("Proceso '{}' no puede llamarse a sí mismo", nombre),
                                0, 0
                            ));
                        }
                    }
                }
                Instruccion::Si { entonces, sino, .. } => {
                    self.verificar_invocaciones_en_instrucciones(entonces, procesos_declarados, contexto);
                    self.verificar_invocaciones_en_instrucciones(sino, procesos_declarados, contexto);
                }
                Instruccion::Mientras { cuerpo, .. } => {
                    self.verificar_invocaciones_en_instrucciones(cuerpo, procesos_declarados, contexto);
                }
                Instruccion::Repetir { cuerpo, .. } => {
                    self.verificar_invocaciones_en_instrucciones(cuerpo, procesos_declarados, contexto);
                }
                _ => {}
            }
        }
    }
    
    fn verificar_variables_locales(&mut self, programa: &Program) {
        // Verificar variables en procesos
        for proceso in &programa.procesos {
            let mut variables_declaradas = HashMap::new();
            
            // Agregar parámetros como variables declaradas
            for param in &proceso.parametros {
                variables_declaradas.insert(param.nombre.clone(), param.tipo_dato.clone());
            }
            
            // Agregar variables locales
            for var in &proceso.variables {
                variables_declaradas.insert(var.nombre.clone(), var.tipo_dato.clone());
            }
            
            // Verificar uso de variables en instrucciones
            self.verificar_variables_en_instrucciones(&proceso.instrucciones, &variables_declaradas, &proceso.nombre);
        }
        
        // Verificar variables en robots
        for robot in &programa.robots_definidos {
            let mut variables_declaradas = HashMap::new();
            
            // Agregar variables del robot
            for var in &robot.variables {
                variables_declaradas.insert(var.nombre.clone(), var.tipo_dato.clone());
            }
            
            // Verificar uso de variables en instrucciones
            self.verificar_variables_en_instrucciones(&robot.instrucciones, &variables_declaradas, &robot.nombre);
        }
    }
    
    fn verificar_variables_en_instrucciones(&mut self, instrucciones: &[Instruccion], 
                                          variables_declaradas: &HashMap<String, String>, contexto: &str) {
        for instruccion in instrucciones {
            match instruccion {
                Instruccion::Elemental { nombre } => {
                    
                }
                Instruccion::Asignacion { variable, valor } => {
                    // Verificar que la variable esté declarada
                    if !variables_declaradas.contains_key(variable) {
                        self.errores.push(CompilerError::new(
                            format!("Variable '{}' no declarada en '{}'", variable, contexto),
                            0, 0
                        ));
                    } else {
                        // Verificar tipo de la expresión de asignación
                        let tipo_declarado = &variables_declaradas[variable];
                        let tipo_expresion = self.obtener_tipo_expresion(valor, variables_declaradas);
                        
                        if let Some(tipo_exp) = tipo_expresion {
                            if tipo_declarado != &tipo_exp {
                                self.errores.push(CompilerError::new(
                                    format!("Tipo incorrecto en asignación a '{}': esperado '{}', encontrado '{}' (en '{}')",
                                            variable, tipo_declarado, tipo_exp, contexto),
                                    0, 0
                                ));
                            }
                        }
                    }
                    
                    // Verificar variables en la expresión
                    self.verificar_variables_en_expresion(valor, variables_declaradas, contexto);
                }
                Instruccion::LlamadaFuncion { argumentos, .. } => {
                    for arg in argumentos {
                        self.verificar_variables_en_expresion(arg, variables_declaradas, contexto);
                    }
                }
                Instruccion::Si { condicion, entonces, sino } => {
                    // Verificar variables en la condición
                    self.verificar_variables_en_expresion(condicion, variables_declaradas, contexto);
                    
                    // Verificar variables en los bloques
                    self.verificar_variables_en_instrucciones(entonces, variables_declaradas, contexto);
                    self.verificar_variables_en_instrucciones(sino, variables_declaradas, contexto);
                }
                Instruccion::Mientras { condicion, cuerpo } => {
                    self.verificar_variables_en_expresion(condicion, variables_declaradas, contexto);
                    self.verificar_variables_en_instrucciones(cuerpo, variables_declaradas, contexto);
                }
                Instruccion::Repetir { condicion, cuerpo } => {
                    self.verificar_variables_en_expresion(condicion, variables_declaradas, contexto);
                    self.verificar_variables_en_instrucciones(cuerpo, variables_declaradas, contexto);
                }
            }
        }
    }
    
    fn verificar_variables_en_expresion(&mut self, expresion: &Expresion, 
                                       variables_declaradas: &HashMap<String, String>, contexto: &str) {
        match expresion {
            Expresion::Identificador(nombre) => {
                if !variables_declaradas.contains_key(nombre) {
                    self.errores.push(CompilerError::new(
                        format!("Variable '{}' no declarada en expresión (en '{}')", nombre, contexto),
                        0, 0
                    ));
                }
            }
            Expresion::Binaria { izquierda, derecha, .. } => {
                self.verificar_variables_en_expresion(izquierda, variables_declaradas, contexto);
                self.verificar_variables_en_expresion(derecha, variables_declaradas, contexto);
            }
            _ => {} // Numero y Booleano no tienen variables
        }
    }
    
    fn obtener_tipo_expresion(&self, expresion: &Expresion, 
                             variables_declaradas: &HashMap<String, String>) -> Option<String> {
        match expresion {
            Expresion::Identificador(nombre) => {
                variables_declaradas.get(nombre).cloned()
            }
            Expresion::Elemental { nombre } => {
                // Aquí puedes manejar expresiones elementales si es necesario
                None
            }

            Expresion::Numero(_) => Some("numero".to_string()),
            Expresion::Booleano(_) => Some("booleano".to_string()),
            Expresion::Binaria { izquierda, operador, derecha } => {
                let tipo_izq = self.obtener_tipo_expresion(izquierda, variables_declaradas);
                let tipo_der = self.obtener_tipo_expresion(derecha, variables_declaradas);
                
                if let (Some(tipo_i), Some(tipo_d)) = (tipo_izq, tipo_der) {
                    // Verificar compatibilidad de tipos
                    if tipo_i == tipo_d {
                        // Para operaciones aritméticas
                        if ["+", "-", "*", "/"].contains(&operador.as_str()) {
                            if tipo_i == "numero" {
                                return Some("numero".to_string());
                            } else {
                                return None; // Error de tipo
                            }
                        }
                        // Para operaciones de comparación
                        else if ["<", "<=", ">", ">=", "==", "<>"].contains(&operador.as_str()) {
                            return Some("booleano".to_string());
                        }
                        // Para operaciones booleanas
                        else if ["&", "|"].contains(&operador.as_str()) {
                            if tipo_i == "booleano" {
                                return Some("booleano".to_string());
                            } else {
                                return None; // Error de tipo
                            }
                        }
                    }
                }
                None
            }
        }
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