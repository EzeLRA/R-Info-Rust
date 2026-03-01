use std::collections::HashMap;

#[derive(Debug, PartialEq, Clone)]
pub enum TokenOp {
    Num(i32),
    Var(String),
    Op(char),
    ParenLeft,
    ParenRight,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expression {
    Constant(i32),
    Variable(String),
    BinaryOp(Box<Expression>, char, Box<Expression>),
}

impl Expression {
    fn to_string(&self) -> String {
        match self {
            Expression::Constant(n) => n.to_string(),
            Expression::Variable(name) => name.clone(),
            Expression::BinaryOp(left, op, right) => {
                // Mejorar la representación para evitar paréntesis innecesarios
                let left_str = match **left {
                    Expression::BinaryOp(_, op_left, _) if self.op_precedence(*op) > self.op_precedence(op_left) => {
                        format!("({})", left.to_string())
                    },
                    _ => left.to_string(),
                };
                
                let right_str = match **right {
                    Expression::BinaryOp(_, op_right, _) if self.op_precedence(*op) >= self.op_precedence(op_right) => {
                        format!("({})", right.to_string())
                    },
                    _ => right.to_string(),
                };
                
                format!("{} {} {}", left_str, op, right_str)
            }
        }
    }
    
    fn op_precedence(&self, op: char) -> u8 {
        match op {
            '+' | '-' => 1,
            '*' | '/' => 2,
            _ => 0
        }
    }
    
    fn simplify(&self) -> Expression {
        match self {
            Expression::BinaryOp(left, op, right) => {
                let left_simp = left.simplify();
                let right_simp = right.simplify();
                
                // Primero intentar simplificaciones básicas
                if let Some(simplified) = self.simplify_basic(&left_simp, op, &right_simp) {
                    return simplified;
                }
                
                // Luego intentar simplificaciones algebraicas
                if let Some(simplified) = self.simplify_algebraic(&left_simp, op, &right_simp) {
                    return simplified;
                }
                
                // Si no se puede simplificar más, mantener la operación
                Expression::BinaryOp(Box::new(left_simp), *op, Box::new(right_simp))
            },
            _ => self.clone(),
        }
    }
    
    fn simplify_basic(&self, left: &Expression, op: &char, right: &Expression) -> Option<Expression> {
        match (op, left, right) {
            // Simplificaciones con 0
            ('+', Expression::Constant(0), expr) => Some(expr.clone()),
            ('+', expr, Expression::Constant(0)) => Some(expr.clone()),
            ('-', expr, Expression::Constant(0)) => Some(expr.clone()),
            ('*', Expression::Constant(0), _) => Some(Expression::Constant(0)),
            ('*', _, Expression::Constant(0)) => Some(Expression::Constant(0)),
            ('*', Expression::Constant(1), expr) => Some(expr.clone()),
            ('*', expr, Expression::Constant(1)) => Some(expr.clone()),
            ('/', expr, Expression::Constant(1)) => Some(expr.clone()),
            
            // Simplificaciones con números negativos
            ('+', expr, Expression::Constant(n)) if *n < 0 => {
                Some(Expression::BinaryOp(
                    Box::new(expr.clone()), 
                    '-', 
                    Box::new(Expression::Constant(-n))
                ))
            },
            
            // Evaluación de constantes
            (op, Expression::Constant(a), Expression::Constant(b)) => {
                match op {
                    '+' => Some(Expression::Constant(a + b)),
                    '-' => Some(Expression::Constant(a - b)),
                    '*' => Some(Expression::Constant(a * b)),
                    '/' => {
                        if *b != 0 {
                            Some(Expression::Constant(a / b))
                        } else {
                            None // Mantener división por cero como expresión
                        }
                    }
                    _ => None,
                }
            },
            
            _ => None,
        }
    }
    
