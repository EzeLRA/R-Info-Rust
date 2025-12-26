use std::collections::{HashMap, HashSet};
use crate::lib::compilerError::CompilerError;
use super::super::parser::ast::{ASTNode, Condition, Parameter};

// Estructuras para estadísticas de comunicación
#[derive(Debug, Clone)]
pub struct CommunicationStats {
    pub sends: HashMap<String, usize>,
    pub receives: HashMap<String, usize>,
    pub connections: HashSet<String>,
    pub robot_communications: HashMap<String, RobotCommStats>,
}

#[derive(Debug, Clone)]
pub struct RobotCommStats {
    pub sends: usize,
    pub receives: usize,
    pub total: usize,
}

// Estructura para información de procesos
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub variables: Vec<ASTNode>,
    pub body_statements: usize,
    pub scope: String,
}

// Estructura para llamadas a procesos
#[derive(Debug, Clone)]
pub struct ProcessCallInfo {
    pub name: String,
    pub parameters: Vec<String>,
    pub line: usize,
    pub is_valid: bool,
}

// Estructura para código ejecutable
#[derive(Debug, Clone)]
pub struct ExecutableCode {
    pub programa: String,
    pub areas: Vec<AreaInfo>,
    pub robots: Vec<RobotExecutable>,
    pub procesos: Vec<ProcessExecutable>,
    pub main: Vec<ExecutableInstruction>,
    pub variables: HashMap<String, VariableInfo>,
}

#[derive(Debug, Clone)]
pub struct AreaInfo {
    pub name: String,
    pub area_type: String,
    pub dimensions: Vec<String>,
    pub bounds: AreaBounds,
}

#[derive(Debug, Clone)]
pub struct AreaBounds {
    pub x1: i32,
    pub y1: i32,
    pub x2: i32,
    pub y2: i32,
}

#[derive(Debug, Clone)]
pub struct RobotExecutable {
    pub name: String,
    pub instructions: Vec<ExecutableInstruction>,
    pub position: (i32, i32),
    pub direction: String,
    pub bag: (usize, usize), // (flores, papeles)
    pub active: bool,
}

#[derive(Debug, Clone)]
pub struct ProcessExecutable {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub instructions: Vec<ExecutableInstruction>,
}

#[derive(Debug, Clone)]
pub enum ExecutableInstruction {
    Elemental {
        instruction: String,
        parameters: Vec<String>,
        line: usize,
    },
    ProcessCall {
        process_name: String,
        parameters: Vec<String>,
        line: usize,
    },
    If {
        condition: Condition,
        consequent: Vec<ExecutableInstruction>,
        alternate: Vec<ExecutableInstruction>,
        line: usize,
    },
    While {
        condition: Condition,
        body: Vec<ExecutableInstruction>,
        line: usize,
    },
    Repeat {
        count: String,
        body: Vec<ExecutableInstruction>,
        line: usize,
    },
    Assignment {
        target: Option<String>,
        operator: String,
        value: ExpressionValue,
        line: usize,
    },
    Expression {
        expression: ExpressionValue,
        line: usize,
    },
    Unknown {
        original: ASTNode,
    },
}

#[derive(Debug, Clone)]
pub enum ExpressionValue {
    Variable(String),
    Number(i32),
    Boolean(bool),
    BinaryOperation {
        left: Box<ExpressionValue>,
        operator: String,
        right: Box<ExpressionValue>,
    },
    UnaryOperation {
        operator: String,
        operand: Box<ExpressionValue>,
    },
    ProcessCall {
        name: String,
        parameters: Vec<ExpressionValue>,
    },
}

#[derive(Debug, Clone)]
pub struct VariableInfo {
    pub name: String,
    pub var_type: String,
    pub value: Option<ExpressionValue>,
}

// Estructura para tabla de símbolos
#[derive(Debug, Clone)]
pub struct SymbolInfo {
    pub name: String,
    pub var_type: String,
    pub scope: String,
    pub initialized: bool,
    pub is_constant: bool,
}

// Resultado del análisis semántico
#[derive(Debug, Clone)]
pub struct SemanticAnalysisResult {
    pub symbol_table: Vec<SymbolInfo>,
    pub processes: Vec<ProcessInfo>,
    pub process_calls: Vec<ProcessCallInfo>,
    pub executable: ExecutableCode,
    pub errors: Vec<CompilerError>,
    pub success: bool,
    pub summary: AnalysisSummary,
    pub communication_stats: CommunicationResult,
}

#[derive(Debug, Clone)]
pub struct AnalysisSummary {
    pub total_instructions: usize,
    pub total_processes: usize,
    pub total_process_calls: usize,
    pub valid_process_calls: usize,
    pub total_errors: usize,
    pub total_variables: usize,
    pub total_robots: usize,
    pub total_areas: usize,
    pub total_conexiones: usize,
}

#[derive(Debug, Clone)]
pub struct CommunicationResult {
    pub total_sends: usize,
    pub total_receives: usize,
    pub total_connections: usize,
    pub effective_connections: usize,
    pub communicating_entities: Vec<String>,
    pub total_communicating_robots: usize,
    pub by_robot: Vec<RobotCommSummary>,
    pub total_conexiones: usize,
}

#[derive(Debug, Clone)]
pub struct RobotCommSummary {
    pub name: String,
    pub sends: usize,
    pub receives: usize,
    pub total: usize,
    pub is_communicating: bool,
}

// Estado temporal para seguimiento de expresiones
#[derive(Debug, Clone)]
struct ExpressionContext {
    current_expression: Vec<ExpressionValue>,
    in_assignment: bool,
    assignment_target: Option<String>,
    current_operator: Option<String>,
}

