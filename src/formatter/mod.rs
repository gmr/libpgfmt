mod expr;
mod lexical;
mod pgdump;
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
    /// Wrap CASE expressions when ELSE is present (AWeber, mattmc3).
    pub wrap_case_else: bool,
    /// Use the dedicated pg_dump/ruleutils renderer instead of the standard
    /// statement formatter.
    pub pg_dump: bool,
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
                wrap_case_else: false,
                pg_dump: false,
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
                wrap_case_else: false,
                pg_dump: false,
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
                wrap_case_else: true,
                pg_dump: false,
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
                wrap_case_else: false,
                pg_dump: false,
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
                wrap_case_else: false,
                pg_dump: false,
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
                wrap_case_else: false,
                pg_dump: false,
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
                wrap_case_else: true,
                pg_dump: false,
            },
            // PgDump uses a dedicated renderer (see formatter::pgdump); these
            // values are placeholders and are not consulted on that path.
            Style::PgDump => Self {
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
                wrap_case_else: false,
                pg_dump: true,
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
        let statements = root
            .named_children(&mut root.walk())
            .filter(|c| c.kind() == "toplevel_stmt")
            .count();
        for child in root.named_children(&mut cursor) {
            if child.kind() == "toplevel_stmt" {
                // Most statements are wrapped in a `stmt` node, but some parse
                // as a direct child of `toplevel_stmt` (e.g. the legacy
                // transaction statement `BEGIN`, which is a
                // `TransactionStmtLegacy`). Format the inner statement in
                // either case so it is not silently dropped.
                let stmt = child.find_child("stmt").unwrap_or(child);
                let source = self.text(stmt);
                if self.config.pg_dump {
                    let text = lexical::restore_literals(source, &self.format_pgdump_stmt(stmt)?);
                    let mut text = self.restore_comments(child, text);
                    // The deparser writes a lone statement without `;`, and
                    // the pg_dump fixtures keep that, but a second statement
                    // needs the first terminated or the two run together --
                    // see https://github.com/gmr/libpgfmt/issues/61.
                    if statements > 1 && !text.trim_end().ends_with(';') {
                        if lexical::ends_in_line_comment(&text) {
                            text.push('\n');
                        }
                        text.push(';');
                    }
                    results.push(text);
                } else {
                    let text = lexical::restore_literals(source, &self.format_stmt(stmt)?);
                    results.push(self.restore_comments(child, text));
                }
            } else if child.kind() == "comment"
                && let Some(prev) = child.prev_sibling()
                && prev.kind() != "comment"
                && prev.end_position().row == child.start_position().row
                && let Some(last) = results.last_mut()
            {
                // A comment on the same line as the end of a statement --
                // `SELECT 1; -- why` -- stays on that line. Moving it to a
                // paragraph of its own also made formatting unstable, since
                // restore_comments can place a comment after the `;`.
                last.push(' ');
                last.push_str(self.text(child).trim_end());
            } else if child.kind() == "comment" {
                // Standalone comments between/around top-level statements are
                // direct children of `source_file`; preserve them verbatim in
                // source order so they are not silently discarded. Comments
                // inside a statement are handled by restore_comments.
                results.push(self.text(child).trim_end().to_string());
            }
        }
        if results.is_empty() {
            return Ok(String::new());
        }
        Ok(results.join("\n\n"))
    }

    /// Put back the comments inside `stmt` that formatting dropped.
    ///
    /// When one cannot be placed, the statement is emitted as written, with
    /// only its whitespace collapsed: losing the formatting is better than
    /// losing the comment. See [`lexical::restore_comments`].
    fn restore_comments(&self, stmt: Node<'a>, formatted: String) -> String {
        let source = self.text(stmt);
        let mut comments = Vec::new();
        let mut stack = vec![stmt];
        while let Some(node) = stack.pop() {
            if node.kind() == "comment" {
                let offset = source[..node.start_byte() - stmt.start_byte()]
                    .chars()
                    .count();
                comments.push((offset, self.text(node).trim_end().to_string()));
            }
            let mut cursor = node.walk();
            stack.extend(node.children(&mut cursor));
        }
        if comments.is_empty() {
            return formatted;
        }
        comments.sort_by_key(|(offset, _)| *offset);
        if let Some(restored) = lexical::restore_comments(source, &comments, &formatted) {
            return restored;
        }
        let mut verbatim = lexical::collapse_whitespace(source);
        if formatted.trim_end().ends_with(';') && !verbatim.ends_with(';') {
            if lexical::ends_in_line_comment(&verbatim) {
                verbatim.push('\n');
            }
            verbatim.push(';');
        }
        verbatim
    }

    /// Format a PL/pgSQL root node.
    pub fn format_plpgsql_root(&self, root: Node<'a>) -> Result<String, FormatError> {
        if let Some(block) = root.find_child("pl_block") {
            let mut body = self.format_plpgsql_block(block, 0);
            // The outermost PL/pgSQL block should end with "END;" (semicolon
            // before the closing $$ delimiter).
            if !body.trim_end().ends_with(';') {
                body.push(';');
            }
            return Ok(body);
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
