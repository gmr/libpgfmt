pub mod error;
mod formatter;
mod node_helpers;
pub mod style;

use error::FormatError;
use formatter::Formatter;
use style::Style;
use tree_sitter::Parser;
use tree_sitter_postgres::{LANGUAGE, LANGUAGE_PLPGSQL};

/// Format one or more PostgreSQL SQL statements according to the specified style.
///
/// Input may contain multiple semicolon-separated statements.
/// Statements without trailing semicolons are handled gracefully.
pub fn format(sql: &str, style: Style) -> Result<String, FormatError> {
    let trimmed = sql.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    // The grammar requires trailing semicolons; ensure they are present.
    // If the input ends with a line comment (--), append the semicolon on a
    // new line so it doesn't become part of the comment.
    let input = if trimmed.ends_with(';') {
        trimmed.to_string()
    } else if trimmed.lines().last().is_some_and(|l| l.contains("--")) {
        format!("{trimmed}\n;")
    } else {
        format!("{trimmed};")
    };
    let mut parser = Parser::new();
    parser
        .set_language(&LANGUAGE.into())
        .map_err(|e| FormatError::Parser(e.to_string()))?;
    let tree = parser
        .parse(&input, None)
        .ok_or_else(|| FormatError::Parser("Failed to parse SQL".into()))?;
    let root = tree.root_node();
    // The tree-sitter-postgres grammar doesn't handle some valid SQL
    // constructs (e.g., decimal literals like 800.00). When errors are
    // limited to leaf nodes, attempt to format anyway so the rest of the
    // statement is still properly styled. Only bail out when the tree
    // structure is fundamentally broken (ERROR at the top level wrapping
    // major statement parts).
    if root.has_error() && has_structural_error(&root) {
        return Err(FormatError::Syntax(find_error_message(&root, &input)));
    }
    let fmt = Formatter::new(&input, style);
    fmt.format_root(root)
}

/// Format PL/pgSQL code according to the specified style.
///
/// The input should be the body of a PL/pgSQL function (the content between
/// the dollar-quote delimiters, typically starting with DECLARE or BEGIN).
pub fn format_plpgsql(code: &str, style: Style) -> Result<String, FormatError> {
    let trimmed = code.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }
    let mut parser = Parser::new();
    parser
        .set_language(&LANGUAGE_PLPGSQL.into())
        .map_err(|e| FormatError::Parser(e.to_string()))?;
    let tree = parser
        .parse(trimmed, None)
        .ok_or_else(|| FormatError::Parser("Failed to parse PL/pgSQL".into()))?;
    let root = tree.root_node();
    if root.has_error() {
        return Err(FormatError::Syntax(find_error_message(&root, trimmed)));
    }
    let fmt = Formatter::new(trimmed, style);
    fmt.format_plpgsql_root(root)
}

/// Check whether the parse tree has a structural error (ERROR node wrapping
/// significant content). Small ERROR nodes (e.g., unparsed decimal fraction
/// like ".00") are tolerable and can be passed through as-is.
fn has_structural_error(node: &tree_sitter::Node) -> bool {
    if node.is_error() {
        // Small leaf ERROR nodes (e.g., ".00" decimal part = 3 bytes) are
        // tolerable grammar gaps. Anything larger likely indicates a genuine
        // parse failure that would produce garbled output.
        let size = node.end_byte() - node.start_byte();
        return size > 4;
    }
    if node.is_missing() {
        return true;
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.has_error() && has_structural_error(&child) {
            return true;
        }
    }
    false
}

fn find_error_message(node: &tree_sitter::Node, source: &str) -> String {
    if node.is_error() || node.is_missing() {
        let start = node.start_position();
        return format!(
            "Syntax error at line {}, column {}: {:?}",
            start.row + 1,
            start.column + 1,
            &source[node.byte_range()]
        );
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.has_error() {
            return find_error_message(&child, source);
        }
    }
    "Unknown syntax error".into()
}