// Analizador semántico principal
pub struct SemanticAnalyzer {
    symbol_table: Vec<SymbolInfo>,
    scope_stack: Vec<HashMap<String, SymbolInfo>>,
    errors: Vec<CompilerError>,
    current_scope: String,
    processes_info: Vec<ProcessInfo>,
    process_calls: Vec<ProcessCallInfo>,
    executable_code: ExecutableCode,
    message_communications: CommunicationStats,
    expression_context: Option<ExpressionContext>,
    process_calls_in_expressions: Vec<(String, Vec<ExpressionValue>)>,
}

impl SemanticAnalyzer {
    pub fn new() -> Self {
        Self {
            symbol_table: Vec::new(),
            scope_stack: vec![HashMap::new()],
            errors: Vec::new(),
            current_scope: "global".to_string(),
            processes_info: Vec::new(),
            process_calls: Vec::new(),
            executable_code: ExecutableCode {
                programa: String::new(),
                areas: Vec::new(),
                robots: Vec::new(),
                procesos: Vec::new(),
                main: Vec::new(),
                variables: HashMap::new(),
            },
            message_communications: CommunicationStats {
                sends: HashMap::new(),
                receives: HashMap::new(),
                connections: HashSet::new(),
                robot_communications: HashMap::new(),
            },
            expression_context: None,
            process_calls_in_expressions: Vec::new(),
        }
    }

    pub fn analyze(&mut self, ast: &ASTNode) -> SemanticAnalysisResult {
        // Reiniciar estado
        self.symbol_table.clear();
        self.errors.clear();
        self.scope_stack = vec![HashMap::new()];
        self.processes_info.clear();
        self.process_calls.clear();
        self.executable_code = ExecutableCode {
            programa: String::new(),
            areas: Vec::new(),
            robots: Vec::new(),
            procesos: Vec::new(),
            main: Vec::new(),
            variables: HashMap::new(),
        };
        self.message_communications = CommunicationStats {
            sends: HashMap::new(),
            receives: HashMap::new(),
            connections: HashSet::new(),
            robot_communications: HashMap::new(),
        };
        self.expression_context = None;
        self.process_calls_in_expressions.clear();

        // Realizar análisis
        self.visit_program(ast);

        // Construir resultado
        SemanticAnalysisResult {
            symbol_table: self.get_formatted_symbol_table(),
            processes: self.processes_info.clone(),
            process_calls: self.process_calls.clone(),
            executable: self.executable_code.clone(),
            errors: self.errors.clone(),
            success: self.errors.is_empty(),
            summary: self.get_analysis_summary(),
            communication_stats: self.get_communication_stats(),
        }
    }

    // NUEVO: Iniciar contexto de expresión
    fn start_expression_context(&mut self, in_assignment: bool, target: Option<String>) {
        self.expression_context = Some(ExpressionContext {
            current_expression: Vec::new(),
            in_assignment,
            assignment_target: target,
            current_operator: None,
        });
    }

    // NUEVO: Finalizar contexto de expresión
    fn end_expression_context(&mut self) -> Option<ExpressionValue> {
        if let Some(mut context) = self.expression_context.take() {
            if context.current_expression.len() == 1 {
                return Some(context.current_expression.remove(0));
            } else if !context.current_expression.is_empty() {
                // Construir expresión binaria si hay múltiples elementos
                return self.build_expression_from_context(context);
            }
        }
        None
    }

    // NUEVO: Construir expresión binaria desde contexto
    fn build_expression_from_context(&mut self, context: ExpressionContext) -> Option<ExpressionValue> {
        let mut expression_stack = context.current_expression;
        
        if expression_stack.len() % 2 == 1 {
            // Número impar de elementos: a + b - c
            while expression_stack.len() > 1 {
                let right = expression_stack.pop()?;
                let operator = if let Some(op) = context.current_operator.clone() {
                    op
                } else {
                    "+".to_string() // Operador por defecto
                };
                let left = expression_stack.pop()?;
                
                let binary_op = ExpressionValue::BinaryOperation {
                    left: Box::new(left),
                    operator: operator.clone(),
                    right: Box::new(right),
                };
                expression_stack.push(binary_op);
            }
            expression_stack.pop()
        } else {
            self.errors.push(CompilerError::new(
                "Expresión mal formada".to_string(),
                0, 0
            ));
            None
        }
    }

    // NUEVO: Agregar valor al contexto de expresión actual
    fn add_to_expression(&mut self, value: ExpressionValue) {
        if let Some(context) = &mut self.expression_context {
            context.current_expression.push(value);
        }
    }

    // NUEVO: Agregar operador al contexto de expresión actual
    fn add_operator_to_expression(&mut self, operator: String) {
        if let Some(context) = &mut self.expression_context {
            context.current_operator = Some(operator);
        }
    }

    // Método para registrar envío de mensajes
    fn register_message_send(&mut self, sender: &str, target: Option<&str>) {
        let sender_name = self.get_current_entity_name();
        
        // Registrar envío del sender
        *self.message_communications.sends.entry(sender_name.clone())
            .or_insert(0) += 1;
        
        // Registrar comunicación si hay un target específico
        if let Some(target) = target {
            let connection_key = format!("{}->{}", sender_name, target);
            self.message_communications.connections.insert(connection_key);
        }
        
        // Actualizar estadísticas del robot/proceso actual
        self.update_robot_communication_stats(&sender_name, "send");
    }

    // Método para registrar recepción de mensajes
    fn register_message_receive(&mut self, receiver: &str, source: Option<&str>) {
        let receiver_name = self.get_current_entity_name();
        
        // Registrar recepción del receiver
        *self.message_communications.receives.entry(receiver_name.clone())
            .or_insert(0) += 1;
        
        // Registrar comunicación si hay un source específico
        if let Some(source) = source {
            let connection_key = format!("{}->{}", source, receiver_name);
            self.message_communications.connections.insert(connection_key);
        }
        
        // Actualizar estadísticas del robot/proceso actual
        self.update_robot_communication_stats(&receiver_name, "receive");
    }

