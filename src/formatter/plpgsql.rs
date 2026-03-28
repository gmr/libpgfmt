/// PL/pgSQL formatting.
use crate::node_helpers::NodeExt;
use tree_sitter::Node;

use super::Formatter;

impl<'a> Formatter<'a> {
    /// Format a PL/pgSQL block.
    pub(crate) fn format_plpgsql_block(&self, node: Node<'a>, indent_level: usize) -> String {
        let indent = "  ".repeat(indent_level);
        let mut lines = Vec::new();

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "decl_sect" => {
                    lines.push(format!("{indent}{}", self.kw("DECLARE")));
                    self.format_decl_sect(child, indent_level + 1, &mut lines);
                }
                "kw_begin" => {
                    lines.push(format!("{indent}{}", self.kw("BEGIN")));
                }
                "proc_sect" => {
                    self.format_proc_sect(child, indent_level + 1, &mut lines);
                }
                "exception_sect" => {
                    lines.push(format!("{indent}{}", self.kw("EXCEPTION")));
                    self.format_exception_sect(child, indent_level + 1, &mut lines);
                }
                "kw_end" => {
                    lines.push(format!("{indent}{}", self.kw("END")));
                }
                _ => {}
            }
        }

        lines.join("\n")
    }

    fn format_decl_sect(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "decl_stmt" {
                self.format_decl_stmt(child, indent_level, lines);
            }
        }
    }

    fn format_decl_stmt(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        if let Some(decl) = node.find_child("decl_statement") {
            let var_name = decl
                .find_child("decl_varname")
                .map(|n| self.text(n).trim().to_string())
                .unwrap_or_default();

            let mut parts = vec![var_name];

            // Constant?
            if decl.has_child("kw_constant") {
                parts.push(self.kw("CONSTANT"));
            }

            // Data type.
            if let Some(dt) = decl.find_child("decl_datatype") {
                let type_text = self.text(dt).trim().to_string();
                parts.push(type_text);
            }

            // Collation.
            if let Some(coll) = decl.find_child("decl_collate") {
                let coll_text = self.text(coll).trim().to_string();
                parts.push(coll_text);
            }

            // NOT NULL.
            if decl.has_child("kw_not") {
                parts.push(format!("{} {}", self.kw("NOT"), self.kw("NULL")));
            }

            // Default value.
            if let Some(defval) = decl.find_child("decl_defval") {
                let def_text = self.text(defval).trim().to_string();
                parts.push(def_text);
            }

            lines.push(format!("{indent}{};", parts.join(" ")));
        }
    }

    fn format_proc_sect(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "proc_stmt" {
                self.format_proc_stmt(child, indent_level, lines);
            }
        }
    }

    fn format_proc_stmt(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "stmt_if" => self.format_stmt_if(child, indent_level, lines),
                "stmt_loop" | "stmt_while" => self.format_stmt_loop(child, indent_level, lines),
                "stmt_for" => self.format_stmt_for(child, indent_level, lines),
                "stmt_foreach_a" => self.format_stmt_foreach(child, indent_level, lines),
                "stmt_case" => self.format_stmt_case(child, indent_level, lines),
                "stmt_return" => {
                    let expr = child
                        .find_child("sql_expression")
                        .map(|n| self.text(n).trim())
                        .unwrap_or("");
                    if expr.is_empty() {
                        lines.push(format!("{indent}{};", self.kw("RETURN")));
                    } else {
                        lines.push(format!("{indent}{} {expr};", self.kw("RETURN")));
                    }
                }
                "stmt_raise" => self.format_stmt_raise(child, indent_level, lines),
                "stmt_null" => {
                    lines.push(format!("{indent}{};", self.kw("NULL")));
                }
                // Statements whose source text is passed through with indentation.
                "stmt_assign" | "stmt_execsql" | "stmt_perform" | "stmt_call"
                | "stmt_dynexecute" | "stmt_exit" | "stmt_continue" | "stmt_open"
                | "stmt_close" | "stmt_fetch" | "stmt_move" | "stmt_commit" | "stmt_rollback"
                | "stmt_assert" => {
                    let text = self.text(child).trim();
                    lines.push(format!("{indent}{text}"));
                }
                "pl_block" => {
                    let block_text = self.format_plpgsql_block(child, indent_level);
                    lines.push(block_text);
                }
                _ => {
                    let text = self.text(child).trim();
                    if !text.is_empty() {
                        lines.push(format!("{indent}{text}"));
                    }
                }
            }
        }
    }

    fn format_stmt_if(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let inner_indent_level = indent_level + 1;

        let mut cursor = node.walk();
        let children: Vec<_> = node.named_children(&mut cursor).collect();

        let mut i = 0;
        while i < children.len() {
            let child = children[i];
            match child.kind() {
                "kw_if" => {
                    if i == 0 {
                        // Opening IF.
                        let cond = children
                            .get(i + 1)
                            .filter(|c| c.kind() == "sql_expression")
                            .map(|c| self.text(*c).trim())
                            .unwrap_or("");
                        lines.push(format!(
                            "{indent}{} {cond} {}",
                            self.kw("IF"),
                            self.kw("THEN")
                        ));
                        i += 3; // skip cond and THEN
                    }
                    // Closing IF (END IF).
                }
                "sql_expression" => {
                    i += 1; // handled with IF/ELSIF
                }
                "kw_then" => {
                    i += 1; // handled with IF/ELSIF
                }
                "proc_sect" => {
                    self.format_proc_sect(child, inner_indent_level, lines);
                    i += 1;
                }
                "elsif_clause" => {
                    self.format_elsif_clause(child, indent_level, lines);
                    i += 1;
                }
                "else_clause" => {
                    lines.push(format!("{indent}{}", self.kw("ELSE")));
                    if let Some(proc) = child.find_child("proc_sect") {
                        self.format_proc_sect(proc, inner_indent_level, lines);
                    }
                    i += 1;
                }
                "kw_end" => {
                    i += 1; // END IF handled below
                }
                _ => {
                    i += 1;
                }
            }
        }
        lines.push(format!("{indent}{} {};", self.kw("END"), self.kw("IF")));
    }

    fn format_elsif_clause(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let inner_indent_level = indent_level + 1;

        let cond = node
            .find_child("sql_expression")
            .map(|n| self.text(n).trim())
            .unwrap_or("");
        lines.push(format!(
            "{indent}{} {cond} {}",
            self.kw("ELSIF"),
            self.kw("THEN")
        ));

        if let Some(proc) = node.find_child("proc_sect") {
            self.format_proc_sect(proc, inner_indent_level, lines);
        }
    }

    fn format_stmt_loop(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let inner_indent_level = indent_level + 1;

        // WHILE condition or just LOOP.
        if node.kind() == "stmt_while" {
            let cond = node
                .find_child("sql_expression")
                .map(|n| self.text(n).trim())
                .unwrap_or("");
            lines.push(format!(
                "{indent}{} {cond} {}",
                self.kw("WHILE"),
                self.kw("LOOP")
            ));
        } else {
            lines.push(format!("{indent}{}", self.kw("LOOP")));
        }

        if let Some(body) = node.find_child("loop_body")
            && let Some(proc) = body.find_child("proc_sect")
        {
            self.format_proc_sect(proc, inner_indent_level, lines);
        }

        lines.push(format!("{indent}{} {};", self.kw("END"), self.kw("LOOP")));
    }

    fn format_stmt_for(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let inner_indent_level = indent_level + 1;

        let var = node
            .find_child("for_variable")
            .map(|n| self.text(n).trim())
            .unwrap_or("");

        // Determine if it's a FOR ... IN range or FOR ... IN query.
        let in_clause = if let Some(range) = node.find_child("for_integer_range") {
            self.text(range).trim().to_string()
        } else if let Some(query) = node.find_child("for_control") {
            self.text(query).trim().to_string()
        } else {
            // Fallback: reconstruct from source.
            let text = self.text(node);
            if let Some(start) = text.find("IN") {
                if let Some(end) = text.find("LOOP") {
                    text[start + 2..end].trim().to_string()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        };

        let for_kw = self.kw("FOR");
        let in_kw = self.kw("IN");
        let loop_kw = self.kw("LOOP");
        lines.push(format!(
            "{indent}{for_kw} {var} {in_kw} {in_clause} {loop_kw}"
        ));

        if let Some(body) = node.find_child("loop_body")
            && let Some(proc) = body.find_child("proc_sect")
        {
            self.format_proc_sect(proc, inner_indent_level, lines);
        }

        lines.push(format!("{indent}{} {};", self.kw("END"), self.kw("LOOP")));
    }

    fn format_stmt_foreach(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        // FOREACH ... SLICE ... IN ARRAY ... LOOP ... END LOOP
        let text = self.text(node);
        lines.push(format!("{indent}{text}"));
    }

    fn format_stmt_case(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let inner_indent_level = indent_level + 1;
        let inner_indent = "  ".repeat(inner_indent_level);

        let expr = node
            .find_child("sql_expression")
            .map(|n| self.text(n).trim())
            .unwrap_or("");
        lines.push(format!("{indent}{} {expr}", self.kw("CASE")));

        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "case_when" {
                let when_expr = child
                    .find_child("sql_expression")
                    .map(|n| self.text(n).trim())
                    .unwrap_or("");
                lines.push(format!(
                    "{inner_indent}{} {when_expr} {}",
                    self.kw("WHEN"),
                    self.kw("THEN")
                ));
                if let Some(proc) = child.find_child("proc_sect") {
                    self.format_proc_sect(proc, inner_indent_level + 1, lines);
                }
            } else if child.kind() == "else_clause" {
                lines.push(format!("{inner_indent}{}", self.kw("ELSE")));
                if let Some(proc) = child.find_child("proc_sect") {
                    self.format_proc_sect(proc, inner_indent_level + 1, lines);
                }
            }
        }

        lines.push(format!("{indent}{} {};", self.kw("END"), self.kw("CASE")));
    }

    fn format_stmt_raise(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let mut parts = vec![self.kw("RAISE")];

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() {
                match child.kind() {
                    "kw_raise" => {} // already handled
                    "raise_level" => {
                        let level = self.text(child).trim();
                        parts.push(self.kw(level));
                    }
                    "string_literal" => {
                        parts.push(self.text(child).to_string());
                    }
                    "sql_expression" => {
                        parts.push(self.text(child).trim().to_string());
                    }
                    _ => {}
                }
            } else {
                let text = self.text(child).trim();
                if text == "," {
                    // Append comma to the previous part instead of adding a separate token.
                    if let Some(last) = parts.last_mut() {
                        last.push(',');
                    }
                }
            }
        }

        lines.push(format!("{indent}{};", parts.join(" ")));
    }

    fn format_exception_sect(&self, node: Node<'a>, indent_level: usize, lines: &mut Vec<String>) {
        let indent = "  ".repeat(indent_level);
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "proc_conditions" => {
                    let cond_text = self.text(child).trim();
                    lines.push(format!(
                        "{indent}{} {cond_text} {}",
                        self.kw("WHEN"),
                        self.kw("THEN")
                    ));
                }
                "proc_sect" => {
                    self.format_proc_sect(child, indent_level + 1, lines);
                }
                _ => {}
            }
        }
    }
}
