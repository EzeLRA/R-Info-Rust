// Definiciones de AST
#[derive(Debug, Clone)]
pub enum ASTNode {
    Program {
        name: String,
        body: Vec<ASTNode>,
    },
    ProcesosSection {
        procesos: Vec<ASTNode>,
    },
    Proceso {
        name: String,
        parameters: Vec<Parameter>,
        variables: Option<Box<ASTNode>>,
        body: Vec<ASTNode>,
    },
    Parameter {
        direction: String,
        name: String,
        param_type: String,
    },
    AreasSection {
        areas: Vec<ASTNode>,
    },
    AreaDefinition {
        name: String,
        area_type: String,
        dimensions: Vec<String>,
    },
    RobotsSection {
        robots: Vec<ASTNode>,
    },
    Robot {
        name: String,
        variables: Option<Vec<ASTNode>>,
        body: Vec<ASTNode>,
    },
    VariablesSection {
        declarations: Vec<ASTNode>,
    },
    VariableDeclaration {
        name: String,
        variable_type: String,
    },
    MainBlock {
        body: Vec<ASTNode>,
    },
    IfStatement {
        condition: Condition,
        consequent: Vec<ASTNode>,
        alternate: Option<Vec<ASTNode>>,
    },
    WhileStatement {
        condition: Condition,
        body: Vec<ASTNode>,
    },
    RepeatStatement {
        count: String,
        body: Vec<ASTNode>,
    },
    Assignment {
        target: Option<String>,
        operator: String,
        value: String,
    },
    ElementalInstruction {
        instruction: String,
        parameters: Vec<String>,
    },
    ProcessCall {
        name: String,
        parameters: Vec<String>,
    },
    Condition {
        expression: String,
    },
    Operator {
        operator: String,
    },
    Value {
        value: String,
    },
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub direction: String,
    pub name: String,
    pub param_type: String,
}

#[derive(Debug, Clone)]
pub struct Condition {
    pub expression: String,
}