    fn simplify_algebraic(&self, left: &Expression, op: &char, right: &Expression) -> Option<Expression> {
        match (op, left, right) {
            // Propiedad conmutativa: ordenar términos para mejor simplificación
            ('+', Expression::Variable(_), Expression::Constant(_)) => {
                Some(Expression::BinaryOp(
                    Box::new(right.clone()),
                    '+',
                    Box::new(left.clone())
                ))
            },
            ('*', Expression::Variable(_), Expression::Constant(_)) => {
                Some(Expression::BinaryOp(
                    Box::new(right.clone()),
                    '*',
                    Box::new(left.clone())
                ))
            },
            
            // Combinar términos semejantes: x + x = 2x
            ('+', Expression::Variable(v1), Expression::Variable(v2)) if v1 == v2 => {
                Some(Expression::BinaryOp(
                    Box::new(Expression::Constant(2)),
                    '*',
                    Box::new(Expression::Variable(v1.clone()))
                ))
            },
            
            // x * x = x^2 (aunque no tenemos exponentes, podemos representarlo como x*x)
            ('*', Expression::Variable(v1), Expression::Variable(v2)) if v1 == v2 => {
                Some(Expression::BinaryOp(
                    Box::new(Expression::Variable(v1.clone())),
                    '*',
                    Box::new(Expression::Variable(v1.clone()))
                ))
            },
            
            // Propiedad distributiva: a*(b + c) = a*b + a*c
            ('*', left_expr, Expression::BinaryOp(add_left, add_op, add_right)) if *add_op == '+' => {
                Some(Expression::BinaryOp(
                    Box::new(Expression::BinaryOp(
                        Box::new(left_expr.clone()),
                        '*',
                        add_left.clone()
                    )),
                    '+',
                    Box::new(Expression::BinaryOp(
                        Box::new(left_expr.clone()),
                        '*',
                        add_right.clone()
                    ))
                ))
            },
            
            // (a + b)*c = a*c + b*c
            ('*', Expression::BinaryOp(add_left, add_op, add_right), right_expr) if *add_op == '+' => {
                Some(Expression::BinaryOp(
                    Box::new(Expression::BinaryOp(
                        add_left.clone(),
                        '*',
                        Box::new(right_expr.clone())
                    )),
                    '+',
                    Box::new(Expression::BinaryOp(
                        add_right.clone(),
                        '*',
                        Box::new(right_expr.clone())
                    ))
                ))
            },
            
            // Simplificar 2*x + 3*x = 5*x
            ('+', 
            Expression::BinaryOp(left1, op1, right1),
            Expression::BinaryOp(left2, op2, right2)
            ) if *op1 == '*' && *op2 == '*' => {
                // Caso: constante*x + constante*x
                match (left1.as_ref(), right1.as_ref(), left2.as_ref(), right2.as_ref()) {
                    (Expression::Constant(c1), Expression::Variable(v1), 
                    Expression::Constant(c2), Expression::Variable(v2)) if v1 == v2 => {
                        Some(Expression::BinaryOp(
                            Box::new(Expression::Constant(c1 + c2)),
                            '*',
                            Box::new(Expression::Variable(v1.clone()))
                        ))
                    },
                    _ => None
                }
            },
            
            // Simplificar x*2 + x*3 = x*5
            ('+',
            Expression::BinaryOp(left1, op1, right1),
            Expression::BinaryOp(left2, op2, right2)
            ) if *op1 == '*' && *op2 == '*' => {
                match (left1.as_ref(), right1.as_ref(), left2.as_ref(), right2.as_ref()) {
                    (Expression::Variable(v1), Expression::Constant(c1),
                    Expression::Variable(v2), Expression::Constant(c2)) if v1 == v2 => {
                        Some(Expression::BinaryOp(
                            Box::new(Expression::Variable(v1.clone())),
                            '*',
                            Box::new(Expression::Constant(c1 + c2))
                        ))
                    },
                    _ => None
                }
            },
            
            // x - x = 0
            ('-', Expression::Variable(v1), Expression::Variable(v2)) if v1 == v2 => {
                Some(Expression::Constant(0))
            },
            
            // x*2 - x = x
            ('-', 
            Expression::BinaryOp(left1, op1, right1),
            Expression::Variable(v2)
            ) if *op1 == '*' => {
                match (left1.as_ref(), right1.as_ref()) {
                    (Expression::Variable(v1), Expression::Constant(c)) if v1 == v2 && *c == 2 => {
                        Some(Expression::Variable(v1.clone()))
                    },
                    _ => None
                }
            },
            
            _ => None,
        }
    }
    
    fn collect_terms(&self) -> HashMap<String, i32> {
        let mut terms = HashMap::new();
        self.collect_terms_rec(&mut terms, 1);
        terms
    }
    
