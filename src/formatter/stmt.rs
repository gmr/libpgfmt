/// Statement-level formatting: dispatches to specific statement formatters.
use crate::error::FormatError;
use crate::node_helpers::{NodeExt, flatten_list};
use tree_sitter::Node;

use super::Formatter;

/// Classification of table elements for river-style CREATE TABLE.
enum TableElementKind {
    /// PRIMARY KEY constraint (should be first).
    PrimaryKey(String),
    /// Column definition: (name, typename, constraints_text).
    Column(String, String, String),
    /// Table constraint: (optional_name, body).
    Constraint(Option<String>, String),
}

impl<'a> Formatter<'a> {
    /// Format a `stmt` node, dispatching based on the statement type.
    pub(crate) fn format_stmt(&self, node: Node<'a>) -> Result<String, FormatError> {
        let mut cursor = node.walk();
        if let Some(child) = node.named_children(&mut cursor).next() {
            let result = match child.kind() {
                "SelectStmt" => self.format_select_stmt(child),
                "InsertStmt" => self.format_insert_stmt(child),
                "UpdateStmt" => self.format_update_stmt(child),
                "DeleteStmt" => self.format_delete_stmt(child),
                "CreateStmt" => self.format_create_table_stmt(child),
                "ViewStmt" => self.format_view_stmt(child),
                "CreateFunctionStmt" => self.format_create_function_stmt(child),
                "CreateDomainStmt" => self.format_create_domain_stmt(child),
                "CreateForeignTableStmt" => self.format_create_foreign_table_stmt(child),
                "CreateTableAsStmt" | "CreateMatViewStmt" => {
                    self.format_create_table_as_stmt(child)
                }
                _ => {
                    let text = self.text(child);
                    normalize_whitespace(text)
                }
            };
            let trimmed = result.trim_end_matches(';');
            // If the last line contains a line comment (--), appending ;
            // directly would put the semicolon inside the comment.
            let needs_newline = trimmed
                .lines()
                .last()
                .map(|line| line.contains("--"))
                .unwrap_or(false);
            return if needs_newline {
                Ok(format!("{trimmed}\n;"))
            } else {
                Ok(format!("{trimmed};"))
            };
        }
        Ok(String::new())
    }

    // ── INSERT ──────────────────────────────────────────────────────────

    pub(crate) fn format_insert_stmt(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();

        // INSERT INTO target.
        let target = node
            .find_child("insert_target")
            .map(|n| self.format_qualified_name_from(n))
            .unwrap_or_default();
        parts.push(format!(
            "{} {} {target}",
            self.kw("INSERT"),
            self.kw("INTO")
        ));

        // Column list.
        let insert_rest = node.find_child("insert_rest");
        if let Some(rest) = insert_rest {
            if let Some(col_list) = rest.find_child("insert_column_list") {
                let cols = flatten_list(col_list, "insert_column_list");
                let formatted: Vec<_> = cols.iter().map(|c| self.format_expr(*c)).collect();
                parts[0] = format!("{} ({})", parts[0], formatted.join(", "));
            }

            // VALUES or SELECT.
            if let Some(select) = rest.find_child("SelectStmt") {
                let formatted = self.format_select_stmt(select);
                // Check if it's VALUES or a sub-SELECT.
                let select_text = formatted.trim_end_matches(';');
                let values_kw = self.kw("VALUES");
                let is_values = select_text.trim_start().starts_with(&values_kw);

                if is_values && self.config.river {
                    // River: VALUES aligned with INSERT INTO.
                    // Compute padding to right-align VALUES with INSERT INTO.
                    let insert_kw_len = parts[0]
                        .split(' ')
                        .take(2)
                        .collect::<Vec<_>>()
                        .join(" ")
                        .len();
                    let river_width = std::cmp::max(insert_kw_len, values_kw.len());

                    // Strip VALUES keyword and any pre-existing indentation from
                    // the formatter's multi-line output.
                    let raw_content = select_text.trim_start_matches(&values_kw);
                    let trimmed_lines: Vec<_> = raw_content.lines().map(|l| l.trim()).collect();
                    let content = trimmed_lines.join("\n");

                    if self.config.leading_commas && trimmed_lines.len() > 1 {
                        // For leading commas, handle continuation lines manually:
                        // the `, ` replaces 2 chars of the indent padding.
                        let kw_padding = if values_kw.len() < river_width {
                            " ".repeat(river_width - values_kw.len())
                        } else {
                            String::new()
                        };
                        let first_line_content = trimmed_lines[0].trim();
                        parts.push(format!("{kw_padding}{values_kw} {first_line_content}"));
                        let content_col = river_width + 1; // where content starts
                        for line in &trimmed_lines[1..] {
                            let trimmed = line.trim();
                            if trimmed.starts_with(',') {
                                // Leading comma: put it 2 chars before content col.
                                let padding = " ".repeat(content_col - 2);
                                parts.push(format!("{padding}{trimmed}"));
                            } else if !trimmed.is_empty() {
                                let padding = " ".repeat(content_col);
                                parts.push(format!("{padding}{trimmed}"));
                            }
                        }
                    } else {
                        parts.push(self.river_line(&values_kw, content.trim(), river_width));
                    }
                } else {
                    // Real SELECT or non-river VALUES: emit as-is.
                    parts.push(select_text.to_string());
                }
            }
        }

        parts.join("\n")
    }

