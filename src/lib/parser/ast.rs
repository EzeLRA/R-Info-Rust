// Definiciones de AST con información de ubicación

#[derive(Debug, Clone)]
pub struct RobotInstanciado {
    pub nombre: String,
    pub tipo: String,
    pub ubicacion: (usize, usize), // (línea, columna)
}

#[derive(Debug, Clone)]
pub struct AsignacionArea {
    pub robot: Expresion,
    pub area: Expresion,
    pub ubicacion: (usize, usize), // (línea, columna)
}

#[derive(Debug, Clone)]
pub struct InicializacionRobot {
    pub robot: Expresion,
    pub pos_x: Expresion,
    pub pos_y: Expresion,
    pub ubicacion: (usize, usize), // (línea, columna)
}

// Estructura principal del Ast
#[derive(Debug, Clone)]
pub struct Program {
    pub nombre: String,
    pub procesos: Vec<Proceso>,
    pub areas: Vec<Area>,
    pub robots_declarados: Vec<String>,
    pub robots_definidos: Vec<Robot>,
    pub robots_instanciados: Vec<RobotInstanciado>,
    pub asignaciones_areas: Vec<AsignacionArea>,
    pub inicializaciones: Vec<InicializacionRobot>,
    pub ubicacion: (usize, usize), // (línea, columna) donde comienza el programa
}

#[derive(Debug, Clone)]
pub struct Proceso {
    pub nombre: String,
    pub parametros: Vec<Parametro>,
    pub variables: Vec<Variable>,
    pub instrucciones: Vec<Instruccion>,
    pub ubicacion: (usize, usize), // (línea, columna) donde se declara el proceso
}

#[derive(Debug, Clone)]
pub struct Parametro {
    pub tipo: String,
    pub nombre: String,
    pub tipo_dato: String,
    pub ubicacion: (usize, usize), // (línea, columna) donde se declara el parámetro
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub nombre: String,
    pub tipo_dato: String,
    pub ubicacion: (usize, usize), // (línea, columna) donde se declara la variable
}

#[derive(Debug, Clone)]
pub struct Area {
    pub nombre: String,
    pub tipo: String,
    pub coordenadas: (i32, i32, i32, i32),
    pub ubicacion: (usize, usize), // (línea, columna) donde se declara el área
}

#[derive(Debug, Clone)]
pub struct Robot {
    pub nombre: String,
    pub variables: Vec<Variable>,
    pub instrucciones: Vec<Instruccion>,
    pub ubicacion: (usize, usize), // (línea, columna) donde se define el robot
}

