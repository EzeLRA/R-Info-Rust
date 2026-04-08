use crate::lib::{optimizer::bot_simplyfier::BotSimplifier, parser::ast::{
    Area, AsignacionArea, Expresion, InicializacionRobot, Instruccion, Proceso, Program, Robot, RobotInstanciado, Variable
}};

#[derive(Debug, Clone)]
pub struct RobotPrototype {
    pub type_name: String,
    pub my_variables: Vec<Variable>,
    pub instructions: Vec<Instruccion>,
}

impl RobotPrototype {
    pub fn new(type_name: String) -> Self {
        RobotPrototype {
            type_name,
            my_variables: Vec::new(),
            instructions: Vec::new(),
        }
    }

    pub fn load_robot_definition(&mut self, robot_def: &Robot) {
        self.my_variables = robot_def.variables.clone();
        self.instructions = robot_def.instrucciones.clone();
    }

    pub fn optimize_instructions(&mut self, instructions: &mut Vec<Instruccion>) {
        let mut math_bot = BotSimplifier::new();

        instructions.iter_mut().for_each(|instr| {
            match instr {
                Instruccion::Asignacion { variable, expresion_texto } => {
                    math_bot.process_ecuation(expresion_texto);
                    *expresion_texto = math_bot.get_simplified_expression().unwrap();
                },
                Instruccion::Si { condicion_texto, entonces, sino } => {
                    math_bot.process_ecuation(condicion_texto);
                    *condicion_texto = math_bot.get_simplified_expression().unwrap();
                    self.optimize_instructions(entonces);
                    if !sino.is_empty() {
                        self.optimize_instructions(sino);
                    }
                },
                Instruccion::Mientras { condicion_texto, cuerpo } => {
                    math_bot.process_ecuation(condicion_texto);
                    *condicion_texto = math_bot.get_simplified_expression().unwrap();
                    self.optimize_instructions(cuerpo);
                },
                Instruccion::Repetir { condicion_texto, cuerpo } => {
                    math_bot.process_ecuation(condicion_texto);
                    *condicion_texto = math_bot.get_simplified_expression().unwrap();
                    self.optimize_instructions(cuerpo);
                },
                _ => {}
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
            let mut robot_prototype = RobotPrototype::new(robot_def.nombre.clone());
            
            // Cargar la definición del robot
            robot_prototype.load_robot_definition(robot_def);
            
            self.info.robots.push(robot_prototype);
        }

        // Aquí iría la lógica adicional de optimización
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