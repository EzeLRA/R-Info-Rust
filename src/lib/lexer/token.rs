use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TokenType {
    Parameter,
    Str,
    Num,
    Bool,
    BoolValue,
    Keyword,
    Indent,
    Dedent,
    ControlSentence,
    ElementalInstruction,
    Identifier,
    EndFile,
    // Operadores específicos
    Declaration,
    Assign,
    Equals,
    NotEquals,
    Less,
    LessEqual,
    GreaterEqual,
    Greater,
    And,
    Or,
    Not,
    Comma,
    Plus,
    Minus,
    Multiply,
    Divide,
}

impl TokenType {
    pub fn as_str(&self) -> &'static str {
        match self {
            TokenType::Parameter => "PARAMETER",
            TokenType::Str => "STRING",
            TokenType::Num => "NUMBER",
            TokenType::Bool => "BOOLEAN",
            TokenType::BoolValue => "BOOLEAN_VALUE",
            TokenType::Keyword => "KEYWORD",
            TokenType::Indent => "INDENT",
            TokenType::Dedent => "DEDENT",
            TokenType::ControlSentence => "CONTROL_SENTENCE",
            TokenType::ElementalInstruction => "ELEMENTAL_INSTRUCTION",
            TokenType::Identifier => "IDENTIFIER",
            TokenType::EndFile => "EOF",
            TokenType::Declaration => "DECLARATION",
            TokenType::Assign => "ASSIGN",
            TokenType::Equals => "EQUALS",
            TokenType::NotEquals => "NOT_EQUALS",
            TokenType::LessEqual => "LESS_EQUAL",
            TokenType::Less => "LESS",
            TokenType::Greater => "GREATER",
            TokenType::GreaterEqual => "GREATER_EQUAL",
            TokenType::And => "AND",
            TokenType::Or => "OR",
            TokenType::Not => "NOT",
            TokenType::Comma => "COMMA",
            TokenType::Plus => "PLUS",
            TokenType::Minus => "MINUS",
            TokenType::Multiply => "MULTIPLY",
            TokenType::Divide => "DIVIDE"
        }
    }
}

impl std::fmt::Display for TokenType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub value: String,
    pub line: usize,
    pub column: usize,
}

impl Token {
    pub fn new(token_type: TokenType, value: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            token_type,
            value: value.into(),
            line,
            column,
        }
    }
    pub fn is_mat_operator(&self) -> bool {
        matches!(self.token_type, TokenType::Plus | TokenType::Minus | TokenType::Multiply | TokenType::Divide)||matches!(self.token_type, TokenType::Equals | TokenType::NotEquals | TokenType::Less | TokenType::LessEqual | TokenType::Greater | TokenType::GreaterEqual)||matches!(self.token_type, TokenType::And | TokenType::Or | TokenType::Not)
    }
}

#[derive(Debug, Clone)]
pub struct Keywords {
    pub basic_keywords: HashMap<&'static str, &'static str>,
    pub control_sentences: HashMap<&'static str, &'static str>,
    pub elemental_instructions: HashMap<&'static str, &'static str>,
    pub keyword_map: HashMap<String, TokenType>,
    pub types_defined: HashMap<String, TokenType>,
}

impl Keywords {
    pub fn new() -> Self {
        let mut basic_keywords = HashMap::new();
        basic_keywords.insert("KEYWORD1", "proceso");
        basic_keywords.insert("KEYWORD2", "robot");
        basic_keywords.insert("KEYWORD3", "variables");
        basic_keywords.insert("KEYWORD4", "comenzar");
        basic_keywords.insert("KEYWORD5", "fin");
        basic_keywords.insert("KEYWORD6", "programa");
        basic_keywords.insert("KEYWORD7", "procesos");
        basic_keywords.insert("KEYWORD8", "areas");
        basic_keywords.insert("KEYWORD9", "robots");

        let mut control_sentences = HashMap::new();
        control_sentences.insert("CONTROL_SENTENCE1", "si");
        control_sentences.insert("CONTROL_SENTENCE2", "sino");
        control_sentences.insert("CONTROL_SENTENCE3", "mientras");
        control_sentences.insert("CONTROL_SENTENCE4", "repetir");

        let mut elemental_instructions = HashMap::new();
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION1", "Iniciar");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION2", "derecha");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION3", "mover");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION4", "tomarFlor");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION5", "tomarPapel");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION6", "depositarFlor");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION7", "depositarPapel");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION8", "PosAv");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION9", "PosCa");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION10", "HayFlorEnLaBolsa");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION11", "HayPapelEnLaBolsa");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION12", "HayFlorEnLaEsquina");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION13", "HayPapelEnLaEsquina");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION14", "Pos");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION15", "Informar");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION16", "AsignarArea");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION17", "AreaC");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION18", "AreaPC");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION19", "AreaP");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION20", "Leer");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION21", "Random");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION22", "BloquearEsquina");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION23", "LiberarEsquina");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION24", "EnviarMensaje");
        elemental_instructions.insert("ELEMENTAL_INSTRUCTION25", "RecibirMensaje");

        let mut types_defined = HashMap::new();
        types_defined.insert("numero".to_string(), TokenType::Num);
        types_defined.insert("booleano".to_string(), TokenType::Bool);
        types_defined.insert("V".to_string(), TokenType::BoolValue);
        types_defined.insert("F".to_string(), TokenType::BoolValue);

        let mut keyword_map = HashMap::new();

        for (_, value) in basic_keywords.iter() {
            keyword_map.insert(value.to_string(), TokenType::Keyword);
        }

        for (_, value) in control_sentences.iter() {
            keyword_map.insert(value.to_string(), TokenType::ControlSentence);
        }

        for (_, value) in elemental_instructions.iter() {
            keyword_map.insert(value.to_string(), TokenType::ElementalInstruction);
        }

        Self {
            basic_keywords,
            control_sentences,
            elemental_instructions,
            keyword_map,
            types_defined,
        }
    }

    pub fn get_token_type(&self, word: &str) -> Option<TokenType> {
        self.keyword_map.get(word).copied()
            .or_else(|| self.types_defined.get(word).copied())
    }

    pub fn is_basic_keyword(&self, word: &str) -> bool {
        self.basic_keywords.values().any(|&v| v == word)
    }

    pub fn is_control_sentence(&self, word: &str) -> bool {
        self.control_sentences.values().any(|&v| v == word)
    }

    pub fn is_elemental_instruction(&self, word: &str) -> bool {
        self.elemental_instructions.values().any(|&v| v == word)
    }

    pub fn is_type_defined(&self, word: &str) -> bool {
        self.types_defined.contains_key(word)
    }
}

impl Default for Keywords {
    fn default() -> Self {
        Self::new()
    }
}