    fn collect_terms_rec(&self, terms: &mut HashMap<String, i32>, coefficient: i32) {
        match self {
            Expression::Constant(n) => {
                *terms.entry("__constant__".to_string()).or_insert(0) += n * coefficient;
            },
            Expression::Variable(name) => {
                *terms.entry(name.clone()).or_insert(0) += coefficient;
            },
            Expression::BinaryOp(left, op, right) => {
                match op {
                    '+' => {
                        left.collect_terms_rec(terms, coefficient);
                        right.collect_terms_rec(terms, coefficient);
                    },
                    '-' => {
                        left.collect_terms_rec(terms, coefficient);
                        right.collect_terms_rec(terms, -coefficient);
                    },
                    '*' => {
                        match (left.as_ref(), right.as_ref()) {
                            (Expression::Constant(c), expr) => {
                                expr.collect_terms_rec(terms, coefficient * c);
                            },
                            (expr, Expression::Constant(c)) => {
                                expr.collect_terms_rec(terms, coefficient * c);
                            },
                            _ => {
                                // Producto de variables, por ahora lo dejamos como está
                                let term = format!("({}*{})", left.to_string(), right.to_string());
                                *terms.entry(term).or_insert(0) += coefficient;
                            }
                        }
                    },
                    '/' => {
                        match (left.as_ref(), right.as_ref()) {
                            (expr, Expression::Constant(c)) => {
                                if *c != 0 {
                                    expr.collect_terms_rec(terms, coefficient / c);
                                }
                            },
                            _ => {
                                let term = format!("({}/{})", left.to_string(), right.to_string());
                                *terms.entry(term).or_insert(0) += coefficient;
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }
    
    fn from_terms(terms: HashMap<String, i32>) -> Expression {
        let mut expr: Option<Expression> = None;
        let mut constant = 0;
        
        for (term, coeff) in terms {
            if coeff == 0 {
                continue;
            }
            
            if term == "__constant__" {
                constant = coeff;
                continue;
            }
            
            let term_expr = if term.contains('*') || term.contains('/') {
                // Es una expresión compuesta, necesitamos parsearla de nuevo
                // Por simplicidad, creamos una variable especial
                Expression::Variable(term)
            } else if coeff == 1 {
                Expression::Variable(term)
            } else {
                Expression::BinaryOp(
                    Box::new(Expression::Constant(coeff)),
                    '*',
                    Box::new(Expression::Variable(term))
                )
            };
            
            expr = match expr {
                None => Some(term_expr),
                Some(e) => Some(Expression::BinaryOp(
                    Box::new(e),
                    '+',
                    Box::new(term_expr)
                ))
            };
        }
        
        if constant != 0 {
            let const_expr = Expression::Constant(constant);
            expr = match expr {
                None => Some(const_expr),
                Some(e) => Some(Expression::BinaryOp(
                    Box::new(e),
                    '+',
                    Box::new(const_expr)
                ))
            };
        }
        
        expr.unwrap_or(Expression::Constant(0))
    }
    
    fn evaluate(&self, context: &HashMap<String, i32>) -> Result<i32, String> {
        match self {
            Expression::Constant(n) => Ok(*n),
            Expression::Variable(name) => {
                if let Some(&value) = context.get(name) {
                    Ok(value)
                } else {
                    Err(format!("Variable '{}' no definida", name))
                }
            },
            Expression::BinaryOp(left, op, right) => {
                let left_val = left.evaluate(context)?;
                let right_val = right.evaluate(context)?;
                
                match op {
                    '+' => Ok(left_val + right_val),
                    '-' => Ok(left_val - right_val),
                    '*' => Ok(left_val * right_val),
                    '/' => {
                        if right_val == 0 {
                            Err("División por cero".to_string())
                        } else {
                            Ok(left_val / right_val)
                        }
                    },
                    _ => Err(format!("Operador '{}' no válido", op)),
                }
            }
        }
    }
}

/*
    Bot Core - Funcionalidad base de procesamiento de tokens y construcción de árbol
*/
trait TokenProcessor {
    fn level_priority(&self, op: char) -> u8 {
        match op {
            '+' | '-' => 1,
            '*' | '/' => 2,
            _ => 0
        }
    }
    
    fn process_ecuation_string(&self, ecuation: &str) -> Vec<TokenOp> {
        let mut tokens = Vec::new();
        let mut it = ecuation.chars().filter(|c| !c.is_whitespace()).peekable();

        let mut wait_number = true; 

        while let Some(&c) = it.peek() {
            match c {
                '0'..='9' => {
                    let mut num_str = String::new();
                    while let Some(&c) = it.peek() {
                        if c.is_ascii_digit() { 
                            num_str.push(it.next().unwrap()); 
                        } else { 
                            break; 
                        }
                    }
                    tokens.push(TokenOp::Num(num_str.parse::<i32>().unwrap()));
                    wait_number = false;
                }
                'a'..='z' | 'A'..='Z' | '_' => {
                    let mut var_str = String::new();
                    while let Some(&c) = it.peek() {
                        if c.is_ascii_alphabetic() || c == '_' || c.is_ascii_digit() { 
                            var_str.push(it.next().unwrap()); 
                        } else { 
                            break; 
                        }
                    }
                    tokens.push(TokenOp::Var(var_str));
                    wait_number = false;
                }
                '-' if wait_number => {
                    tokens.push(TokenOp::Num(0));
                    tokens.push(TokenOp::Op(it.next().unwrap()));
                }
                '+' | '-' | '*' | '/' => {
                    tokens.push(TokenOp::Op(it.next().unwrap()));
                    wait_number = true;
                }
                '(' => {
                    tokens.push(TokenOp::ParenLeft);
                    it.next();
                    wait_number = true;
                }
                ')' => {
                    tokens.push(TokenOp::ParenRight);
                    it.next();
                    wait_number = false;
                }
                _ => { it.next(); }
            }
        }
        tokens
    }
    
    fn shunting_yard(&self, tokens: &[TokenOp]) -> Vec<TokenOp> {
        let mut result = Vec::new();
        let mut stack_ops = Vec::new();

        for token in tokens {
            match token {
                TokenOp::Num(_) | TokenOp::Var(_) => result.push(token.clone()),
                TokenOp::ParenLeft => stack_ops.push(token.clone()),
                TokenOp::ParenRight => {
                    while let Some(top) = stack_ops.pop() {
                        if top == TokenOp::ParenLeft { break; }
                        result.push(top);
                    }
                }
                TokenOp::Op(c) => {
                    while let Some(TokenOp::Op(top_c)) = stack_ops.last() {
                        if self.level_priority(*top_c) >= self.level_priority(*c) {
                            result.push(stack_ops.pop().unwrap());
                        } else { break; }
                    }
                    stack_ops.push(TokenOp::Op(*c));
                }
            }
        }
        while let Some(op) = stack_ops.pop() {
            result.push(op);
        }
        result
    }
    
    fn build_expression_tree(&self, tokens: &[TokenOp]) -> Option<Expression> {
        let rpn_tokens = self.shunting_yard(tokens);
        let mut stack: Vec<Expression> = Vec::new();
        
        for token in rpn_tokens {
            match token {
                TokenOp::Num(n) => stack.push(Expression::Constant(n)),
                TokenOp::Var(name) => stack.push(Expression::Variable(name)),
                TokenOp::Op(op) => {
                    if stack.len() < 2 {
                        return None;
                    }
                    let right = Box::new(stack.pop().unwrap());
                    let left = Box::new(stack.pop().unwrap());
                    stack.push(Expression::BinaryOp(left, op, right));
                }
                _ => {}
            }
        }
        
        stack.pop()
    }
}

/*
    Bot Simplifier - Simplifyies expressions by applying algebraic rules and collecting like terms
*/
#[derive(Debug, PartialEq, Clone)]
pub struct BotSimplifier {
    tokens: Vec<TokenOp>,
    expression: Option<Expression>,
}

impl TokenProcessor for BotSimplifier {}

impl BotSimplifier {
    pub fn new() -> BotSimplifier {
        BotSimplifier {
            tokens: Vec::new(),
            expression: None,
        }
    }

    pub fn process_ecuation(&mut self, ecuation: &str) {
        self.tokens = self.process_ecuation_string(ecuation);
        self.expression = self.build_expression_tree(&self.tokens);
    }

    pub fn get_simplified_expression(&self) -> Option<String> {
        if let Some(expr) = &self.expression {
            // Primero simplificar con reglas básicas
            let simplified = expr.simplify();
            
            // Luego recolectar términos semejantes
            let terms = simplified.collect_terms();
            
            // Reconstruir expresión a partir de términos
            let final_expr = Expression::from_terms(terms);
            
            Some(final_expr.to_string())
        } else {
            None
        }
    }
    
    pub fn get_expression_tree(&self) -> Option<Expression> {
        self.expression.clone()
    }
}