    // Obtener el nombre de la entidad actual
    fn get_current_entity_name(&self) -> String {
        if self.current_scope.starts_with("robot:") {
            self.current_scope.replace("robot:", "")
        } else if self.current_scope.starts_with("proceso:") {
            self.current_scope.replace("proceso:", "")
        } else if self.current_scope == "main" {
            "main".to_string()
        } else {
            "global".to_string()
        }
    }

    // Actualizar estadísticas de comunicación
    fn update_robot_communication_stats(&mut self, entity_name: &str, comm_type: &str) {
        let stats = self.message_communications.robot_communications
            .entry(entity_name.to_string())
            .or_insert(RobotCommStats {
                sends: 0,
                receives: 0,
                total: 0,
            });
        
        match comm_type {
            "send" => stats.sends += 1,
            "receive" => stats.receives += 1,
            _ => {}
        }
        
        stats.total = stats.sends + stats.receives;
    }

    // Analizar comunicaciones en instrucciones de mensajes
    fn analyze_message_communication(&mut self, node: &ASTNode) {
        if let ASTNode::ElementalInstruction { instruction, parameters, .. } = node {
            let current_entity = self.get_current_entity_name();
            
            if instruction == "EnviarMensaje" {
                // Registrar envío de mensaje
                let target = if !parameters.is_empty() {
                    self.extract_parameter_value(&parameters[0])
                } else {
                    None
                };
                self.register_message_send(&current_entity, target.as_deref());
                
            } else if instruction == "RecibirMensaje" {
                // Registrar recepción de mensaje
                let source = if !parameters.is_empty() {
                    self.extract_parameter_value(&parameters[0])
                } else {
                    None
                };
                self.register_message_receive(&current_entity, source.as_deref());
            }
        }
    }

    // Extraer valor de parámetro
    fn extract_parameter_value(&self, param: &str) -> Option<String> {
        // Aquí podrías implementar lógica más compleja para extraer valores
        Some(param.to_string())
    }

    // Obtener estadísticas de comunicación
    fn get_communication_stats(&self) -> CommunicationResult {
        let total_sends: usize = self.message_communications.sends.values().sum();
        let total_receives: usize = self.message_communications.receives.values().sum();
        
        // Calcular entidades que participaron en comunicación
        let mut communicating_entities = HashSet::new();
        
        for (entity, &count) in &self.message_communications.sends {
            if count > 0 {
                communicating_entities.insert(entity.clone());
            }
        }
        
        for (entity, &count) in &self.message_communications.receives {
            if count > 0 {
                communicating_entities.insert(entity.clone());
            }
        }
        
        // Calcular conexiones efectivas
        let effective_connections = self.message_communications.connections
            .iter()
            .filter(|conn| {
                let parts: Vec<&str> = conn.split("->").collect();
                if parts.len() == 2 {
                    let receiver = parts[1];
                    self.message_communications.receives.get(receiver)
                        .map_or(false, |&count| count > 0)
                } else {
                    false
                }
            })
            .count();
        
        let by_robot = self.message_communications.robot_communications
            .iter()
            .map(|(name, stats)| RobotCommSummary {
                name: name.clone(),
                sends: stats.sends,
                receives: stats.receives,
                total: stats.total,
                is_communicating: stats.total > 0,
            })
            .collect();
        
        CommunicationResult {
            total_sends,
            total_receives,
            total_connections: self.message_communications.connections.len(),
            effective_connections,
            communicating_entities: communicating_entities.clone().into_iter().collect(),
            total_communicating_robots: communicating_entities.len(),
            by_robot,
            total_conexiones: self.calculate_total_conexiones(),
        }
    }

    // Calcular total de conexiones
    fn calculate_total_conexiones(&self) -> usize {
        let mut communicating_entities = HashSet::new();
        
        for (entity, stats) in &self.message_communications.robot_communications {
            if stats.sends > 0 {
                // Verificar si hay algún receptor para los mensajes de este robot
                let mut has_receiver = false;
                
                for conn in &self.message_communications.connections {
                    if conn.starts_with(&format!("{}->", entity)) {
                        let parts: Vec<&str> = conn.split("->").collect();
                        if parts.len() == 2 {
                            let receiver = parts[1];
                            if self.message_communications.receives.get(receiver)
                                .map_or(false, |&count| count > 0) {
                                has_receiver = true;
                                break;
                            }
                        }
                    }
                }
                
                if has_receiver {
                    communicating_entities.insert(entity.clone());
                }
            }
        }
        
        communicating_entities.len()
    }

    // Métodos de visita del AST
    fn visit_program(&mut self, node: &ASTNode) {
        if let ASTNode::Program { name, body } = node {
            self.enter_scope("global".to_string());
            self.executable_code.programa = name.clone();
            
            for section in body {
                match section {
                    ASTNode::VariablesSection { declarations } => {
                        self.visit_variables_section(declarations);
                    }
                    ASTNode::ProcesosSection { procesos } => {
                        self.visit_procesos_section(procesos);
                    }
                    ASTNode::RobotsSection { robots } => {
                        self.visit_robots_section(robots);
                    }
                    ASTNode::MainBlock { body } => {
                        self.visit_main_block(body);
                    }
                    ASTNode::AreasSection { areas } => {
                        self.visit_areas_section(areas);
                    }
                    _ => {
                        self.errors.push(CompilerError::new(
                            format!("Sección no reconocida en programa: {:?}", section),
                            0, 0
                        ));
                    }
                }
            }
            
            self.exit_scope();
        }
    }

