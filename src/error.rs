use std::fmt;

/// Errors that can occur during formatting.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FormatError {
    /// The tree-sitter parser could not be initialized.
    Parser(String),
    /// The input SQL contains a syntax error.
    Syntax(String),
}

impl fmt::Display for FormatError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FormatError::Parser(msg) => write!(f, "Parser error: {msg}"),
            FormatError::Syntax(msg) => write!(f, "Syntax error: {msg}"),
        }
    }
}

impl std::error::Error for FormatError {}
