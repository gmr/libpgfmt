mod expr;
mod plpgsql;
mod select;
mod stmt;

use crate::error::FormatError;
use crate::node_helpers::NodeExt;
use crate::style::Style;
use tree_sitter::Node;

/// Configuration derived from the style.
#[derive(Debug, Clone)]
pub(crate) struct StyleConfig {
    /// Keyword casing: true = UPPER, false = lower.
    pub upper_keywords: bool,
    /// Indentation string for left-aligned styles.
    pub indent: &'static str,
    /// Use leading commas instead of trailing.
    pub leading_commas: bool,
    /// JOINs participate in river alignment (AWeber, mattmc3).
    pub joins_in_river: bool,
    /// Always use explicit INNER JOIN (never plain JOIN).
    pub explicit_inner_join: bool,
    /// Insert blank lines between major clauses.
    pub blank_lines_between_clauses: bool,
    /// Use river (right-aligned keyword) layout.
    pub river: bool,
    /// Compact CTE chaining (Kickstarter: "), name AS (").
    pub compact_ctes: bool,
    /// JOIN ON on same line as JOIN (Kickstarter).
    pub join_on_same_line: bool,
    /// Blank lines inside CTE bodies (GitLab).
    pub blank_lines_in_ctes: bool,
    /// Strip INNER keyword from INNER JOIN (mattmc3: use plain JOIN).
    pub strip_inner_join: bool,
}

impl StyleConfig {
    pub fn from_style(style: Style) -> Self {
        match style {
            Style::River => Self {
                upper_keywords: true,
                indent: "    ",
                leading_commas: false,
                joins_in_river: false,
                explicit_inner_join: false,
                blank_lines_between_clauses: false,
                river: true,
                compact_ctes: false,
                join_on_same_line: false,
                blank_lines_in_ctes: false,
                strip_inner_join: false,
            },
            Style::Mozilla => Self {
                upper_keywords: true,
                indent: "    ",
                leading_commas: false,
                joins_in_river: false,
                explicit_inner_join: false,
                blank_lines_between_clauses: false,
                river: false,
                compact_ctes: false,
                join_on_same_line: false,
                blank_lines_in_ctes: false,
                strip_inner_join: false,
            },
            Style::Aweber => Self {
                upper_keywords: true,
                indent: "    ",
                leading_commas: false,
                joins_in_river: true,
                explicit_inner_join: false,
                blank_lines_between_clauses: false,
                river: true,
                compact_ctes: false,
                join_on_same_line: false,
                blank_lines_in_ctes: false,
                strip_inner_join: false,
            },
            Style::Dbt => Self {
                upper_keywords: false,
                indent: "    ",
                leading_commas: false,
                joins_in_river: false,
                explicit_inner_join: true,
                blank_lines_between_clauses: true,
                river: false,
                compact_ctes: false,
                join_on_same_line: false,
                blank_lines_in_ctes: false,
                strip_inner_join: false,
            },
            Style::Gitlab => Self {
                upper_keywords: true,
                indent: "  ",
                leading_commas: false,
                joins_in_river: false,
                explicit_inner_join: true,
                blank_lines_between_clauses: false,
                river: false,
                compact_ctes: false,
                join_on_same_line: false,
                blank_lines_in_ctes: true,
                strip_inner_join: false,
            },
            Style::Kickstarter => Self {
                upper_keywords: true,
                indent: "  ",
                leading_commas: false,
                joins_in_river: false,
                explicit_inner_join: true,
                blank_lines_between_clauses: false,
                river: false,
                compact_ctes: true,
                join_on_same_line: true,
                blank_lines_in_ctes: false,
                strip_inner_join: false,
            },
            Style::Mattmc3 => Self {
                upper_keywords: false,
                indent: "    ",
                leading_commas: true,
                joins_in_river: true,
                explicit_inner_join: false,
                blank_lines_between_clauses: false,
                river: true,
                compact_ctes: false,
                join_on_same_line: false,
                blank_lines_in_ctes: false,
                strip_inner_join: true,
            },
        }
    }
}

/// The core SQL formatter.
pub(crate) struct Formatter<'a> {
    pub source: &'a str,
    pub style: Style,
    pub config: StyleConfig,
}

impl<'a> Formatter<'a> {
    pub fn new(source: &'a str, style: Style) -> Self {
        Self {
            source,
            style,
            config: StyleConfig::from_style(style),
        }
    }

    /// Format the root `source_file` node containing one or more statements.
    pub fn format_root(&self, root: Node<'a>) -> Result<String, FormatError> {
        let mut results = Vec::new();
        let mut cursor = root.walk();
        for child in root.named_children(&mut cursor) {
            if child.kind() == "toplevel_stmt"
                && let Some(stmt) = child.find_child("stmt")
            {
                results.push(self.format_stmt(stmt)?);
            }
        }
        if results.is_empty() {
            return Ok(String::new());
        }
        Ok(results.join("\n\n"))
    }

    /// Format a PL/pgSQL root node.
    pub fn format_plpgsql_root(&self, root: Node<'a>) -> Result<String, FormatError> {
        if let Some(block) = root.find_child("pl_block") {
            return Ok(self.format_plpgsql_block(block, 0));
        }
        // Fallback: return normalized source.
        Ok(root.text(self.source).to_string())
    }

    /// Apply keyword casing.
    pub fn kw(&self, keyword: &str) -> String {
        if self.config.upper_keywords {
            keyword.to_uppercase()
        } else {
            keyword.to_lowercase()
        }
    }

    /// Format a two-word keyword pair like "GROUP BY" or "ORDER BY".
    pub fn kw_pair(&self, first: &str, second: &str) -> String {
        format!("{} {}", self.kw(first), self.kw(second))
    }

    /// Get the text of a node.
    pub fn text(&self, node: Node<'a>) -> &'a str {
        node.text(self.source)
    }
}