    fn visit_variables_section(&mut self, declarations: &[ASTNode]) {
        for decl in declarations {
            if let ASTNode::VariableDeclaration { name, variable_type } = decl {
                self.declare_variable(name, variable_type, "global".to_string());
                
                self.executable_code.variables.insert(
                    name.clone(),
                    VariableInfo {
                        name: name.clone(),
                        var_type: variable_type.clone(),
                        value: None,
                    }
                );
            }
        }
    }

    fn visit_areas_section(&mut self, areas: &[ASTNode]) {
        for area in areas {
            if let ASTNode::AreaDefinition { name, area_type, dimensions } = area {
                self.declare_variable(name, "area", "global".to_string());
                
                self.executable_code.areas.push(AreaInfo {
                    name: name.clone(),
                    area_type: area_type.clone(),
                    dimensions: dimensions.clone(),
                    bounds: self.calculate_area_bounds(dimensions),
                });
            }
        }
    }

    fn visit_procesos_section(&mut self, procesos: &[ASTNode]) {
        for proceso in procesos {
            if let ASTNode::Proceso { name, parameters, variables, body } = proceso {
                self.declare_process(name, parameters);
                
                let var_nodes = if let Some(boxed_vars) = variables {
                    if let ASTNode::VariablesSection { declarations } = &**boxed_vars {
                        declarations.clone()
                    } else {
                        Vec::new()
                    }
                } else {
                    Vec::new()
                };
                
                self.processes_info.push(ProcessInfo {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    variables: var_nodes,
                    body_statements: body.len(),
                    scope: format!("proceso:{}", name),
                });

                let executable_proceso = ProcessExecutable {
                    name: name.clone(),
                    parameters: parameters.clone(),
                    instructions: self.compile_instructions(body),
                };
                
                self.executable_code.procesos.push(executable_proceso);
                self.visit_proceso(proceso);
            }
        }
    }

    fn visit_proceso(&mut self, node: &ASTNode) {
        if let ASTNode::Proceso { name, parameters, body, .. } = node {
            self.enter_scope(format!("proceso:{}", name));
            
            for param in parameters {
                self.declare_variable(&param.name, &param.param_type, format!("proceso:{}", name));
            }

            self.visit_block(body);
            self.exit_scope();
        }
    }

    fn visit_robots_section(&mut self, robots: &[ASTNode]) {
        for robot in robots {
            if let ASTNode::Robot { name, variables, body } = robot {
                self.declare_variable(name, "robot", "global".to_string());
                
                self.executable_code.robots.push(RobotExecutable {
                    name: name.clone(),
                    instructions: self.compile_instructions(body),
                    position: (0, 0),
                    direction: "este".to_string(),
                    bag: (0, 0),
                    active: false,
                });
                
                self.enter_scope(format!("robot:{}", name));
                
                if let Some(vars) = variables {
                    for var in vars {
                        if let ASTNode::VariableDeclaration { name, variable_type } = var {
                            self.declare_variable(name, variable_type, format!("robot:{}", name));
                        }
                    }
                }
                
                self.visit_block(body);
                self.exit_scope();
            }
        }
    }

    fn visit_main_block(&mut self, body: &[ASTNode]) {
        self.enter_scope("main".to_string());
        self.executable_code.main = self.compile_instructions(body);
        self.visit_block(body);
        self.exit_scope();
    }

    fn visit_block(&mut self, statements: &[ASTNode]) {
        let mut i = 0;
        while i < statements.len() {
            let stmt = &statements[i];
            
            // Verificar si es un Value seguido de Assignment u Operator
            if let ASTNode::Value { value } = stmt {
                let next_stmt = if i + 1 < statements.len() {
                    Some(&statements[i + 1])
                } else {
                    None
                };
                
                match next_stmt {
                    Some(ASTNode::Assignment { target, operator, value: assign_value }) => {
                        // Caso: Value seguido de Assignment
                        self.visit_assignment_with_target(value, operator, assign_value);
                        i += 2; // Saltar ambos statements
                        continue;
                    }
                    Some(ASTNode::Operator { operator }) => {
                        // Caso: Value seguido de Operator (expresión)
                        self.start_expression_context(false, None);
                        self.visit_value_as_expression(value);
                        self.add_operator_to_expression(operator.clone());
                        
                        // Continuar procesando expresión
                        let mut j = i + 2;
                        while j < statements.len() {
                            match &statements[j] {
                                ASTNode::Value { value: next_val } => {
                                    self.visit_value_as_expression(next_val);
                                }
                                ASTNode::Operator { operator: next_op } => {
                                    self.add_operator_to_expression(next_op.clone());
                                }
                                _ => break,
                            }
                            j += 1;
                        }
                        
                        // Finalizar expresión
                        if let Some(expr) = self.end_expression_context() {
                            // Buscar Assignment después de la expresión
                            if j < statements.len() {
                                if let ASTNode::Assignment { target, operator: assign_op, value: assign_val } = &statements[j] {
                                    // Aquí deberíamos manejar la asignación completa
                                    // Por ahora, solo avanzamos el índice
                                    i = j + 1;
                                    continue;
                                }
                            }
                        }
                        
                        i = j;
                        continue;
                    }
                    Some(ASTNode::ProcessCall { name: call_name, parameters }) => {
                        // Caso: Value (nombre de proceso) seguido de ProcessCall
                        if self.lookup_process(value).is_some() {
                            // Es una llamada a proceso válida
                            self.visit_process_call(value, parameters);
                            i += 2; // Saltar Value y ProcessCall
                            continue;
                        }
                    }
                    _ => {}
                }
                
                // Si no es ninguno de los casos especiales, procesar como expresión simple
                self.visit_value_statement(value);
                i += 1;
            } else {
                // Procesar statement normal
                self.visit_statement(stmt);
                i += 1;
            }
        }
    }

