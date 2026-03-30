//! A Rust library for formatting PostgreSQL SQL and PL/pgSQL.
//!
//! Uses [tree-sitter-postgres](https://crates.io/crates/tree-sitter-postgres)
//! for parsing, supporting 7 formatting styles based on popular SQL style guides.
//!
//! # Quick start
//!
//! ```
//! use libpgfmt::{format, style::Style};
//!
//! let sql = "SELECT id, name FROM users WHERE active = TRUE";
//! let formatted = format(sql, Style::River).unwrap();
//! assert_eq!(formatted, "SELECT id,\n       name\n  FROM users\n WHERE active = TRUE;");
//! ```
//!
//! # Styles
//!
//! The [`Style`] enum provides 7 formatting variants:
//!
//! - **River** — keywords right-aligned to form a visual river
//! - **Mozilla** — keywords left-aligned, content indented 4 spaces
//! - **AWeber** (default) — river with JOINs in keyword alignment
//! - **Dbt** — lowercase keywords, blank lines between clauses
//! - **Gitlab** — 2-space indent, uppercase keywords
//! - **Kickstarter** — 2-space indent, compact JOINs
//! - **Mattmc3** — lowercase river with leading commas
//!
//! Styles can be parsed from strings:
//!
//! ```
//! use libpgfmt::style::Style;
//!
//! let style: Style = "dbt".parse().unwrap();
//! assert_eq!(style, Style::Dbt);
//! ```
//!
//! # PL/pgSQL
//!
//! Format PL/pgSQL function bodies (the content between `$$` delimiters)
//! with [`format_plpgsql`]:
//!
//! ```
//! use libpgfmt::{format_plpgsql, style::Style};
//!
//! let body = "BEGIN RETURN 1; END";
//! let formatted = format_plpgsql(body, Style::River).unwrap();
//! assert_eq!(formatted, "BEGIN\n  RETURN 1;\nEND;");
//! ```

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
///
/// # Examples
///
/// ```
/// use libpgfmt::{format, style::Style};
///
/// // River style
/// let result = format("SELECT id FROM users WHERE active = TRUE", Style::River).unwrap();
/// assert_eq!(result, "SELECT id\n  FROM users\n WHERE active = TRUE;");
///
/// // dbt style (lowercase, blank lines)
/// let result = format("SELECT id FROM users", Style::Dbt).unwrap();
/// assert_eq!(result, "select id\n\nfrom users;");
/// ```
///
/// # Errors
///
/// Returns [`FormatError::Syntax`] if the input contains a syntax error that
/// prevents formatting, or [`FormatError::Parser`] if the tree-sitter parser
/// cannot be initialized.
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
///
/// # Examples
///
/// ```
/// use libpgfmt::{format_plpgsql, style::Style};
///
/// let body = "BEGIN RETURN 1; END";
/// let formatted = format_plpgsql(body, Style::River).unwrap();
/// assert_eq!(formatted, "BEGIN\n  RETURN 1;\nEND;");
/// ```
///
/// # Errors
///
/// Returns [`FormatError::Syntax`] if the PL/pgSQL body contains a syntax
/// error, or [`FormatError::Parser`] if the parser cannot be initialized.
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

/// Check whether the parse tree has a structural error that would prevent
/// meaningful formatting.
///
/// The tree-sitter-postgres grammar has known limitations that produce ERROR
/// nodes for valid SQL (e.g., `IS NOT NULL AND`, parenthesized boolean
/// expressions, dollar-quoted function bodies). We only reject input when
/// the parser couldn't produce any valid statement structure at all.
fn has_structural_error(root: &tree_sitter::Node) -> bool {
    // If the parser produced at least one valid toplevel_stmt, the errors
    // are grammar limitations (expression-level conflicts, dollar-quoted
    // bodies, etc.) — not fundamentally broken SQL. Format what we can.
    let mut cursor = root.walk();
    let has_valid_stmt = root
        .named_children(&mut cursor)
        .any(|c| c.kind() == "toplevel_stmt");
    if has_valid_stmt {
        return false;
    }
    // No valid statements at all — this is genuinely broken input.
    true
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
