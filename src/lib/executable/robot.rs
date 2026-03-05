/*
    Robot Instruments
*/
pub enum VariableType {
    Numero(i64),
    Boolean(bool),
    Null
}

pub struct SimpleMathBot{
    variables : Vec<(String,VariableType)>,
    last_result : VariableType
}

impl SimpleMathBot {
    pub fn new() -> Self {
        SimpleMathBot {
            variables: Vec::new(),
            last_result: VariableType::Null,
        }
    }
}


pub struct InstructionExecuter{
    instrucctions : Vec<String>,
    next_line : (u128, i64),
    procedures : Vec<String>,
    interrumption : char
}

impl InstructionExecuter{
    fn new(inst : Vec<String>) -> Self {
        InstructionExecuter {
            instrucctions: inst,
            next_line: (0, -1),
            procedures: Vec::new(),
            interrumption: '_',
        }
    }
}   

pub struct SuperSensor {
    HayFlorEnLaEsquina : bool,
    HayPapelEnLaEsquina : bool,
    // HayObstaculoEnLaEsquina : bool, "A futuro implementar"
}

impl SuperSensor {
    pub fn new() -> Self {
        SuperSensor {
            HayFlorEnLaEsquina: false,
            HayPapelEnLaEsquina: false,
        }
    }
}

pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

pub struct SuperNavigator {
    actual_position: (u8, u8),
    destination: (u8, u8),
    direction : Direction,
    my_areas : Vec<String>,
    actual_area : String
}

impl SuperNavigator {
    pub fn new(pos_x : u8, pos_y : u8 , areas : Vec<String>) -> Self {
        SuperNavigator {
            actual_position: (pos_x, pos_y),
            destination: (pos_x, pos_y),
            direction: Direction::Up,
            my_areas: areas,
            actual_area: String::new(),
        }
    }
}

pub struct Conmutron{
    waiting_message : bool,
    message : String,
    send_to : String
}

impl Conmutron {
    pub fn new() -> Self {
        Conmutron {
            waiting_message: false,
            message: String::new(),
            send_to: String::new(),
        }
    }
}

pub struct SuperBag {
    HayFlorEnLaBolsa : bool,
    HayPapelEnLaBolsa : bool,
    flowers : u32,
    papers : u32,
}

impl SuperBag {
    pub fn new() -> Self {
        SuperBag {
            HayFlorEnLaBolsa: false,
            HayPapelEnLaBolsa: false,
            flowers: 0,
            papers: 0,
        }
    }
}

/* 
    Robot Principal
*/

pub struct Robot {
    //Atributtes
    name: String,
    robot_type: String,
    color: String,
    is_active: bool,
    //Instrumments
    bag : SuperBag,
    sensor : SuperSensor,
    navigator : SuperNavigator,
    communicator : Conmutron,
    aritmetic_bot : SimpleMathBot,
    instruction_executer : InstructionExecuter
}

impl Robot{
    pub fn new(name : String, robot_type : String, color : String, pos_x : u8, pos_y : u8, areas : Vec<String>,instr : Vec<String>) -> Self {
        Robot {
            name,
            robot_type,
            color,
            is_active: false,
            bag: SuperBag::new(),
            sensor: SuperSensor::new(),
            navigator: SuperNavigator::new(pos_x, pos_y, areas),
            communicator: Conmutron::new(),
            aritmetic_bot : SimpleMathBot::new(),
            instruction_executer : InstructionExecuter::new(instr),
        }
    }
}