    fn visit_statement(&mut self, node: &ASTNode) {
        match node {
            ASTNode::VariableDeclaration { name, variable_type } => {
                self.visit_variable_declaration(name, variable_type);
            }
            ASTNode::IfStatement { condition, consequent, alternate } => {
                self.visit_if_statement(condition, consequent, alternate);
            }
            ASTNode::WhileStatement { condition, body } => {
                self.visit_while_statement(condition, body);
            }
            ASTNode::RepeatStatement { count, body } => {
                self.visit_repeat_statement(count, body);
            }
            ASTNode::Assignment { target, operator, value } => {
                self.visit_assignment(target, operator, value);
            }
            ASTNode::ProcessCall { name, parameters } => {
                self.visit_process_call(name, parameters);
            }
            ASTNode::ElementalInstruction { instruction, parameters } => {
                self.visit_elemental_instruction(instruction, parameters);
            }
            ASTNode::AreaDefinition { name, area_type, dimensions } => {
                self.visit_area_definition(name, area_type, dimensions);
            }
            ASTNode::Value { value } => {
                self.visit_value_statement(value);
            }
            ASTNode::Operator { operator } => {
                // Esto solo debería ocurrir dentro de expresiones
                self.errors.push(CompilerError::new(
                    "Operador fuera de contexto de expresión".to_string(),
                    0, 0
                ));
            }
            _ => {
                self.errors.push(CompilerError::new(
                    format!("Tipo de statement no reconocido: {:?}", node),
                    0, 0
                ));
            }
        }
    }

    // NUEVO: Visitar Value como statement individual
    fn visit_value_statement(&mut self, value: &str) {
        // Verificar si es una variable declarada
        if let Some(symbol) = self.lookup_variable(value) {
            // Es una variable válida usada como statement
            // Esto podría ser para lectura o para pasar como parámetro
            if !symbol.initialized && !symbol.is_constant {
                self.errors.push(CompilerError::new(
                    format!("Variable '{}' no inicializada", value),
                    0, 0
                ));
            }
            
            // Agregar al contexto de expresión si estamos en uno
            if self.expression_context.is_some() {
                self.add_to_expression(ExpressionValue::Variable(value.to_string()));
            }
        } else if let Some(_) = self.lookup_process(value) {
            // Es un nombre de proceso, pero sin parámetros
            self.errors.push(CompilerError::new(
                format!("Llamada a proceso '{}' sin parámetros", value),
                0, 0
            ));
        } else if self.is_number(value) {
            // Es un número
            if let Ok(num) = value.parse::<i32>() {
                if self.expression_context.is_some() {
                    self.add_to_expression(ExpressionValue::Number(num));
                }
            }
        } else if self.is_boolean(value) {
            // Es un booleano
            let bool_val = matches!(value.to_lowercase().as_str(), "true" | "verdadero" | "v");
            if self.expression_context.is_some() {
                self.add_to_expression(ExpressionValue::Boolean(bool_val));
            }
        } else {
            // Identificador no reconocido
            self.errors.push(CompilerError::new(
                format!("Identificador '{}' no declarado", value),
                0, 0
            ));
        }
    }

    // NUEVO: Visitar Value como parte de expresión
    fn visit_value_as_expression(&mut self, value: &str) {
        self.visit_value_statement(value); // Misma lógica por ahora
    }

    // NUEVO: Visitar Assignment con target explícito
    fn visit_assignment_with_target(&mut self, target: &str, operator: &str, value: &str) {
        // Verificar que el target sea una variable declarada
        let target_symbol = self.lookup_variable(target);
        if target_symbol.is_none() {
            self.errors.push(CompilerError::new(
                format!("Variable '{}' no declarada", target),
                0, 0
            ));
        }
        
        // Verificar el valor
        if self.is_number(value) {
            // Es una asignación numérica
            if let Ok(num) = value.parse::<i32>() {
                // Verificar tipos si target existe
                if let Some(symbol) = target_symbol {
                    if symbol.var_type != "numero" {
                        self.errors.push(CompilerError::new(
                            format!("Tipo incompatible: no se puede asignar número a variable de tipo '{}'", symbol.var_type),
                            0, 0
                        ));
                    }
                }
            }
        } else if self.is_boolean(value) {
            // Es una asignación booleana
            if let Some(symbol) = target_symbol {
                if symbol.var_type != "booleano" && symbol.var_type != "bool" {
                    self.errors.push(CompilerError::new(
                        format!("Tipo incompatible: no se puede asignar booleano a variable de tipo '{}'", symbol.var_type),
                        0, 0
                    ));
                }
            }
        } else {
            // Es probablemente otra variable
            let value_symbol = self.lookup_variable(value);
            if let Some(value_sym) = value_symbol {
                if let Some(target_sym) = target_symbol {
                    if target_sym.var_type != value_sym.var_type {
                        self.errors.push(CompilerError::new(
                            format!("Tipo incompatible: no se puede asignar {} (tipo: {}) a {} (tipo: {})",
                                value, value_sym.var_type, target, target_sym.var_type),
                            0, 0
                        ));
                    }
                    
                    if !value_sym.initialized && !value_sym.is_constant {
                        self.errors.push(CompilerError::new(
                            format!("Variable '{}' no inicializada", value),
                            0, 0
                        ));
                    }
                }
            } else {
                // Variable no declarada
                self.errors.push(CompilerError::new(
                    format!("Variable '{}' no declarada", value),
                    0, 0
                ));
            }
        }
        
        // Marcar target como inicializado
        if let Some(mut symbol) = target_symbol {
            symbol.initialized = true;
        }
    }

    fn visit_variable_declaration(&mut self, name: &str, var_type: &str) {
        self.declare_variable(name, var_type, self.current_scope.clone());
    }

    fn visit_if_statement(&mut self, condition: &Condition, consequent: &[ASTNode], alternate: &Option<Vec<ASTNode>>) {
        self.visit_condition(condition);
        self.enter_scope("if".to_string());
        self.visit_block(consequent);
        self.exit_scope();
        
        if let Some(alt) = alternate {
            self.enter_scope("else".to_string());
            self.visit_block(alt);
            self.exit_scope();
        }
    }