#[derive(Debug, Clone)]
pub enum Instruccion {
    Elemental { 
        nombre: String, 
        ubicacion: (usize, usize) 
    },
    Asignacion { 
        variable: String, 
        expresion_texto: String, 
        ubicacion: (usize, usize) 
    },
    LlamadaFuncion { 
        nombre: String, 
        argumentos: Vec<Expresion>, 
        ubicacion: (usize, usize) 
    },
    Si { 
        condicion_texto: String, 
        entonces: Vec<Instruccion>, 
        sino: Vec<Instruccion>, 
        ubicacion: (usize, usize) 
    },
    Mientras { 
        condicion_texto: String, 
        cuerpo: Vec<Instruccion>, 
        ubicacion: (usize, usize) 
    },
    Repetir { 
        condicion_texto: String, 
        cuerpo: Vec<Instruccion>, 
        ubicacion: (usize, usize) 
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expresion {
    Elemental { 
        nombre: String, 
        ubicacion: (usize, usize) 
    },
    Identificador(String, (usize, usize)),
    Numero(i32, (usize, usize)),
    Booleano(bool, (usize, usize)),
}

// Implementación de métodos útiles para Expresion
impl Expresion {
    pub fn obtener_ubicacion(&self) -> (usize, usize) {
        match self {
            Expresion::Elemental { ubicacion, .. } => *ubicacion,
            Expresion::Identificador(_, ubicacion) => *ubicacion,
            Expresion::Numero(_, ubicacion) => *ubicacion,
            Expresion::Booleano(_, ubicacion) => *ubicacion,
        }
    }
    
    pub fn obtener_valor_string(&self) -> String {
        match self {
            Expresion::Elemental { nombre, .. } => nombre.clone(),
            Expresion::Identificador(nombre, _) => nombre.clone(),
            Expresion::Numero(valor, _) => valor.to_string(),
            Expresion::Booleano(valor, _) => if *valor { "V".to_string() } else { "F".to_string() },
        }
    }
    
    pub fn es_identificador(&self) -> bool {
        matches!(self, Expresion::Identificador(_, _))
    }
    
    pub fn es_numero(&self) -> bool {
        matches!(self, Expresion::Numero(_, _))
    }
    
    pub fn es_booleano(&self) -> bool {
        matches!(self, Expresion::Booleano(_, _))
    }
    
    pub fn es_elemental(&self) -> bool {
        matches!(self, Expresion::Elemental { .. })
    }
}

// Implementación de métodos útiles para Instruccion
impl Instruccion {
    pub fn obtener_ubicacion(&self) -> (usize, usize) {
        match self {
            Instruccion::Elemental { ubicacion, .. } => *ubicacion,
            Instruccion::Asignacion { ubicacion, .. } => *ubicacion,
            Instruccion::LlamadaFuncion { ubicacion, .. } => *ubicacion,
            Instruccion::Si { ubicacion, .. } => *ubicacion,
            Instruccion::Mientras { ubicacion, .. } => *ubicacion,
            Instruccion::Repetir { ubicacion, .. } => *ubicacion,
        }
    }
    
    pub fn es_llamada_funcion(&self) -> bool {
        matches!(self, Instruccion::LlamadaFuncion { .. })
    }
    
    pub fn es_asignacion(&self) -> bool {
        matches!(self, Instruccion::Asignacion { .. })
    }
    
    pub fn es_elemental(&self) -> bool {
        matches!(self, Instruccion::Elemental { .. })
    }
    
    pub fn obtener_nombre_funcion(&self) -> Option<&str> {
        match self {
            Instruccion::LlamadaFuncion { nombre, .. } => Some(nombre),
            Instruccion::Elemental { nombre, .. } => Some(nombre),
            _ => None,
        }
    }
}

// Implementación de métodos útiles para Program
impl Program {
    pub fn obtener_robot_definido(&self, nombre: &str) -> Option<&Robot> {
        self.robots_definidos.iter().find(|r| r.nombre == nombre)
    }
    
    pub fn obtener_proceso(&self, nombre: &str) -> Option<&Proceso> {
        self.procesos.iter().find(|p| p.nombre == nombre)
    }
    
    pub fn obtener_area(&self, nombre: &str) -> Option<&Area> {
        self.areas.iter().find(|a| a.nombre == nombre)
    }
    
    pub fn robot_esta_declarado(&self, tipo: &str) -> bool {
        self.robots_declarados.contains(&tipo.to_string())
    }
}

// Implementación de métodos útiles para Proceso
impl Proceso {
    pub fn obtener_parametro(&self, nombre: &str) -> Option<&Parametro> {
        self.parametros.iter().find(|p| p.nombre == nombre)
    }
    
    pub fn obtener_variable(&self, nombre: &str) -> Option<&Variable> {
        self.variables.iter().find(|v| v.nombre == nombre)
    }
    
    pub fn tiene_parametro(&self, nombre: &str) -> bool {
        self.parametros.iter().any(|p| p.nombre == nombre)
    }
    
    pub fn tiene_variable(&self, nombre: &str) -> bool {
        self.variables.iter().any(|v| v.nombre == nombre)
    }
}

// Implementación de métodos útiles para Robot
impl Robot {
    pub fn obtener_variable(&self, nombre: &str) -> Option<&Variable> {
        self.variables.iter().find(|v| v.nombre == nombre)
    }
    
    pub fn tiene_variable(&self, nombre: &str) -> bool {
        self.variables.iter().any(|v| v.nombre == nombre)
    }
}

// Implementación de Display para mejor debugging
impl std::fmt::Display for Expresion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expresion::Elemental { nombre, .. } => write!(f, "{}", nombre),
            Expresion::Identificador(nombre, _) => write!(f, "{}", nombre),
            Expresion::Numero(valor, _) => write!(f, "{}", valor),
            Expresion::Booleano(valor, _) => write!(f, "{}", if *valor { "V" } else { "F" }),
        }
    }
}

impl std::fmt::Display for Instruccion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Instruccion::Elemental { nombre, ubicacion: (line, col) } => {
                write!(f, "{} (línea {}, columna {})", nombre, line, col)
            }
            Instruccion::Asignacion { variable, expresion_texto, ubicacion: (line, col) } => {
                write!(f, "{} := {} (línea {}, columna {})", variable, expresion_texto, line, col)
            }
            Instruccion::LlamadaFuncion { nombre, argumentos: _, ubicacion: (line, col) } => {
                write!(f, "{}(...) (línea {}, columna {})", nombre, line, col)
            }
            Instruccion::Si { condicion_texto, ubicacion: (line, col), .. } => {
                write!(f, "si {} entonces ... (línea {}, columna {})", condicion_texto, line, col)
            }
            Instruccion::Mientras { condicion_texto, ubicacion: (line, col), .. } => {
                write!(f, "mientras {} hacer ... (línea {}, columna {})", condicion_texto, line, col)
            }
            Instruccion::Repetir { condicion_texto, ubicacion: (line, col), .. } => {
                write!(f, "repetir {} ... (línea {}, columna {})", condicion_texto, line, col)
            }
        }
    }
}