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
            "a_expr" | "b_expr" => self.format_a_expr(node),
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
            "qualified_name" | "any_name" => self.format_qualified_name(node),
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
            "array_expr" => self.format_array_expr(node),
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
        let mut parts: Vec<String> = Vec::new();
        let mut cursor = node.walk();
        // Check if this a_expr contains an inline expr_list (e.g., IN (...)).
        // If so, skip unnamed parens since we format them with the expr_list.
        let has_expr_list = node.find_child("expr_list").is_some();
        let mut pending_cast = false;
        for child in node.children(&mut cursor) {
            if child.is_named() {
                // After ::, the next named child is a Typename — append directly
                // to the previous part without spaces.
                if pending_cast {
                    pending_cast = false;
                    let typename = self.format_expr(child);
                    if let Some(last) = parts.last_mut() {
                        last.push_str("::");
                        last.push_str(&typename);
                    }
                    continue;
                }
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
                    // Typecast operator :: — defer and attach to next Typename.
                    if text == "::" {
                        pending_cast = true;
                        continue;
                    }
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
        let mut parts: Vec<String> = Vec::new();
        let mut cursor = node.walk();
        let mut pending_cast = false;
        for child in node.children(&mut cursor) {
            if child.is_named() {
                if pending_cast {
                    pending_cast = false;
                    let typename = self.format_expr(child);
                    if let Some(last) = parts.last_mut() {
                        last.push_str("::");
                        last.push_str(&typename);
                    }
                    continue;
                }
                parts.push(self.format_expr(child));
            } else {
                let text = self.text(child).trim();
                if !text.is_empty() {
                    if text == "::" {
                        pending_cast = true;
                        continue;
                    }
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
        let mut paren_depth: u32 = 0;
        let mut paren_parts: Vec<String> = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                let formatted = match child.kind() {
                    "columnref" => self.format_columnref(child),
                    "AexprConst" => self.format_const(child),
                    "func_expr" | "func_application" => self.format_func(child),
                    "case_expr" => self.format_case_expr(child),
                    "select_with_parens" => {
                        let f = self.format_select_with_parens(child);
                        if f.starts_with("(\n") {
                            has_block_subquery = true;
                        }
                        f
                    }
                    "kw_exists" => self.kw("EXISTS"),
                    "kw_row" => self.kw("ROW"),
                    "kw_array" => self.kw("ARRAY"),
                    "array_expr" => self.format_array_expr(child),
                    _ if child.kind().starts_with("kw_") => self.format_keyword_node(child),
                    _ => self.format_expr(child),
                };
                // Merge ARRAY with following [...] bracket expression.
                let target = if paren_depth > 0 {
                    &mut paren_parts
                } else {
                    &mut parts
                };
                if formatted.starts_with('[')
                    && let Some(last) = target.last_mut()
                    && (*last == "ARRAY" || *last == "array")
                {
                    last.push_str(&formatted);
                    continue;
                }
                // Field selection (e.g. `.bar`) attaches to the preceding
                // expression with no space. When that base was parenthesized
                // (its parens stripped above), restore them so `(foo).bar`
                // keeps its composite-field meaning rather than becoming the
                // column reference `foo.bar`.
                if formatted.starts_with('.')
                    && let Some(last) = target.last_mut()
                {
                    // Re-wrap unless the base is already fully enclosed in
                    // outer parens. `contains('(')` would be too broad: a bare
                    // function call like `foo(x)` contains `(` yet still needs
                    // wrapping so `(foo(x)).bar` keeps its field-selection
                    // meaning instead of collapsing to `foo(x).bar`.
                    if !(last.starts_with('(') && last.ends_with(')')) {
                        *last = format!("({last})");
                    }
                    last.push_str(&formatted);
                    continue;
                }
                target.push(formatted);
            } else {
                let text = self.text(child).trim();
                if text == "(" {
                    if paren_depth > 0 {
                        // Nested paren — include it as content.
                        paren_parts.push("(".to_string());
                    }
                    paren_depth += 1;
                } else if text == ")" && paren_depth > 0 {
                    paren_depth -= 1;
                    if paren_depth == 0 {
                        // Close outermost paren group.
                        let inner = paren_parts.join(" ");
                        // Strip redundant parens around a single simple
                        // expression (column ref, literal) that doesn't
                        // contain operators or keywords.
                        if paren_parts.len() == 1 && !inner.contains(' ') && !inner.contains('\n') {
                            parts.push(inner);
                        } else {
                            parts.push(format!("({inner})"));
                        }
                        paren_parts.clear();
                    } else {
                        // Closing a nested paren.
                        paren_parts.push(")".to_string());
                    }
                } else if !text.is_empty() {
                    if paren_depth > 0 {
                        paren_parts.push(text.to_string());
                    } else {
                        parts.push(text.to_string());
                    }
                }
            }
        }
        // Unclosed parens — flush as-is.
        if paren_depth > 0 {
            parts.push(format!("({}", paren_parts.join(" ")));
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
                "identifier" => self.text(child).to_string(),
                "unreserved_keyword" => {
                    // Special pseudo-variables like VALUE in domain CHECK
                    // constraints are conventionally lowercased regardless
                    // of keyword casing.
                    if let Some(kw) = child.named_children(&mut child.walk()).next()
                        && kw.kind() == "kw_value"
                    {
                        return "value".to_string();
                    }
                    self.text(child).to_string()
                }
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
        let named: Vec<_> = node.named_children(&mut cursor).collect();

        // A single named child is a plain literal or boolean/null keyword.
        if named.len() == 1 {
            let child = named[0];
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

        // Multiple children: a typed string literal such as
        // `INTERVAL '2 days'`, `DATE '2020-01-01'`, `INTERVAL '2' DAY`, or
        // `INTERVAL(6) '2 days'`. Walk the children in order so the leading
        // type name, optional `(precision)`, the string, and any trailing
        // interval qualifier are all preserved.
        if !named.is_empty() {
            let mut result = String::new();
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.is_named() {
                    match child.kind() {
                        "Sconst" | "string_literal" => {
                            result.push(' ');
                            result.push_str(&self.format_string_const(child));
                        }
                        // Precision digit inside `(...)`, e.g. INTERVAL(6).
                        "Iconst" | "integer_literal" => result.push_str(self.text(child)),
                        // Trailing interval qualifier, e.g. DAY, HOUR TO MINUTE.
                        "opt_interval" => {
                            result.push(' ');
                            result.push_str(&self.format_opt_interval(child));
                        }
                        // The leading type: ConstInterval, ConstTypename,
                        // func_name (DATE), Numeric, etc.
                        _ => result.push_str(&self.format_expr(child)),
                    }
                } else {
                    match self.text(child).trim() {
                        "(" => result.push('('),
                        ")" => result.push(')'),
                        _ => {}
                    }
                }
            }
            return result;
        }

        self.text(node).to_string()
    }

    /// Format an interval qualifier (`opt_interval`), e.g. `DAY` or
    /// `HOUR TO MINUTE`, applying keyword casing to each component.
    fn format_opt_interval(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        let parts: Vec<String> = node
            .named_children(&mut cursor)
            .map(|child| self.format_expr(child))
            .collect();
        if parts.is_empty() {
            self.text(node).trim().to_string()
        } else {
            parts.join(" ")
        }
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
                    let mut result = self.format_func(app);
                    // Trailing clauses appear in grammar order:
                    // WITHIN GROUP, FILTER, RESPECT/IGNORE NULLS (PG19), OVER.
                    if let Some(wg) = node.find_child("within_group_clause") {
                        result.push(' ');
                        result.push_str(&self.format_within_group_clause(wg));
                    }
                    if let Some(filter) = node.find_child("filter_clause") {
                        result.push(' ');
                        result.push_str(&self.format_filter_clause(filter));
                    }
                    if let Some(nt) = node.find_child("null_treatment") {
                        result.push(' ');
                        result.push_str(&self.format_null_treatment(nt));
                    }
                    // Check for OVER clause at the func_expr level
                    // (window functions like RANK() OVER (...)).
                    if let Some(over) = node.find_child("over_clause") {
                        result.push(' ');
                        result.push_str(&self.format_over_clause(over));
                    }
                    return result;
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

        // ANY, ALL, SOME are special SQL constructs that conventionally
        // have a space before the opening paren.
        let lower = cased_name.to_lowercase();
        let space = if lower == "any" || lower == "all" || lower == "some" {
            " "
        } else {
            ""
        };
        let mut result = format!("{cased_name}{space}({inner})");

        if let Some(over) = over_clause {
            result.push(' ');
            result.push_str(&self.format_over_clause(over));
        }

        result
    }

    fn format_func_expr_common(&self, node: Node<'a>) -> String {
        // Check for func_expr_common_subexpr children.
        let subexpr = node.find_child("func_expr_common_subexpr").unwrap_or(node);

        // CAST(expr AS type) → expr::type
        // Parenthesize when the formatted operand contains spaces that indicate
        // a compound expression (e.g. "a + b"), because the :: typecast operator
        // has higher precedence than arithmetic operators in PostgreSQL.
        // Simple expressions (column refs, literals, function calls, already-
        // parenthesized expressions) do not need extra parens.
        if subexpr.has_child("kw_cast")
            && let Some(expr) = subexpr.find_child_any(&["a_expr", "c_expr"])
            && let Some(typename) = subexpr.find_child("Typename")
        {
            let formatted = self.format_expr(expr);
            // A compound operand carries a top-level operator (a space outside
            // any parens/quotes), e.g. "a + b", "foo(x) + 1", "x IS NOT NULL".
            // Because :: binds tighter than those operators, such operands must
            // be wrapped. Bare identifiers, literals, and single function calls
            // like foo(x) have no top-level space and are left alone.
            let needs_parens = has_top_level_space(&formatted);
            return if needs_parens {
                format!("({formatted})::{}", self.format_typename(typename))
            } else {
                format!("{formatted}::{}", self.format_typename(typename))
            };
        }

        // Handle COALESCE, GREATEST, LEAST, NULLIF, etc.
        // These are function-like: KEYWORD(args)
        if let Some(expr_list) = subexpr.find_child("expr_list") {
            let items = flatten_list(expr_list, "expr_list");
            let mut formatted: Vec<String> = items.iter().map(|i| self.format_expr(*i)).collect();
            // Merge decimal fragments split by tree-sitter ERROR nodes
            // (e.g., "0" + ".00" → "0.00").
            let mut i = 0;
            while i + 1 < formatted.len() {
                if formatted[i].chars().all(|c| c.is_ascii_digit())
                    && formatted[i + 1].starts_with('.')
                    && formatted[i + 1][1..].chars().all(|c| c.is_ascii_digit())
                {
                    let merged = format!("{}{}", formatted[i], formatted[i + 1]);
                    formatted[i] = merged;
                    formatted.remove(i + 1);
                } else {
                    i += 1;
                }
            }
            // Find the keyword name.
            let mut name = String::new();
            let mut cursor = subexpr.walk();
            for child in subexpr.named_children(&mut cursor) {
                if child.kind().starts_with("kw_") {
                    name = self.kw(self.text(child));
                    break;
                }
            }
            return format!("{name}({})", formatted.join(", "));
        }

        // Other forms (CURRENT_TIMESTAMP, etc.).
        let mut cursor = subexpr.walk();
        let mut parts = Vec::new();
        for child in subexpr.children(&mut cursor) {
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

    /// Format a `WITHIN GROUP (ORDER BY ...)` clause on an ordered-set or
    /// hypothetical-set aggregate (e.g. `percentile_cont`).
    fn format_within_group_clause(&self, node: Node<'a>) -> String {
        let order = node
            .find_child("sort_clause")
            .map(|sc| self.format_sort_clause_inline(sc))
            .unwrap_or_default();
        format!("{} ({order})", self.kw_pair("WITHIN", "GROUP"))
    }

    /// Format a `FILTER (WHERE <condition>)` aggregate filter clause.
    fn format_filter_clause(&self, node: Node<'a>) -> String {
        let cond = node
            .find_child_any(&["a_expr", "c_expr"])
            .map(|e| self.format_expr(e))
            .unwrap_or_default();
        format!("{} ({} {cond})", self.kw("FILTER"), self.kw("WHERE"))
    }

    /// Format a `null_treatment` node: `RESPECT NULLS` or `IGNORE NULLS` (PG19).
    fn format_null_treatment(&self, node: Node<'a>) -> String {
        let leading = if node.has_child("kw_ignore") {
            self.kw("IGNORE")
        } else {
            self.kw("RESPECT")
        };
        format!("{leading} {}", self.kw("NULLS"))
    }

    fn format_over_clause(&self, node: Node<'a>) -> String {
        // `OVER (...)` with an inline window_specification holds the actual
        // partition/order/frame clauses and stays parenthesized.
        if let Some(spec) = node.find_child("window_specification") {
            return format!(
                "{} ({})",
                self.kw("OVER"),
                self.format_window_spec_body(spec)
            );
        }
        // `OVER window_name` references a window defined in the WINDOW clause.
        // This bare form is distinct from `OVER (window_name)` in PostgreSQL,
        // so preserve it without adding parentheses.
        if let Some(name) = node.find_child("ColId") {
            return format!("{} {}", self.kw("OVER"), self.format_expr(name));
        }
        format!(
            "{} ({})",
            self.kw("OVER"),
            self.format_window_spec_body(node)
        )
    }

    /// Format the inner contents (PARTITION BY / ORDER BY / frame) of a
    /// window specification, without the surrounding parentheses.
    pub(crate) fn format_window_spec_body(&self, spec: Node<'a>) -> String {
        let mut inner = Vec::new();
        let mut cursor = spec.walk();
        for child in spec.named_children(&mut cursor) {
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
        inner.join(" ")
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
        let case_kw = self.kw("CASE");
        let end_kw = self.kw("END");
        let mut case_arg: Option<String> = None;
        let mut when_clauses: Vec<String> = Vec::new();
        let mut else_parts: Option<String> = None;

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "kw_case" | "kw_end" => {}
                "case_arg" => {
                    if let Some(expr) = child.find_child_any(&["a_expr", "c_expr"]) {
                        case_arg = Some(self.format_expr(expr));
                    }
                }
                "when_clause_list" => {
                    let clauses = flatten_list(child, "when_clause_list");
                    for clause in clauses {
                        when_clauses.push(self.format_when_clause(clause));
                    }
                }
                "case_default" => {
                    if let Some(expr) = child.find_child_any(&["a_expr", "c_expr", "a_expr_prec"]) {
                        else_parts =
                            Some(format!("{} {}", self.kw("ELSE"), self.format_expr(expr)));
                    }
                }
                _ => {}
            }
        }

        // Build the single-line version first.
        let mut inline_parts = vec![case_kw.clone()];
        if let Some(ref arg) = case_arg {
            inline_parts.push(arg.clone());
        }
        for wc in &when_clauses {
            inline_parts.push(wc.clone());
        }
        if let Some(ref ep) = else_parts {
            inline_parts.push(ep.clone());
        }
        inline_parts.push(end_kw.clone());
        let single_line = inline_parts.join(" ");

        // Wrap CASE when the style wraps CASE+ELSE and there's an ELSE clause,
        // or when the single-line version is excessively long.
        let should_wrap = if self.config.wrap_case_else && else_parts.is_some() {
            true
        } else {
            single_line.len() > 120
        };
        if !should_wrap {
            return single_line;
        }

        // Multi-line: align WHEN/ELSE under the first WHEN, END indented 1 space.
        // First line: "CASE [arg] WHEN ..."
        // Continuation: "     WHEN ..." (indent = len("CASE ") + len(arg + " ") if present)
        let prefix = match &case_arg {
            Some(arg) => format!("{case_kw} {arg} "),
            None => format!("{case_kw} "),
        };
        let when_indent = " ".repeat(prefix.len());
        let end_indent = " ";

        let mut lines = Vec::new();
        for (i, wc) in when_clauses.iter().enumerate() {
            if i == 0 {
                lines.push(format!("{prefix}{wc}"));
            } else {
                lines.push(format!("{when_indent}{wc}"));
            }
        }
        if let Some(ep) = &else_parts {
            lines.push(format!("{when_indent}{ep}"));
        }
        lines.push(format!("{end_indent}{end_kw}"));
        lines.join("\n")
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

    fn format_array_expr(&self, node: Node<'a>) -> String {
        // array_expr: [ expr_list ]
        if let Some(expr_list) = node.find_child("expr_list") {
            let items = flatten_list(expr_list, "expr_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
            return format!("[{}]", formatted.join(", "));
        }
        self.text(node).to_string()
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

    /// Flatten the wrapper nodes the grammar inserts for character/bit types
    /// (`CharacterWithLength > character > kw_character + opt_varying`, etc.)
    /// so the keyword, VARYING qualifier, and length token are all visible to
    /// the single-pass type renderer below.
    fn flatten_type_children(&self, node: Node<'a>) -> Vec<Node<'a>> {
        const WRAPPERS: &[&str] = &[
            "CharacterWithLength",
            "CharacterWithoutLength",
            "character",
            "BitWithLength",
            "BitWithoutLength",
            "bit",
            "opt_varying",
        ];
        let mut out = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() && WRAPPERS.contains(&child.kind()) {
                out.extend(self.flatten_type_children(child));
            } else {
                out.push(child);
            }
        }
        out
    }

    fn format_typename_inner(&self, node: Node<'a>) -> String {
        // Get the base type name.
        let mut base = String::new();
        let mut modifiers = String::new();
        let mut extra_keywords = Vec::new();
        let mut timezone_keywords = Vec::new();

        for child in self.flatten_type_children(node) {
            if child.is_named() {
                match child.kind() {
                    "kw_integer" | "kw_int" | "kw_smallint" | "kw_bigint" | "kw_real"
                    | "kw_boolean" | "kw_float" | "kw_decimal" | "kw_numeric" => {
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
                    "opt_timezone" => {
                        // WITH/WITHOUT TIME ZONE — must appear after modifiers
                        // so TIMESTAMP(6) WITH TIME ZONE is correct, not
                        // TIMESTAMP WITH TIME ZONE(6).
                        let mut tz_cursor = child.walk();
                        for tz_child in child.named_children(&mut tz_cursor) {
                            match tz_child.kind() {
                                "kw_with" => timezone_keywords.push(self.kw("WITH")),
                                "kw_without" => timezone_keywords.push(self.kw("WITHOUT")),
                                "kw_time" => timezone_keywords.push(self.kw("TIME")),
                                "kw_zone" => timezone_keywords.push(self.kw("ZONE")),
                                _ => {}
                            }
                        }
                    }
                    "kw_with" => timezone_keywords.push(self.kw("WITH")),
                    "kw_without" => timezone_keywords.push(self.kw("WITHOUT")),
                    "kw_time" => {
                        if base.is_empty() {
                            base = self.kw("TIME");
                        } else {
                            timezone_keywords.push(self.kw("TIME"));
                        }
                    }
                    "kw_zone" => timezone_keywords.push(self.kw("ZONE")),
                    "kw_timestamp" => base = self.kw("TIMESTAMP"),
                    "type_function_name" | "unreserved_keyword" => {
                        let name = self.format_first_named_child(child);
                        base = self.map_type_name(&name.to_lowercase());
                    }
                    "opt_type_modifiers" => {
                        modifiers = self.format_type_modifiers(child);
                    }
                    "Iconst" => {
                        // Bare length token inside CharacterWithLength.
                        modifiers = format!("({})", self.text(child).trim());
                    }
                    "expr_list" => {
                        // Parenthesized length/precision args (e.g. BitWithLength
                        // renders the length as ( expr_list )).
                        let items: Vec<String> = flatten_list(child, "expr_list")
                            .iter()
                            .map(|e| self.format_expr(*e))
                            .collect();
                        modifiers = format!("({})", items.join(", "));
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
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&extra_keywords.join(" "));
        }
        if !modifiers.is_empty() {
            result.push_str(&modifiers);
        }
        // Timezone qualifiers (WITH/WITHOUT TIME ZONE) must follow modifiers
        // so that TIMESTAMP(6) WITH TIME ZONE is produced, not
        // TIMESTAMP WITH TIME ZONE(6).
        if !timezone_keywords.is_empty() {
            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(&timezone_keywords.join(" "));
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
                    "attrs" => parts.push(self.format_attrs(child)),
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
        // Clean up any double dots produced by joining a separator with an
        // attr's own leading dot, but never touch dots inside quoted
        // identifiers (e.g. "a..b" is a single, distinct object name).
        collapse_double_dots_outside_quotes(&result)
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
        let mut has_as = false;
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "kw_as" => {
                    has_as = true;
                    parts.push(self.kw("AS"));
                }
                "ColId" => {
                    // Always add AS keyword for bare aliases.
                    if !has_as {
                        parts.push(self.kw("AS"));
                    }
                    parts.push(self.format_col_id(child));
                }
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

    // ── GRAPH_TABLE (SQL/PGQ property graphs, PG19) ──────────────────────
    //
    // `GRAPH_TABLE (graph MATCH <pattern> COLUMNS (<exprs>)) [AS alias]`.
    // The graph pattern is a contiguous token stream (`(a)-[e]->(b)`) whose
    // vertex/edge elements concatenate without separators, while the contents
    // inside each `(...)`/`[...]` are space-separated. Rendered inline.

    fn format_graph_table(&self, node: Node<'a>) -> String {
        let mut graph_name = String::new();
        let mut pattern = String::new();
        let mut columns = String::new();
        let mut alias = String::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "qualified_name" => graph_name = self.format_expr(child),
                "graph_pattern" => pattern = self.format_graph_pattern(child),
                "labeled_expr_list" => columns = self.format_labeled_expr_list(child),
                "opt_alias_clause" | "alias_clause" => alias = self.format_alias(child),
                _ => {}
            }
        }
        // Indented block: the graph name, MATCH and COLUMNS each get their own
        // line one indent level in; the closing paren de-dents to the
        // `GRAPH_TABLE` column. Inner lines are indented relative to column 0
        // so the FROM layout (river_line / left-aligned) can re-anchor them.
        let indent = self.config.indent;
        let close = if alias.is_empty() {
            ")".to_string()
        } else {
            format!(") {alias}")
        };
        format!(
            "{gt} (\n{indent}{graph_name}\n{indent}{match_kw} {pattern}\n{indent}{cols_kw} ({columns})\n{close}",
            gt = self.kw("GRAPH_TABLE"),
            match_kw = self.kw("MATCH"),
            cols_kw = self.kw("COLUMNS"),
        )
    }

    fn format_graph_pattern(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        if let Some(list) = node.find_child("path_pattern_list") {
            let items = flatten_list(list, "path_pattern_list");
            let formatted: Vec<_> = items.iter().map(|i| self.render_path(*i)).collect();
            parts.push(formatted.join(", "));
        }
        if let Some(wc) = node.find_child("where_clause") {
            parts.push(self.format_graph_where(wc));
        }
        parts.join(" ")
    }

    /// Render a path element node, concatenating vertices, edges, and
    /// quantifiers without separators (e.g. `(a)-[e]->(b){2}`).
    fn render_path(&self, node: Node<'a>) -> String {
        match node.kind() {
            "path_primary" => self.format_path_primary(node),
            "opt_graph_pattern_quantifier" => {
                let mut cursor = node.walk();
                node.children(&mut cursor).map(|c| self.text(c)).collect()
            }
            // path_pattern / path_pattern_expression / path_term / path_factor
            // are structural wrappers whose children concatenate directly.
            _ => {
                let mut cursor = node.walk();
                node.named_children(&mut cursor)
                    .map(|c| self.render_path(c))
                    .collect()
            }
        }
    }

    fn format_path_primary(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();
        let named: Vec<usize> = children
            .iter()
            .enumerate()
            .filter(|(_, c)| c.is_named())
            .map(|(i, _)| i)
            .collect();
        // Bare connectors like `->`, `-`, `<-` have no named children.
        if named.is_empty() {
            return children.iter().map(|c| self.text(*c)).collect();
        }
        let first = named[0];
        let last = *named.last().unwrap();
        // Delimiters/arrows around the content glue directly (`-[`, `]->`, `(`).
        let prefix: String = children[..first].iter().map(|c| self.text(*c)).collect();
        let suffix: String = children[last + 1..].iter().map(|c| self.text(*c)).collect();
        let inner: Vec<String> = named
            .iter()
            .map(|&i| self.render_path_inner(children[i]))
            .collect();
        format!("{prefix}{}{suffix}", inner.join(" "))
    }

    /// Render the space-separated contents inside a vertex/edge pattern:
    /// an element variable, an `IS <label>` test, and/or a `WHERE <cond>`.
    fn render_path_inner(&self, node: Node<'a>) -> String {
        match node.kind() {
            "opt_colid" => node
                .find_child("ColId")
                .map(|c| self.text(c).to_string())
                .unwrap_or_default(),
            "opt_is_label_expression" => {
                let label = node
                    .find_child("label_expression")
                    .map(|l| self.format_label_expression(l))
                    .unwrap_or_default();
                format!("{} {label}", self.kw("IS"))
            }
            "where_clause" => self.format_graph_where(node),
            "path_pattern_expression" => self.render_path(node),
            _ => self.text(node).to_string(),
        }
    }

    fn format_label_expression(&self, node: Node<'a>) -> String {
        match node.kind() {
            // `A | B` label alternation.
            "label_disjunction" => {
                let mut cursor = node.walk();
                let parts: Vec<_> = node
                    .named_children(&mut cursor)
                    .map(|c| self.format_label_expression(c))
                    .collect();
                parts.join(" | ")
            }
            "label_term" => self.text(node).to_string(),
            // `label_expression` wraps a single label_term or label_disjunction.
            _ => {
                let mut cursor = node.walk();
                match node.named_children(&mut cursor).next() {
                    Some(child) => self.format_label_expression(child),
                    None => self.text(node).to_string(),
                }
            }
        }
    }

    fn format_graph_where(&self, node: Node<'a>) -> String {
        let cond = node
            .find_child_any(&["a_expr", "c_expr"])
            .map(|e| self.format_expr(e))
            .unwrap_or_default();
        format!("{} {cond}", self.kw("WHERE"))
    }

    pub(crate) fn format_labeled_expr_list(&self, node: Node<'a>) -> String {
        let items = flatten_list(node, "labeled_expr_list");
        let formatted: Vec<_> = items.iter().map(|i| self.format_labeled_expr(*i)).collect();
        formatted.join(", ")
    }

    fn format_labeled_expr(&self, node: Node<'a>) -> String {
        let expr = node
            .find_child_any(&["a_expr", "c_expr"])
            .map(|e| self.format_expr(e))
            .unwrap_or_default();
        if let Some(label) = node.find_child("ColLabel") {
            format!("{expr} {} {}", self.kw("AS"), self.text(label))
        } else {
            expr
        }
    }

    // ── CREATE PROPERTY GRAPH (SQL/PGQ, PG19) ────────────────────────────

    /// Format a `CREATE PROPERTY GRAPH` statement with its VERTEX/EDGE TABLES
    /// clauses laid out one element definition per line, like CREATE TABLE.
    pub(crate) fn format_create_prop_graph_stmt(&self, node: Node<'a>) -> String {
        let name = node
            .find_child("qualified_name")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        let mut header = self.kw("CREATE");
        if let Some(temp) = node.find_child("OptTemp") {
            header.push(' ');
            header.push_str(&self.render_graph_inline(temp));
        }
        header.push_str(&format!(
            " {} {} {name}",
            self.kw("PROPERTY"),
            self.kw("GRAPH")
        ));
        let mut lines = vec![header];
        for (clause, synonym, list) in [
            (
                "opt_vertex_tables_clause",
                "vertex_synonym",
                "vertex_table_list",
            ),
            ("opt_edge_tables_clause", "edge_synonym", "edge_table_list"),
        ] {
            if let Some(inner) = node
                .find_child(clause)
                .and_then(|c| c.find_child_any(&["vertex_tables_clause", "edge_tables_clause"]))
            {
                self.format_graph_tables_clause(inner, synonym, list, "", &mut lines);
            }
        }
        lines.join("\n")
    }

    pub(crate) fn format_alter_prop_graph_stmt(&self, node: Node<'a>) -> String {
        let name = node
            .find_child("qualified_name")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        // Only the `ADD ... TABLES` forms carry vertex/edge table clauses; give
        // those the same structured block layout as CREATE. The other forms
        // (DROP TABLES, ALTER ... TABLE ... {ADD,DROP,ALTER} LABEL/PROPERTIES)
        // are short and render on one line via the keyword-casing walker.
        let has_tables =
            node.has_child("vertex_tables_clause") || node.has_child("edge_tables_clause");
        if !has_tables {
            return self.render_graph_inline(node);
        }
        let add = format!("{} ", self.kw("ADD"));
        let mut lines = vec![format!(
            "{} {} {} {name}",
            self.kw("ALTER"),
            self.kw("PROPERTY"),
            self.kw("GRAPH")
        )];
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "vertex_tables_clause" => self.format_graph_tables_clause(
                    child,
                    "vertex_synonym",
                    "vertex_table_list",
                    &add,
                    &mut lines,
                ),
                "edge_tables_clause" => self.format_graph_tables_clause(
                    child,
                    "edge_synonym",
                    "edge_table_list",
                    &add,
                    &mut lines,
                ),
                _ => {}
            }
        }
        lines.join("\n")
    }

    fn format_graph_tables_clause(
        &self,
        clause: Node<'a>,
        synonym_kind: &str,
        list_kind: &str,
        lead: &str,
        lines: &mut Vec<String>,
    ) {
        let indent = self.config.indent;
        let synonym = clause
            .find_child(synonym_kind)
            .map(|s| self.kw(self.text(s)))
            .unwrap_or_default();
        lines.push(format!("{indent}{lead}{synonym} {} (", self.kw("TABLES")));
        if let Some(list) = clause.find_child(list_kind) {
            let defs = flatten_list(list, list_kind);
            if self.config.river {
                self.graph_defs_aligned(&defs, lines);
            } else {
                // pg_dump and the other left-aligned styles put each element on
                // one line. pg_dump's 4-space indent yields the 8-space element
                // indent that pg_get_propgraphdef emits, so this is idempotent.
                self.graph_defs_simple(&defs, lines);
            }
        }
        lines.push(format!("{indent})"));
    }

    /// One element per line, no column alignment (non-river left-aligned styles).
    fn graph_defs_simple(&self, defs: &[Node<'a>], lines: &mut Vec<String>) {
        let elem = self.config.indent.repeat(2);
        let last = defs.len().saturating_sub(1);
        for (i, def) in defs.iter().enumerate() {
            let comma = if i < last { "," } else { "" };
            lines.push(format!("{elem}{}{comma}", self.render_graph_inline(*def)));
        }
    }

    /// River-family layout: one element per line with each field
    /// (name, KEY, SOURCE, DESTINATION, LABEL/PROPERTIES) padded into an
    /// aligned column, like the CREATE TABLE column/type/constraint alignment.
    fn graph_defs_aligned(&self, defs: &[Node<'a>], lines: &mut Vec<String>) {
        let elem = self.config.indent.repeat(2);
        let rows: Vec<[String; 5]> = defs.iter().map(|d| self.graph_element_cells(*d)).collect();
        // Column widths over the fields present in at least one element.
        let mut widths = [0usize; 5];
        for row in &rows {
            for (c, cell) in row.iter().enumerate() {
                widths[c] = widths[c].max(cell.chars().count());
            }
        }
        let active: Vec<usize> = (0..5).filter(|&c| widths[c] > 0).collect();
        let last_row = rows.len().saturating_sub(1);
        for (i, row) in rows.iter().enumerate() {
            let mut line = elem.clone();
            for (j, &c) in active.iter().enumerate() {
                if j + 1 == active.len() {
                    line.push_str(&row[c]);
                } else {
                    // Pad the field to its column width and separate with a
                    // single space, exactly as CREATE TABLE's column/type
                    // alignment does (see render_aligned_column in stmt.rs).
                    let pad = widths[c] - row[c].chars().count();
                    line.push_str(&row[c]);
                    line.push_str(&" ".repeat(pad + 1));
                }
            }
            let mut line = line.trim_end().to_string();
            if i < last_row {
                line.push(',');
            }
            lines.push(line);
        }
    }

    /// Split an element definition into its five aligned columns:
    /// name (with any alias), KEY, SOURCE, DESTINATION, LABEL/PROPERTIES.
    /// Absent fields are empty strings.
    fn graph_element_cells(&self, def: Node<'a>) -> [String; 5] {
        let mut cells: [String; 5] = Default::default();
        let mut cursor = def.walk();
        for child in def.named_children(&mut cursor) {
            match child.kind() {
                "qualified_name" => cells[0] = self.format_expr(child),
                "opt_propgraph_table_alias" => {
                    cells[0].push(' ');
                    cells[0].push_str(&self.render_graph_inline(child));
                }
                "opt_graph_table_key_clause" => cells[1] = self.render_graph_inline(child),
                "source_vertex_table" => cells[2] = self.render_graph_inline(child),
                "destination_vertex_table" => cells[3] = self.render_graph_inline(child),
                "opt_element_table_label_and_properties" => {
                    cells[4] = self.render_graph_inline(child)
                }
                _ => {}
            }
        }
        cells
    }

    /// Render a property-graph clause node inline, casing keyword leaves and
    /// gluing punctuation. Used for element table definitions, which mix
    /// identifiers, `LABEL`/`SOURCE`/`DESTINATION`/`KEY` keywords, and lists.
    fn render_graph_inline(&self, node: Node<'a>) -> String {
        let kind = node.kind();
        if kind.starts_with("kw_") {
            return self.kw(self.text(node));
        }
        match kind {
            "qualified_name" => return self.format_expr(node),
            "labeled_expr_list" => return self.format_labeled_expr_list(node),
            "name" | "ColId" | "columnref" | "identifier" | "quoted_identifier" => {
                return self.text(node).to_string();
            }
            _ => {}
        }
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();
        if children.is_empty() {
            return self.text(node).to_string();
        }
        let mut out = String::new();
        for child in children {
            let piece = self.render_graph_inline(child);
            if piece.is_empty() {
                continue;
            }
            if out.is_empty() {
                out.push_str(&piece);
                continue;
            }
            let tok = self.text(child);
            let last = out.chars().last().unwrap_or(' ');
            if tok == ")" || tok == "," || last == '(' {
                out.push_str(&piece);
            } else {
                out.push(' ');
                out.push_str(&piece);
            }
        }
        out
    }

    /// Format a table reference (for FROM clause), returning the table name with alias.
    pub(crate) fn format_table_ref(&self, node: Node<'a>) -> String {
        // GRAPH_TABLE (...) SQL/PGQ property-graph query (PG19).
        if node.has_child("kw_graph_table") {
            return self.format_graph_table(node);
        }
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

/// Collapse consecutive dots (`..` → `.`) that arise from joining a qualified
/// name's separator with an attr's own leading dot, while leaving dots inside
/// double-quoted identifiers untouched.
fn collapse_double_dots_outside_quotes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_quote = false;
    let mut prev_dot = false;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        let ch = chars[i];
        if in_quote {
            result.push(ch);
            if ch == '"' {
                if i + 1 < len && chars[i + 1] == '"' {
                    result.push('"');
                    i += 2;
                    continue;
                }
                in_quote = false;
            }
            i += 1;
            continue;
        }
        if ch == '"' {
            in_quote = true;
            prev_dot = false;
            result.push(ch);
        } else if ch == '.' {
            if !prev_dot {
                result.push('.');
            }
            prev_dot = true;
        } else {
            prev_dot = false;
            result.push(ch);
        }
        i += 1;
    }
    result
}

/// Returns true when `s` contains whitespace at the top level (outside any
/// parentheses/brackets and outside quoted strings), indicating a compound
/// operand such as `a + b` or `foo(x) + 1` rather than a bare identifier,
/// literal, or single function call.
fn has_top_level_space(s: &str) -> bool {
    let mut depth: i32 = 0;
    let mut in_single = false;
    // True when the current single-quoted string is a PostgreSQL escape string
    // (E'...'), which uses backslash escaping rather than only '' doubling.
    let mut escape_string = false;
    let mut in_double = false;
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;
    while i < len {
        let ch = chars[i];
        if in_single {
            // In an E'...' string a backslash escapes the next character, so a
            // \' is not a terminator.
            if escape_string && ch == '\\' && i + 1 < len {
                i += 2;
                continue;
            }
            if ch == '\'' {
                // A doubled '' is an escaped quote, not a terminator.
                if i + 1 < len && chars[i + 1] == '\'' {
                    i += 2;
                    continue;
                }
                in_single = false;
                escape_string = false;
            }
            i += 1;
            continue;
        }
        if in_double {
            if ch == '"' {
                if i + 1 < len && chars[i + 1] == '"' {
                    i += 2;
                    continue;
                }
                in_double = false;
            }
            i += 1;
            continue;
        }
        match ch {
            '\'' => {
                in_single = true;
                // An E/e immediately preceding the quote (and not part of a
                // longer identifier) marks a backslash-escaped string literal.
                escape_string = i >= 1
                    && matches!(chars[i - 1], 'E' | 'e')
                    && (i < 2 || !(chars[i - 2].is_ascii_alphanumeric() || chars[i - 2] == '_'));
            }
            '"' => in_double = true,
            '(' | '[' => depth += 1,
            ')' | ']' => depth -= 1,
            c if c.is_whitespace() && depth == 0 => return true,
            _ => {}
        }
        i += 1;
    }
    false
}

/// Collapse runs of whitespace to single spaces, but preserve whitespace
/// inside single-quoted or double-quoted strings.
fn collapse_whitespace_outside_quotes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut in_single_quote = false;
    // True when the current single-quoted string is a PostgreSQL escape string
    // (E'...'), which uses backslash escaping rather than only '' doubling.
    let mut escape_string = false;
    let mut in_double_quote = false;
    let mut prev_was_space = false;

    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        let ch = chars[i];

        if in_single_quote {
            result.push(ch);
            // In an E'...' string a backslash escapes the next character, so a
            // \' is not a terminator and its following bytes stay untouched.
            if escape_string && ch == '\\' && i + 1 < len {
                result.push(chars[i + 1]);
                i += 2;
                continue;
            }
            if ch == '\'' {
                // Check for escaped quote ('').
                if i + 1 < len && chars[i + 1] == '\'' {
                    result.push('\'');
                    i += 2;
                    continue;
                }
                in_single_quote = false;
                escape_string = false;
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
            // An E/e immediately preceding the quote (and not part of a longer
            // identifier) marks a backslash-escaped string literal.
            escape_string = i >= 1
                && matches!(chars[i - 1], 'E' | 'e')
                && (i < 2 || !(chars[i - 2].is_ascii_alphanumeric() || chars[i - 2] == '_'));
            prev_was_space = false;
            result.push(ch);
        } else if ch == '"' {
            in_double_quote = true;
            prev_was_space = false;
            result.push(ch);
        } else if ch == '$' {
            // Dollar-quoted string: $$...$$ or $tag$...$tag$.
            let tag_start = i;
            let mut tag_end = i + 1;
            while tag_end < len && (chars[tag_end].is_ascii_alphanumeric() || chars[tag_end] == '_')
            {
                tag_end += 1;
            }
            if tag_end < len && chars[tag_end] == '$' {
                let tag: String = chars[tag_start..=tag_end].iter().collect();
                result.push_str(&tag);
                i = tag_end + 1;
                while i < len {
                    let remaining: String = chars[i..].iter().collect();
                    if remaining.starts_with(&tag) {
                        result.push_str(&tag);
                        i += tag.len();
                        break;
                    }
                    result.push(chars[i]);
                    i += 1;
                }
                prev_was_space = false;
                continue;
            }
            // Not a dollar-quote, just a dollar sign.
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