    fn visit_while_statement(&mut self, condition: &Condition, body: &[ASTNode]) {
        self.visit_condition(condition);
        self.enter_scope("while".to_string());
        self.visit_block(body);
        self.exit_scope();
    }

    fn visit_repeat_statement(&mut self, count: &str, body: &[ASTNode]) {
        // Verificar que count sea un número o variable numérica
        if self.is_number(count) {
            if let Ok(count_val) = count.parse::<i32>() {
                if count_val <= 0 {
                    self.errors.push(CompilerError::new(
                        "El contador de repetición debe ser mayor a 0".to_string(),
                        0, 0
                    ));
                }
            }
        } else if let Some(symbol) = self.lookup_variable(count) {
            if symbol.var_type != "numero" {
                self.errors.push(CompilerError::new(
                    format!("El contador de repetición debe ser numérico, se obtuvo: {}", symbol.var_type),
                    0, 0
                ));
            }
            if !symbol.initialized {
                self.errors.push(CompilerError::new(
                    format!("Variable '{}' no inicializada en contador de repetición", count),
                    0, 0
                ));
            }
        } else {
            self.errors.push(CompilerError::new(
                format!("Contador de repetición no válido: '{}'", count),
                0, 0
            ));
        }
        
        self.enter_scope("repeat".to_string());
        self.visit_block(body);
        self.exit_scope();
    }

    fn visit_assignment(&mut self, target: &Option<String>, operator: &str, value: &str) {
        // Este método solo maneja asignaciones sin target explícito
        // (target: None, operator: ":=", value: "algo")
        
        if operator != ":=" {
            self.errors.push(CompilerError::new(
                format!("Operador de asignación no válido: '{}'", operator),
                0, 0
            ));
        }
        
        // Verificar el valor
        if self.is_number(value) {
            // Asignación de número (sin target)
            // Esto podría ser para inicializar algo, pero sin target es raro
        } else if self.is_boolean(value) {
            // Asignación de booleano (sin target)
        } else {
            // Posible variable
            if let Some(symbol) = self.lookup_variable(value) {
                if !symbol.initialized && !symbol.is_constant {
                    self.errors.push(CompilerError::new(
                        format!("Variable '{}' no inicializada", value),
                        0, 0
                    ));
                }
            } else {
                self.errors.push(CompilerError::new(
                    format!("Identificador '{}' no declarado", value),
                    0, 0
                ));
            }
        }
    }

    fn visit_process_call(&mut self, name: &str, parameters: &[String]) {
        self.process_calls.push(ProcessCallInfo {
            name: name.to_string(),
            parameters: parameters.to_vec(),
            line: 0, // TODO: Agregar línea del AST
            is_valid: false,
        });

        let process = self.lookup_process(name);
        if process.is_none() {
            self.errors.push(CompilerError::new(
                format!("Proceso '{}' no declarado", name),
                0, 0
            ));
            return;
        }

        let call_index = self.process_calls.len() - 1;
        self.process_calls[call_index].is_valid = true;

        // TODO: Verificar parámetros
        // Por ahora, solo registramos la llamada
    }

    fn visit_elemental_instruction(&mut self, instruction: &str, parameters: &[String]) {
        let valid_instructions = vec![
            "Iniciar", "derecha", "mover", "tomarFlor", "tomarPapel",
            "depositarFlor", "depositarPapel", "PosAv", "PosCa",
            "HayFlorEnLaBolsa", "HayPapelEnLaBolsa", "HayFlorEnLaEsquina", 
            "HayPapelEnLaEsquina", "Pos", "Informar", "AsignarArea",
            "Random", "BloquearEsquina", "LiberarEsquina",
            "EnviarMensaje", "RecibirMensaje"
        ];

        if !valid_instructions.contains(&instruction) {
            self.errors.push(CompilerError::new(
                format!("Instrucción elemental no reconocida: '{}'", instruction),
                0, 0
            ));
        }

        // Analizar comunicación para instrucciones de mensajes
        if instruction == "EnviarMensaje" || instruction == "RecibirMensaje" {
            // Crear un nodo temporal para análisis
            let temp_node = ASTNode::ElementalInstruction {
                instruction: instruction.to_string(),
                parameters: parameters.to_vec(),
            };
            self.analyze_message_communication(&temp_node);
        }

        // Verificar parámetros
        for param in parameters {
            if !self.is_number(param) && !self.is_boolean(param) {
                if let Some(symbol) = self.lookup_variable(param) {
                    if !symbol.initialized && !symbol.is_constant {
                        self.errors.push(CompilerError::new(
                            format!("Variable '{}' no inicializada en parámetro de instrucción", param),
                            0, 0
                        ));
                    }
                } else {
                    self.errors.push(CompilerError::new(
                        format!("Parámetro no válido: '{}'", param),
                        0, 0
                    ));
                }
            }
        }
    }

    fn visit_area_definition(&mut self, name: &str, area_type: &str, dimensions: &[String]) {
        let valid_area_types = vec!["AreaC", "AreaPC", "AreaP"];
        
        if !valid_area_types.contains(&area_type) {
            self.errors.push(CompilerError::new(
                format!("Tipo de área no reconocido: '{}'", area_type),
                0, 0
            ));
        }

        if dimensions.len() != 4 {
            self.errors.push(CompilerError::new(
                format!("El área '{}' debe tener exactamente 4 dimensiones", name),
                0, 0
            ));
        }

        for dim in dimensions {
            if !self.is_number(dim) && !self.lookup_variable(dim).is_some() {
                self.errors.push(CompilerError::new(
                    format!("Dimensión inválida en área '{}': {}", name, dim),
                    0, 0
                ));
            }
        }
    }

