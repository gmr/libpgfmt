/// Expression formatting — converts expression AST nodes to inline SQL text.
use crate::node_helpers::{NodeExt, flatten_list};
use tree_sitter::Node;

use super::Formatter;

/// SQL built-in aggregate and function names that should follow keyword casing.
/// User-defined function names are preserved as-is.
const SQL_BUILTIN_FUNCTIONS: &[&str] = &[
    "abs",
    "avg",
    "array_agg",
    "bit_and",
    "bit_or",
    "bool_and",
    "bool_or",
    "cardinality",
    "cast",
    "ceil",
    "ceiling",
    "char_length",
    "character_length",
    "coalesce",
    "concat",
    "concat_ws",
    "convert",
    "corr",
    "count",
    "covar_pop",
    "covar_samp",
    "cume_dist",
    "current_date",
    "current_time",
    "current_timestamp",
    "date_part",
    "date_trunc",
    "dense_rank",
    "every",
    "exists",
    "exp",
    "extract",
    "first_value",
    "floor",
    "format",
    "generate_series",
    "greatest",
    "json_agg",
    "json_object_agg",
    "jsonb_agg",
    "jsonb_object_agg",
    "lag",
    "last_value",
    "lead",
    "least",
    "left",
    "length",
    "ln",
    "localtime",
    "localtimestamp",
    "log",
    "lower",
    "lpad",
    "ltrim",
    "max",
    "min",
    "mod",
    "now",
    "nth_value",
    "ntile",
    "nullif",
    "octet_length",
    "overlay",
    "percent_rank",
    "position",
    "power",
    "rank",
    "regexp_matches",
    "regexp_replace",
    "regexp_split_to_array",
    "regexp_split_to_table",
    "repeat",
    "replace",
    "reverse",
    "right",
    "round",
    "row_number",
    "rpad",
    "rtrim",
    "sign",
    "split_part",
    "sqrt",
    "stddev",
    "stddev_pop",
    "stddev_samp",
    "string_agg",
    "strpos",
    "substr",
    "substring",
    "sum",
    "to_char",
    "to_date",
    "to_number",
    "to_timestamp",
    "translate",
    "trim",
    "trunc",
    "unnest",
    "upper",
    "var_pop",
    "var_samp",
    "variance",
    "width_bucket",
    "xmlagg",
];

fn is_sql_builtin_function(name: &str) -> bool {
    let lower = name.to_lowercase();
    SQL_BUILTIN_FUNCTIONS.contains(&lower.as_str())
}

/// PostgreSQL internal type name → standard SQL type name mapping.
const PG_TYPE_MAP: &[(&str, &str)] = &[
    ("bigint", "BIGINT"),
    ("bigserial", "BIGSERIAL"),
    ("bool", "BOOLEAN"),
    ("boolean", "BOOLEAN"),
    ("bytea", "BYTEA"),
    ("char", "CHAR"),
    ("character", "CHAR"),
    ("character varying", "VARCHAR"),
    ("date", "DATE"),
    ("double precision", "DOUBLE PRECISION"),
    ("float4", "REAL"),
    ("float8", "DOUBLE PRECISION"),
    ("int", "INTEGER"),
    ("int2", "SMALLINT"),
    ("int4", "INTEGER"),
    ("int8", "BIGINT"),
    ("integer", "INTEGER"),
    ("interval", "INTERVAL"),
    ("json", "JSON"),
    ("jsonb", "JSONB"),
    ("name", "NAME"),
    ("numeric", "NUMERIC"),
    ("oid", "OID"),
    ("real", "REAL"),
    ("serial", "SERIAL"),
    ("serial4", "SERIAL"),
    ("serial8", "BIGSERIAL"),
    ("smallint", "SMALLINT"),
    ("smallserial", "SMALLSERIAL"),
    ("text", "TEXT"),
    ("time", "TIME"),
    ("timestamp", "TIMESTAMP"),
    ("timestamptz", "TIMESTAMP WITH TIME ZONE"),
    ("timetz", "TIME WITH TIME ZONE"),
    ("trigger", "TRIGGER"),
    ("uuid", "UUID"),
    ("varchar", "VARCHAR"),
    ("xml", "XML"),
];

