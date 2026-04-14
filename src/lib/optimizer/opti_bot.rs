use crate::lib::{optimizer::bot_simplyfier::BotSimplifier, parser::ast::{
    Area, AsignacionArea, Expresion, InicializacionRobot, Instruccion, Proceso, Program, Robot, RobotInstanciado, Variable
}};

#[derive(Debug, Clone)]
pub struct RobotPrototype {
    pub type_name: String,
    pub my_variables: Vec<Variable>,
    pub instructions: Vec<Instruccion>,
    pub ubicacion: (usize, usize), // Ubicación donde se define el robot
}

impl RobotPrototype {
    pub fn new(type_name: String, ubicacion: (usize, usize)) -> Self {
        RobotPrototype {
            type_name,
            my_variables: Vec::new(),
            instructions: Vec::new(),
            ubicacion,
        }
    }

    pub fn load_robot_definition(&mut self, robot_def: &Robot) {
        self.my_variables = robot_def.variables.clone();
        self.instructions = robot_def.instrucciones.clone();
        self.ubicacion = robot_def.ubicacion;
    }

    pub fn optimize_instructions(&mut self, instructions: &mut Vec<Instruccion>) {
        let mut math_bot = BotSimplifier::new();

        instructions.iter_mut().for_each(|instr| {
            match instr {
                Instruccion::Asignacion { variable: _, expresion_texto, ubicacion: _ } => {
                    math_bot.process_ecuation(expresion_texto);
                    if let Some(simplified) = math_bot.get_simplified_expression() {
                        *expresion_texto = simplified;
                    }
                },
                Instruccion::Si { condicion_texto, entonces, sino, ubicacion: _ } => {
                    math_bot.process_ecuation(condicion_texto);
                    if let Some(simplified) = math_bot.get_simplified_expression() {
                        *condicion_texto = simplified;
                    }
                    self.optimize_instructions(entonces);
                    if !sino.is_empty() {
                        self.optimize_instructions(sino);
                    }
                },
                Instruccion::Mientras { condicion_texto, cuerpo, ubicacion: _ } => {
                    math_bot.process_ecuation(condicion_texto);
                    if let Some(simplified) = math_bot.get_simplified_expression() {
                        *condicion_texto = simplified;
                    }
                    self.optimize_instructions(cuerpo);
                },
                Instruccion::Repetir { condicion_texto, cuerpo, ubicacion: _ } => {
                    math_bot.process_ecuation(condicion_texto);
                    if let Some(simplified) = math_bot.get_simplified_expression() {
                        *condicion_texto = simplified;
                    }
                    self.optimize_instructions(cuerpo);
                },
                Instruccion::LlamadaFuncion { nombre: _, argumentos: _, ubicacion: _ } => {
                    // Las llamadas a función no se optimizan por ahora
                },
                Instruccion::Elemental { .. } => {
                    // Las instrucciones elementales no requieren optimización
                }
            }
        });
    }
}

#[derive(Debug, Clone)]
pub struct Info {
    pub name_program: String,
    pub procedures: Vec<Proceso>,
    pub robots: Vec<RobotPrototype>,
    pub areas_robot: Vec<AsignacionArea>,
    pub areas_definidas: Vec<Area>,
    pub inicializaciones: Vec<InicializacionRobot>,
    pub robots_instanciados: Vec<RobotInstanciado>,
}

impl Info {
    pub fn new() -> Self {
        Info {
            name_program: String::new(),
            procedures: Vec::new(),
            robots: Vec::new(),
            areas_robot: Vec::new(),
            areas_definidas: Vec::new(),
            inicializaciones: Vec::new(),
            robots_instanciados: Vec::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Optimizer {
    info: Info,
}

impl Optimizer {
    pub fn new() -> Self {
        Optimizer { info: Info::new() }
    }

    pub fn process(&mut self, program: &Program) {
        
        // Configurar datos básicos para el optimizador
        self.info.name_program = program.nombre.clone();
        self.info.procedures = program.procesos.clone();
        self.info.areas_robot = program.asignaciones_areas.clone();
        self.info.areas_definidas = program.areas.clone();
        self.info.inicializaciones = program.inicializaciones.clone();
        self.info.robots_instanciados = program.robots_instanciados.clone();

        // Procesar cada robot definido
        for robot_def in &program.robots_definidos {
            let mut robot_prototype = RobotPrototype::new(
                robot_def.nombre.clone(),
                robot_def.ubicacion
            );
            
            // Cargar la definición del robot
            robot_prototype.load_robot_definition(robot_def);
            
            self.info.robots.push(robot_prototype);
        }

        // Optimizar instrucciones de cada robot
       for robot in &mut self.info.robots {
            let mut instructions = robot.instructions.clone();
            robot.optimize_instructions(&mut instructions);
            robot.instructions = instructions;
        }
    }

    // Método para obtener información del estado actual
    pub fn get_robot_info(&self, robot_name: &str) -> Option<&RobotPrototype> {
        self.info.robots.iter().find(|r| r.type_name == robot_name)
    }

    pub fn get_all_robots(&self) -> &Vec<RobotPrototype> {
        &self.info.robots
    }

    pub fn get_areas_definidas(&self) -> &Vec<Area> {
        &self.info.areas_definidas
    }

    pub fn get_inicializaciones(&self) -> &Vec<InicializacionRobot> {
        &self.info.inicializaciones
    }

    pub fn get_info(&self) -> &Info {
        &self.info
    }
}