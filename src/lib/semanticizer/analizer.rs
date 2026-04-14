use std::collections::{HashMap, HashSet};
use crate::lib::compilerError::CompilerError;
use crate::lib::parser::ast::{Program, Proceso, Robot, Instruccion, Expresion, Variable};

#[derive(Debug, Clone)]
pub struct SemanticAnalyzer {
    errores: Vec<CompilerError>,
    advertencias: Vec<String>,
    // Mapa de funciones declaradas con su información
    funciones_declaradas: HashMap<String, FuncionInfo>,
    // Seguimiento de contexto actual para mejor mensajes de error
    contexto_actual: Vec<String>,
}

#[derive(Debug, Clone)]
struct FuncionInfo {
    nombre: String,
    parametros: Vec<(String, String, (usize, usize))>, // (nombre, tipo, ubicacion)
    tipo_retorno: Option<String>,
    es_proceso: bool,
    ubicacion: (usize, usize),
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            errores: Vec::new(),
            advertencias: Vec::new(),
            funciones_declaradas: HashMap::new(),
            contexto_actual: Vec::new(),
        }
    }
    
    pub fn analizar(&mut self, programa: &Program) -> Result<(), Vec<CompilerError>> {
        // Fase 1: Recolectar todas las funciones/procesos declarados
        self.recolectar_funciones_declaradas(programa);
        
        // Fase 2: Analizar procesos (nombres únicos, parámetros únicos, variables únicas)
        let procesos_info = self.analizar_procesos(programa);
        
        // Fase 3: Analizar robots (nombres únicos, variables únicas)
        self.analizar_robots(programa);
        
        // Fase 4: Verificar invocaciones de procesos
        self.verificar_invocaciones_procesos(programa, &procesos_info);
        
        // Fase 5: Verificar uso de variables locales
        self.verificar_variables_locales(programa);
        
        // Fase 6: Verificar robots instanciados vs definidos
        self.verificar_instancias_robots(programa);
        
        // Fase 7: Verificar asignaciones de área e inicializaciones
        self.verificar_configuracion_robots(programa);
        
        if self.errores.is_empty() {
            Ok(())
        } else {
            Err(self.errores.clone())
        }
    }
    
    fn recolectar_funciones_declaradas(&mut self, programa: &Program) {
        // Recolectar procesos
        for proceso in &programa.procesos {
            self.funciones_declaradas.insert(
                proceso.nombre.clone(),
                FuncionInfo {
                    nombre: proceso.nombre.clone(),
                    parametros: proceso.parametros.iter()
                        .map(|p| (p.nombre.clone(), p.tipo_dato.clone(), p.ubicacion))
                        .collect(),
                    tipo_retorno: None,
                    es_proceso: true,
                    ubicacion: proceso.ubicacion,
                }
            );
        }
        
        // Agregar funciones built-in/elementales
        self.agregar_funciones_builtin();
    }
    
    fn agregar_funciones_builtin(&mut self) {
        let builtins = vec![
            ("mover", vec![], None),
            ("derecha", vec![], None),
            ("Pos", vec![("av".to_string(), "numero".to_string()), 
                        ("ca".to_string(), "numero".to_string())], None),
            ("AsignarArea", vec![("robot".to_string(), "identificador".to_string()),
                                ("area".to_string(), "identificador".to_string())], None),
            ("Iniciar", vec![("robot".to_string(), "identificador".to_string()),
                           ("x".to_string(), "numero".to_string()),
                           ("y".to_string(), "numero".to_string())], None),
            ("HayFlorEnLaBolsa", vec![], Some("booleano".to_string())),
            ("HayPapelEnLaBolsa", vec![], Some("booleano".to_string())),
            ("HayFlorEnLaEsquina", vec![], Some("booleano".to_string())),
            ("HayPapelEnLaEsquina", vec![], Some("booleano".to_string())),
        ];
        
        for (nombre, params, retorno) in builtins {
            self.funciones_declaradas.insert(
                nombre.to_string(),
                FuncionInfo {
                    nombre: nombre.to_string(),
                    parametros: params.into_iter()
                        .map(|(n, t)| (n, t, (0, 0)))
                        .collect(),
                    tipo_retorno: retorno,
                    es_proceso: false,
                    ubicacion: (0, 0),
                }
            );
        }
    }
    
    fn analizar_procesos(&mut self, programa: &Program) -> HashMap<String, ProcesoInfo> {
        let mut procesos_info = HashMap::new();
        let mut nombres_procesos = HashSet::new();
        
        for proceso in &programa.procesos {
            // Establecer contexto para mensajes de error
            self.contexto_actual.push(format!("Proceso '{}'", proceso.nombre));
            
            // Verificar nombre único
            if nombres_procesos.contains(&proceso.nombre) {
                let (line, col) = proceso.ubicacion;
                self.agregar_error_con_ubicacion(
                    &format!("Proceso '{}' declarado múltiples veces", proceso.nombre),
                    line, col
                );
                self.contexto_actual.pop();
                continue;
            }
            nombres_procesos.insert(proceso.nombre.clone());
            
            // Verificar parámetros únicos
            let mut nombres_parametros = HashSet::new();
            let mut parametros_info = Vec::new();
            
            for param in &proceso.parametros {
                if nombres_parametros.contains(&param.nombre) {
                    let (line, col) = param.ubicacion;
                    self.agregar_error_con_ubicacion(
                        &format!("Parámetro '{}' duplicado en proceso '{}'", 
                                param.nombre, proceso.nombre),
                        line, col
                    );
                } else {
                    nombres_parametros.insert(param.nombre.clone());
                    parametros_info.push((param.nombre.clone(), param.tipo_dato.clone()));
                }
            }
            
            // Verificar variables locales únicas
            let mut nombres_variables = HashSet::new();
            for var in &proceso.variables {
                if nombres_variables.contains(&var.nombre) {
                    let (line, col) = var.ubicacion;
                    self.agregar_error_con_ubicacion(
                        &format!("Variable '{}' declarada múltiples veces en proceso '{}'", 
                                var.nombre, proceso.nombre),
                        line, col
                    );
                } else {
                    nombres_variables.insert(var.nombre.clone());
                }
            }
            
            // Verificar que no haya conflicto entre parámetros y variables
            for param in &proceso.parametros {
                if nombres_variables.contains(&param.nombre) {
                    let (line, col) = param.ubicacion;
                    self.agregar_error_con_ubicacion(
                        &format!("Conflicto: parámetro '{}' también declarado como variable en proceso '{}'",
                                param.nombre, proceso.nombre),
                        line, col
                    );
                }
            }
            
            procesos_info.insert(proceso.nombre.clone(), ProcesoInfo {
                parametros: parametros_info.clone(),
                variables: proceso.variables.clone(),
                ubicacion: proceso.ubicacion,
            });
            
            self.contexto_actual.pop();
        }
        
        procesos_info
    }
    
    fn analizar_robots(&mut self, programa: &Program) {
        let mut nombres_robots = HashSet::new();
        
        for robot in &programa.robots_definidos {
            self.contexto_actual.push(format!("Robot '{}'", robot.nombre));
            
            // Verificar nombre único de robot
            if nombres_robots.contains(&robot.nombre) {
                let (line, col) = robot.ubicacion;
                self.agregar_error_con_ubicacion(
                    &format!("Robot '{}' definido múltiples veces", robot.nombre),
                    line, col
                );
            }
            nombres_robots.insert(robot.nombre.clone());
            
            // Verificar variables locales únicas en robot
            let mut nombres_variables = HashSet::new();
            for var in &robot.variables {
                if nombres_variables.contains(&var.nombre) {
                    let (line, col) = var.ubicacion;
                    self.agregar_error_con_ubicacion(
                        &format!("Variable '{}' declarada múltiples veces en robot '{}'", 
                                var.nombre, robot.nombre),
                        line, col
                    );
                }
                nombres_variables.insert(var.nombre.clone());
            }
            
            self.contexto_actual.pop();
        }
    }
    
    fn verificar_invocaciones_procesos(&mut self, programa: &Program, 
                                      procesos_info: &HashMap<String, ProcesoInfo>) {
        // Verificar en cada robot
        for robot in &programa.robots_definidos {
            self.contexto_actual.push(format!("Robot '{}'", robot.nombre));
            self.verificar_invocaciones_en_instrucciones(
                &robot.instrucciones, 
                procesos_info,
                &robot.nombre
            );
            self.contexto_actual.pop();
        }
        
        // Verificar en cada proceso (para detectar llamadas entre procesos)
        for proceso in &programa.procesos {
            self.contexto_actual.push(format!("Proceso '{}'", proceso.nombre));
            self.verificar_invocaciones_en_instrucciones(
                &proceso.instrucciones,
                procesos_info,
                &proceso.nombre
            );
            self.contexto_actual.pop();
        }
    }
    
    fn verificar_invocaciones_en_instrucciones(&mut self, instrucciones: &[Instruccion], 
                                              procesos_info: &HashMap<String, ProcesoInfo>,
                                              contexto: &str) {
        let mut pila_llamadas = Vec::new();
        
        for instruccion in instrucciones {
            self.verificar_instruccion_llamadas(instruccion, procesos_info, contexto, &mut pila_llamadas);
        }
    }
    
    fn verificar_instruccion_llamadas(&mut self, instruccion: &Instruccion,
                                     procesos_info: &HashMap<String, ProcesoInfo>,
                                     contexto: &str,
                                     pila_llamadas: &mut Vec<String>) {
        match instruccion {
            Instruccion::LlamadaFuncion { nombre, argumentos, ubicacion } => {
                // Verificar existencia primero
                match self.funciones_declaradas.get(nombre) {
                    Some(func_info) => {
                        // Extraer datos necesarios inmediatamente
                        let num_params = func_info.parametros.len();
                        let params_clone = func_info.parametros.clone();
                        let nombre_clone = nombre.clone();
                        let contexto_clone = contexto.to_string();
                        let (line, col) = *ubicacion;
                        
                        // Verificar número de argumentos
                        if argumentos.len() != num_params {
                            self.agregar_error_con_ubicacion(
                                &format!("La función/proceso '{}' espera {} argumentos, pero se proporcionaron {} (en '{}')",
                                        nombre_clone, num_params, argumentos.len(), contexto_clone),
                                line, col
                            );
                        }
                        
                        // Verificar tipos de argumentos
                        for (i, arg) in argumentos.iter().enumerate() {
                            if i < params_clone.len() {
                                self.verificar_tipo_argumento(arg, &params_clone[i].1, &nombre_clone);
                            }
                        }
                        
                        // Detectar recursión
                        if nombre == contexto {
                            self.agregar_advertencia_con_ubicacion(
                                &format!("⚠ Posible recursión infinita: '{}' se llama a sí mismo", nombre_clone),
                                line, col
                            );
                        }
                        
                        if pila_llamadas.contains(&nombre_clone) {
                            self.agregar_advertencia_con_ubicacion(
                                &format!("⚠ Posible recursión indirecta: ciclo de llamadas detectado con '{}'", nombre_clone),
                                line, col
                            );
                        }
                    }
                    None => {
                        let (line, col) = *ubicacion;
                        self.agregar_error_con_ubicacion(
                            &format!("Función/proceso '{}' no declarado (llamado en '{}')", nombre, contexto),
                            line, col
                        );
                    }
                }
            }
            Instruccion::Si { entonces, sino, .. } => {
                self.verificar_invocaciones_en_instrucciones(entonces, procesos_info, contexto);
                self.verificar_invocaciones_en_instrucciones(sino, procesos_info, contexto);
            }
            Instruccion::Mientras { cuerpo, .. } => {
                self.verificar_invocaciones_en_instrucciones(cuerpo, procesos_info, contexto);
            }
            Instruccion::Repetir { cuerpo, .. } => {
                self.verificar_invocaciones_en_instrucciones(cuerpo, procesos_info, contexto);
            }
            _ => {}
        }
    }
    
    fn verificar_tipo_argumento(&mut self, arg: &Expresion, tipo_esperado: &str, funcion: &str) {
        let (tipo_real, ubicacion) = match arg {
            Expresion::Numero(_, ubic) => ("numero", *ubic),
            Expresion::Booleano(_, ubic) => ("booleano", *ubic),
            Expresion::Identificador(_, ubic) => ("identificador", *ubic),
            Expresion::Elemental { ubicacion, .. } => ("elemental", *ubicacion),
        };
        
        // Verificación básica de compatibilidad de tipos
        let compatible = match (tipo_esperado, tipo_real) {
            ("numero", "numero") => true,
            ("booleano", "booleano") => true,
            ("identificador", "identificador") => true,
            (_, "elemental") => true, // Las funciones elementales pueden devolver cualquier tipo
            _ => false,
        };
        
        if !compatible && tipo_real != "identificador" {
            let (line, col) = ubicacion;
            self.agregar_advertencia_con_ubicacion(
                &format!("⚠ Posible incompatibilidad de tipos: se esperaba '{}' pero se proporcionó '{}' en llamada a '{}'",
                        tipo_esperado, tipo_real, funcion),
                line, col
            );
        }
    }
    
    fn verificar_variables_locales(&mut self, programa: &Program) {
        // Verificar variables en procesos
        for proceso in &programa.procesos {
            self.contexto_actual.push(format!("Proceso '{}'", proceso.nombre));
            
            let mut ambito = HashMap::new();
            
            // Agregar parámetros al ámbito
            for param in &proceso.parametros {
                ambito.insert(param.nombre.clone(), (param.tipo_dato.clone(), param.ubicacion));
            }
            
            // Agregar variables locales al ámbito
            for var in &proceso.variables {
                ambito.insert(var.nombre.clone(), (var.tipo_dato.clone(), var.ubicacion));
            }
            
            // Verificar uso en instrucciones
            self.verificar_variables_en_instrucciones(&proceso.instrucciones, &ambito, &proceso.nombre);
            
            self.contexto_actual.pop();
        }
        
        // Verificar variables en robots
        for robot in &programa.robots_definidos {
            self.contexto_actual.push(format!("Robot '{}'", robot.nombre));
            
            let mut ambito = HashMap::new();
            
            // Agregar variables del robot al ámbito
            for var in &robot.variables {
                ambito.insert(var.nombre.clone(), (var.tipo_dato.clone(), var.ubicacion));
            }
            
            // Verificar uso en instrucciones
            self.verificar_variables_en_instrucciones(&robot.instrucciones, &ambito, &robot.nombre);
            
            self.contexto_actual.pop();
        }
    }
    
    fn verificar_variables_en_instrucciones(&mut self, instrucciones: &[Instruccion], 
                                          ambito: &HashMap<String, (String, (usize, usize))>, 
                                          contexto: &str) {
        for instruccion in instrucciones {
            match instruccion {
                Instruccion::Asignacion { variable, expresion_texto, ubicacion } => {
                    // Verificar que la variable esté declarada
                    if !ambito.contains_key(variable) {
                        let (line, col) = *ubicacion;
                        self.agregar_error_con_ubicacion(
                            &format!("Variable '{}' no declarada en '{}'", variable, contexto),
                            line, col
                        );
                    }
                    
                    // Verificar variables en la expresión texto
                    self.verificar_variables_en_texto(expresion_texto, ambito, contexto);
                }
                Instruccion::LlamadaFuncion { argumentos, nombre, ubicacion } => {
                    // Verificar argumentos
                    for arg in argumentos {
                        self.verificar_variables_en_expresion(arg, ambito, contexto);
                    }
                    
                    // Verificar que la función llamada existe
                    if !self.funciones_declaradas.contains_key(nombre) {
                        let (line, col) = *ubicacion;
                        self.agregar_error_con_ubicacion(
                            &format!("Función/proceso '{}' no declarado (llamado en '{}')", nombre, contexto),
                            line, col
                        );
                    }
                }
                Instruccion::Si { condicion_texto, entonces, sino, .. } => {
                    self.verificar_variables_en_texto(condicion_texto, ambito, contexto);
                    self.verificar_variables_en_instrucciones(entonces, ambito, contexto);
                    self.verificar_variables_en_instrucciones(sino, ambito, contexto);
                }
                Instruccion::Mientras { condicion_texto, cuerpo, .. } => {
                    self.verificar_variables_en_texto(condicion_texto, ambito, contexto);
                    self.verificar_variables_en_instrucciones(cuerpo, ambito, contexto);
                }
                Instruccion::Repetir { condicion_texto, cuerpo, .. } => {
                    self.verificar_variables_en_texto(condicion_texto, ambito, contexto);
                    self.verificar_variables_en_instrucciones(cuerpo, ambito, contexto);
                }
                Instruccion::Elemental { .. } => {}
            }
        }
    }
    
    fn verificar_variables_en_texto(&mut self, texto: &str, 
                                   ambito: &HashMap<String, (String, (usize, usize))>, 
                                   contexto: &str) {
        let palabras: Vec<&str> = texto.split(|c: char| !c.is_alphanumeric() && c != '_')
                                       .filter(|s| !s.is_empty())
                                       .collect();
        
        for palabra in palabras {
            if palabra.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_') {
                if self.es_valor_booleano_literal(palabra) {
                    continue;
                }
                
                if palabra.parse::<i32>().is_err() && 
                   !self.funciones_declaradas.contains_key(palabra) &&
                   !ambito.contains_key(palabra) {
                    self.agregar_advertencia(&format!(
                        "Posible variable no declarada '{}' en expresión en '{}'", 
                        palabra, contexto
                    ));
                }
            }
        }
    }
    
    fn verificar_variables_en_expresion(&mut self, expresion: &Expresion, 
                                       ambito: &HashMap<String, (String, (usize, usize))>, 
                                       contexto: &str) {
        match expresion {
            Expresion::Identificador(nombre, ubicacion) => {
                if !ambito.contains_key(nombre) && 
                   !self.funciones_declaradas.contains_key(nombre) &&
                   !self.es_valor_booleano_literal(nombre) {
                    let (line, col) = *ubicacion;
                    self.agregar_error_con_ubicacion(
                        &format!("Variable '{}' no declarada en '{}'", nombre, contexto),
                        line, col
                    );
                }
            }
            Expresion::Elemental { nombre, ubicacion } => {
                if !self.funciones_declaradas.contains_key(nombre) {
                    let (line, col) = *ubicacion;
                    self.agregar_error_con_ubicacion(
                        &format!("Función elemental desconocida: '{}'", nombre),
                        line, col
                    );
                }
            }
            Expresion::Numero(_, _) => {}
            Expresion::Booleano(_, _) => {}
        }
    }
    
    fn verificar_instancias_robots(&mut self, programa: &Program) {
        for instancia in &programa.robots_instanciados {
            if !programa.robots_declarados.contains(&instancia.tipo) {
                let (line, col) = instancia.ubicacion;
                self.agregar_error_con_ubicacion(
                    &format!("Robot instanciado '{}' con tipo no declarado '{}'", 
                            instancia.nombre, instancia.tipo),
                    line, col
                );
            }
            
            let instancias_con_mismo_nombre: Vec<_> = programa.robots_instanciados.iter()
                .filter(|r| r.nombre == instancia.nombre)
                .collect();
            
            if instancias_con_mismo_nombre.len() > 1 {
                let (line, col) = instancia.ubicacion;
                self.agregar_error_con_ubicacion(
                    &format!("Múltiples instancias de robot con nombre '{}'", instancia.nombre),
                    line, col
                );
            }
        }
    }
    
    fn verificar_configuracion_robots(&mut self, programa: &Program) {
        for instancia in &programa.robots_instanciados {
            let nombre_instancia = Expresion::Identificador(instancia.nombre.clone(), instancia.ubicacion);
            
            let tiene_area = programa.asignaciones_areas.iter()
                .any(|a| a.robot == nombre_instancia);
            
            if !tiene_area {
                let (line, col) = instancia.ubicacion;
                self.agregar_advertencia_con_ubicacion(
                    &format!("Robot '{}' no tiene asignación de área", instancia.nombre),
                    line, col
                );
            }
            
            let tiene_inicializacion = programa.inicializaciones.iter()
                .any(|i| i.robot == nombre_instancia);
            
            if !tiene_inicializacion {
                let (line, col) = instancia.ubicacion;
                self.agregar_advertencia_con_ubicacion(
                    &format!("Robot '{}' no tiene inicialización de posición", instancia.nombre),
                    line, col
                );
            }
        }
        
        for asignacion in &programa.asignaciones_areas {
            if let Expresion::Identificador(nombre_area, ubicacion) = &asignacion.area {
                let area_existe = programa.areas.iter()
                    .any(|a| a.nombre == *nombre_area);
                
                if !area_existe {
                    let (line, col) = *ubicacion;
                    self.agregar_error_con_ubicacion(
                        &format!("Área '{}' no declarada", nombre_area),
                        line, col
                    );
                }
            }
        }
    }
    
    fn es_valor_booleano_literal(&self, valor: &str) -> bool {
        matches!(valor, "V" | "F")
    }
    
    fn agregar_error(&mut self, mensaje: &str) {
        let contexto = if self.contexto_actual.is_empty() {
            String::new()
        } else {
            format!(" en {}", self.contexto_actual.join(" -> "))
        };
        
        self.errores.push(CompilerError::new(
            format!("{}{}", mensaje, contexto),
            0, 0
        ));
    }
    
    fn agregar_error_con_ubicacion(&mut self, mensaje: &str, line: usize, column: usize) {
        let contexto = if self.contexto_actual.is_empty() {
            String::new()
        } else {
            format!(" en {}", self.contexto_actual.join(" -> "))
        };
        
        self.errores.push(CompilerError::new(
            format!("{}{}", mensaje, contexto),
            line,
            column
        ));
    }
    
    fn agregar_advertencia(&mut self, mensaje: &str) {
        self.advertencias.push(mensaje.to_string());
    }
    
    fn agregar_advertencia_con_ubicacion(&mut self, mensaje: &str, line: usize, column: usize) {
        self.advertencias.push(format!("{} (línea {}, columna {})", mensaje, line, column));
    }
    
    pub fn obtener_errores(&self) -> &[CompilerError] {
        &self.errores
    }
    
    pub fn obtener_advertencias(&self) -> &[String] {
        &self.advertencias
    }
    
    pub fn mostrar_resultados(&self) {
        if self.errores.is_empty() && self.advertencias.is_empty() {
            println!("✅ Análisis semántico completado sin errores ni advertencias.");
            return;
        }
        
        if !self.errores.is_empty() {
            println!("\n❌ ERRORES SEMÁNTICOS ({}):", self.errores.len());
            for (i, error) in self.errores.iter().enumerate() {
                println!("  {}. [Línea {}, Columna {}] {}", 
                        i + 1, error.line, error.column, error.message);
            }
        }
        
        if !self.advertencias.is_empty() {
            println!("\n⚠️  ADVERTENCIAS ({}):", self.advertencias.len());
            for (i, advertencia) in self.advertencias.iter().enumerate() {
                println!("  {}. {}", i + 1, advertencia);
            }
        }
        
        println!();
    }
}

#[derive(Debug, Clone)]
struct ProcesoInfo {
    parametros: Vec<(String, String)>,
    variables: Vec<Variable>,
    ubicacion: (usize, usize),
}