    fn visit_condition(&mut self, condition: &Condition) {
        let expr = &condition.expression;
        
        // Analizar expresión de condición
        // Separar en tokens y verificar cada uno
        let tokens: Vec<&str> = expr.split_whitespace().collect();
        let mut i = 0;
        
        while i < tokens.len() {
            let token = tokens[i];
            
            if self.is_operator(token) {
                // Es un operador (+, -, *, /, <, >, etc.)
                // Verificar que tenga operandos válidos a ambos lados
                if i == 0 || i == tokens.len() - 1 {
                    self.errors.push(CompilerError::new(
                        format!("Operador '{}' en posición inválida en condición", token),
                        0, 0
                    ));
                } else {
                    let left = tokens[i - 1];
                    let right = tokens[i + 1];
                    
                    // Verificar operandos
                    self.verify_operand_in_condition(left, token);
                    self.verify_operand_in_condition(right, token);
                    
                    i += 2; // Saltar operador y operando derecho
                }
            } else if !self.is_keyword(token) {
                // Es un identificador o valor
                self.verify_operand_in_condition(token, "");
            }
            
            i += 1;
        }
    }

    fn verify_operand_in_condition(&mut self, operand: &str, operator: &str) {
        if self.is_number(operand) {
            // Número válido
        } else if self.is_boolean(operand) {
            // Booleano válido
        } else if let Some(symbol) = self.lookup_variable(operand) {
            // Variable declarada
            if !symbol.initialized && !symbol.is_constant {
                self.errors.push(CompilerError::new(
                    format!("Variable '{}' no inicializada en condición", operand),
                    0, 0
                ));
            }
            
            // Verificar compatibilidad con operador
            match operator {
                "<" | ">" | "<=" | ">=" => {
                    if symbol.var_type != "numero" {
                        self.errors.push(CompilerError::new(
                            format!("Operador '{}' requiere operandos numéricos, se obtuvo: {}",
                                operator, symbol.var_type),
                            0, 0
                        ));
                    }
                }
                "&" | "|" => {
                    if symbol.var_type != "booleano" && symbol.var_type != "bool" {
                        self.errors.push(CompilerError::new(
                            format!("Operador '{}' requiere operandos booleanos, se obtuvo: {}",
                                operator, symbol.var_type),
                            0, 0
                        ));
                    }
                }
                _ => {}
            }
        } else {
            // Identificador no reconocido
            self.errors.push(CompilerError::new(
                format!("Identificador '{}' no declarado en condición", operand),
                0, 0
            ));
        }
    }

    // Métodos de compilación
    fn compile_instructions(&self, statements: &[ASTNode]) -> Vec<ExecutableInstruction> {
        let mut compiled = Vec::new();
        let mut i = 0;
        
        while i < statements.len() {
            let stmt = &statements[i];
            
            // Manejar patrones especiales
            if let ASTNode::Value { value } = stmt {
                let next_stmt = if i + 1 < statements.len() {
                    Some(&statements[i + 1])
                } else {
                    None
                };
                
                match next_stmt {
                    Some(ASTNode::Assignment { target, operator, value: assign_value }) => {
                        // Compilar como Assignment con target
                        let expr_value = self.compile_expression_value(assign_value);
                        compiled.push(ExecutableInstruction::Assignment {
                            target: Some(value.clone()),
                            operator: operator.clone(),
                            value: expr_value,
                            line: 0,
                        });
                        i += 2;
                        continue;
                    }
                    Some(ASTNode::ProcessCall { name: call_name, parameters }) => {
                        // Compilar como ProcessCall
                        compiled.push(ExecutableInstruction::ProcessCall {
                            process_name: value.clone(),
                            parameters: parameters.clone(),
                            line: 0,
                        });
                        i += 2;
                        continue;
                    }
                    _ => {}
                }
            }
            
            // Compilación normal
            compiled.push(self.compile_instruction(stmt));
            i += 1;
        }
        
        compiled
    }

    fn compile_instruction(&self, statement: &ASTNode) -> ExecutableInstruction {
        match statement {
            ASTNode::ElementalInstruction { instruction, parameters } => {
                ExecutableInstruction::Elemental {
                    instruction: instruction.clone(),
                    parameters: parameters.clone(),
                    line: 0, // TODO: Agregar línea
                }
            }
            ASTNode::ProcessCall { name, parameters } => {
                ExecutableInstruction::ProcessCall {
                    process_name: name.clone(),
                    parameters: parameters.clone(),
                    line: 0, // TODO: Agregar línea
                }
            }
            ASTNode::IfStatement { condition, consequent, alternate } => {
                ExecutableInstruction::If {
                    condition: condition.clone(),
                    consequent: self.compile_instructions(consequent),
                    alternate: alternate.as_ref().map_or_else(Vec::new, |alt| self.compile_instructions(alt)),
                    line: 0, // TODO: Agregar línea
                }
            }
            ASTNode::WhileStatement { condition, body } => {
                ExecutableInstruction::While {
                    condition: condition.clone(),
                    body: self.compile_instructions(body),
                    line: 0, // TODO: Agregar línea
                }
            }
            ASTNode::RepeatStatement { count, body } => {
                ExecutableInstruction::Repeat {
                    count: count.clone(),
                    body: self.compile_instructions(body),
                    line: 0, // TODO: Agregar línea
                }
            }
            ASTNode::Assignment { target, operator, value } => {
                ExecutableInstruction::Assignment {
                    target: target.clone(),
                    operator: operator.clone(),
                    value: self.compile_expression_value(value),
                    line: 0,
                }
            }
            ASTNode::Value { value } => {
                ExecutableInstruction::Expression {
                    expression: self.compile_expression_value(value),
                    line: 0,
                }
            }
            _ => ExecutableInstruction::Unknown {
                original: statement.clone(),
            }
        }
    }

