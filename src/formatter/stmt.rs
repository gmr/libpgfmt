/// Statement-level formatting: dispatches to specific statement formatters.
use crate::error::FormatError;
use crate::node_helpers::{NodeExt, flatten_list, flatten_list_keeping_comments};
use tree_sitter::Node;

use super::Formatter;
use super::lexical;

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
                "MergeStmt" => self.format_merge_stmt(child),
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

        // River aligns the CTE to the INSERT INTO keyword pair, the way the
        // VALUES clause below aligns to it. Held aside rather than pushed:
        // the column-list, OVERRIDING and DEFAULT VALUES branches below all
        // rewrite `parts[0]`, which must stay the INSERT header.
        let with_clause = self.dml_with_clause(node, self.kw_pair("INSERT", "INTO").len());

        // INSERT INTO target.
        let target = node
            .find_child("insert_target")
            .map(|n| self.format_insert_target(n))
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

            // OVERRIDING {SYSTEM|USER} VALUE (before VALUES/SELECT).
            if rest.has_child("kw_overriding") {
                let kind = rest
                    .find_child("override_kind")
                    .map(|ok| {
                        if ok.has_child("kw_system") {
                            self.kw("SYSTEM")
                        } else {
                            self.kw("USER")
                        }
                    })
                    .unwrap_or_default();
                parts[0] = format!(
                    "{} {} {kind} {}",
                    parts[0],
                    self.kw("OVERRIDING"),
                    self.kw("VALUE")
                );
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
            } else if rest.has_child("kw_default") {
                // INSERT INTO t DEFAULT VALUES.
                parts[0] = format!("{} {} {}", parts[0], self.kw("DEFAULT"), self.kw("VALUES"));
            }
        }

        // ON CONFLICT ... DO UPDATE/DO NOTHING.
        if let Some(on_conflict) = node.find_child("opt_on_conflict") {
            parts.push(self.format_on_conflict(on_conflict));
        }

        // RETURNING clause.
        if let Some(ret) = node.find_child("returning_clause")
            && let Some(text) = self.returning_text(ret)
        {
            if self.config.river {
                let width = format!("{} {}", self.kw("INSERT"), self.kw("INTO")).len();
                parts.push(self.river_line(&self.kw("RETURNING"), &text, width));
            } else {
                parts.push(format!("{} {text}", self.kw("RETURNING")));
            }
        }

        if let Some(with) = with_clause {
            parts.insert(0, with);
        }

        parts.join("\n")
    }

    /// Format an `opt_on_conflict` node into a single-line clause.
    fn format_on_conflict(&self, node: Node<'a>) -> String {
        let mut parts = vec![self.kw("ON"), self.kw("CONFLICT")];

        // Conflict target: (col, ...) or ON CONSTRAINT name.
        if let Some(conf) = node.find_child("opt_conf_expr") {
            if let Some(idx) = conf.find_child("index_params") {
                let items = flatten_list(idx, "index_params");
                let cols: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
                parts.push(format!("({})", cols.join(", ")));
            } else if let Some(name) = conf.find_child("name") {
                parts.push(format!(
                    "{} {}",
                    self.kw_pair("ON", "CONSTRAINT"),
                    self.format_expr(name)
                ));
            }
        }

        // DO NOTHING | DO UPDATE SET ... | DO SELECT [FOR ...]. Treating every
        // non-NOTHING form as DO UPDATE emitted `DO UPDATE SET` with an empty
        // SET for the PG19 DO SELECT form, which does not parse.
        parts.push(self.kw("DO"));
        if node.has_child("kw_nothing") {
            parts.push(self.kw("NOTHING"));
        } else if node.has_child("kw_select") {
            parts.push(self.kw("SELECT"));
            if let Some(lock) = node.find_child("opt_for_locking_strength") {
                parts.push(self.render_clause_inline(lock));
            }
        } else {
            parts.push(self.kw("UPDATE"));
            parts.push(self.kw("SET"));
            if let Some(set_list) = node.find_child("set_clause_list") {
                let clauses = flatten_list(set_list, "set_clause_list");
                let formatted: Vec<_> =
                    clauses.iter().map(|c| self.format_set_clause(*c)).collect();
                parts.push(formatted.join(", "));
            }
        }
        if let Some(where_c) = node.find_child("where_clause")
            && let Some(expr) = where_c.find_child_any(&["a_expr", "c_expr"])
        {
            parts.push(format!("{} {}", self.kw("WHERE"), self.format_expr(expr)));
        }

        parts.retain(|p| !p.is_empty());
        parts.join(" ")
    }

    /// Format the target list of a `returning_clause` into an inline,
    /// comma-separated string. Returns `None` when there are no targets.
    fn returning_text(&self, node: Node<'a>) -> Option<String> {
        let target_list = node.find_child("target_list")?;
        let targets = flatten_list(target_list, "target_list");
        if targets.is_empty() {
            return None;
        }
        let formatted: Vec<_> = targets.iter().map(|t| self.format_target_el(*t)).collect();
        Some(formatted.join(", "))
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
            if node.has_child("from_clause") {
                keywords.push(self.kw("FROM"));
            }
            if node.has_child("where_or_current_clause") {
                keywords.push(self.kw("WHERE"));
            }
            if node.has_child("returning_clause") {
                keywords.push(self.kw("RETURNING"));
            }
            let width = keywords.iter().map(|k| k.len()).max().unwrap_or(6);

            if let Some(with) = self.dml_with_clause(node, width) {
                lines.push(with);
            }
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

            // FROM clause.
            if let Some(from_c) = node.find_child("from_clause")
                && let Some(from_list) = from_c.find_child("from_list")
            {
                self.format_relation_list_river(from_list, &self.kw("FROM"), width, &mut lines);
            }

            // WHERE clause.
            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_river(where_c, width, &mut lines);
            }

            // RETURNING clause.
            if let Some(ret) = node.find_child("returning_clause")
                && let Some(text) = self.returning_text(ret)
            {
                lines.push(self.river_line(&self.kw("RETURNING"), &text, width));
            }
        } else {
            if let Some(with) = self.dml_with_clause(node, 0) {
                lines.push(with);
            }
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

            // FROM clause.
            if let Some(from_c) = node.find_child("from_clause")
                && let Some(from_list) = from_c.find_child("from_list")
            {
                self.format_relation_list_left_aligned(from_list, &self.kw("FROM"), &mut lines);
            }

            // WHERE clause.
            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_left_aligned(where_c, &mut lines);
            }

            // RETURNING clause.
            if let Some(ret) = node.find_child("returning_clause")
                && let Some(text) = self.returning_text(ret)
            {
                lines.push(format!("{} {text}", self.kw("RETURNING")));
            }
        }

        lines.join("\n")
    }

    fn format_set_clause(&self, node: Node<'a>) -> String {
        let value = node
            .find_child_any(&["a_expr", "c_expr"])
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        // Multi-column form: SET (a, b, c) = (...). The columns live in a
        // set_target_list, not a set_target, so looking only for the latter
        // emitted `SET  = ...` and dropped every column name.
        if let Some(list) = node.find_child("set_target_list") {
            let items = flatten_list(list, "set_target_list");
            let cols: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
            return format!("({}) = {value}", cols.join(", "));
        }
        let target = node
            .find_child("set_target")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();
        format!("{target} = {value}")
    }

    /// Render a `from_list` (the relations of an UPDATE ... FROM or a
    /// DELETE ... USING) river-aligned under the given keyword. The first
    /// relation shares the keyword line; the rest are indented to the content
    /// column.
    fn format_relation_list_river(
        &self,
        from_list: Node<'a>,
        keyword: &str,
        width: usize,
        lines: &mut Vec<String>,
    ) {
        let tables = flatten_list(from_list, "from_list");
        let last = tables.len().saturating_sub(1);
        for (i, table) in tables.iter().enumerate() {
            let mut text = self.format_table_ref(*table);
            if i < last {
                text.push(',');
            }
            if i == 0 {
                lines.push(self.river_line(keyword, &text, width));
            } else {
                let padding = " ".repeat(width + 1);
                lines.push(format!("{padding}{text}"));
            }
        }
    }

    /// Left-aligned counterpart of [`Self::format_relation_list_river`].
    fn format_relation_list_left_aligned(
        &self,
        from_list: Node<'a>,
        keyword: &str,
        lines: &mut Vec<String>,
    ) {
        let indent = self.config.indent;
        let tables = flatten_list(from_list, "from_list");
        if tables.len() == 1 {
            lines.push(format!("{keyword} {}", self.format_table_ref(tables[0])));
            return;
        }
        lines.push(keyword.to_string());
        let last = tables.len().saturating_sub(1);
        for (i, table) in tables.iter().enumerate() {
            let mut text = self.format_table_ref(*table);
            if i < last {
                text.push(',');
            }
            lines.push(format!("{indent}{text}"));
        }
    }

    // ── MERGE ───────────────────────────────────────────────────────────

    /// Format `[WITH ...] MERGE INTO <target> USING <source> ON <cond>
    /// <WHEN clauses> [RETURNING ...]`.
    ///
    /// Each WHEN clause takes its own line with the action indented beneath
    /// it, since an action is itself a whole UPDATE/INSERT/DELETE and reads
    /// poorly trailing a long condition.
    pub(crate) fn format_merge_stmt(&self, node: Node<'a>) -> String {
        let target = node
            .find_child("relation_expr_opt_alias")
            .map(|n| self.format_relation_expr_opt_alias(n))
            .unwrap_or_default();
        let source = node
            .find_child("table_ref")
            .map(|n| self.format_table_ref(n))
            .unwrap_or_default();
        let whens: Vec<Node<'a>> = node
            .find_child("merge_when_list")
            .map(|l| flatten_list(l, "merge_when_list"))
            .unwrap_or_default()
            .into_iter()
            .filter(|n| n.kind() == "merge_when_clause")
            .collect();

        let mut lines = Vec::new();

        if self.config.river {
            let merge_kw = self.kw_pair("MERGE", "INTO");
            let mut keywords = vec![merge_kw.clone(), self.kw("USING"), self.kw("ON")];
            if !whens.is_empty() {
                keywords.push(self.kw("WHEN"));
            }
            if node.has_child("returning_clause") {
                keywords.push(self.kw("RETURNING"));
            }
            let width = keywords.iter().map(|k| k.len()).max().unwrap_or(0);
            let content_col = width + 1;

            if let Some(with) = self.dml_with_clause(node, width) {
                lines.push(with);
            }
            lines.push(self.river_line(&merge_kw, &target, width));
            lines.push(self.river_line(&self.kw("USING"), &source, width));
            if let Some(cond) = node.find_child("a_expr") {
                let conditions = self.split_top_level_conditions(cond);
                lines.push(self.river_line(&self.kw("ON"), &conditions[0].1, width));
                for (op, text) in &conditions[1..] {
                    lines.push(self.river_line(op, text, width));
                }
            }
            let pad = " ".repeat(content_col);
            for when in &whens {
                let (head, action) = self.format_merge_when_clause(*when);
                lines.push(self.river_line(&self.kw("WHEN"), &head, width));
                for line in action {
                    lines.push(format!("{pad}{line}"));
                }
            }
            if let Some(ret) = node.find_child("returning_clause")
                && let Some(text) = self.returning_text(ret)
            {
                lines.push(self.river_line(&self.kw("RETURNING"), &text, width));
            }
        } else {
            let indent = self.config.indent;
            if let Some(with) = self.dml_with_clause(node, 0) {
                lines.push(with);
            }
            lines.push(format!("{} {target}", self.kw_pair("MERGE", "INTO")));
            lines.push(format!("{} {source}", self.kw("USING")));
            if let Some(cond) = node.find_child("a_expr") {
                let conditions = self.split_top_level_conditions(cond);
                lines.push(format!("{} {}", self.kw("ON"), conditions[0].1));
                for (op, text) in &conditions[1..] {
                    lines.push(format!("{op} {text}"));
                }
            }
            for when in &whens {
                let (head, action) = self.format_merge_when_clause(*when);
                lines.push(format!("{} {head}", self.kw("WHEN")));
                for line in action {
                    lines.push(format!("{indent}{line}"));
                }
            }
            if let Some(ret) = node.find_child("returning_clause")
                && let Some(text) = self.returning_text(ret)
            {
                lines.push(format!("{} {text}", self.kw("RETURNING")));
            }
        }

        lines.join("\n")
    }

    /// Split a `merge_when_clause` into its `WHEN` head (everything up to and
    /// including THEN, with the leading WHEN removed) and the action lines.
    fn format_merge_when_clause(&self, node: Node<'a>) -> (String, Vec<String>) {
        let mut head = Vec::new();
        if let Some(tgt) =
            node.find_child_any(&["merge_when_tgt_matched", "merge_when_tgt_not_matched"])
        {
            // NOT / MATCHED / BY SOURCE|TARGET, minus the leading WHEN that
            // the caller emits as the river keyword.
            let mut cursor = tgt.walk();
            for child in tgt.named_children(&mut cursor) {
                if child.kind() != "kw_when" {
                    head.push(self.kw(self.text(child)));
                }
            }
        }
        if let Some(cond) = node.find_child("opt_merge_when_condition") {
            head.push(self.render_clause_inline(cond));
        }
        head.push(self.kw("THEN"));

        let mut action = Vec::new();
        if let Some(upd) = node.find_child("merge_update") {
            let sets = upd
                .find_child("set_clause_list")
                .map(|l| flatten_list(l, "set_clause_list"))
                .unwrap_or_default();
            let formatted: Vec<_> = sets.iter().map(|c| self.format_set_clause(*c)).collect();
            action.push(format!(
                "{} {} {}",
                self.kw("UPDATE"),
                self.kw("SET"),
                formatted.join(", ")
            ));
        } else if node.has_child("merge_delete") {
            action.push(self.kw("DELETE"));
        } else if let Some(ins) = node.find_child("merge_insert") {
            action.extend(self.format_merge_insert(ins));
        } else if node.has_child("kw_do") {
            action.push(self.kw_pair("DO", "NOTHING"));
        }
        (head.join(" "), action)
    }

    /// Render a `merge_insert` action: the INSERT header (optional column
    /// list and OVERRIDING clause) and its VALUES on a following line.
    fn format_merge_insert(&self, node: Node<'a>) -> Vec<String> {
        let mut header = self.kw("INSERT");
        if let Some(cols) = node.find_child("insert_column_list") {
            let items = flatten_list(cols, "insert_column_list");
            let formatted: Vec<_> = items.iter().map(|c| self.format_expr(*c)).collect();
            header = format!("{header} ({})", formatted.join(", "));
        }
        if node.has_child("kw_overriding") {
            let kind = node
                .find_child("override_kind")
                .map(|ok| {
                    if ok.has_child("kw_system") {
                        self.kw("SYSTEM")
                    } else {
                        self.kw("USER")
                    }
                })
                .unwrap_or_default();
            header = format!(
                "{header} {} {kind} {}",
                self.kw("OVERRIDING"),
                self.kw("VALUE")
            );
        }
        if node.has_child("kw_default") {
            return vec![format!("{header} {}", self.kw_pair("DEFAULT", "VALUES"))];
        }
        let bare = header == self.kw("INSERT");
        let mut out = vec![header];
        if let Some(values) = node.find_child("merge_values_clause")
            && let Some(list) = values.find_child("expr_list")
        {
            let items = flatten_list(list, "expr_list");
            let formatted: Vec<_> = items.iter().map(|e| self.format_expr(*e)).collect();
            let values = format!("{} ({})", self.kw("VALUES"), formatted.join(", "));
            // `INSERT VALUES (...)` fits one line; a column list or OVERRIDING
            // clause makes the header long enough to want its own.
            if bare {
                out[0] = format!("{} {values}", out[0]);
            } else {
                out.push(values);
            }
        }
        out
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
            if node.has_child("using_clause") {
                keywords.push(self.kw("USING"));
            }
            if node.has_child("where_or_current_clause") {
                keywords.push(self.kw("WHERE"));
            }
            if node.has_child("returning_clause") {
                keywords.push(self.kw("RETURNING"));
            }
            let width = keywords.iter().map(|k| k.len()).max().unwrap_or(6);

            if let Some(with) = self.dml_with_clause(node, width) {
                lines.push(with);
            }
            lines.push(delete_kw);
            lines.push(self.river_line(&self.kw("FROM"), &table, width));

            // USING clause.
            if let Some(using_c) = node.find_child("using_clause")
                && let Some(from_list) = using_c.find_child("from_list")
            {
                self.format_relation_list_river(from_list, &self.kw("USING"), width, &mut lines);
            }

            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_river(where_c, width, &mut lines);
            }

            // RETURNING clause.
            if let Some(ret) = node.find_child("returning_clause")
                && let Some(text) = self.returning_text(ret)
            {
                lines.push(self.river_line(&self.kw("RETURNING"), &text, width));
            }
        } else {
            if let Some(with) = self.dml_with_clause(node, 0) {
                lines.push(with);
            }
            lines.push(format!("{} {} {table}", self.kw("DELETE"), self.kw("FROM")));

            // USING clause.
            if let Some(using_c) = node.find_child("using_clause")
                && let Some(from_list) = using_c.find_child("from_list")
            {
                self.format_relation_list_left_aligned(from_list, &self.kw("USING"), &mut lines);
            }

            if let Some(where_c) = node.find_child("where_or_current_clause") {
                self.format_where_left_aligned(where_c, &mut lines);
            }

            // RETURNING clause.
            if let Some(ret) = node.find_child("returning_clause")
                && let Some(text) = self.returning_text(ret)
            {
                lines.push(format!("{} {text}", self.kw("RETURNING")));
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

        let if_not_exists = if node.has_child("kw_if") {
            format!(
                "{} {} {} ",
                self.kw("IF"),
                self.kw("NOT"),
                self.kw("EXISTS")
            )
        } else {
            String::new()
        };

        // TEMP / TEMPORARY / UNLOGGED, and the GLOBAL/LOCAL variants.
        let temp = node
            .find_child("OptTemp")
            .map(|n| format!("{} ", self.kw(normalize_whitespace(self.text(n)).as_str())))
            .unwrap_or_default();

        let mut header = format!(
            "{} {temp}{} {if_not_exists}{table_name}",
            self.kw("CREATE"),
            self.kw("TABLE")
        );

        // `PARTITION OF parent` and `OF type` name a second relation or type
        // between the table name and the element list.
        let parent = node
            .named_children_vec()
            .into_iter()
            .filter(|c| c.kind() == "qualified_name")
            .nth(1);
        if let Some(parent) = parent.filter(|_| node.has_child("kw_partition")) {
            header.push_str(&format!(
                " {} {}",
                self.kw_pair("PARTITION", "OF"),
                self.format_qualified_name(parent)
            ));
        } else if let Some(of_type) = node
            .find_child("any_name")
            .filter(|_| node.has_child("kw_of"))
        {
            header.push_str(&format!(
                " {} {}",
                self.kw("OF"),
                self.render_clause_inline(of_type)
            ));
        }

        // A typed or partition table may omit the element list entirely.
        let elem_list = node
            .find_child("OptTableElementList")
            .and_then(|n| n.find_child("TableElementList"))
            .or_else(|| {
                node.find_child("OptTypedTableElementList")
                    .and_then(|n| n.find_child("TypedTableElementList"))
            });

        let mut lines = Vec::new();
        if elem_list.is_none() {
            // An empty `()` is still required before INHERITS or SERVER.
            let mut cursor = node.walk();
            if node.children(&mut cursor).any(|c| c.kind() == "(") {
                header.push_str(" ()");
            }
            lines.push(header);
        } else {
            lines.push(format!("{header} ("));
        }

        // Column definitions and constraints.
        if let Some(elem_list) = elem_list {
            let indent = self.config.indent;

            // Comments interspersed with the columns/constraints are associated
            // with the element they trail so they render as end-of-line
            // comments instead of bogus columns.
            let (leading_comments, grouped) = self.collect_table_elements(node, elem_list);
            for c in &leading_comments {
                lines.push(format!("{indent}{c}"));
            }

            if self.config.river {
                // River style: PRIMARY KEY first, padded columns, constraint
                // on separate indented line.
                let mut pk_elements: Vec<(String, Vec<String>)> = Vec::new();
                let mut col_elements: Vec<(String, String, String, Vec<String>)> = Vec::new();
                let mut constraint_elements: Vec<(Option<String>, String, Vec<String>)> =
                    Vec::new();

                for (elem_node, comments) in &grouped {
                    match self.classify_table_element(*elem_node) {
                        TableElementKind::PrimaryKey(text) => {
                            pk_elements.push((text, comments.clone()));
                        }
                        TableElementKind::Column(name, typename, constraints) => {
                            col_elements.push((name, typename, constraints, comments.clone()));
                        }
                        TableElementKind::Constraint(name, body) => {
                            constraint_elements.push((name, body, comments.clone()));
                        }
                    }
                }

                // Calculate max column name and type widths for alignment.
                let max_name_len = col_elements
                    .iter()
                    .map(|(n, ..)| n.len())
                    .max()
                    .unwrap_or(0);
                let max_type_len = col_elements
                    .iter()
                    .map(|(_, t, ..)| t.len())
                    .max()
                    .unwrap_or(0);

                // Build ordered list of (rendered item, trailing comments):
                // PKs first, then columns, then constraints.
                let mut all_items: Vec<(String, Vec<String>)> = Vec::new();
                for (pk, comments) in &pk_elements {
                    all_items.push((pk.clone(), comments.clone()));
                }
                for (name, typename, constraints, comments) in &col_elements {
                    all_items.push((
                        render_aligned_column(
                            name,
                            typename,
                            constraints,
                            max_name_len,
                            max_type_len,
                        ),
                        comments.clone(),
                    ));
                }
                // Table constraints: CONSTRAINT name on one line,
                // CHECK(...) on the next, both aligned with the type column.
                for (name, body, comments) in &constraint_elements {
                    let constraint_padding = " ".repeat(max_name_len + 1);
                    if let Some(cname) = name {
                        all_items.push((
                            format!(
                                "{constraint_padding}{} {cname}\n{constraint_padding}{body}",
                                self.kw("CONSTRAINT")
                            ),
                            comments.clone(),
                        ));
                    } else {
                        all_items.push((format!("{constraint_padding}{body}"), comments.clone()));
                    }
                }

                // Render each item to its physical line(s), with the trailing
                // comma on the last line (constraints span two lines).
                let mut rendered: Vec<(Vec<String>, Vec<String>)> = Vec::new();
                let total = all_items.len();
                for (i, (item, comments)) in all_items.iter().enumerate() {
                    let comma = if i < total - 1 { "," } else { "" };
                    let mut phys: Vec<String> = Vec::new();
                    if item.contains('\n') {
                        let item_lines: Vec<&str> = item.lines().collect();
                        for (j, line) in item_lines.iter().enumerate() {
                            if j == item_lines.len() - 1 {
                                phys.push(format!("{indent}{line}{comma}"));
                            } else {
                                phys.push(format!("{indent}{line}"));
                            }
                        }
                    } else {
                        phys.push(format!("{indent}{item}{comma}"));
                    }
                    rendered.push((phys, comments.clone()));
                }

                // Align trailing comments to a common column so they line up
                // with the rest of the river-style layout.
                let comment_col = rendered
                    .iter()
                    .filter(|(_, c)| !c.is_empty())
                    .filter_map(|(phys, _)| phys.last().map(|l| l.len()))
                    .max()
                    .unwrap_or(0);
                for (phys, comments) in &rendered {
                    let last = phys.len() - 1;
                    for (j, line) in phys.iter().enumerate() {
                        if j == last && !comments.is_empty() {
                            let pad = " ".repeat(comment_col.saturating_sub(line.len()));
                            lines.push(format!("{line}{pad} {}", comments[0]));
                            for extra in &comments[1..] {
                                lines.push(format!("{indent}{extra}"));
                            }
                        } else {
                            lines.push(line.clone());
                        }
                    }
                }
            } else {
                lines.extend(self.render_grouped_elements_left_aligned(&grouped, indent));
            }
        }

        if elem_list.is_some() {
            lines.push(")".to_string());
        }

        // FOR VALUES ... / DEFAULT, the bound of a PARTITION OF table.
        if let Some(bound) = node.find_child("PartitionBoundSpec") {
            lines.push(self.render_clause_inline(bound));
        }

        // INHERITS (parent, ...).
        if let Some(inh) = node.find_child("OptInherit")
            && let Some(list) = inh.find_child("qualified_name_list")
        {
            let items = flatten_list(list, "qualified_name_list");
            let formatted: Vec<_> = items
                .iter()
                .map(|q| self.format_qualified_name(*q))
                .collect();
            lines.push(format!(
                "{} ({})",
                self.kw("INHERITS"),
                formatted.join(", ")
            ));
        }

        // PARTITION BY { RANGE | LIST | HASH } (...).
        if let Some(spec) = node
            .find_child("OptPartitionSpec")
            .and_then(|n| n.find_child("PartitionSpec"))
        {
            let method = spec
                .find_child("ColId")
                .map(|n| self.kw(self.text(n)))
                .unwrap_or_default();
            let cols = spec
                .find_child("part_params")
                .map(|pp| {
                    // A part_elem's parentheses are literal tokens, and
                    // PostgreSQL requires them around any key that is not a
                    // column or a function call, so render the element as
                    // written rather than as an expression.
                    let items = flatten_list(pp, "part_params");
                    let formatted: Vec<_> = items
                        .iter()
                        .map(|i| self.render_clause_inline(*i))
                        .collect();
                    formatted.join(", ")
                })
                .unwrap_or_default();
            lines.push(format!(
                "{} {method} ({cols})",
                self.kw_pair("PARTITION", "BY")
            ));
        }

        // USING <access method>.
        if let Some(am) = node.find_child("table_access_method_clause") {
            lines.push(self.render_clause_inline(am));
        }

        // WITH clause for storage parameters.
        // OptWith already contains the WITH keyword, so just normalize.
        if let Some(with) = node.find_child("OptWith") {
            let text = normalize_whitespace(self.text(with));
            if !text.is_empty() {
                lines.push(text);
            }
        }

        // ON COMMIT { DROP | DELETE ROWS | PRESERVE ROWS }.
        if let Some(oc) = node.find_child("OnCommitOption") {
            lines.push(self.render_clause_inline(oc));
        }

        // TABLESPACE name.
        if let Some(ts) = node.find_child("OptTableSpace")
            && let Some(name) = ts.find_child("name")
        {
            lines.push(format!(
                "{} {}",
                self.kw("TABLESPACE"),
                self.format_expr(name)
            ));
        }

        lines.join("\n")
    }

    /// Split a flattened `TableElementList` into real elements, each paired
    /// with the trailing `-- ...` comments that follow it, plus any comments
    /// that precede the first element. Comments parse as sibling nodes inside
    /// the list (after an element's comma); associating each with the element
    /// it trails lets us re-emit it as an end-of-line comment rather than
    /// treating it as a bogus column.
    fn group_table_elements(
        &self,
        elements: &[Node<'a>],
    ) -> (Vec<String>, Vec<(Node<'a>, Vec<String>)>) {
        let mut leading: Vec<String> = Vec::new();
        let mut grouped: Vec<(Node<'a>, Vec<String>)> = Vec::new();
        for &e in elements {
            if e.kind() == "comment" {
                let text = self.text(e).trim_end().to_string();
                match grouped.last_mut() {
                    Some((_, comments)) => comments.push(text),
                    None => leading.push(text),
                }
            } else {
                grouped.push((e, Vec::new()));
            }
        }
        (leading, grouped)
    }

    /// Collect the comments belonging to a table element list, shared by
    /// CREATE TABLE and CREATE FOREIGN TABLE. Returns the leading comments
    /// (rendered before the first element, in source order) and each element
    /// paired with the trailing `-- ...` comments that follow it.
    ///
    /// Comments interspersed within the list parse as siblings of the
    /// elements; a comment after the last element parses as a direct child of
    /// the statement node, between the list and the closing paren. Comments
    /// after the closing paren are put back by `Formatter::restore_comments`.
    fn collect_table_elements(
        &self,
        node: Node<'a>,
        elem_list: Node<'a>,
    ) -> (Vec<String>, Vec<(Node<'a>, Vec<String>)>) {
        let raw = flatten_list_keeping_comments(elem_list, elem_list.kind());
        let (group_leading, mut grouped) = self.group_table_elements(&raw);
        let list_start = elem_list.start_byte();
        let close_paren = {
            let mut paren_cursor = node.walk();
            node.children(&mut paren_cursor)
                .find(|c| c.kind() == ")")
                .map(|c| c.start_byte())
                .unwrap_or_else(|| node.end_byte())
        };
        // Comments that precede the list are kept ahead of comments inside the
        // list so leading comments render in source order.
        let mut before_list: Vec<String> = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() != "comment" {
                continue;
            }
            let text = self.text(child).trim_end().to_string();
            if child.start_byte() < list_start {
                before_list.push(text);
            } else if child.start_byte() < close_paren {
                match grouped.last_mut() {
                    Some((_, comments)) => comments.push(text),
                    None => before_list.push(text),
                }
            }
            // Comments after the closing paren are put back later.
        }
        let mut leading = before_list;
        leading.extend(group_leading);
        (leading, grouped)
    }

    /// Render grouped table elements in left-aligned (non-river) style,
    /// appending trailing commas and attached comments. Shared by
    /// `format_create_table_stmt` and `format_create_foreign_table_stmt`.
    fn render_grouped_elements_left_aligned(
        &self,
        grouped: &[(Node<'a>, Vec<String>)],
        indent: &str,
    ) -> Vec<String> {
        let mut lines = Vec::new();
        let total = grouped.len();
        for (i, (elem_node, comments)) in grouped.iter().enumerate() {
            let elem = self.format_table_element(*elem_node);
            let comma = if i < total - 1 { "," } else { "" };
            let line = format!("{indent}{elem}{comma}");
            if let Some((first, rest)) = comments.split_first() {
                lines.push(format!("{line} {first}"));
                for extra in rest {
                    lines.push(format!("{indent}{extra}"));
                }
            } else {
                lines.push(line);
            }
        }
        lines
    }

    /// Classify a table element for river-style CREATE TABLE formatting.
    fn classify_table_element(&self, node: Node<'a>) -> TableElementKind {
        // Matches the element's inner child rather than the wrapper kind, so
        // TypedTableElement (CREATE TABLE ... OF type) classifies the same way
        // as TableElement instead of falling through to an untyped column,
        // which the river layout then padded with a trailing space.
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
            if let Some(opts) = col.find_child("create_generic_options") {
                constraint_parts.push(self.format_col_generic_options_inline(opts));
            }
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
        if let Some(like) = node.find_child("TableLikeClause") {
            return TableElementKind::Constraint(None, self.render_clause_inline(like));
        }
        if let Some(opts) = node.find_child("columnOptions") {
            return TableElementKind::Constraint(None, self.render_clause_inline(opts));
        }
        TableElementKind::Constraint(None, normalize_whitespace(self.text(node)))
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
        // Matches the inner child rather than the wrapper kind, so
        // TypedTableElement (CREATE TABLE ... OF type) is formatted like
        // TableElement instead of falling back to raw source text with its
        // keyword casing and whitespace unnormalized.
        if let Some(col) = node.find_child("columnDef") {
            return self.format_column_def(col);
        }
        if let Some(constraint) = node.find_child("TableConstraint") {
            return self.format_table_constraint(constraint);
        }
        if let Some(like) = node.find_child("TableLikeClause") {
            return self.render_clause_inline(like);
        }
        if let Some(opts) = node.find_child("columnOptions") {
            return self.render_clause_inline(opts);
        }
        normalize_whitespace(self.text(node))
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

        if let Some(opts) = node.find_child("create_generic_options") {
            parts.push(self.format_col_generic_options_inline(opts));
        }

        if let Some(qual_list) = node.find_child("ColQualList") {
            let constraints = flatten_list(qual_list, "ColQualList");
            for child in constraints {
                if child.kind() == "ColConstraint" {
                    parts.push(self.format_col_constraint(child));
                }
            }
        }

        parts.retain(|p| !p.is_empty());
        parts.join(" ")
    }

    fn format_col_constraint(&self, node: Node<'a>) -> String {
        // ColConstraint is one of: CONSTRAINT <name> <elem>, a bare <elem>,
        // a ConstraintAttr (DEFERRABLE, INITIALLY DEFERRED/IMMEDIATE,
        // [NOT] ENFORCED), or COLLATE <name>. Only the first two carry a
        // ColConstraintElem; the rest are flat keyword clauses.
        let Some(elem) = node.find_child("ColConstraintElem") else {
            return self.render_clause_inline(node);
        };
        let mut parts = Vec::new();
        if node.has_child("kw_constraint")
            && let Some(name) = node.find_child("name")
        {
            parts.push(self.kw("CONSTRAINT"));
            parts.push(self.format_expr(name));
        }
        parts.push(self.format_col_constraint_elem(elem));
        parts.retain(|p| !p.is_empty());
        parts.join(" ")
    }

    fn format_col_constraint_elem(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut has_check = false;
        // GENERATED ... AS (expr) also requires its parentheses; only the
        // IDENTITY form has no expression.
        let generated_expr = node.has_child("kw_generated") && !node.has_child("kw_identity");
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "kw_not" => parts.push(self.kw("NOT")),
                "kw_null" => parts.push(self.kw("NULL")),
                "kw_primary" => parts.push(self.kw("PRIMARY")),
                "kw_key" => parts.push(self.kw("KEY")),
                "kw_unique" => parts.push(self.kw("UNIQUE")),
                "kw_default" => parts.push(self.kw("DEFAULT")),
                "kw_check" => {
                    has_check = true;
                    parts.push(self.kw("CHECK"));
                }
                "kw_references" => parts.push(self.kw("REFERENCES")),
                "a_expr" | "c_expr" | "b_expr" => {
                    // A column-level CHECK constraint requires its expression to
                    // be parenthesized; DEFAULT and others do not. Only add a
                    // pair when the expression is not already a single
                    // parenthesized group, so `CHECK ((x))` collapses to
                    // `CHECK (x)` rather than doubling up.
                    let expr = self.format_expr(child);
                    if (has_check || generated_expr) && !is_wrapped_in_parens(&expr) {
                        parts.push(format!("({expr})"));
                    } else {
                        parts.push(expr);
                    }
                }
                _ if child.kind().starts_with("kw_") => {
                    parts.push(self.kw(self.text(child)));
                }
                // opt_column_list, key_match, key_actions, opt_no_inherit,
                // opt_unique_null_treatment, opt_virtual_or_stored,
                // OptParenthesizedSeqOptList, opt_definition, OptConsTableSpace.
                // These are flat keyword/punctuation clauses; the old
                // format_expr fallback returned garbage or nothing.
                _ => parts.push(self.render_clause_inline(child)),
            }
        }
        parts.retain(|p| !p.is_empty());
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
        parts.retain(|p| !p.is_empty());
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
                        let mut formatted: Vec<_> =
                            items.iter().map(|i| self.format_expr(*i)).collect();
                        // WITHOUT OVERLAPS qualifies the last key column and
                        // sits inside the parens, not after them.
                        if let Some(wo) = node.find_child("opt_without_overlaps")
                            && let Some(last) = formatted.last_mut()
                        {
                            *last = format!("{last} {}", self.render_clause_inline(wo));
                        }
                        let mut rendered = formatted.join(", ");
                        // FOREIGN KEY (cols, PERIOD name) — optionalPeriodName
                        // carries its own leading comma and sits inside the
                        // parens.
                        if let Some(period) = node.find_child("optionalPeriodName") {
                            rendered.push_str(&self.render_clause_inline(period));
                        }
                        parts.push(format!("({rendered})"));
                    }
                    "opt_without_overlaps" | "optionalPeriodName" => {}
                    "ExclusionConstraintList" => {
                        parts.push(format!("({})", self.render_clause_inline(child)));
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
                    // access_method_clause, ExclusionConstraintList,
                    // OptWhereClause, opt_without_overlaps, opt_c_include,
                    // opt_definition, OptConsTableSpace, key_actions,
                    // ConstraintAttributeSpec, optionalPeriodName. Flat
                    // keyword/punctuation clauses that the old format_expr
                    // fallback dropped or mangled.
                    _ => parts.push(self.render_clause_inline(child)),
                }
            }
            // Unnamed parentheses are re-emitted by the columnList, EXCLUDE
            // list, and CHECK expression arms above.
        }
        parts.retain(|p| !p.is_empty());
        parts.join(" ")
    }

    // ── CREATE VIEW ─────────────────────────────────────────────────────

    fn format_view_stmt(&self, node: Node<'a>) -> String {
        let mut head = vec![self.kw("CREATE")];

        // OR REPLACE.
        if node.has_child("kw_or") && node.has_child("kw_replace") {
            head.push(self.kw("OR"));
            head.push(self.kw("REPLACE"));
        }

        // TEMP / TEMPORARY (and GLOBAL/LOCAL variants).
        if let Some(temp) = node.find_child("OptTemp") {
            head.push(self.kw(normalize_whitespace(self.text(temp)).as_str()));
        }

        if node.has_child("kw_recursive") {
            head.push(self.kw("RECURSIVE"));
        }

        head.push(self.kw("VIEW"));
        let mut prefix = head.join(" ");

        // View name.
        let name = node
            .find_child("qualified_name")
            .or_else(|| node.find_child("view_name"))
            .map(|n| self.format_qualified_name(n))
            .unwrap_or_default();
        prefix = format!("{prefix} {name}");

        // Column names, required for a RECURSIVE view.
        // A plain view nests the list under opt_column_list.
        if let Some(columns) = node.find_child("columnList").or_else(|| {
            node.find_child("opt_column_list")
                .and_then(|n| n.find_child("columnList"))
        }) {
            prefix = format!("{prefix} ({})", self.render_clause_inline(columns));
        }

        // WITH (security_barrier, check_option = ...). security_barrier is
        // what keeps a view's WHERE from being bypassed by a leaky function,
        // so dropping it is a security change -- see
        // https://github.com/gmr/libpgfmt/issues/58.
        if let Some(options) = node.find_child("opt_reloptions") {
            let options = self.render_clause_inline(options);
            if !options.is_empty() {
                prefix = format!("{prefix} {options}");
            }
        }
        prefix = format!("{prefix} {}", self.kw("AS"));

        // WITH [CASCADED | LOCAL] CHECK OPTION, which makes writes through
        // the view obey its WHERE clause.
        let check_option = node
            .find_child("opt_check_option")
            .map(|n| self.render_clause_inline(n))
            .filter(|t| !t.is_empty())
            .map(|t| format!("\n{t}"))
            .unwrap_or_default();

        // The SELECT body.
        if let Some(select) = node.find_child("SelectStmt") {
            let body = self.format_select_stmt(select);
            format!("{prefix}\n{}{check_option}", body.trim_end_matches(';'))
        } else {
            prefix
        }
    }

    // ── CREATE TABLE AS / CREATE MATERIALIZED VIEW ──────────────────────

    fn format_create_table_as_stmt(&self, node: Node<'a>) -> String {
        // Everything before AS, in order: CREATE [UNLOGGED] MATERIALIZED VIEW
        // [IF NOT EXISTS] name [(cols)] [USING am] [WITH (...)] [TABLESPACE
        // ts]. Picking these out one by one dropped every one not picked --
        // see https://github.com/gmr/libpgfmt/issues/58.
        let mut prefix_parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "kw_as" {
                break;
            }
            let piece = self.render_clause_inline(child);
            if !piece.is_empty() {
                prefix_parts.push(piece);
            }
        }
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

        // WITH [NO] DATA.
        let suffix = node
            .find_child("opt_with_data")
            .map(|n| self.render_clause_inline(n))
            .filter(|t| !t.is_empty())
            .map(|t| format!("\n{t}"))
            .unwrap_or_default();

        format!("{prefix}\n{body}{suffix}")
    }

    // ── CREATE FUNCTION ─────────────────────────────────────────────────

    fn format_create_function_stmt(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();

        // CREATE FUNCTION/PROCEDURE name(args)
        let mut header = vec![self.kw("CREATE")];
        if node.has_child("opt_or_replace") {
            header.push(self.kw("OR"));
            header.push(self.kw("REPLACE"));
        }
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

        // RETURNS type, or the RETURNS TABLE (...) production, which carries
        // a table_func_column_list instead of a func_return.
        if let Some(ret) = node.find_child("func_return") {
            let ret_type = self.format_func_return(ret);
            header.push(format!("{} {ret_type}", self.kw("RETURNS")));
        } else if let Some(cols) = node.find_child("table_func_column_list") {
            let items = flatten_list(cols, "table_func_column_list");
            let formatted: Vec<_> = items
                .iter()
                .map(|i| self.render_clause_inline(*i))
                .collect();
            header.push(format!(
                "{} {} ({})",
                self.kw("RETURNS"),
                self.kw("TABLE"),
                formatted.join(", ")
            ));
        }

        parts.push(header.join(" "));

        // Function options (LANGUAGE, AS, etc.).
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "opt_createfunc_opt_list" | "createfunc_opt_list" => {
                    self.format_createfunc_opts(child, &mut parts);
                }
                "opt_routine_body" => {
                    parts.push(self.format_routine_body(child));
                }
                _ => {}
            }
        }

        parts.join(&format!("\n{}", self.config.indent))
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

    /// The LANGUAGE of the function an option item belongs to, lower-cased
    /// and unquoted.
    fn function_language(&self, item: Node<'a>) -> Option<String> {
        let mut stmt = item;
        while !matches!(stmt.kind(), "CreateFunctionStmt") {
            stmt = stmt.parent()?;
        }
        let mut stack = vec![stmt];
        while let Some(node) = stack.pop() {
            if node.kind() == "createfunc_opt_item"
                && node.has_child("kw_language")
                && let Some(lang) = node.find_child("NonReservedWord_or_Sconst")
            {
                return Some(self.text(lang).trim_matches('\'').to_lowercase());
            }
            stack.extend(node.named_children_vec());
        }
        None
    }

    fn format_createfunc_opt_item(&self, node: Node<'a>, parts: &mut Vec<String>) {
        if node.has_child("kw_language") {
            if let Some(lang) = node.find_child("NonReservedWord_or_Sconst") {
                parts.push(format!("{} {}", self.kw("LANGUAGE"), self.text(lang)));
            }
            return;
        }
        if let Some(child) = node.find_child("func_as") {
            // AS $$ ... $$
            // Preserve original line breaks in the body — collapsing
            // newlines to spaces would break line-comment (--) semantics.
            // For single-line bodies, normalize whitespace (safe since
            // there are no newlines that could affect -- comments).
            // For multi-line bodies, re-indent each line to preserve
            // the original structure.
            let text = self.text(child).trim();
            if let Some((tag, body)) = parse_dollar_quoted(text) {
                // The body is a string constant. Re-laying it out is safe
                // only in a language where whitespace outside strings means
                // nothing, and only when no string in it spans a line;
                // anything else -- PL/Perl, PL/Python, a multi-line literal
                // in PL/pgSQL -- is emitted exactly as written. See
                // https://github.com/gmr/libpgfmt/issues/57.
                let relayout = self
                    .function_language(node)
                    .is_some_and(|l| l == "sql" || l == "plpgsql")
                    && lexical::layout_is_safe(body);
                if !relayout {
                    parts.push(format!("{} {text}", self.kw("AS")));
                } else if body.contains('\n') {
                    let body = reindent_body(body, " ");
                    parts.push(format!("{} {tag}\n{body}\n{tag}", self.kw("AS")));
                } else {
                    let body = normalize_whitespace(body);
                    parts.push(format!("{} {tag}\n {body}\n{tag}", self.kw("AS")));
                }
            } else {
                parts.push(format!("{} {text}", self.kw("AS")));
            }
            return;
        }
        // WINDOW, and everything reachable through common_func_opt_item:
        // STRICT, IMMUTABLE/STABLE/VOLATILE, [EXTERNAL] SECURITY
        // DEFINER/INVOKER, LEAKPROOF, CALLED/RETURNS NULL ON NULL INPUT, COST,
        // ROWS, SUPPORT, PARALLEL, and SET/RESET. These are flat keyword
        // clauses with no layout decision, so render them inline.
        let text = self.render_clause_inline(node);
        if !text.is_empty() {
            parts.push(text);
        }
    }

    /// Format a SQL-standard routine body: `RETURN expr`, or a `BEGIN ATOMIC
    /// ... END` block with one statement per line.
    ///
    /// Only the first line is left unindented — the caller's option join
    /// supplies that one. Every later line carries its own indent, so blank
    /// lines (dbt separates clauses with one) stay genuinely blank instead of
    /// becoming a line of spaces.
    fn format_routine_body(&self, node: Node<'a>) -> String {
        if let Some(ret) = node.find_child("ReturnStmt") {
            return self.render_clause_inline(ret);
        }
        let indent = self.config.indent;
        let mut lines = vec![format!("{} {}", self.kw("BEGIN"), self.kw("ATOMIC"))];
        if let Some(list) = node.find_child("routine_body_stmt_list") {
            for stmt in flatten_list(list, "routine_body_stmt_list") {
                if stmt.kind() != "routine_body_stmt" {
                    continue;
                }
                let Some(mut inner) = stmt.named_children_vec().into_iter().next() else {
                    continue;
                };
                // routine_body_stmt wraps a `stmt`, which wraps the statement.
                if inner.kind() == "stmt"
                    && let Some(unwrapped) = inner.named_children_vec().into_iter().next()
                {
                    inner = unwrapped;
                }
                let body = match inner.kind() {
                    "SelectStmt" => self.format_select_stmt(inner),
                    "InsertStmt" => self.format_insert_stmt(inner),
                    "UpdateStmt" => self.format_update_stmt(inner),
                    "DeleteStmt" => self.format_delete_stmt(inner),
                    _ => normalize_whitespace(self.text(inner)),
                };
                let body = body.trim_end_matches(';');
                for line in body.lines() {
                    if line.is_empty() {
                        lines.push(String::new());
                    } else {
                        lines.push(format!("{indent}{indent}{line}"));
                    }
                }
                if let Some(last) = lines.last_mut() {
                    last.push(';');
                }
            }
        }
        lines.push(format!("{indent}{}", self.kw("END")));
        lines.join("\n")
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
        let if_not_exists = if node.has_child("kw_if") {
            format!(
                "{} {} {} ",
                self.kw("IF"),
                self.kw("NOT"),
                self.kw("EXISTS")
            )
        } else {
            String::new()
        };

        let mut header = format!(
            "{} {} {} {if_not_exists}{table_name}",
            self.kw("CREATE"),
            self.kw("FOREIGN"),
            self.kw("TABLE")
        );

        // `PARTITION OF parent ... FOR VALUES ...`, as in CREATE TABLE.
        // Dropping it made the table a plain foreign table with an empty
        // column list -- see https://github.com/gmr/libpgfmt/issues/58.
        let parent = node
            .named_children_vec()
            .into_iter()
            .filter(|c| c.kind() == "qualified_name")
            .nth(1);
        if let Some(parent) = parent.filter(|_| node.has_child("kw_partition")) {
            header.push_str(&format!(
                " {} {}",
                self.kw_pair("PARTITION", "OF"),
                self.format_qualified_name(parent)
            ));
        }

        // A partition may omit the element list entirely.
        let elem_list = node
            .find_child("OptTableElementList")
            .and_then(|n| n.find_child("TableElementList"))
            .or_else(|| {
                node.find_child("OptTypedTableElementList")
                    .and_then(|n| n.find_child("TypedTableElementList"))
            });

        let mut lines = Vec::new();
        if elem_list.is_none() {
            // An empty `()` is still required before INHERITS or SERVER.
            let mut cursor = node.walk();
            if node.children(&mut cursor).any(|c| c.kind() == "(") {
                header.push_str(" ()");
            }
            lines.push(header);
        } else {
            lines.push(format!("{header} ("));
        }

        // Column definitions (same as CREATE TABLE).
        if let Some(elem_list) = elem_list {
            let indent = self.config.indent;

            // Inline comments trail the element they follow (see
            // collect_table_elements); render them as end-of-line comments
            // rather than treating them as bogus columns.
            let (leading_comments, grouped) = self.collect_table_elements(node, elem_list);
            for c in &leading_comments {
                lines.push(format!("{indent}{c}"));
            }

            if self.config.river {
                // Classify all elements, keeping their original order.
                let classified: Vec<_> = grouped
                    .iter()
                    .map(|(e, comments)| (self.classify_table_element(*e), comments.clone()))
                    .collect();

                // Compute column-alignment widths from Column elements only.
                let max_name_len = classified
                    .iter()
                    .filter_map(|(e, _)| {
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
                    .filter_map(|(e, _)| {
                        if let TableElementKind::Column(_, t, _) = e {
                            Some(t.len())
                        } else {
                            None
                        }
                    })
                    .max()
                    .unwrap_or(0);

                // Render each element to a single line (foreign-table
                // constraints render inline), then align trailing comments.
                let total = classified.len();
                let mut rendered: Vec<(String, Vec<String>)> = Vec::new();
                for (i, (elem, comments)) in classified.iter().enumerate() {
                    let comma = if i < total - 1 { "," } else { "" };
                    let text = match elem {
                        TableElementKind::Column(name, typename, constraints) => {
                            render_aligned_column(
                                name,
                                typename,
                                constraints,
                                max_name_len,
                                max_type_len,
                            )
                        }
                        TableElementKind::Constraint(Some(name), body) => {
                            format!("{} {name} {body}", self.kw("CONSTRAINT"))
                        }
                        TableElementKind::PrimaryKey(text)
                        | TableElementKind::Constraint(None, text) => text.clone(),
                    };
                    rendered.push((format!("{indent}{text}{comma}"), comments.clone()));
                }

                let comment_col = rendered
                    .iter()
                    .filter(|(_, c)| !c.is_empty())
                    .map(|(line, _)| line.len())
                    .max()
                    .unwrap_or(0);
                for (line, comments) in &rendered {
                    if let Some((first, rest)) = comments.split_first() {
                        let pad = " ".repeat(comment_col.saturating_sub(line.len()));
                        lines.push(format!("{line}{pad} {first}"));
                        for extra in rest {
                            lines.push(format!("{indent}{extra}"));
                        }
                    } else {
                        lines.push(line.clone());
                    }
                }
            } else {
                lines.extend(self.render_grouped_elements_left_aligned(&grouped, indent));
            }
        }

        if elem_list.is_some() {
            lines.push(")".to_string());
        }

        // FOR VALUES ... / DEFAULT, the bound of a PARTITION OF table.
        if let Some(bound) = node.find_child("PartitionBoundSpec") {
            lines.push(self.render_clause_inline(bound));
        }

        // INHERITS (parent, ...).
        if let Some(inh) = node.find_child("OptInherit") {
            let text = self.render_clause_inline(inh);
            if !text.is_empty() {
                lines.push(text);
            }
        }

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

    fn collect_generic_options(&self, node: Node<'a>) -> Vec<String> {
        let Some(opt_list) = node.find_child("generic_option_list") else {
            return Vec::new();
        };
        flatten_list(opt_list, "generic_option_list")
            .iter()
            .map(|item| self.format_generic_option(*item))
            .collect()
    }

    fn format_generic_options(&self, node: Node<'a>, lines: &mut Vec<String>) {
        let items = self.collect_generic_options(node);
        if items.is_empty() {
            return;
        }
        let indent = self.config.indent;
        lines.push(format!("{} (", self.kw("OPTIONS")));
        let last = items.len() - 1;
        for (i, item) in items.iter().enumerate() {
            let comma = if i < last { "," } else { "" };
            lines.push(format!("{indent}{item}{comma}"));
        }
        lines.push(")".to_string());
    }

    fn format_col_generic_options_inline(&self, node: Node<'a>) -> String {
        let items = self.collect_generic_options(node);
        format!("{} ({})", self.kw("OPTIONS"), items.join(", "))
    }

    fn format_generic_option(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            parts.push(self.format_expr(child));
        }
        parts.retain(|p| !p.is_empty());
        parts.join(" ")
    }

    // ── Helpers ─────────────────────────────────────────────────────────

    /// Render the leading `WITH ...` clause of a DML statement, if it has one.
    ///
    /// `opt_with_clause` wraps the `with_clause` that the shared helpers
    /// expect; passing the outer node yields nothing, which is how INSERT,
    /// UPDATE and DELETE came to discard their CTEs entirely.
    fn dml_with_clause(&self, node: Node<'a>, river_width: usize) -> Option<String> {
        let with = node
            .find_child("opt_with_clause")?
            .find_child("with_clause")?;
        Some(if self.config.river {
            self.format_with_clause_river_inner(with, river_width, true)
        } else {
            self.format_with_clause_left(with)
        })
    }

    /// Render a clause subtree inline: case keyword leaves, delegate real
    /// expressions to `format_expr`, and glue punctuation. Used as the
    /// fallback for the keyword-and-punctuation clauses hanging off
    /// constraints and function options, where there is no layout decision to
    /// make and the old per-kind match silently dropped anything it did not
    /// name.
    pub(crate) fn render_clause_inline(&self, node: Node<'a>) -> String {
        let kind = node.kind();
        if kind.starts_with("kw_") {
            return self.kw(self.text(node));
        }
        match kind {
            "a_expr" | "b_expr" | "c_expr" | "func_expr_windowless" => {
                return self.format_expr(node);
            }
            "Typename" => return self.format_typename(node),
            "qualified_name" => return self.format_qualified_name(node),
            // An identifier keeps the spelling it was written with. Many are
            // unreserved keywords in the grammar -- a column named `data`
            // parses as `ColId > unreserved_keyword > kw_data` -- and
            // recursing would apply keyword casing to a name.
            "ColId" | "ColLabel" | "attr_name" | "name" => {
                return self.text(node).trim().to_string();
            }
            _ => {}
        }
        let mut cursor = node.walk();
        let children: Vec<Node> = node.children(&mut cursor).collect();
        if children.is_empty() {
            return self.text(node).trim().to_string();
        }
        let mut out = String::new();
        for child in children {
            let piece = self.render_clause_inline(child);
            if piece.is_empty() {
                continue;
            }
            if out.is_empty() {
                out.push_str(&piece);
                continue;
            }
            let glue_left = piece.starts_with([')', ',', ';']);
            let glue_right = out.ends_with('(');
            if glue_left || glue_right {
                out.push_str(&piece);
            } else {
                out.push(' ');
                out.push_str(&piece);
            }
        }
        out
    }

    fn format_relation_expr_opt_alias(&self, node: Node<'a>) -> String {
        let mut parts = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "relation_expr" => parts.push(self.format_relation_expr(child)),
                "opt_alias_clause" | "alias_clause" | "func_alias_clause" => {
                    // alias_clause already includes the AS keyword.
                    let alias = self.format_expr(child);
                    if !alias.is_empty() {
                        parts.push(alias);
                    }
                }
                "ColId" => {
                    let alias = self.format_expr(child);
                    if !alias.is_empty() {
                        // The `relation_expr AS ColId` form already carries
                        // kw_as as a sibling, which the catch-all arm emits.
                        // Only the bare `relation_expr ColId` form needs AS
                        // supplied, otherwise the output doubles it and, on a
                        // second pass, drops the alias entirely.
                        if node.has_child("kw_as") {
                            parts.push(alias);
                        } else {
                            parts.push(format!("{} {alias}", self.kw("AS")));
                        }
                    }
                }
                _ => parts.push(self.format_expr(child)),
            }
        }
        parts.retain(|p| !p.is_empty());
        parts.join(" ")
    }

    /// The table an INSERT writes to, with its alias. ON CONFLICT and
    /// RETURNING refer to the table by the alias once one is given, so
    /// dropping it leaves them naming a table that no longer exists -- see
    /// https://github.com/gmr/libpgfmt/issues/58.
    fn format_insert_target(&self, node: Node<'a>) -> String {
        let Some(qn) = node.find_child("qualified_name") else {
            return self.format_expr(node);
        };
        let name = self.format_expr(qn);
        match node.find_child("ColId") {
            Some(alias) => format!("{name} {} {}", self.kw("AS"), self.format_expr(alias)),
            None => name,
        }
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

/// Render one river-aligned column: the name padded to `max_name_len`, then the
/// type padded to `max_type_len` only when `constraints` follow it — a column
/// with no constraints is left unpadded so it emits no trailing whitespace.
fn render_aligned_column(
    name: &str,
    typename: &str,
    constraints: &str,
    max_name_len: usize,
    max_type_len: usize,
) -> String {
    let padded_name = format!("{:width$}", name, width = max_name_len);
    if constraints.is_empty() {
        format!("{padded_name} {typename}")
    } else {
        let padded_type = format!("{:width$}", typename, width = max_type_len);
        format!("{padded_name} {padded_type} {constraints}")
    }
}

/// Returns true when `s` is a single expression already enclosed in one outer
/// pair of parentheses, e.g. `(a AND b)` — but not `(a) AND (b)`, where the
/// first `(` closes before the end. Parentheses inside string literals are
/// ignored.
fn is_wrapped_in_parens(s: &str) -> bool {
    let s = s.trim();
    let bytes = s.as_bytes();
    if bytes.first() != Some(&b'(') || bytes.last() != Some(&b')') {
        return false;
    }
    let mut depth = 0usize;
    let mut in_str = false;
    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'\'' => in_str = !in_str,
            b'(' if !in_str => depth += 1,
            b')' if !in_str => {
                depth -= 1;
                if depth == 0 && i != bytes.len() - 1 {
                    return false;
                }
            }
            _ => {}
        }
    }
    depth == 0
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

/// Collapse whitespace, keeping string constants, quoted identifiers and
/// comments verbatim, and keeping the newlines that carry meaning: the one
/// after a `--` comment, which would otherwise swallow the rest of the
/// statement, and the one between two string constants, which PostgreSQL
/// requires to join them. See [`lexical::collapse_whitespace`].
pub(crate) fn normalize_whitespace(s: &str) -> String {
    lexical::collapse_whitespace(s)
}
