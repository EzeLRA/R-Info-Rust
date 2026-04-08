// Definiciones de AST

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
    pub tipo: String,
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
    Elemental { nombre: String },
    Asignacion { variable: String, expresion_texto: String },
    LlamadaFuncion { nombre: String, argumentos: Vec<Expresion> },
    Si { condicion_texto: String, entonces: Vec<Instruccion>, sino: Vec<Instruccion> },
    Mientras { condicion_texto: String, cuerpo: Vec<Instruccion> },
    Repetir { condicion_texto: String, cuerpo: Vec<Instruccion> },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expresion {
    Elemental { nombre: String },
    Identificador(String),
    Numero(i32),
    Booleano(bool),
}