use std::fmt;

// Error del compilador
#[derive(Debug, Clone)]
pub struct CompilerError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

impl CompilerError {
    pub fn new(message: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            message: message.into(),
            line,
            column,
        }
    }
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (línea {}, columna {})", self.message, self.line, self.column)
    }
}

impl std::error::Error for CompilerError {}