impl<'a> Formatter<'a> {
    /// Format any expression node into inline SQL text.
    pub(crate) fn format_expr(&self, node: Node<'a>) -> String {
        match node.kind() {
            "a_expr" => self.format_a_expr(node),
            "a_expr_prec" => self.format_a_expr_prec(node),
            "c_expr" => self.format_c_expr(node),
            "columnref" => self.format_columnref(node),
            "AexprConst" => self.format_const(node),
            "func_expr" | "func_application" => self.format_func(node),
            "case_expr" => self.format_case_expr(node),
            "target_el" => self.format_target_el(node),
            "Typename" => self.format_typename(node),
            "SimpleTypename" => self.format_simple_typename(node),
            "select_with_parens" => self.format_select_with_parens(node),
            "Sconst" | "string_literal" => self.format_string_const(node),
            "Iconst" | "integer_literal" => self.text(node).to_string(),
            "Fconst" | "float_literal" => self.text(node).to_string(),
            "sortby" => self.format_sortby(node),
            "identifier" => self.text(node).to_string(),
            "type_function_name" => self.format_first_named_child(node),
            "ColId" => self.format_col_id(node),
            "ColLabel" => self.format_first_named_child(node),
            "qualified_name" => self.format_qualified_name(node),
            "indirection" => self.format_indirection(node),
            "indirection_el" => self.format_indirection_el(node),
            "attr_name" => self.format_first_named_child(node),
            "relation_expr" => self.format_relation_expr(node),
            "func_name" => self.format_func_name(node),
            "Numeric" | "GenericType" => self.format_typename_inner(node),
            // Unreserved keywords used as identifiers should preserve casing.
            "unreserved_keyword" => self.text(node).to_string(),
            "expr_list" => {
                let items = flatten_list(node, "expr_list");
                let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
                formatted.join(", ")
            }
            "func_arg_expr" => self.format_first_named_child(node),
            "opt_alias_clause" | "alias_clause" => self.format_alias(node),
            "group_by_item" => self.format_first_named_child(node),
            "ERROR" => self.text(node).to_string(),
            _ if node.kind().starts_with("kw_") => self.format_keyword_node(node),
            _ => {
                // Fallback: reconstruct from children or use source text.
                if node.named_child_count() == 0 {
                    self.text(node).to_string()
                } else {
                    self.format_first_named_child(node)
                }
            }
        }
    }