    // ── UPDATE ──────────────────────────────────────────────────────────

    pub(crate) fn format_update_stmt(&self, node: Node<'a>) -> String {
        let table = node
            .find_child("relation_expr_opt_alias")
            .map(|n| self.format_relation_expr_opt_alias(n))
            .unwrap_or_default();

        let mut lines = Vec::new();

        if self.config.river {
            // Collect keywords for river width.
            let mut keywords = vec![self.kw("UPDATE"), self.kw("SET")];
            if node.has_child("where_or_current_clause") {
                keywords.push(self.kw("WHERE"));
            }
            let width = keywords.iter().map(|k| k.len()).max().unwrap_or(6);

            lines.push(self.river_line(&self.kw("UPDATE"), &table, width));

            // SET clause.
            if let Some(set_list) = node.find_child("set_clause_list") {
                let clauses = flatten_list(set_list, "set_clause_list");
                let formatted: Vec<_> =
                    clauses.iter().map(|c| self.format_set_clause(*c)).collect();
                if formatted.len() == 1 {
                    lines.push(self.river_line(&self.kw("SET"), &formatted[0], width));
                } else if self.config.leading_commas {
                    // Leading commas: first item without comma, subsequent with leading ", ".
                    lines.push(self.river_line(&self.kw("SET"), &formatted[0], width));
                    let content_col = width + 1;
                    for clause in &formatted[1..] {
                        let padding = " ".repeat(content_col - 2);
                        lines.push(format!("{padding}, {clause}"));
                    }
                } else {
                    lines.push(self.river_line(
                        &self.kw("SET"),
                        &format!("{},", formatted[0]),
                        width,
                    ));
                    let content_col = width + 1;
                    for (i, clause) in formatted[1..].iter().enumerate() {
                        let padding = " ".repeat(content_col);
                        if i < formatted.len() - 2 {
                            lines.push(format!("{padding}{clause},"));
                        } else {
                            lines.push(format!("{padding}{clause}"));
                        }
                    }
                }
            }

            // WHERE clause.
            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_river(where_c, width, &mut lines);
            }
        } else {
            lines.push(format!("{} {table}", self.kw("UPDATE")));

            // SET clause.
            let indent = self.config.indent;
            if let Some(set_list) = node.find_child("set_clause_list") {
                let clauses = flatten_list(set_list, "set_clause_list");
                let formatted: Vec<_> =
                    clauses.iter().map(|c| self.format_set_clause(*c)).collect();
                lines.push(self.kw("SET"));
                for (i, clause) in formatted.iter().enumerate() {
                    if i < formatted.len() - 1 {
                        lines.push(format!("{indent}{clause},"));
                    } else {
                        lines.push(format!("{indent}{clause}"));
                    }
                }
            }

            // WHERE clause.
            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_left_aligned(where_c, &mut lines);
            }
        }

        lines.join("\n")
    }

    fn format_set_clause(&self, node: Node<'a>) -> String {
        let target = node
            .find_child("set_target")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        let value = node
            .find_child_any(&["a_expr", "c_expr"])
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        format!("{target} = {value}")
    }

    // ── DELETE ──────────────────────────────────────────────────────────

    pub(crate) fn format_delete_stmt(&self, node: Node<'a>) -> String {
        let table = node
            .find_child("relation_expr_opt_alias")
            .map(|n| self.format_relation_expr_opt_alias(n))
            .unwrap_or_default();

        let mut lines = Vec::new();

        if self.config.river {
            let delete_kw = self.kw("DELETE");
            let mut keywords = vec![delete_kw.clone(), self.kw("FROM")];
            if node.has_child("where_or_current_clause") {
                keywords.push(self.kw("WHERE"));
            }
            let width = keywords.iter().map(|k| k.len()).max().unwrap_or(6);

            lines.push(delete_kw);
            lines.push(self.river_line(&self.kw("FROM"), &table, width));

            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_river(where_c, width, &mut lines);
            }
        } else {
            lines.push(format!("{} {} {table}", self.kw("DELETE"), self.kw("FROM")));
            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_left_aligned(where_c, &mut lines);
            }
        }

        lines.join("\n")
    }

    // ── CREATE TABLE ────────────────────────────────────────────────────

    fn format_create_table_stmt(&self, node: Node<'a>) -> String {
        let table_name = node
            .find_child("qualified_name")
            .map(|n| self.format_qualified_name(n))
            .unwrap_or_default();

        let mut lines = Vec::new();
        lines.push(format!(
            "{} {} {table_name} (",
            self.kw("CREATE"),
            self.kw("TABLE")
        ));

        // Column definitions and constraints.
        if let Some(elem_list) = node
            .find_child("OptTableElementList")
            .and_then(|n| n.find_child("TableElementList"))
        {
            let elements = flatten_list(elem_list, "TableElementList");
            let indent = self.config.indent;

            if self.config.river {
                // River style: PRIMARY KEY first, padded columns, constraint
                // on separate indented line.
                let mut pk_elements = Vec::new();
                let mut col_elements = Vec::new();
                let mut constraint_elements = Vec::new();

                for e in &elements {
                    let elem = self.classify_table_element(*e);
                    match elem {
                        TableElementKind::PrimaryKey(text) => pk_elements.push(text),
                        TableElementKind::Column(name, typename, constraints) => {
                            col_elements.push((name, typename, constraints));
                        }
                        TableElementKind::Constraint(name, body) => {
                            constraint_elements.push((name, body));
                        }
                    }
                }

                // Calculate max column name and type widths for alignment.
                let max_name_len = col_elements
                    .iter()
                    .map(|(n, _, _)| n.len())
                    .max()
                    .unwrap_or(0);
                let max_type_len = col_elements
                    .iter()
                    .map(|(_, t, _)| t.len())
                    .max()
                    .unwrap_or(0);

                // Build ordered list: PKs first, then columns, then constraints.
                let mut all_items: Vec<String> = Vec::new();
                for pk in &pk_elements {
                    all_items.push(pk.clone());
                }
                for (name, typename, constraints) in &col_elements {
                    let padded_name = format!("{:width$}", name, width = max_name_len);
                    let padded_type = format!("{:width$}", typename, width = max_type_len);
                    let mut item = format!("{padded_name} {padded_type}");
                    if !constraints.is_empty() {
                        item = format!("{item} {constraints}");
                    }
                    all_items.push(item);
                }
                // Table constraints: CONSTRAINT name on one line,
                // CHECK(...) on the next, both aligned with the type column.
                for (name, body) in &constraint_elements {
                    let constraint_padding = " ".repeat(max_name_len + 1);
                    if let Some(cname) = name {
                        all_items.push(format!(
                            "{constraint_padding}{} {cname}\n{constraint_padding}{body}",
                            self.kw("CONSTRAINT")
                        ));
                    } else {
                        all_items.push(format!("{constraint_padding}{body}"));
                    }
                }

                for (i, item) in all_items.iter().enumerate() {
                    let comma = if i < all_items.len() - 1 { "," } else { "" };
                    if item.contains('\n') {
                        // Multi-line item (constraint): only add comma to last line.
                        let item_lines: Vec<&str> = item.lines().collect();
                        for (j, line) in item_lines.iter().enumerate() {
                            if j == item_lines.len() - 1 {
                                lines.push(format!("{indent}{line}{comma}"));
                            } else {
                                lines.push(format!("{indent}{line}"));
                            }
                        }
                    } else {
                        lines.push(format!("{indent}{item}{comma}"));
                    }
                }
            } else {
                let formatted: Vec<_> = elements
                    .iter()
                    .map(|e| self.format_table_element(*e))
                    .collect();

                for (i, elem) in formatted.iter().enumerate() {
                    let comma = if i < formatted.len() - 1 { "," } else { "" };
                    lines.push(format!("{indent}{elem}{comma}"));
                }
            }
        }

        lines.push(")".to_string());

        // WITH clause for storage parameters.
        // OptWith already contains the WITH keyword, so just normalize.
        if let Some(with) = node.find_child("OptWith") {
            let text = normalize_whitespace(self.text(with));
            if !text.is_empty() {
                lines.push(text);
            }
        }

        lines.join("\n")
    }

    /// Classify a table element for river-style CREATE TABLE formatting.
    fn classify_table_element(&self, node: Node<'a>) -> TableElementKind {
        match node.kind() {
            "TableElement" => {
                if let Some(col) = node.find_child("columnDef") {
                    let name = col
                        .find_child("ColId")
                        .map(|n| self.format_col_id(n))
                        .unwrap_or_default();
                    let typename = col
                        .find_child("Typename")
                        .map(|n| self.format_typename(n))
                        .unwrap_or_default();
                    let mut constraint_parts = Vec::new();
                    if let Some(qual_list) = col.find_child("ColQualList") {
                        let constraints = flatten_list(qual_list, "ColQualList");
                        for child in constraints {
                            if child.kind() == "ColConstraint" {
                                constraint_parts.push(self.format_col_constraint(child));
                            }
                        }
                    }
                    return TableElementKind::Column(name, typename, constraint_parts.join(" "));
                }
                if let Some(constraint) = node.find_child("TableConstraint") {
                    return self.classify_table_constraint(constraint);
                }
                TableElementKind::Column(self.text(node).to_string(), String::new(), String::new())
            }
            _ => {
                TableElementKind::Column(self.text(node).to_string(), String::new(), String::new())
            }
        }
    }

    fn classify_table_constraint(&self, node: Node<'a>) -> TableElementKind {
        let constraint_name = node.find_child("name").map(|n| self.format_expr(n));

        if let Some(elem) = node.find_child("ConstraintElem") {
            // Check if it's PRIMARY KEY.
            let mut is_pk = false;
            let mut cursor = elem.walk();
            for child in elem.named_children(&mut cursor) {
                if child.kind() == "kw_primary" {
                    is_pk = true;
                    break;
                }
            }
            if is_pk {
                let formatted = self.format_constraint_elem(elem);
                return if let Some(cname) = constraint_name {
                    TableElementKind::PrimaryKey(format!(
                        "{} {cname} {formatted}",
                        self.kw("CONSTRAINT")
                    ))
                } else {
                    TableElementKind::PrimaryKey(formatted)
                };
            }
            let body = self.format_constraint_elem(elem);
            return TableElementKind::Constraint(constraint_name, body);
        }
        TableElementKind::Constraint(constraint_name, self.text(node).to_string())
    }

    fn format_table_element(&self, node: Node<'a>) -> String {
        match node.kind() {
            "TableElement" => {
                if let Some(col) = node.find_child("columnDef") {
                    return self.format_column_def(col);
                }
                if let Some(constraint) = node.find_child("TableConstraint") {
                    return self.format_table_constraint(constraint);
                }
                self.text(node).to_string()
            }
            _ => self.text(node).to_string(),
        }
    }

    fn format_column_def(&self, node: Node<'a>) -> String {
        let name = node
            .find_child("ColId")
            .map(|n| self.format_col_id(n))
            .unwrap_or_default();
        let typename = node
            .find_child("Typename")
            .map(|n| self.format_typename(n))
            .unwrap_or_default();

        let mut parts = vec![name, typename];

        // Column constraints.
        if let Some(qual_list) = node.find_child("ColQualList") {
            let constraints = flatten_list(qual_list, "ColQualList");
            for child in constraints {
                if child.kind() == "ColConstraint" {
                    parts.push(self.format_col_constraint(child));
                }
            }
        }

        parts.join(" ")
    }

    fn format_col_constraint(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        // Optional constraint name.
        if let Some(name) = node.find_child("name") {
            parts.push(self.kw("CONSTRAINT"));
            parts.push(self.format_expr(name));
        }
        if let Some(elem) = node.find_child("ColConstraintElem") {
            parts.push(self.format_col_constraint_elem(elem));
        }
        parts.join(" ")
    }

    fn format_col_constraint_elem(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "kw_not" => parts.push(self.kw("NOT")),
                "kw_null" => parts.push(self.kw("NULL")),
                "kw_primary" => parts.push(self.kw("PRIMARY")),
                "kw_key" => parts.push(self.kw("KEY")),
                "kw_unique" => parts.push(self.kw("UNIQUE")),
                "kw_default" => parts.push(self.kw("DEFAULT")),
                "kw_check" => parts.push(self.kw("CHECK")),
                "kw_references" => parts.push(self.kw("REFERENCES")),
                "a_expr" | "c_expr" | "b_expr" => {
                    parts.push(self.format_expr(child));
                }
                _ if child.kind().starts_with("kw_") => {
                    parts.push(self.kw(self.text(child)));
                }
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.join(" ")
    }

    fn format_table_constraint(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        if let Some(name) = node.find_child("name") {
            parts.push(self.kw("CONSTRAINT"));
            parts.push(self.format_expr(name));
        }
        if let Some(elem) = node.find_child("ConstraintElem") {
            parts.push(self.format_constraint_elem(elem));
        }
        parts.join(" ")
    }

    fn format_constraint_elem(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut has_check = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "kw_primary" => parts.push(self.kw("PRIMARY")),
                    "kw_key" => parts.push(self.kw("KEY")),
                    "kw_unique" => parts.push(self.kw("UNIQUE")),
                    "kw_check" => {
                        has_check = true;
                        parts.push(self.kw("CHECK"));
                    }
                    "kw_foreign" => parts.push(self.kw("FOREIGN")),
                    "kw_references" => parts.push(self.kw("REFERENCES")),
                    "columnList" => {
                        let items = flatten_list(child, "columnList");
                        let formatted: Vec<_> =
                            items.iter().map(|i| self.format_expr(*i)).collect();
                        parts.push(format!("({})", formatted.join(", ")));
                    }
                    "a_expr" | "c_expr" => {
                        let expr_text = format!("({})", self.format_expr(child));
                        if has_check && self.config.river {
                            // River style: CHECK(expr) without space.
                            if let Some(last) = parts.last_mut() {
                                *last = format!("{last}{expr_text}");
                            }
                        } else {
                            parts.push(expr_text);
                        }
                    }
                    _ if child.kind().starts_with("kw_") => {
                        parts.push(self.kw(self.text(child)));
                    }
                    _ => parts.push(self.format_expr(child)),
                }
            } else {
                let text = self.text(child).trim();
                if text == "(" || text == ")" {
                    // Handled by columnList formatting.
                }
            }
        }
        parts.join(" ")
    }

    // ── CREATE VIEW ─────────────────────────────────────────────────────

    fn format_view_stmt(&self, node: Node<'a>) -> String {
        let mut prefix = format!("{} {}", self.kw("CREATE"), self.kw("VIEW"));

        // View name.
        let name = node
            .find_child("qualified_name")
            .or_else(|| node.find_child("view_name"))
            .map(|n| self.format_qualified_name(n))
            .unwrap_or_default();
        prefix = format!("{prefix} {name} {}", self.kw("AS"));

        // The SELECT body.
        if let Some(select) = node.find_child("SelectStmt") {
            let body = self.format_select_stmt(select);
            format!("{prefix}\n{}", body.trim_end_matches(';'))
        } else {
            prefix
        }
    }

    // ── CREATE TABLE AS / CREATE MATERIALIZED VIEW ──────────────────────

    fn format_create_table_as_stmt(&self, node: Node<'a>) -> String {
        let kind = node.kind();
        let mut prefix_parts = vec![self.kw("CREATE")];

        if kind == "CreateMatViewStmt" {
            prefix_parts.push(self.kw("MATERIALIZED"));
            prefix_parts.push(self.kw("VIEW"));
        } else {
            // Could be CREATE TABLE AS or CREATE MATERIALIZED VIEW AS.
            if node.has_child("kw_materialized") {
                prefix_parts.push(self.kw("MATERIALIZED"));
                prefix_parts.push(self.kw("VIEW"));
            } else {
                prefix_parts.push(self.kw("TABLE"));
            }
        }

        let name = self.find_name_in_create(node);
        prefix_parts.push(name);
        prefix_parts.push(self.kw("AS"));

        let prefix = prefix_parts.join(" ");

        // The SELECT body.
        let mut body = String::new();
        if let Some(select) = node.find_child("SelectStmt") {
            body = self.format_select_stmt(select);
        } else if let Some(query) = node.find_child("create_as_target")
            && let Some(select) = query.find_child("SelectStmt")
        {
            body = self.format_select_stmt(select);
        }

        let body = body.trim_end_matches(';');

        // Check for WITH NO DATA.
        let mut suffix = String::new();
        if node.has_child("kw_no") || self.text(node).contains("WITH NO DATA") {
            suffix = format!(
                "\n{} {} {}",
                self.kw("WITH"),
                self.kw("NO"),
                self.kw("DATA")
            );
        }

        format!("{prefix}\n{body}{suffix}")
    }

    // ── CREATE FUNCTION ─────────────────────────────────────────────────

    fn format_create_function_stmt(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();

        // CREATE FUNCTION/PROCEDURE name(args)
        let mut header = vec![self.kw("CREATE")];
        if node.has_child("kw_procedure") {
            header.push(self.kw("PROCEDURE"));
        } else {
            header.push(self.kw("FUNCTION"));
        }

        let func_name = node
            .find_child("func_name")
            .map(|n| self.format_func_name(n))
            .unwrap_or_default();
        header.push(func_name);

        // Arguments.
        if let Some(args) = node.find_child("func_args_with_defaults") {
            let args_text = self.format_func_args(args);
            let last = header.last_mut().unwrap();
            *last = format!("{last}{args_text}");
        }

        // RETURNS type.
        if let Some(ret) = node.find_child("func_return") {
            let ret_type = self.format_func_return(ret);
            header.push(format!("{} {ret_type}", self.kw("RETURNS")));
        }

        parts.push(header.join(" "));

        // Function options (LANGUAGE, AS, etc.).
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "opt_createfunc_opt_list" | "createfunc_opt_list" => {
                    self.format_createfunc_opts(child, &mut parts);
                }
                _ => {}
            }
        }

        parts.join("\n    ")
    }

    fn format_func_args(&self, node: Node<'a>) -> String {
        // Reconstruct the function arguments.
        let text = self.text(node);
        // For now, normalize whitespace in the args.

        normalize_whitespace(text)
    }

    fn format_func_return(&self, node: Node<'a>) -> String {
        if let Some(ft) = node.find_child("func_type")
            && let Some(tn) = ft.find_child("Typename")
        {
            return self.format_typename(tn);
        }
        self.text(node).trim().to_string()
    }

    fn format_createfunc_opts(&self, node: Node<'a>, parts: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "createfunc_opt_item" => {
                    self.format_createfunc_opt_item(child, parts);
                }
                "createfunc_opt_list" => {
                    self.format_createfunc_opts(child, parts);
                }
                _ => {}
            }
        }
    }

    fn format_createfunc_opt_item(&self, node: Node<'a>, parts: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "kw_language" => {
                    if let Some(lang) = node.find_child("NonReservedWord_or_Sconst") {
                        parts.push(format!("{} {}", self.kw("LANGUAGE"), self.text(lang)));
                    }
                }
                "func_as" => {
                    // AS $$ ... $$
                    // Preserve original line breaks in the body — collapsing
                    // newlines to spaces would break line-comment (--) semantics.
                    // For single-line bodies, normalize whitespace (safe since
                    // there are no newlines that could affect -- comments).
                    // For multi-line bodies, re-indent each line to preserve
                    // the original structure.
                    let text = self.text(child).trim();
                    if let Some((tag, body)) = parse_dollar_quoted(text) {
                        if body.contains('\n') {
                            let body = reindent_body(body, " ");
                            parts.push(format!("{} {tag}\n{body}\n{tag}", self.kw("AS")));
                        } else {
                            let body = normalize_whitespace(body);
                            parts.push(format!("{} {tag}\n {body}\n{tag}", self.kw("AS")));
                        }
                    } else {
                        parts.push(format!("{} {text}", self.kw("AS")));
                    }
                }
                _ => {}
            }
        }
    }

    // ── CREATE DOMAIN ───────────────────────────────────────────────────

    fn format_create_domain_stmt(&self, node: Node<'a>) -> String {
        let name = self.find_name_in_create(node);
        let mut parts = vec![format!(
            "{} {} {name}",
            self.kw("CREATE"),
            self.kw("DOMAIN")
        )];

        // AS typename.
        if let Some(tn) = node.find_child("Typename") {
            parts[0] = format!(
                "{} {} {}",
                parts[0],
                self.kw("AS"),
                self.format_typename(tn)
            );
        }

        // Constraints.
        if let Some(constraints) = node.find_child("ColQualList") {
            let mut cursor = constraints.walk();
            for child in constraints.named_children(&mut cursor) {
                if child.kind() == "ColConstraint" {
                    let indent = self.config.indent;
                    parts.push(format!("{indent}{}", self.format_col_constraint(child)));
                }
            }
        }

        parts.join("\n")
    }

    // ── CREATE FOREIGN TABLE ────────────────────────────────────────────

    fn format_create_foreign_table_stmt(&self, node: Node<'a>) -> String {
        let table_name = node
            .find_child("qualified_name")
            .map(|n| self.format_qualified_name(n))
            .unwrap_or_default();

        let mut lines = Vec::new();
        lines.push(format!(
            "{} {} {} {table_name} (",
            self.kw("CREATE"),
            self.kw("FOREIGN"),
            self.kw("TABLE")
        ));

        // Column definitions (same as CREATE TABLE).
        if let Some(elem_list) = node
            .find_child("OptTableElementList")
            .and_then(|n| n.find_child("TableElementList"))
        {
            let elements = flatten_list(elem_list, "TableElementList");
            let indent = self.config.indent;

            if self.config.river {
                // Classify all elements, keeping their original order.
                let classified: Vec<_> = elements
                    .iter()
                    .map(|e| self.classify_table_element(*e))
                    .collect();

                // Compute column-alignment widths from Column elements only.
                let max_name_len = classified
                    .iter()
                    .filter_map(|e| {
                        if let TableElementKind::Column(n, _, _) = e {
                            Some(n.len())
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(0);
                let max_type_len = classified
                    .iter()
                    .filter_map(|e| {
                        if let TableElementKind::Column(_, t, _) = e {
                            Some(t.len())
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(0);

                let total = classified.len();
                for (i, elem) in classified.iter().enumerate() {
                    let comma = if i < total - 1 { "," } else { "" };
                    match elem {
                        TableElementKind::Column(name, typename, constraints) => {
                            let padded_name = format!("{:width$}", name, width = max_name_len);
                            let padded_type = format!("{:width$}", typename, width = max_type_len);
                            let mut item = format!("{padded_name} {padded_type}");
                            if !constraints.is_empty() {
                                item = format!("{item} {constraints}");
                            }
                            lines.push(format!("{indent}{item}{comma}"));
                        }
                        TableElementKind::PrimaryKey(text)
                        | TableElementKind::Constraint(_, text) => {
                            let text = match elem {
                                TableElementKind::Constraint(Some(name), body) => {
                                    format!("{} {name} {body}", self.kw("CONSTRAINT"))
                                }
                                _ => text.clone(),
                            };
                            lines.push(format!("{indent}{text}{comma}"));
                        }
                    }
                }
            } else {
                let formatted: Vec<_> = elements
                    .iter()
                    .map(|e| self.format_table_element(*e))
                    .collect();
                for (i, elem) in formatted.iter().enumerate() {
                    let comma = if i < formatted.len() - 1 { "," } else { "" };
                    lines.push(format!("{indent}{elem}{comma}"));
                }
            }
        }

        lines.push(")".to_string());

        // SERVER name.
        if let Some(server_name) = node.find_child("name") {
            lines.push(format!(
                "{} {}",
                self.kw("SERVER"),
                self.format_expr(server_name)
            ));
        }

        // OPTIONS (...).
        if let Some(opts) = node.find_child("create_generic_options") {
            self.format_generic_options(opts, &mut lines);
        }

        lines.join("\n")
    }

    fn format_generic_options(&self, node: Node<'a>, lines: &mut Vec<String>) {
        if let Some(opt_list) = node.find_child("generic_option_list") {
            let items = flatten_list(opt_list, "generic_option_list");
            let indent = self.config.indent;

            lines.push(format!("{} (", self.kw("OPTIONS")));
            for (i, item) in items.iter().enumerate() {
                let formatted = self.format_generic_option(*item);
                let comma = if i < items.len() - 1 { "," } else { "" };
                lines.push(format!("{indent}{formatted}{comma}"));
            }
            lines.push(")".to_string());
        }
    }

    fn format_generic_option(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "generic_option_name" => parts.push(self.format_expr(child)),
                "generic_option_arg" => parts.push(self.format_expr(child)),
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.join(" ")
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    fn format_relation_expr_opt_alias(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "relation_expr" => parts.push(self.format_relation_expr(child)),
                "opt_alias_clause" | "alias_clause" => {
                    // alias_clause already includes the AS keyword.
                    let alias = self.format_expr(child);
                    if !alias.is_empty() {
                        parts.push(alias);
                    }
                }
                "ColId" => {
                    // Bare identifier alias without AS keyword.
                    let alias = self.format_expr(child);
                    if !alias.is_empty() {
                        parts.push(format!("{} {alias}", self.kw("AS")));
                    }
                }
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.join(" ")
    }

    fn format_qualified_name_from(&self, node: Node<'a>) -> String {
        // insert_target wraps a qualified_name.
        if let Some(qn) = node.find_child("qualified_name") {
            return self.format_expr(qn);
        }
        self.format_expr(node)
    }

    fn find_name_in_create(&self, node: Node<'a>) -> String {
        // Look for qualified_name, any_name, or create_as_target.
        if let Some(qn) = node.find_child("qualified_name") {
            return self.format_expr(qn);
        }
        if let Some(an) = node.find_child("any_name") {
            return self.format_expr(an);
        }
        if let Some(cat) = node.find_child("create_as_target") {
            if let Some(qn) = cat.find_child("qualified_name") {
                return self.format_expr(qn);
            }
            return self.format_expr(cat);
        }
        if let Some(mv) = node.find_child("create_mv_target") {
            if let Some(qn) = mv.find_child("qualified_name") {
                return self.format_expr(qn);
            }
            return self.format_expr(mv);
        }
        String::new()
    }

    // format_where_river and format_where_left_aligned are defined in select.rs
}

/// Parse a dollar-quoted string into (tag, body).
/// E.g., `$$ body $$` → Some(("$$", " body "))
/// E.g., `$fn$ body $fn$` → Some(("$fn$", " body "))
fn parse_dollar_quoted(s: &str) -> Option<(&str, &str)> {
    if !s.starts_with('$') {
        return None;
    }
    // Find the end of the opening tag.
    let tag_end = s[1..].find('$')? + 2; // +1 for the inner offset, +1 for the closing $
    let tag = &s[..tag_end];
    let rest = &s[tag_end..];
    // Find the closing tag.
    let body_end = rest.rfind(tag)?;
    let body = &rest[..body_end];
    Some((tag, body))
}

/// Re-indent a multi-line body (e.g., a dollar-quoted function body) so that
/// each non-empty line starts with the given `indent` prefix. Strips leading
/// and trailing blank lines, and removes the common leading whitespace from all
/// lines before applying the new indent.
fn reindent_body(s: &str, indent: &str) -> String {
    let lines: Vec<&str> = s.lines().collect();
    // Skip leading/trailing empty lines.
    let start = lines.iter().position(|l| !l.trim().is_empty()).unwrap_or(0);
    let end = lines
        .iter()
        .rposition(|l| !l.trim().is_empty())
        .map(|i| i + 1)
        .unwrap_or(lines.len());
    let body_lines = &lines[start..end];
    if body_lines.is_empty() {
        return String::new();
    }
    // Determine the minimum leading whitespace across non-empty lines.
    let min_indent = body_lines
        .iter()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.len() - l.trim_start().len())
        .min()
        .unwrap_or(0);
    body_lines
        .iter()
        .map(|line| {
            if line.trim().is_empty() {
                String::new()
            } else {
                format!("{indent}{}", &line[min_indent..])
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Collapse runs of whitespace to single spaces, but preserve whitespace
/// inside single-quoted strings, double-quoted identifiers, and dollar-quoted
/// strings so that literal content is not altered.
fn normalize_whitespace(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let chars: Vec<char> = s.chars().collect();
    let len = chars.len();
    let mut i = 0;
    let mut in_space_run = false;

    while i < len {
        let ch = chars[i];

        // Single-quoted string.
        if ch == '\'' {
            in_space_run = false;
            result.push(ch);
            i += 1;
            while i < len {
                result.push(chars[i]);
                if chars[i] == '\'' {
                    i += 1;
                    if i < len && chars[i] == '\'' {
                        result.push(chars[i]);
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Double-quoted identifier.
        if ch == '"' {
            in_space_run = false;
            result.push(ch);
            i += 1;
            while i < len {
                result.push(chars[i]);
                if chars[i] == '"' {
                    i += 1;
                    if i < len && chars[i] == '"' {
                        result.push(chars[i]);
                        i += 1;
                    } else {
                        break;
                    }
                } else {
                    i += 1;
                }
            }
            continue;
        }

        // Dollar-quoted string.
        if ch == '$' {
            let tag_start = i;
            let mut tag_end = i + 1;
            while tag_end < len && (chars[tag_end].is_ascii_alphanumeric() || chars[tag_end] == '_')
            {
                tag_end += 1;
            }
            if tag_end < len && chars[tag_end] == '$' {
                in_space_run = false;
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
                continue;
            }
        }

        // Normal whitespace collapsing.
        if ch.is_whitespace() {
            if !in_space_run && !result.is_empty() {
                result.push(' ');
            }
            in_space_run = true;
            i += 1;
        } else {
            in_space_run = false;
            result.push(ch);
            i += 1;
        }
    }

    result.trim().to_string()
}