    fn compile_expression_value(&self, value: &str) -> ExpressionValue {
        if let Ok(num) = value.parse::<i32>() {
            ExpressionValue::Number(num)
        } else if self.is_boolean(value) {
            let bool_val = matches!(value.to_lowercase().as_str(), "true" | "verdadero" | "v");
            ExpressionValue::Boolean(bool_val)
        } else {
            ExpressionValue::Variable(value.to_string())
        }
    }

    // Métodos auxiliares
    fn calculate_area_bounds(&self, dimensions: &[String]) -> AreaBounds {
        if dimensions.len() != 4 {
            return AreaBounds {
                x1: 0, y1: 0, x2: 99, y2: 99,
            };
        }
        
        AreaBounds {
            x1: dimensions[0].parse().unwrap_or(0),
            y1: dimensions[1].parse().unwrap_or(0),
            x2: dimensions[2].parse().unwrap_or(99),
            y2: dimensions[3].parse().unwrap_or(99),
        }
    }

    fn is_operator(&self, word: &str) -> bool {
        matches!(word, "+" | "-" | "*" | "/" | "=" | "<" | ">" | "!" | "&" | "|" | "," | ":" | "~" | "<=" | ">=" | "==" | "!=")
    }

    fn is_keyword(&self, word: &str) -> bool {
        let keywords = vec![
            "si", "sino", "mientras", "repetir", "proceso", "robot", "variables",
            "numero", "booleano", "comenzar", "fin", "programa", "procesos",
            "areas", "robots", "V", "F", "HayFlorEnLaEsquina" // Agregado como ejemplo
        ];
        keywords.contains(&word)
    }

    fn is_number(&self, word: &str) -> bool {
        word.parse::<i32>().is_ok()
    }

    fn is_boolean(&self, word: &str) -> bool {
        matches!(word.to_lowercase().as_str(), "true" | "false" | "verdadero" | "falso" | "v" | "f")
    }

    fn is_identifier(&self, word: &str) -> bool {
        let first_char = word.chars().next();
        first_char.map_or(false, |c| c.is_alphabetic() || c == '_')
    }

    fn enter_scope(&mut self, scope_name: String) {
        self.scope_stack.push(HashMap::new());
        self.current_scope = scope_name;
    }

    fn exit_scope(&mut self) {
        if self.scope_stack.len() > 1 {
            self.scope_stack.pop();
            // Actualizar current_scope basado en el nuevo scope superior
            if let Some(top) = self.scope_stack.last() {
                // Buscar algún símbolo que tenga información de scope
                for (_, symbol) in top {
                    self.current_scope = symbol.scope.clone();
                    break;
                }
            } else {
                self.current_scope = "global".to_string();
            }
        }
    }

    fn declare_variable(&mut self, name: &str, var_type: &str, scope: String) {
        let current_scope = self.scope_stack.last_mut().unwrap();
        
        if current_scope.contains_key(name) {
            self.errors.push(CompilerError::new(
                format!("Variable '{}' ya declarada en este ámbito", name),
                0, 0
            ));
        } else {
            let symbol = SymbolInfo {
                name: name.to_string(),
                var_type: var_type.to_string(),
                scope: scope.clone(),
                initialized: var_type == "robot" || var_type == "area", // Robots y áreas se consideran inicializados
                is_constant: false,
            };
            current_scope.insert(name.to_string(), symbol);
            
            // También agregar a la tabla de símbolos global
            self.symbol_table.push(SymbolInfo {
                name: name.to_string(),
                var_type: var_type.to_string(),
                scope,
                initialized: var_type == "robot" || var_type == "area",
                is_constant: false,
            });
        }
    }

    fn lookup_variable(&mut self, name: &str) -> Option<&mut SymbolInfo> {
        for scope in self.scope_stack.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return Some(symbol);
            }
        }
        None
    }

    fn declare_process(&mut self, name: &str, parameters: &[Parameter]) {
        let global_scope = self.scope_stack.first_mut().unwrap();
        global_scope.insert(format!("process:{}", name), SymbolInfo {
            name: name.to_string(),
            var_type: "process".to_string(),
            scope: "global".to_string(),
            initialized: true,
            is_constant: true,
        });
    }

    fn lookup_process(&self, name: &str) -> Option<&SymbolInfo> {
        self.scope_stack.first()
            .and_then(|scope| scope.get(&format!("process:{}", name)))
    }

    fn get_formatted_symbol_table(&self) -> Vec<SymbolInfo> {
        self.symbol_table.clone()
    }

    fn get_analysis_summary(&self) -> AnalysisSummary {
        let total_instructions = self.get_total_instructions();
        let valid_process_calls = self.process_calls.iter()
            .filter(|call| call.is_valid)
            .count();
        
        AnalysisSummary {
            total_instructions,
            total_processes: self.processes_info.len(),
            total_process_calls: self.process_calls.len(),
            valid_process_calls,
            total_errors: self.errors.len(),
            total_variables: self.get_formatted_symbol_table().len(),
            total_robots: self.executable_code.robots.len(),
            total_areas: self.executable_code.areas.len(),
            total_conexiones: self.calculate_total_conexiones(),
        }
    }

    fn get_total_instructions(&self) -> usize {
        let mut total = 0;
        
        // Instrucciones en procesos
        for proceso in &self.processes_info {
            total += proceso.body_statements;
        }
        
        // Instrucciones en robots (excluyendo llamadas a procesos)
        for robot in &self.executable_code.robots {
            for instr in &robot.instructions {
                if !matches!(instr, ExecutableInstruction::ProcessCall { .. }) {
                    total += 1;
                }
            }
        }
        
        // Instrucciones en main
        total += self.executable_code.main.len();
        
        total
    }
}