    /// Format an a_expr node (the main expression type with operators).
    fn format_a_expr(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        // Check if this a_expr contains an inline expr_list (e.g., IN (...)).
        // If so, skip unnamed parens since we format them with the expr_list.
        let has_expr_list = node.find_child("expr_list").is_some();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "a_expr_prec" | "a_expr" | "c_expr" => {
                        parts.push(self.format_expr(child));
                    }
                    "kw_and" => parts.push(self.kw("AND")),
                    "kw_or" => parts.push(self.kw("OR")),
                    "kw_not" => parts.push(self.kw("NOT")),
                    "kw_is" => parts.push(self.kw("IS")),
                    "kw_null" => parts.push(self.kw("NULL")),
                    "kw_true" => parts.push(self.kw("TRUE")),
                    "kw_false" => parts.push(self.kw("FALSE")),
                    "kw_in" => parts.push(self.kw("IN")),
                    "kw_any" => parts.push(self.kw("ANY")),
                    "kw_all" => parts.push(self.kw("ALL")),
                    "kw_some" => parts.push(self.kw("SOME")),
                    "kw_like" => parts.push(self.kw("LIKE")),
                    "kw_ilike" => parts.push(self.kw("ILIKE")),
                    "kw_between" => parts.push(self.kw("BETWEEN")),
                    "kw_exists" => parts.push(self.kw("EXISTS")),
                    "kw_as" => parts.push(self.kw("AS")),
                    "select_with_parens" => {
                        parts.push(self.format_select_with_parens(child));
                    }
                    "in_expr" => {
                        parts.push(self.format_in_expr(child));
                    }
                    "expr_list" => {
                        // Inline expr_list (e.g., IN (a, b, c)) — format with parens.
                        let items = flatten_list(child, "expr_list");
                        let formatted: Vec<_> =
                            items.iter().map(|i| self.format_expr(*i)).collect();
                        parts.push(format!("({})", formatted.join(", ")));
                    }
                    "qual_all_Op" | "all_Op" | "MathOp" | "sub_type" | "qual_Op" => {
                        let op_text = self.text(child).trim();
                        let normalized = if op_text == "!=" { "<>" } else { op_text };
                        parts.push(normalized.to_string());
                    }
                    _ if child.kind().starts_with("kw_") => {
                        parts.push(self.format_keyword_node(child));
                    }
                    _ => parts.push(self.format_expr(child)),
                }
            } else {
                // Unnamed children are operators like =, <, >, !=, etc.
                let text = self.text(child).trim();
                if !text.is_empty() {
                    // Skip parens that surround an expr_list (handled inline).
                    if has_expr_list && (text == "(" || text == ")") {
                        continue;
                    }
                    // Normalize != to <>
                    let op = if text == "!=" { "<>" } else { text };
                    parts.push(op.to_string());
                }
            }
        }
        Self::join_with_multiline_indent(&parts)
    }

    /// Join parts with spaces, properly indenting multi-line parts.
    /// When a part contains newlines, continuation lines are indented
    /// to align with where that part starts in the joined output.
    fn join_with_multiline_indent(parts: &[String]) -> String {
        if parts.is_empty() {
            return String::new();
        }
        // Fast path: no multi-line parts.
        if !parts.iter().any(|p| p.contains('\n')) {
            return parts.join(" ");
        }

        let mut result = String::new();
        let mut col = 0usize;
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                result.push(' ');
                col += 1;
            }
            if part.contains('\n') {
                // This part starts at column `col` in the output.
                // Indent continuation lines by `col` spaces.
                let indent_str = " ".repeat(col);
                let mut lines = part.lines();
                if let Some(first) = lines.next() {
                    result.push_str(first);
                    // Update col to end of first line (for any subsequent parts).
                    col += first.len();
                }
                for line in lines {
                    result.push('\n');
                    result.push_str(&indent_str);
                    result.push_str(line);
                }
            } else {
                result.push_str(part);
                col += part.len();
            }
        }
        result
    }

    fn format_a_expr_prec(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                parts.push(self.format_expr(child));
            } else {
                let text = self.text(child).trim();
                if !text.is_empty() {
                    let op = if text == "!=" { "<>" } else { text };
                    parts.push(op.to_string());
                }
            }
        }
        Self::join_with_multiline_indent(&parts)
    }

    fn format_c_expr(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut has_block_subquery = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "columnref" => parts.push(self.format_columnref(child)),
                    "AexprConst" => parts.push(self.format_const(child)),
                    "func_expr" | "func_application" => parts.push(self.format_func(child)),
                    "case_expr" => parts.push(self.format_case_expr(child)),
                    "select_with_parens" => {
                        let formatted = self.format_select_with_parens(child);
                        if formatted.starts_with("(\n") {
                            has_block_subquery = true;
                        }
                        parts.push(formatted);
                    }
                    "kw_exists" => parts.push(self.kw("EXISTS")),
                    "kw_row" => parts.push(self.kw("ROW")),
                    _ if child.kind().starts_with("kw_") => {
                        parts.push(self.format_keyword_node(child));
                    }
                    _ => parts.push(self.format_expr(child)),
                }
            } else {
                let text = self.text(child).trim();
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        }

        // For block-format subqueries (left-aligned styles), join with simple
        // spaces without column-based multiline indentation; the subquery
        // already has proper internal indentation.
        let result = if has_block_subquery {
            parts.join(" ")
        } else {
            Self::join_with_multiline_indent(&parts)
        };

        // Clean up double spaces on each line, preserving leading whitespace
        // and spaces inside quoted strings.
        result
            .lines()
            .map(|line| {
                let leading = line.len() - line.trim_start().len();
                let prefix = &line[..leading];
                let cleaned = collapse_whitespace_outside_quotes(&line[leading..]);
                format!("{prefix}{cleaned}")
            })
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn format_columnref(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "ColId" => parts.push(self.format_col_id(child)),
                "indirection" => parts.push(self.format_indirection(child)),
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.join("")
    }

    pub(crate) fn format_col_id(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        if let Some(child) = node.named_children(&mut cursor).next() {
            return match child.kind() {
                "identifier" | "unreserved_keyword" => self.text(child).to_string(),
                _ => self.format_expr(child),
            };
        }
        self.text(node).to_string()
    }

    fn format_indirection(&self, node: Node<'a>) -> String {
        let mut result = String::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            result.push_str(&self.format_indirection_el(child));
        }
        result
    }

    fn format_indirection_el(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                parts.push(self.format_expr(child));
            } else {
                parts.push(self.text(child).to_string());
            }
        }
        parts.join("")
    }

    fn format_const(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        if let Some(child) = node.named_children(&mut cursor).next() {
            return match child.kind() {
                "Sconst" | "string_literal" => self.format_string_const(child),
                "Iconst" | "integer_literal" | "Fconst" | "float_literal" => {
                    self.text(child).to_string()
                }
                "kw_true" => self.kw("TRUE"),
                "kw_false" => self.kw("FALSE"),
                "kw_null" => self.kw("NULL"),
                _ if child.kind().starts_with("kw_") => self.format_keyword_node(child),
                _ => self.format_expr(child),
            };
        }
        self.text(node).to_string()
    }

    fn format_string_const(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        if let Some(child) = node
            .named_children(&mut cursor)
            .find(|c| c.kind() == "string_literal")
        {
            return self.text(child).to_string();
        }
        self.text(node).to_string()
    }

    pub(crate) fn format_func(&self, node: Node<'a>) -> String {
        match node.kind() {
            "func_expr" => {
                if let Some(app) = node.find_child("func_application") {
                    return self.format_func(app);
                }
                // func_expr_common_subexpr or other variants.
                self.format_func_expr_common(node)
            }
            "func_application" => self.format_func_application(node),
            _ => self.text(node).to_string(),
        }
    }

    fn format_func_application(&self, node: Node<'a>) -> String {
        let name = node
            .find_child("func_name")
            .map(|n| self.format_func_name(n))
            .unwrap_or_default();

        // Check for special forms: COUNT(*), etc.
        let mut cursor = node.walk();
        let children: Vec<_> = node.children(&mut cursor).collect();

        let mut args = String::new();
        let mut has_star = false;
        let mut has_distinct = false;
        let mut over_clause = None;

        for child in &children {
            if !child.is_named() {
                let text = self.text(*child);
                if text == "*" {
                    has_star = true;
                }
            } else {
                match child.kind() {
                    "func_arg_list" => {
                        let items = flatten_list(*child, "func_arg_list");
                        let formatted: Vec<_> =
                            items.iter().map(|i| self.format_expr(*i)).collect();
                        args = formatted.join(", ");
                    }
                    "distinct_clause" | "kw_distinct" => has_distinct = true,
                    "over_clause" => over_clause = Some(*child),
                    "func_name" => {} // already handled
                    _ => {}
                }
            }
        }

        // Apply keyword casing only to SQL built-in functions; preserve
        // user-defined function names as-is.
        let cased_name = if is_sql_builtin_function(&name) {
            self.kw(&name)
        } else {
            name
        };
        let inner = if has_star {
            "*".to_string()
        } else if has_distinct {
            format!("{} {args}", self.kw("DISTINCT"))
        } else {
            args
        };

        let mut result = format!("{cased_name}({inner})");

        if let Some(over) = over_clause {
            result.push(' ');
            result.push_str(&self.format_over_clause(over));
        }

        result
    }

    fn format_func_expr_common(&self, node: Node<'a>) -> String {
        // Handle COALESCE, GREATEST, LEAST, NULLIF, CURRENT_TIMESTAMP, etc.
        let mut cursor = node.walk();
        let mut parts = Vec::new();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "func_application" => return self.format_func(child),
                    _ if child.kind().starts_with("kw_") => {
                        parts.push(self.format_keyword_node(child));
                    }
                    _ => parts.push(self.format_expr(child)),
                }
            } else {
                let text = self.text(child).trim();
                if !text.is_empty() {
                    parts.push(text.to_string());
                }
            }
        }
        parts.join(" ")
    }

    pub(crate) fn format_func_name(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "type_function_name" => parts.push(self.format_first_named_child(child)),
                "ColId" => parts.push(self.format_col_id(child)),
                "indirection" => parts.push(self.format_indirection(child)),
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.join("")
    }

    fn format_over_clause(&self, node: Node<'a>) -> String {
        let mut parts = vec![self.kw("OVER")];
        parts.push("(".to_string());

        let mut inner = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "opt_partition_clause" => {
                    inner.push(self.format_partition_clause(child));
                }
                "opt_sort_clause" | "sort_clause" => {
                    inner.push(self.format_sort_clause_inline(child));
                }
                "kw_over" => {} // skip
                _ => inner.push(self.format_expr(child)),
            }
        }
        parts.push(inner.join(" "));
        parts.push(")".to_string());
        parts.join("")
    }

    fn format_partition_clause(&self, node: Node<'a>) -> String {
        let mut parts = vec![self.kw("PARTITION"), self.kw("BY")];
        if let Some(list) = node.find_child("expr_list") {
            let items = flatten_list(list, "expr_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
            parts.push(formatted.join(", "));
        }
        parts.join(" ")
    }

    fn format_sort_clause_inline(&self, node: Node<'a>) -> String {
        let actual = if node.kind() == "opt_sort_clause" {
            node.find_child("sort_clause").unwrap_or(node)
        } else {
            node
        };
        let mut parts = vec![self.kw("ORDER"), self.kw("BY")];
        if let Some(list) = actual.find_child("sortby_list") {
            let items = flatten_list(list, "sortby_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_sortby(*i)).collect();
            parts.push(formatted.join(", "));
        }
        parts.join(" ")
    }

    pub(crate) fn format_sortby(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "a_expr" | "c_expr" => parts.push(self.format_expr(child)),
                "opt_asc_desc" => {
                    if let Some(kw) = child.find_child_any(&["kw_asc", "kw_desc"]) {
                        parts.push(self.format_keyword_node(kw));
                    }
                }
                "opt_nulls_order" => {
                    parts.push(self.kw("NULLS"));
                    if child.has_child("kw_first") {
                        parts.push(self.kw("FIRST"));
                    } else {
                        parts.push(self.kw("LAST"));
                    }
                }
                _ => {}
            }
        }
        parts.join(" ")
    }

    fn format_case_expr(&self, node: Node<'a>) -> String {
        let mut parts = vec![self.kw("CASE")];
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "kw_case" | "kw_end" => {}
                "case_arg" => {
                    if let Some(expr) = child.find_child_any(&["a_expr", "c_expr"]) {
                        parts.push(self.format_expr(expr));
                    }
                }
                "when_clause_list" => {
                    let clauses = flatten_list(child, "when_clause_list");
                    for clause in clauses {
                        parts.push(self.format_when_clause(clause));
                    }
                }
                "case_default" => {
                    if let Some(expr) = child.find_child_any(&["a_expr", "c_expr", "a_expr_prec"]) {
                        parts.push(self.kw("ELSE"));
                        parts.push(self.format_expr(expr));
                    }
                }
                _ => {}
            }
        }
        parts.push(self.kw("END"));
        parts.join(" ")
    }

    fn format_when_clause(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let named = node.named_children_vec();
        for child in &named {
            match child.kind() {
                "kw_when" => parts.push(self.kw("WHEN")),
                "kw_then" => parts.push(self.kw("THEN")),
                "a_expr" | "c_expr" | "a_expr_prec" => {
                    parts.push(self.format_expr(*child));
                }
                _ => {}
            }
        }
        parts.join(" ")
    }

    fn format_in_expr(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "select_with_parens" => {
                    return self.format_select_with_parens(child);
                }
                "expr_list" => {
                    let items = flatten_list(child, "expr_list");
                    let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
                    return format!("({})", formatted.join(", "));
                }
                _ => {}
            }
        }
        self.text(node).to_string()
    }

    pub(crate) fn format_select_with_parens(&self, node: Node<'a>) -> String {
        // Contains a sub-SELECT in parentheses.
        if let Some(snp) = node.find_child("select_no_parens") {
            let inner = self.format_select_no_parens(snp);
            let lines: Vec<&str> = inner.lines().collect();
            if lines.len() <= 1 {
                return format!("({inner})");
            }

            if !self.config.river {
                // Left-aligned styles: block format with opening paren on its own,
                // indented body, and closing paren on its own line.
                let indent = self.config.indent;
                let mut result = String::from("(\n");
                for line in &lines {
                    if line.is_empty() {
                        result.push('\n');
                    } else {
                        result.push_str(indent);
                        result.push_str(line);
                        result.push('\n');
                    }
                }
                result.push(')');
                return result;
            }

            // River styles: inline with continuation lines indented after '('.
            let mut result = format!("({}", lines[0]);
            let paren_indent = " ";
            for line in &lines[1..] {
                result.push('\n');
                result.push_str(paren_indent);
                result.push_str(line);
            }
            result.push(')');
            result
        } else {
            format!("({})", self.text(node).trim())
        }
    }

    pub(crate) fn format_target_el(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "a_expr" | "c_expr" => parts.push(self.format_expr(child)),
                    "kw_as" => parts.push(self.kw("AS")),
                    "ColLabel" => parts.push(self.format_expr(child)),
                    _ => parts.push(self.format_expr(child)),
                }
            } else {
                let text = self.text(child).trim();
                if text == "*" {
                    parts.push("*".to_string());
                }
            }
        }
        Self::join_with_multiline_indent(&parts)
    }

    pub(crate) fn format_typename(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        let mut has_setof = false;
        let mut has_array = false;
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "SimpleTypename" => parts.push(self.format_simple_typename(child)),
                    "kw_setof" => {
                        has_setof = true;
                    }
                    "opt_array_bounds" => has_array = true,
                    _ => parts.push(self.format_expr(child)),
                }
            }
        }
        let mut result = String::new();
        if has_setof {
            result.push_str(&self.kw("SETOF"));
            result.push(' ');
        }
        result.push_str(&parts.join(" "));
        if has_array {
            result.push_str("[]");
        }
        result
    }

    pub(crate) fn format_simple_typename(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        if let Some(child) = node.named_children(&mut cursor).next() {
            return match child.kind() {
                "Numeric" | "GenericType" | "Bit" | "Character" | "ConstDatetime"
                | "ConstInterval" => self.format_typename_inner(child),
                _ => self.format_expr(child),
            };
        }
        self.text(node).to_string()
    }

    fn format_typename_inner(&self, node: Node<'a>) -> String {
        // Get the base type name.
        let mut base = String::new();
        let mut modifiers = String::new();
        let mut extra_keywords = Vec::new();

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "kw_integer" | "kw_int" | "kw_smallint" | "kw_bigint" | "kw_real"
                    | "kw_boolean" | "kw_float" | "kw_decimal" => {
                        base = self.map_type_name(&self.text(child).to_lowercase());
                    }
                    "kw_double" => base = "DOUBLE".to_string(),
                    "kw_precision" => {
                        if base == "DOUBLE" {
                            base = "DOUBLE PRECISION".to_string();
                        } else {
                            extra_keywords.push("PRECISION".to_string());
                        }
                    }
                    "kw_varying" => extra_keywords.push("VARYING".to_string()),
                    "kw_with" => extra_keywords.push(self.kw("WITH")),
                    "kw_without" => extra_keywords.push(self.kw("WITHOUT")),
                    "kw_time" => {
                        if base.is_empty() {
                            base = self.kw("TIME");
                        } else {
                            extra_keywords.push(self.kw("TIME"));
                        }
                    }
                    "kw_zone" => extra_keywords.push(self.kw("ZONE")),
                    "kw_timestamp" => base = self.kw("TIMESTAMP"),
                    "type_function_name" | "unreserved_keyword" => {
                        let name = self.format_first_named_child(child);
                        base = self.map_type_name(&name.to_lowercase());
                    }
                    "opt_type_modifiers" => {
                        modifiers = self.format_type_modifiers(child);
                    }
                    "attrs" => {
                        base.push_str(&self.format_attrs(child));
                    }
                    "opt_float" => {
                        // FLOAT(n) precision.
                        let text = self.text(child);
                        if !text.trim().is_empty() {
                            modifiers = text.to_string();
                        }
                    }
                    _ if child.kind().starts_with("kw_") => {
                        let kw_text = self.text(child);
                        extra_keywords.push(self.kw(kw_text));
                    }
                    _ => {
                        if base.is_empty() {
                            base = self.format_expr(child);
                        }
                    }
                }
            } else {
                let text = self.text(child).trim();
                if text == "(" || text == ")" || text == "," {
                    // Part of modifiers — handled by opt_type_modifiers.
                }
            }
        }

        let mut result = if self.config.upper_keywords {
            base.to_uppercase()
        } else {
            base.to_lowercase()
        };

        if !extra_keywords.is_empty() {
            result.push(' ');
            result.push_str(&extra_keywords.join(" "));
        }
        if !modifiers.is_empty() {
            result.push_str(&modifiers);
        }
        result
    }

    fn format_type_modifiers(&self, node: Node<'a>) -> String {
        let mut items = Vec::new();
        if let Some(list) = node.find_child("expr_list") {
            let exprs = flatten_list(list, "expr_list");
            for expr in exprs {
                items.push(self.format_expr(expr));
            }
        }
        if items.is_empty() {
            return String::new();
        }
        format!("({})", items.join(", "))
    }

    fn format_attrs(&self, node: Node<'a>) -> String {
        let mut result = String::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "attr_name" => result.push_str(&self.format_expr(child)),
                    _ => result.push_str(&self.format_expr(child)),
                }
            } else {
                result.push_str(self.text(child));
            }
        }
        result
    }

    fn map_type_name(&self, name: &str) -> String {
        for (pg_name, std_name) in PG_TYPE_MAP {
            if *pg_name == name {
                return if self.config.upper_keywords {
                    std_name.to_string()
                } else {
                    std_name.to_lowercase()
                };
            }
        }
        // If not in the map, return the name with proper casing.
        if self.config.upper_keywords {
            name.to_uppercase()
        } else {
            name.to_lowercase()
        }
    }

    pub(crate) fn format_qualified_name(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "ColId" => parts.push(self.format_col_id(child)),
                    "indirection" => parts.push(self.format_indirection(child)),
                    "attr_name" => {
                        // Schema-qualified: schema.name
                        parts.push(format!(".{}", self.format_expr(child)));
                    }
                    _ => parts.push(self.format_expr(child)),
                }
            } else {
                let text = self.text(child).trim();
                if text == "." {
                    parts.push(".".to_string());
                }
            }
        }
        // Join without extra spaces (dots already included).
        let result = parts.join("");
        // Clean up any double dots.
        result.replace("..", ".")
    }

    pub(crate) fn format_relation_expr(&self, node: Node<'a>) -> String {
        if let Some(qn) = node.find_child("qualified_name") {
            return self.format_qualified_name(qn);
        }
        self.text(node).to_string()
    }

    fn format_alias(&self, node: Node<'a>) -> String {
        if node.kind() == "opt_alias_clause"
            && let Some(ac) = node.find_child("alias_clause")
        {
            return self.format_alias(ac);
        }
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "kw_as" => parts.push(self.kw("AS")),
                "ColId" => parts.push(self.format_col_id(child)),
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.join(" ")
    }

    pub(crate) fn format_keyword_node(&self, node: Node<'a>) -> String {
        self.kw(self.text(node))
    }

    fn format_first_named_child(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        if let Some(child) = node.named_children(&mut cursor).next() {
            return self.format_expr(child);
        }
        self.text(node).to_string()
    }

    /// Format a table reference (for FROM clause), returning the table name with alias.
    pub(crate) fn format_table_ref(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "relation_expr" => parts.push(self.format_relation_expr(child)),
                "opt_alias_clause" | "alias_clause" => {
                    parts.push(self.format_alias(child));
                }
                "joined_table" => return self.text(child).to_string(), // handled elsewhere
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.join(" ")
    }
}

