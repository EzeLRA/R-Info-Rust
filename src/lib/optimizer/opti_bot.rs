use crate::lib::parser::processor::{
    AsignacionArea, Instruccion, Proceso, Program, Variable, 
    Robot, Expresion, InicializacionRobot
};

#[derive(Debug, Clone)]
pub struct RobotPrototype {
    pub type_name: String,
    pub position: (i32, i32),
    pub my_variables: Vec<Variable>,
    pub instructions: Vec<Instruccion>,
    pub area_asignada: Option<Expresion>,
    pub inicializado: bool,
}

impl RobotPrototype {
    pub fn new(type_name: String) -> Self {
        RobotPrototype {
            type_name,
            position: (0, 0),
            my_variables: Vec::new(),
            instructions: Vec::new(),
            area_asignada: None,
            inicializado: false,
        }
    }

    pub fn capture_init_position(&mut self, list: &Vec<AsignacionArea>) {
        if let Some(asignacion) = list.iter().find(|asignacion| {
            if let Expresion::Identificador(robot_nombre) = &asignacion.robot {
                robot_nombre == &self.type_name
            } else {
                false
            }
        }) {
            self.area_asignada = Some(asignacion.area.clone());
        }
    }

    pub fn capture_initialization(&mut self, list: &Vec<InicializacionRobot>) {
        if let Some(inicializacion) = list.iter().find(|init| {
            if let Expresion::Identificador(robot_nombre) = &init.robot {
                robot_nombre == &self.type_name
            } else {
                false
            }
        }) {
            // Capturar las coordenadas de inicialización si son números
            if let Expresion::Numero(x) = inicializacion.pos_x {
                if let Expresion::Numero(y) = inicializacion.pos_y {
                    self.position = (x, y);
                }
            }
            self.inicializado = true;
        }
    }

    pub fn load_robot_definition(&mut self, robot_def: &Robot) {
        self.my_variables = robot_def.variables.clone();
        self.instructions = robot_def.instrucciones.clone();
    }
}

pub struct Info {
    pub name_program: String,
    pub procedures: Vec<Proceso>,
    pub robots: Vec<RobotPrototype>,
    pub areas_robot: Vec<AsignacionArea>,
    pub areas_definidas: Vec<crate::lib::parser::processor::Area>,
    pub inicializaciones: Vec<InicializacionRobot>,
    pub robots_instanciados: Vec<crate::lib::parser::processor::RobotInstanciado>,
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

pub struct Optimizer {
    info: Info,
}

impl Optimizer {
    pub fn new(info: Info) -> Self {
        Optimizer { info }
    }

    pub fn process(&mut self, program: &Program) {
        // Limpiar información previa
        self.info = Info::new();
        
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
            
            // Capturar asignación de área si existe
            robot_prototype.capture_init_position(&program.asignaciones_areas);
            
            // Capturar inicialización si existe
            robot_prototype.capture_initialization(&program.inicializaciones);
            
            self.info.robots.push(robot_prototype);
        }

        // Validaciones y advertencias
        for robot_instanciado in &program.robots_instanciados {
            let robot_proto = self.info.robots.iter()
                .find(|r| r.type_name == robot_instanciado.tipo);
            
            if let Some(robot) = robot_proto {
                if !robot.inicializado {
                    println!("Advertencia: Robot instanciado '{}' de tipo '{}' no tiene inicialización",
                        robot_instanciado.nombre, robot_instanciado.tipo);
                }
                if robot.area_asignada.is_none() {
                    println!("Advertencia: Robot instanciado '{}' de tipo '{}' no tiene área asignada",
                        robot_instanciado.nombre, robot_instanciado.tipo);
                }
            } else {
                println!("Advertencia: Robot instanciado '{}' de tipo '{}' no tiene definición",
                    robot_instanciado.nombre, robot_instanciado.tipo);
            }
        }

        // Aquí iría la lógica adicional de optimización
       
    }

    // Método para obtener información del estado actual
    pub fn get_robot_info(&self, robot_name: &str) -> Option<&RobotPrototype> {
        self.info.robots.iter().find(|r| r.type_name == robot_name)
    }

    pub fn get_all_robots(&self) -> &Vec<RobotPrototype> {
        &self.info.robots
    }

    pub fn get_areas_definidas(&self) -> &Vec<crate::lib::parser::processor::Area> {
        &self.info.areas_definidas
    }

    pub fn get_inicializaciones(&self) -> &Vec<InicializacionRobot> {
        &self.info.inicializaciones
    }
}