/// Collapse runs of whitespace to single spaces, but preserve whitespace
/// inside single-quoted or double-quoted strings.
fn collapse_whitespace_outside_quotes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_single_quote = false;
    let mut in_double_quote = false;
    let mut prev_was_space = false;

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if in_single_quote {
            result.push(ch);
            if ch == '\'' {
                // Check for escaped quote ('').
                if i + 1 < len && chars[i + 1] == '\'' {
                    result.push('\'');
                    i += 2;
                    continue;
                }
                in_single_quote = false;
            }
            i += 1;
            continue;
        }

        if in_double_quote {
            result.push(ch);
            if ch == '"' {
                if i + 1 < len && chars[i + 1] == '"' {
                    result.push('"');
                    i += 2;
                    continue;
                }
                in_double_quote = false;
            }
            i += 1;
            continue;
        }

        if ch == '\'' {
            in_single_quote = true;
            prev_was_space = false;
            result.push(ch);
        } else if ch == '"' {
            in_double_quote = true;
            prev_was_space = false;
            result.push(ch);
        } else if ch.is_whitespace() {
            if !prev_was_space {
                result.push(' ');
                prev_was_space = true;
            }
        } else {
            prev_was_space = false;
            result.push(ch);
        }
        i += 1;
    }

    result
}
