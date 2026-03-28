/// SELECT statement formatting — the most complex formatter.
use crate::node_helpers::{NodeExt, flatten_list};
use crate::style::Style;
use tree_sitter::Node;

use super::Formatter;

/// Collected clauses from a SELECT statement.
pub(crate) struct SelectClauses<'a> {
    pub distinct: Option<Node<'a>>,
    pub targets: Vec<Node<'a>>,
    pub from: Option<Node<'a>>,
    pub where_clause: Option<Node<'a>>,
    pub group_clause: Option<Node<'a>>,
    pub having_clause: Option<Node<'a>>,
    pub sort_clause: Option<Node<'a>>,
    pub limit_clause: Option<Node<'a>>,
    pub offset_clause: Option<Node<'a>>,
    pub with_clause: Option<Node<'a>>,
    /// For UNION / INTERSECT / EXCEPT.
    pub set_op: Option<SetOp<'a>>,
    /// VALUES clause (for INSERT ... VALUES).
    pub values_clause: Option<Node<'a>>,
}

pub(crate) struct SetOp<'a> {
    pub keyword: String,
    pub quantifier: Option<String>,
    pub right: Node<'a>,
    /// Pre-collected clauses for the right side (ERROR recovery).
    pub right_clauses: Option<Box<SelectClauses<'a>>>,
}

impl<'a> Formatter<'a> {
    /// Format a SelectStmt node.
    pub(crate) fn format_select_stmt(&self, node: Node<'a>) -> String {
        let snp = node.find_child("select_no_parens").unwrap_or(node);
        self.format_select_no_parens(snp)
    }

    /// Format a select_no_parens node.
    pub(crate) fn format_select_no_parens(&self, node: Node<'a>) -> String {
        let clauses = self.collect_select_clauses(node);
        if clauses.values_clause.is_some() {
            return self.format_values_only(&clauses);
        }
        if self.config.river {
            self.format_select_river(&clauses)
        } else {
            self.format_select_left_aligned(&clauses)
        }
    }

    /// Collect all clauses from a select_no_parens (or simple_select) node tree.
    fn collect_select_clauses(&self, node: Node<'a>) -> SelectClauses<'a> {
        let mut clauses = SelectClauses {
            distinct: None,
            targets: Vec::new(),
            from: None,
            where_clause: None,
            group_clause: None,
            having_clause: None,
            sort_clause: None,
            limit_clause: None,
            offset_clause: None,
            with_clause: None,
            set_op: None,
            values_clause: None,
        };
        self.collect_clauses_recursive(node, &mut clauses);
        clauses
    }

    fn collect_clauses_recursive(&self, node: Node<'a>, clauses: &mut SelectClauses<'a>) {
        let mut cursor = node.walk();
        let children: Vec<_> = node.named_children(&mut cursor).collect();
        let mut seen_set_op = false;
        for child in &children {
            // Once we've seen a set operation keyword, stop collecting into
            // the left-side clauses. The right side is handled separately.
            if seen_set_op {
                continue;
            }
            match child.kind() {
                "with_clause" => clauses.with_clause = Some(*child),
                "simple_select" => self.collect_clauses_recursive(*child, clauses),
                "select_clause" => self.collect_clauses_recursive(*child, clauses),
                "distinct_clause" => clauses.distinct = Some(*child),
                "target_list" => {
                    clauses.targets = flatten_list(*child, "target_list");
                }
                "opt_target_list" => {
                    if let Some(tl) = child.find_child("target_list") {
                        clauses.targets = flatten_list(tl, "target_list");
                    }
                }
                "from_clause" => clauses.from = Some(*child),
                "where_clause" => clauses.where_clause = Some(*child),
                "group_clause" => clauses.group_clause = Some(*child),
                "having_clause" => clauses.having_clause = Some(*child),
                "opt_sort_clause" => {
                    if let Some(sc) = child.find_child("sort_clause") {
                        clauses.sort_clause = Some(sc);
                    }
                }
                "sort_clause" => clauses.sort_clause = Some(*child),
                "select_limit" => {
                    self.collect_limit_clauses(*child, clauses);
                }
                "limit_clause" => {
                    self.collect_limit_clauses(*child, clauses);
                }
                "offset_clause" => clauses.offset_clause = Some(*child),
                "kw_union" | "kw_intersect" | "kw_except" => {
                    seen_set_op = true;
                    // Set operation — find the right side.
                    let keyword = self.kw(self.text(*child));
                    // Look for quantifier (ALL/DISTINCT) and right select_clause.
                    let mut quantifier = None;
                    let mut found_keyword = false;
                    let mut right_clause = None;
                    for sib in &children {
                        if sib.id() == child.id() {
                            found_keyword = true;
                            continue;
                        }
                        if found_keyword {
                            match sib.kind() {
                                "set_quantifier" => {
                                    quantifier = Some(self.format_set_quantifier(*sib));
                                }
                                "select_clause" | "simple_select" => {
                                    right_clause = Some(*sib);
                                    break;
                                }
                                _ => {}
                            }
                        }
                    }
                    if let Some(right) = right_clause {
                        clauses.set_op = Some(SetOp {
                            keyword,
                            quantifier,
                            right,
                            right_clauses: None,
                        });
                    } else {
                        // ERROR recovery: the right side tokens are loose children
                        // of the same node (not wrapped in select_clause). Collect
                        // the right-side clauses from children after the set op keyword.
                        let mut right_clauses = SelectClauses {
                            distinct: None,
                            targets: Vec::new(),
                            from: None,
                            where_clause: None,
                            group_clause: None,
                            having_clause: None,
                            sort_clause: None,
                            limit_clause: None,
                            offset_clause: None,
                            with_clause: None,
                            set_op: None,
                            values_clause: None,
                        };
                        for sib in &children {
                            if sib.id() == child.id() {
                                found_keyword = true;
                                continue;
                            }
                            if !found_keyword {
                                continue;
                            }
                            match sib.kind() {
                                "set_quantifier" => {} // already handled
                                "opt_target_list" => {
                                    if let Some(tl) = sib.find_child("target_list") {
                                        right_clauses.targets = flatten_list(tl, "target_list");
                                    }
                                }
                                "target_list" => {
                                    right_clauses.targets = flatten_list(*sib, "target_list");
                                }
                                "from_clause" => right_clauses.from = Some(*sib),
                                "where_clause" => right_clauses.where_clause = Some(*sib),
                                "group_clause" => right_clauses.group_clause = Some(*sib),
                                "having_clause" => right_clauses.having_clause = Some(*sib),
                                "sort_clause" => right_clauses.sort_clause = Some(*sib),
                                _ => {}
                            }
                        }
                        clauses.set_op = Some(SetOp {
                            keyword,
                            quantifier,
                            right_clauses: Some(Box::new(right_clauses)),
                            right: node, // unused in this case
                        });
                    }
                }
                "values_clause" => clauses.values_clause = Some(*child),
                _ => {}
            }
        }
    }

    fn collect_limit_clauses(&self, node: Node<'a>, clauses: &mut SelectClauses<'a>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "limit_clause" => {
                    clauses.limit_clause = Some(child);
                }
                "offset_clause" => {
                    clauses.offset_clause = Some(child);
                }
                "kw_limit" => {
                    // This limit_clause node itself is what we want.
                    clauses.limit_clause = Some(node);
                }
                "kw_offset" => {
                    clauses.offset_clause = Some(node);
                }
                _ => {}
            }
        }
    }

    fn format_set_quantifier(&self, node: Node<'a>) -> String {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "kw_all" {
                return self.kw("ALL");
            }
            if child.kind() == "kw_distinct" {
                return self.kw("DISTINCT");
            }
        }
        String::new()
    }

    fn format_values_only(&self, clauses: &SelectClauses<'a>) -> String {
        if let Some(vc) = clauses.values_clause {
            return self.format_values_clause(vc);
        }
        String::new()
    }

    // ── River-style SELECT ──────────────────────────────────────────────

    fn format_select_river(&self, clauses: &SelectClauses<'a>) -> String {
        let mut lines = Vec::new();

        // Calculate river width from all keywords that will appear.
        let keywords = self.collect_river_keywords(clauses);
        let river_width = keywords.iter().map(|k| k.len()).max().unwrap_or(6);

        // WITH clause.
        if let Some(with) = clauses.with_clause {
            lines.push(self.format_with_clause_river(with, river_width));
        }

        // SELECT [DISTINCT] targets.
        let select_kw = if clauses.distinct.is_some() {
            let distinct_text = clauses
                .distinct
                .map(|d| self.format_distinct(d))
                .unwrap_or_else(|| self.kw("DISTINCT"));
            format!("{} {}", self.kw("SELECT"), distinct_text)
        } else {
            self.kw("SELECT")
        };
        self.append_river_targets(&select_kw, &clauses.targets, river_width, &mut lines);

        // FROM clause with JOINs.
        if let Some(from) = clauses.from {
            self.format_from_river(from, river_width, &mut lines);
        }

        // WHERE clause.
        if let Some(where_c) = clauses.where_clause {
            self.format_where_river(where_c, river_width, &mut lines);
        }

        // GROUP BY clause.
        if let Some(group) = clauses.group_clause {
            self.format_group_by_river(group, river_width, &mut lines);
        }

        // HAVING clause.
        if let Some(having) = clauses.having_clause {
            self.format_having_river(having, river_width, &mut lines);
        }

        // ORDER BY clause.
        if let Some(sort) = clauses.sort_clause {
            self.format_order_by_river(sort, river_width, &mut lines);
        }

        // LIMIT / OFFSET.
        if let Some(limit) = clauses.limit_clause {
            self.format_limit_river(limit, river_width, &mut lines);
        }
        if let Some(offset) = clauses.offset_clause {
            self.format_offset_river(offset, river_width, &mut lines);
        }

        let mut result = lines.join("\n");

        // Set operations (UNION, INTERSECT, EXCEPT).
        if let Some(ref set_op) = clauses.set_op {
            let op_text = if let Some(ref q) = set_op.quantifier {
                format!("{} {q}", set_op.keyword)
            } else {
                set_op.keyword.clone()
            };
            result.push_str("\n\n");
            result.push_str(&op_text);
            result.push_str("\n\n");
            if let Some(ref rc) = set_op.right_clauses {
                result.push_str(&self.format_select_river(rc));
            } else {
                let right_clauses = self.collect_select_clauses(set_op.right);
                result.push_str(&self.format_select_river(&right_clauses));
            }
        }

        result
    }

    /// Collect all keywords that will appear in this SELECT for river width calculation.
    fn collect_river_keywords(&self, clauses: &SelectClauses<'a>) -> Vec<String> {
        let mut keywords = Vec::new();

        // For river width, always use just SELECT (not SELECT DISTINCT).
        // DISTINCT is part of the content, not the river keyword.
        keywords.push(self.kw("SELECT"));

        if clauses.from.is_some() {
            keywords.push(self.kw("FROM"));
            // Collect JOIN keywords if they participate in the river.
            if let Some(from) = clauses.from {
                self.collect_join_keywords_for_river(from, &mut keywords);
            }
        }
        if clauses.where_clause.is_some() {
            keywords.push(self.kw("WHERE"));
            // AND/OR keywords participate in river for conditions.
            if let Some(where_c) = clauses.where_clause {
                self.collect_condition_keywords(where_c, &mut keywords);
            }
        }
        if clauses.group_clause.is_some() {
            keywords.push(self.kw_pair("GROUP", "BY"));
        }
        if clauses.having_clause.is_some() {
            keywords.push(self.kw("HAVING"));
        }
        if clauses.sort_clause.is_some() {
            keywords.push(self.kw_pair("ORDER", "BY"));
        }
        if clauses.limit_clause.is_some() {
            keywords.push(self.kw("LIMIT"));
        }
        if clauses.offset_clause.is_some() {
            keywords.push(self.kw("OFFSET"));
        }
        keywords
    }

    fn collect_join_keywords_for_river(&self, from_node: Node<'a>, keywords: &mut Vec<String>) {
        if !self.config.joins_in_river {
            return;
        }
        if let Some(from_list) = from_node.find_child("from_list") {
            let tables = flatten_list(from_list, "from_list");
            for table in tables {
                if table.kind() == "table_ref" {
                    if let Some(jt) = table.find_child("joined_table") {
                        self.collect_join_keywords_inner(jt, keywords);
                    }
                } else {
                    self.collect_join_keywords_inner(table, keywords);
                }
            }
        }
    }

    fn collect_join_keywords_inner(&self, node: Node<'a>, keywords: &mut Vec<String>) {
        if node.kind() == "joined_table" {
            // Get the join keyword.
            let join_kw = self.get_join_keyword(node);
            keywords.push(join_kw);

            // Check for ON/USING.
            if let Some(qual) = node.find_child("join_qual") {
                if qual.has_child("kw_on") {
                    keywords.push(self.kw("ON"));
                    // Collect AND keywords from the ON condition.
                    if let Some(expr) = qual.find_child_any(&["a_expr", "c_expr"]) {
                        self.collect_condition_keywords_from_expr(expr, keywords);
                    }
                } else if qual.has_child("kw_using") {
                    keywords.push(self.kw("USING"));
                }
            }

            // Recurse into left side.
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                if child.kind() == "table_ref"
                    && let Some(jt) = child.find_child("joined_table")
                {
                    self.collect_join_keywords_inner(jt, keywords);
                }
            }
        }
    }

    fn collect_condition_keywords(&self, clause_node: Node<'a>, keywords: &mut Vec<String>) {
        if let Some(expr) = clause_node.find_child_any(&["a_expr", "c_expr"]) {
            self.collect_condition_keywords_from_expr(expr, keywords);
        }
    }

    fn collect_condition_keywords_from_expr(&self, node: Node<'a>, keywords: &mut Vec<String>) {
        if node.kind() == "a_expr" {
            let mut cursor = node.walk();
            for child in node.named_children(&mut cursor) {
                match child.kind() {
                    "kw_and" => keywords.push(self.kw("AND")),
                    "kw_or" => keywords.push(self.kw("OR")),
                    _ => {}
                }
            }
        }
    }

    /// Create a river-aligned line: right-align keyword in the given width.
    pub(crate) fn river_line(&self, keyword: &str, content: &str, width: usize) -> String {
        let padding = if keyword.len() < width {
            " ".repeat(width - keyword.len())
        } else {
            String::new()
        };
        let first_line = format!("{padding}{keyword} ");
        if content.contains('\n') {
            // Multi-line content (e.g. subqueries): indent continuation lines
            // to align with the content start column.
            let indent = " ".repeat(first_line.len());
            let mut lines = content.lines();
            let mut result = format!("{first_line}{}", lines.next().unwrap_or(""));
            for line in lines {
                result.push('\n');
                result.push_str(&indent);
                result.push_str(line);
            }
            result
        } else {
            format!("{first_line}{content}")
        }
    }

    /// Append target list items in river style.
    fn append_river_targets(
        &self,
        select_kw: &str,
        targets: &[Node<'a>],
        width: usize,
        lines: &mut Vec<String>,
    ) {
        if targets.is_empty() {
            lines.push(self.river_line(select_kw, "*", width));
            return;
        }

        let first = self.format_target_el(targets[0]);
        if targets.len() == 1 {
            lines.push(self.river_line(select_kw, &first, width));
            return;
        }

        // First item on the SELECT line.
        if self.config.leading_commas {
            lines.push(self.river_line(select_kw, &first, width));
            let content_col = width + 1; // where content starts
            for target in &targets[1..] {
                let formatted = self.format_target_el(*target);
                let padding = " ".repeat(content_col - 2);
                if formatted.contains('\n') {
                    let mut target_lines = formatted.lines();
                    let first_target_line = target_lines.next().unwrap_or("");
                    lines.push(format!("{padding}, {first_target_line}"));
                    let cont_padding = " ".repeat(content_col);
                    for line in target_lines {
                        lines.push(format!("{cont_padding}{line}"));
                    }
                } else {
                    lines.push(format!("{padding}, {formatted}"));
                }
            }
        } else {
            lines.push(self.river_line(select_kw, &format!("{first},"), width));
            let content_col = width + 1;
            for (i, target) in targets[1..].iter().enumerate() {
                let formatted = self.format_target_el(*target);
                let padding = " ".repeat(content_col);
                let suffix = if i < targets.len() - 2 { "," } else { "" };
                if formatted.contains('\n') {
                    // Multi-line target: indent continuation lines.
                    let mut target_lines = formatted.lines();
                    let first_target_line = target_lines.next().unwrap_or("");
                    lines.push(format!("{padding}{first_target_line}"));
                    for line in target_lines {
                        lines.push(format!("{padding}{line}"));
                    }
                    // Add suffix to last line.
                    if !suffix.is_empty()
                        && let Some(last) = lines.last_mut()
                    {
                        last.push_str(suffix);
                    }
                } else {
                    lines.push(format!("{padding}{formatted}{suffix}"));
                }
            }
        }
    }

    fn format_distinct(&self, node: Node<'a>) -> String {
        let kw = self.kw("DISTINCT");
        // Check for DISTINCT ON (expr_list).
        if let Some(list) = node.find_child("expr_list") {
            let items = flatten_list(list, "expr_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
            format!("{kw} ON ({})", formatted.join(", "))
        } else {
            kw
        }
    }

    fn format_from_river(&self, from_node: Node<'a>, width: usize, lines: &mut Vec<String>) {
        if let Some(from_list) = from_node.find_child("from_list") {
            let tables = flatten_list(from_list, "from_list");
            if tables.is_empty() {
                return;
            }
            for (i, table) in tables.iter().enumerate() {
                if table.kind() == "table_ref" && table.has_child("joined_table") {
                    // Table with JOINs.
                    let jt = table.find_child("joined_table").unwrap();
                    self.format_joined_table_river(jt, width, i == 0, lines);
                } else {
                    let text = self.format_table_ref(*table);
                    if i == 0 {
                        lines.push(self.river_line(&self.kw("FROM"), &text, width));
                    } else {
                        let content_col = width + 1;
                        let padding = " ".repeat(content_col);
                        lines.push(format!("{padding}{text}"));
                    }
                }
            }
        }
    }

    fn format_joined_table_river(
        &self,
        node: Node<'a>,
        width: usize,
        is_first_from: bool,
        lines: &mut Vec<String>,
    ) {
        // A joined_table has: left table_ref, join_type, JOIN, right table_ref, join_qual.
        // Recursion: left table_ref may itself contain a joined_table.
        let named = node.named_children_vec();

        // Find components.
        let mut left_table: Option<Node> = None;
        let mut right_table: Option<Node> = None;
        let mut join_type_node: Option<Node> = None;
        let mut join_qual_node: Option<Node> = None;
        let mut table_count = 0;

        for child in &named {
            match child.kind() {
                "table_ref" => {
                    if table_count == 0 {
                        left_table = Some(*child);
                    } else {
                        right_table = Some(*child);
                    }
                    table_count += 1;
                }
                "join_type" => join_type_node = Some(*child),
                "join_qual" => join_qual_node = Some(*child),
                _ => {}
            }
        }

        // Format left side first (may be recursive).
        if let Some(left) = left_table {
            if let Some(inner_jt) = left.find_child("joined_table") {
                self.format_joined_table_river(inner_jt, width, is_first_from, lines);
            } else {
                let text = self.format_table_ref(left);
                if is_first_from {
                    lines.push(self.river_line(&self.kw("FROM"), &text, width));
                } else {
                    let content_col = width + 1;
                    lines.push(format!("{}{text}", " ".repeat(content_col)));
                }
            }
        }

        // Format the JOIN keyword and right table.
        let join_kw = self.get_join_keyword(node);

        if let Some(right) = right_table {
            let right_text = self.format_table_ref(right);

            if self.config.joins_in_river {
                // JOINs participate in the river (AWeber/mattmc3 style).
                lines.push(self.river_line(&join_kw, &right_text, width));
            } else if join_type_node.is_some() {
                // Typed JOINs (INNER/LEFT/etc.) are indented under FROM content.
                let content_col = width + 1;
                let padding = " ".repeat(content_col);
                // Blank line between typed JOINs (not before the first one).
                // Detect prior JOIN by checking if any previous line contains a JOIN keyword.
                let has_prior_join = lines.iter().any(|l| {
                    let trimmed = l.trim();
                    trimmed.contains("JOIN ")
                });
                if has_prior_join {
                    lines.push(String::new());
                }
                lines.push(format!("{padding}{join_kw} {right_text}"));
            } else {
                // Plain JOIN is river-aligned (like FROM).
                lines.push(self.river_line(&join_kw, &right_text, width));
            }
        }

        // Format ON/USING.
        let is_typed_join = join_type_node.is_some();
        if let Some(qual) = join_qual_node {
            self.format_join_qual_river(qual, width, is_typed_join, lines);
        }
    }

    fn format_join_qual_river(
        &self,
        node: Node<'a>,
        width: usize,
        is_typed_join: bool,
        lines: &mut Vec<String>,
    ) {
        if let Some(on_expr) = node.find_child_any(&["a_expr", "c_expr"]) {
            if self.config.joins_in_river {
                // AWeber/mattmc3: ON participates in the river.
                let conditions = self.split_top_level_conditions(on_expr);
                if conditions.len() <= 1 {
                    lines.push(self.river_line(&self.kw("ON"), &conditions[0].1, width));
                } else {
                    lines.push(self.river_line(&self.kw("ON"), &conditions[0].1, width));
                    for (op, cond_text) in &conditions[1..] {
                        lines.push(self.river_line(op, cond_text, width));
                    }
                }
            } else if is_typed_join {
                // Typed JOINs (INNER/LEFT/etc.): ON indented under FROM content.
                let content_col = width + 1;
                let conditions = self.split_top_level_conditions(on_expr);
                let on_kw = self.kw("ON");
                let padding = " ".repeat(content_col);
                lines.push(format!("{padding}{on_kw} {}", conditions[0].1));
                if conditions.len() > 1 {
                    // AND aligns with the start of ON's condition text.
                    let and_indent = " ".repeat(content_col + on_kw.len() + 1);
                    for (op, cond_text) in &conditions[1..] {
                        lines.push(format!("{and_indent}{op} {cond_text}"));
                    }
                }
            } else {
                // Plain JOIN: ON is river-aligned.
                let conditions = self.split_top_level_conditions(on_expr);
                lines.push(self.river_line(&self.kw("ON"), &conditions[0].1, width));
                if conditions.len() > 1 {
                    for (op, cond_text) in &conditions[1..] {
                        lines.push(self.river_line(op, cond_text, width));
                    }
                }
            }
        } else if node.has_child("kw_using") {
            // USING clause.
            let using_text = self.format_using_clause(node);
            if self.config.joins_in_river {
                lines.push(self.river_line(&self.kw("USING"), &using_text, width));
            } else if is_typed_join {
                let content_col = width + 1;
                let padding = " ".repeat(content_col);
                lines.push(format!("{padding}{} {using_text}", self.kw("USING")));
            } else {
                lines.push(self.river_line(&self.kw("USING"), &using_text, width));
            }
        }
    }

    fn format_using_clause(&self, node: Node<'a>) -> String {
        if let Some(list) = node.find_child("name_list") {
            let items = flatten_list(list, "name_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
            format!("({})", formatted.join(", "))
        } else if let Some(list) = node.find_child("columnList") {
            let items = flatten_list(list, "columnList");
            let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
            format!("({})", formatted.join(", "))
        } else {
            // Fallback.
            let text = self.text(node);
            if let Some(start) = text.find('(') {
                text[start..].to_string()
            } else {
                text.to_string()
            }
        }
    }

    pub(crate) fn format_where_river(&self, node: Node<'a>, width: usize, lines: &mut Vec<String>) {
        self.format_condition_clause_river(node, "WHERE", width, lines);
    }

    fn format_having_river(&self, node: Node<'a>, width: usize, lines: &mut Vec<String>) {
        self.format_condition_clause_river(node, "HAVING", width, lines);
    }

    /// Shared river-style formatting for WHERE and HAVING clauses.
    fn format_condition_clause_river(
        &self,
        node: Node<'a>,
        keyword: &str,
        width: usize,
        lines: &mut Vec<String>,
    ) {
        if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
            let conditions = self.split_top_level_conditions(expr);
            lines.push(self.river_line(&self.kw(keyword), &conditions[0].1, width));
            for (op, cond_text) in &conditions[1..] {
                lines.push(self.river_line(op, cond_text, width));
            }
        }
    }

    fn format_group_by_river(&self, node: Node<'a>, width: usize, lines: &mut Vec<String>) {
        let kw = self.kw_pair("GROUP", "BY");
        if let Some(list) = node.find_child("group_by_list") {
            let items = flatten_list(list, "group_by_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
            lines.push(self.river_line(&kw, &formatted.join(", "), width));
        }
    }

    fn format_order_by_river(&self, node: Node<'a>, width: usize, lines: &mut Vec<String>) {
        let kw = self.kw_pair("ORDER", "BY");
        if let Some(list) = node.find_child("sortby_list") {
            let items = flatten_list(list, "sortby_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_sortby(*i)).collect();
            lines.push(self.river_line(&kw, &formatted.join(", "), width));
        }
    }

    fn format_limit_river(&self, node: Node<'a>, width: usize, lines: &mut Vec<String>) {
        let value = self.extract_limit_value(node);
        if !value.is_empty() {
            lines.push(self.river_line(&self.kw("LIMIT"), &value, width));
        }
    }

    fn format_offset_river(&self, node: Node<'a>, width: usize, lines: &mut Vec<String>) {
        let value = self.extract_offset_value(node);
        if !value.is_empty() {
            lines.push(self.river_line(&self.kw("OFFSET"), &value, width));
        }
    }

    fn extract_limit_value(&self, node: Node<'a>) -> String {
        if let Some(val) = node.find_child("select_limit_value") {
            return self.format_expr(val);
        }
        // Try finding the expression directly.
        if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
            return self.format_expr(expr);
        }
        String::new()
    }

    fn extract_offset_value(&self, node: Node<'a>) -> String {
        if let Some(val) = node.find_child("select_offset_value") {
            return self.format_expr(val);
        }
        if let Some(val) = node.find_child("select_fetch_first_value") {
            return self.format_expr(val);
        }
        if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
            return self.format_expr(expr);
        }
        String::new()
    }

    /// Split a WHERE/HAVING expression into individual conditions separated by AND/OR.
    ///
    /// Returns a vector of (operator, formatted_text) pairs.
    /// The first entry has operator "".
    ///
    /// The tree-sitter grammar can nest AND/OR deeper than expected due to
    /// operator precedence. This method formats the full expression first,
    /// then splits the resulting text on top-level AND/OR boundaries.
    pub(crate) fn split_top_level_conditions(&self, node: Node<'a>) -> Vec<(String, String)> {
        let full_text = self.format_expr(node);
        let and_kw = self.kw("AND");
        let or_kw = self.kw("OR");

        // Split on AND/OR that appear as whole words outside strings and parens.
        // Handles: single-quoted strings (''), double-quoted identifiers (""),
        // dollar-quoted strings ($$..$$, $tag$..$tag$), E'...' escape strings,
        // and BETWEEN...AND (the AND in BETWEEN is not a boolean operator).
        let between_kw = self.kw("BETWEEN");
        let mut conditions = Vec::new();
        let mut current = String::new();
        let mut current_op = String::new();
        let mut paren_depth: u32 = 0;
        let mut in_between = false;
        let bytes = full_text.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut buf = String::new();

        while i < len {
            let ch = bytes[i] as char;

            // Single-quoted string: 'text' with '' as escape.
            if ch == '\'' {
                buf.push(ch);
                i += 1;
                while i < len {
                    let c = bytes[i] as char;
                    buf.push(c);
                    i += 1;
                    if c == '\'' {
                        if i < len && bytes[i] == b'\'' {
                            buf.push('\'');
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
                continue;
            }

            // Double-quoted identifier: "name".
            if ch == '"' {
                buf.push(ch);
                i += 1;
                while i < len {
                    let c = bytes[i] as char;
                    buf.push(c);
                    i += 1;
                    if c == '"' {
                        if i < len && bytes[i] == b'"' {
                            buf.push('"');
                            i += 1;
                        } else {
                            break;
                        }
                    }
                }
                continue;
            }

            // Dollar-quoted string: $$...$$ or $tag$...$tag$.
            if ch == '$' {
                let tag_start = i;
                let mut tag_end = i + 1;
                while tag_end < len
                    && (bytes[tag_end].is_ascii_alphanumeric() || bytes[tag_end] == b'_')
                {
                    tag_end += 1;
                }
                if tag_end < len && bytes[tag_end] == b'$' {
                    let tag = &full_text[tag_start..=tag_end];
                    buf.push_str(tag);
                    i = tag_end + 1;
                    // Scan for closing tag.
                    while i < len {
                        if bytes[i] == b'$' && full_text[i..].starts_with(tag) {
                            buf.push_str(tag);
                            i += tag.len();
                            break;
                        }
                        buf.push(bytes[i] as char);
                        i += 1;
                    }
                    continue;
                }
                // Not a dollar-quote; fall through.
            }

            buf.push(ch);
            i += 1;

            if ch == '(' {
                paren_depth = paren_depth.saturating_add(1);
            } else if ch == ')' {
                paren_depth = paren_depth.saturating_sub(1);
            }
            if paren_depth > 0 {
                continue;
            }

            // Check for keyword at word boundary (next char is space or end).
            let next_is_boundary = i >= len || bytes[i] == b' ';
            if !next_is_boundary {
                continue;
            }

            // Detect BETWEEN keyword (to skip the AND in BETWEEN...AND).
            if buf.ends_with(&format!(" {between_kw}")) {
                in_between = true;
                continue;
            }

            let is_and = buf.ends_with(&format!(" {and_kw}"));
            let is_or = !is_and && buf.ends_with(&format!(" {or_kw}"));

            if is_and || is_or {
                // Skip the AND that belongs to BETWEEN...AND.
                if is_and && in_between {
                    in_between = false;
                    continue;
                }

                let kw = if is_and { &and_kw } else { &or_kw };
                let kw_with_space = kw.len() + 1; // " " + kw
                let cond_text = &buf[..buf.len() - kw_with_space];
                current.push_str(cond_text);
                conditions.push((current_op.clone(), current.trim().to_string()));
                current = String::new();
                current_op = kw.to_string();
                buf.clear();
                // Skip space after the keyword.
                if i < len && bytes[i] == b' ' {
                    i += 1;
                }
            }
        }
        // Push remaining text.
        current.push_str(&buf);
        let remaining = current.trim().to_string();
        if !remaining.is_empty() {
            conditions.push((current_op, remaining));
        }

        if conditions.len() <= 1 {
            vec![(String::new(), full_text)]
        } else {
            conditions
        }
    }

    /// Get the JOIN keyword for a joined_table node.
    pub(crate) fn get_join_keyword(&self, node: Node<'a>) -> String {
        let join_type = node.find_child("join_type");

        if let Some(jt) = join_type {
            let mut parts = Vec::new();
            let mut cursor = jt.walk();
            for child in jt.named_children(&mut cursor) {
                match child.kind() {
                    "kw_inner" => {
                        if !self.config.strip_inner_join {
                            parts.push(self.kw("INNER"));
                        }
                    }
                    "kw_left" => parts.push(self.kw("LEFT")),
                    "kw_right" => parts.push(self.kw("RIGHT")),
                    "kw_full" => parts.push(self.kw("FULL")),
                    "kw_cross" => parts.push(self.kw("CROSS")),
                    "kw_natural" => parts.push(self.kw("NATURAL")),
                    "kw_outer" => {} // implicit, skip
                    _ => parts.push(self.format_keyword_node(child)),
                }
            }
            parts.push(self.kw("JOIN"));
            parts.join(" ")
        } else {
            // Plain JOIN — decide whether to make it INNER JOIN or leave as JOIN.
            if self.config.explicit_inner_join {
                format!("{} {}", self.kw("INNER"), self.kw("JOIN"))
            } else {
                self.kw("JOIN")
            }
        }
    }

    // ── Left-aligned SELECT (Mozilla, dbt, GitLab, Kickstarter) ─────────

    fn format_select_left_aligned(&self, clauses: &SelectClauses<'a>) -> String {
        let mut lines = Vec::new();
        let indent = self.config.indent;
        let blank = self.config.blank_lines_between_clauses;

        // WITH clause.
        if let Some(with) = clauses.with_clause {
            lines.push(self.format_with_clause_left(with));
            // Blank line after CTE block before SELECT for all styles
            // except compact CTEs (Kickstarter).
            if !self.config.compact_ctes {
                lines.push(String::new());
            }
        }

        // SELECT [DISTINCT] targets.
        let select_kw = if clauses.distinct.is_some() {
            let distinct_text = clauses
                .distinct
                .map(|d| self.format_distinct(d))
                .unwrap_or_else(|| self.kw("DISTINCT"));
            format!("{} {}", self.kw("SELECT"), distinct_text)
        } else {
            self.kw("SELECT")
        };

        if clauses.targets.len() <= 1 {
            let target_text = clauses
                .targets
                .first()
                .map(|t| self.format_target_el(*t))
                .unwrap_or_else(|| "*".to_string());
            lines.push(format!("{select_kw} {target_text}"));
        } else {
            lines.push(select_kw);
            for (i, target) in clauses.targets.iter().enumerate() {
                let formatted = self.format_target_el(*target);
                if i < clauses.targets.len() - 1 {
                    lines.push(format!("{indent}{formatted},"));
                } else {
                    lines.push(format!("{indent}{formatted}"));
                }
            }
        }

        // FROM clause.
        if let Some(from) = clauses.from {
            if blank {
                lines.push(String::new());
            }
            self.format_from_left_aligned(from, &mut lines);
        }

        // WHERE clause.
        if let Some(where_c) = clauses.where_clause {
            if blank {
                lines.push(String::new());
            }
            self.format_where_left_aligned(where_c, &mut lines);
        }

        // GROUP BY.
        if let Some(group) = clauses.group_clause {
            if blank {
                lines.push(String::new());
            }
            self.format_group_by_left_aligned(group, &mut lines);
        }

        // HAVING.
        if let Some(having) = clauses.having_clause {
            if blank {
                lines.push(String::new());
            }
            self.format_having_left_aligned(having, &mut lines);
        }

        // ORDER BY.
        if let Some(sort) = clauses.sort_clause {
            if blank {
                lines.push(String::new());
            }
            self.format_order_by_left_aligned(sort, &mut lines);
        }

        // LIMIT.
        if let Some(limit) = clauses.limit_clause {
            if blank {
                lines.push(String::new());
            }
            let value = self.extract_limit_value(limit);
            if !value.is_empty() {
                lines.push(format!("{} {value}", self.kw("LIMIT")));
            }
        }

        // OFFSET.
        if let Some(offset) = clauses.offset_clause {
            let value = self.extract_offset_value(offset);
            if !value.is_empty() {
                lines.push(format!("{} {value}", self.kw("OFFSET")));
            }
        }

        let mut result = lines.join("\n");

        // Set operations.
        if let Some(ref set_op) = clauses.set_op {
            let op_text = if let Some(ref q) = set_op.quantifier {
                format!("{} {q}", set_op.keyword)
            } else {
                set_op.keyword.clone()
            };
            if blank {
                result.push_str("\n\n");
            } else {
                result.push('\n');
            }
            result.push_str(&op_text);
            result.push('\n');
            if let Some(ref rc) = set_op.right_clauses {
                result.push_str(&self.format_select_left_aligned(rc));
            } else {
                let right_clauses = self.collect_select_clauses(set_op.right);
                result.push_str(&self.format_select_left_aligned(&right_clauses));
            }
        }

        result
    }

    fn format_from_left_aligned(&self, node: Node<'a>, lines: &mut Vec<String>) {
        let indent = self.config.indent;
        if let Some(from_list) = node.find_child("from_list") {
            let tables = flatten_list(from_list, "from_list");
            if tables.is_empty() {
                return;
            }

            // Check if any table has joins.
            let first = tables[0];
            if first.kind() == "table_ref" && first.has_child("joined_table") {
                let jt = first.find_child("joined_table").unwrap();
                self.format_joined_table_left_aligned(jt, lines, true);
            } else {
                let text = self.format_table_ref(first);
                if tables.len() == 1 && !first.has_child("joined_table") {
                    lines.push(format!("{} {text}", self.kw("FROM")));
                } else {
                    lines.push(self.kw("FROM"));
                    lines.push(format!("{indent}{text}"));
                }
            }

            for table in &tables[1..] {
                if table.kind() == "table_ref" && table.has_child("joined_table") {
                    let jt = table.find_child("joined_table").unwrap();
                    self.format_joined_table_left_aligned(jt, lines, false);
                } else {
                    let text = self.format_table_ref(*table);
                    lines.push(format!("{indent}{text}"));
                }
            }
        }
    }

    fn format_joined_table_left_aligned(
        &self,
        node: Node<'a>,
        lines: &mut Vec<String>,
        is_first: bool,
    ) {
        let indent = self.config.indent;
        let named = node.named_children_vec();

        let mut left_table: Option<Node> = None;
        let mut right_table: Option<Node> = None;
        let mut join_qual_node: Option<Node> = None;
        let mut table_count = 0;

        for child in &named {
            match child.kind() {
                "table_ref" => {
                    if table_count == 0 {
                        left_table = Some(*child);
                    } else {
                        right_table = Some(*child);
                    }
                    table_count += 1;
                }
                "join_qual" => join_qual_node = Some(*child),
                _ => {}
            }
        }

        // Format left side.
        if let Some(left) = left_table {
            if let Some(inner_jt) = left.find_child("joined_table") {
                self.format_joined_table_left_aligned(inner_jt, lines, is_first);
            } else {
                let text = self.format_table_ref(left);
                if is_first {
                    // GitLab/Kickstarter: FROM and table on same line.
                    // Mozilla/dbt: FROM on its own line, table indented.
                    if self.style == Style::Gitlab || self.style == Style::Kickstarter {
                        lines.push(format!("{} {text}", self.kw("FROM")));
                    } else {
                        lines.push(self.kw("FROM"));
                        lines.push(format!("{indent}{text}"));
                    }
                } else {
                    lines.push(format!("{indent}{text}"));
                }
            }
        }

        // Format JOIN.
        let join_kw = self.get_join_keyword(node);

        if let Some(right) = right_table {
            let right_text = self.format_table_ref(right);
            if self.config.join_on_same_line {
                // Kickstarter: JOIN ... ON on same line.
                let mut join_line = format!("{join_kw} {right_text}");
                if let Some(qual) = join_qual_node {
                    let qual_text = self.format_join_qual_inline(qual);
                    join_line.push_str(&format!(" {qual_text}"));
                    // If there are multiple AND conditions, wrap them.
                    lines.push(join_line);
                    // Additional conditions on indented lines.
                    self.format_extra_join_conditions(qual, lines);
                    return;
                }
                lines.push(join_line);
            } else {
                lines.push(join_kw.to_string());
                lines.push(format!("{indent}{right_text}"));
            }
        }

        // Format ON/USING (non-Kickstarter).
        if !self.config.join_on_same_line
            && let Some(qual) = join_qual_node
        {
            self.format_join_qual_left_aligned(qual, lines);
        }
    }

    fn format_join_qual_inline(&self, node: Node<'a>) -> String {
        if node.has_child("kw_on") {
            if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
                let conditions = self.split_top_level_conditions(expr);
                if !conditions.is_empty() {
                    return format!("{} {}", self.kw("ON"), conditions[0].1);
                }
            }
        } else if node.has_child("kw_using") {
            let using_text = self.format_using_clause(node);
            return format!("{} {using_text}", self.kw("USING"));
        }
        String::new()
    }

    fn format_extra_join_conditions(&self, node: Node<'a>, lines: &mut Vec<String>) {
        if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
            let conditions = self.split_top_level_conditions(expr);
            if conditions.len() > 1 {
                let indent = self.config.indent;
                for (op, cond_text) in &conditions[1..] {
                    lines.push(format!("{indent}{op} {cond_text}"));
                }
            }
        }
    }

    fn format_join_qual_left_aligned(&self, node: Node<'a>, lines: &mut Vec<String>) {
        let indent = self.config.indent;
        if node.has_child("kw_on") {
            if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
                let conditions = self.split_top_level_conditions(expr);
                if conditions.len() <= 1 {
                    lines.push(format!("{indent}{} {}", self.kw("ON"), conditions[0].1));
                } else {
                    lines.push(format!("{indent}{} {}", self.kw("ON"), conditions[0].1));
                    for (op, cond_text) in &conditions[1..] {
                        lines.push(format!("{indent}{op} {cond_text}"));
                    }
                }
            }
        } else if node.has_child("kw_using") {
            let using_text = self.format_using_clause(node);
            lines.push(format!("{indent}{} {using_text}", self.kw("USING")));
        }
    }

    pub(crate) fn format_where_left_aligned(&self, node: Node<'a>, lines: &mut Vec<String>) {
        let indent = self.config.indent;
        if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
            let conditions = self.split_top_level_conditions(expr);
            if conditions.len() <= 1 {
                let text = &conditions[0].1;
                if self.style == Style::Mozilla
                    || self.style == Style::Dbt
                    || self.style == Style::Gitlab
                    || self.style == Style::Kickstarter
                {
                    lines.push(self.kw("WHERE"));
                    Self::push_indented_multiline(lines, indent, text);
                } else {
                    lines.push(format!("{} {text}", self.kw("WHERE")));
                }
            } else {
                lines.push(self.kw("WHERE"));
                Self::push_indented_multiline(lines, indent, &conditions[0].1);
                for (op, cond_text) in &conditions[1..] {
                    Self::push_indented_multiline(lines, indent, &format!("{op} {cond_text}"));
                }
            }
        }
    }

    /// Push a potentially multi-line text with each line prefixed by indent.
    fn push_indented_multiline(lines: &mut Vec<String>, indent: &str, text: &str) {
        for line in text.lines() {
            if line.is_empty() {
                lines.push(String::new());
            } else {
                lines.push(format!("{indent}{line}"));
            }
        }
    }

    fn format_having_left_aligned(&self, node: Node<'a>, lines: &mut Vec<String>) {
        self.format_condition_clause_left_aligned(node, "HAVING", lines);
    }

    /// Shared left-aligned formatting for HAVING (and potentially other) clauses.
    fn format_condition_clause_left_aligned(
        &self,
        node: Node<'a>,
        keyword: &str,
        lines: &mut Vec<String>,
    ) {
        let indent = self.config.indent;
        if let Some(expr) = node.find_child_any(&["a_expr", "c_expr"]) {
            let conditions = self.split_top_level_conditions(expr);
            lines.push(self.kw(keyword));
            lines.push(format!("{indent}{}", conditions[0].1));
            for (op, cond_text) in &conditions[1..] {
                lines.push(format!("{indent}{op} {cond_text}"));
            }
        }
    }

    fn format_group_by_left_aligned(&self, node: Node<'a>, lines: &mut Vec<String>) {
        let kw = self.kw_pair("GROUP", "BY");
        if let Some(list) = node.find_child("group_by_list") {
            let items = flatten_list(list, "group_by_list");
            if items.len() <= 1 || self.style == Style::Kickstarter {
                let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
                lines.push(format!("{kw} {}", formatted.join(", ")));
            } else {
                // dbt/GitLab: each on its own line.
                lines.push(kw);
                let indent = self.config.indent;
                for (i, item) in items.iter().enumerate() {
                    let formatted = self.format_expr(*item);
                    if i < items.len() - 1 {
                        lines.push(format!("{indent}{formatted},"));
                    } else {
                        lines.push(format!("{indent}{formatted}"));
                    }
                }
            }
        }
    }

    fn format_order_by_left_aligned(&self, node: Node<'a>, lines: &mut Vec<String>) {
        let kw = self.kw_pair("ORDER", "BY");
        if let Some(list) = node.find_child("sortby_list") {
            let items = flatten_list(list, "sortby_list");
            let formatted: Vec<_> = items.iter().map(|i| self.format_sortby(*i)).collect();
            lines.push(format!("{kw} {}", formatted.join(", ")));
        }
    }

    // ── WITH / CTE formatting ───────────────────────────────────────────

    fn format_with_clause_river(&self, node: Node<'a>, river_width: usize) -> String {
        let mut lines = Vec::new();
        if let Some(cte_list) = node.find_child("cte_list") {
            let ctes = flatten_list(cte_list, "cte_list");
            for (i, cte) in ctes.iter().enumerate() {
                let cte_text = self.format_cte_river(*cte, river_width);
                if i == 0 {
                    lines.push(format!("{} {cte_text}", self.kw("WITH")));
                } else {
                    lines.push(cte_text);
                }
            }
        }
        lines.join(",\n")
    }

    fn format_cte_river(&self, node: Node<'a>, _river_width: usize) -> String {
        let name = node
            .find_child("name")
            .map(|n| self.format_expr(n))
            .unwrap_or_default();

        let mut body = String::new();
        if let Some(prep) = node.find_child("PreparableStmt")
            && let Some(select) = prep.find_child("SelectStmt")
        {
            body = self.format_select_stmt(select);
        }

        format!("{name} {} (\n{body}\n)", self.kw("AS"))
    }

    fn format_with_clause_left(&self, node: Node<'a>) -> String {
        let mut lines = Vec::new();
        let indent = self.config.indent;
        let blank_in_ctes = self.config.blank_lines_in_ctes;

        if let Some(cte_list) = node.find_child("cte_list") {
            let ctes = flatten_list(cte_list, "cte_list");

            if self.config.blank_lines_between_clauses {
                // dbt style: with\n\nname as (\n...)
                lines.push(format!("{}\n", self.kw("with")));
            }

            for (i, cte) in ctes.iter().enumerate() {
                let name = cte
                    .find_child("name")
                    .map(|n| self.format_expr(n))
                    .unwrap_or_default();

                let mut body = String::new();
                if let Some(prep) = cte.find_child("PreparableStmt")
                    && let Some(select) = prep.find_child("SelectStmt")
                {
                    body = self.format_select_stmt(select);
                }

                let indented_body = body
                    .lines()
                    .map(|l| {
                        if l.is_empty() {
                            String::new()
                        } else {
                            format!("{indent}{l}")
                        }
                    })
                    .collect::<Vec<_>>()
                    .join("\n");

                let cte_prefix = if self.config.compact_ctes && i > 0 {
                    format!("), {name} {} (", self.kw("AS"))
                } else {
                    let as_line = format!("{name} {} (", self.kw("AS"));
                    if i == 0 && !self.config.blank_lines_between_clauses {
                        format!("{} {as_line}", self.kw("WITH"))
                    } else {
                        as_line
                    }
                };

                if self.config.compact_ctes && i > 0 {
                    lines.push(cte_prefix);
                } else {
                    if i > 0 && !self.config.compact_ctes {
                        // Close previous CTE.
                        // Already handled below.
                    }
                    lines.push(cte_prefix);
                }

                if blank_in_ctes || self.config.blank_lines_between_clauses {
                    lines.push(String::new());
                }
                lines.push(indented_body);
                if blank_in_ctes || self.config.blank_lines_between_clauses {
                    lines.push(String::new());
                }

                if !self.config.compact_ctes {
                    lines.push(")".to_string());
                }
            }

            if self.config.compact_ctes {
                lines.push(")".to_string());
            }
        }
        lines.join("\n")
    }

    // ── VALUES clause ───────────────────────────────────────────────────

    pub(crate) fn format_values_clause(&self, node: Node<'a>) -> String {
        // values_clause contains multiple (expr_list) groups.
        let mut value_groups = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "values_clause" => {
                    // Recursive left-linked.
                    let inner_groups = self.collect_value_groups(child);
                    value_groups.extend(inner_groups);
                }
                "expr_list" => {
                    let items = flatten_list(child, "expr_list");
                    let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
                    value_groups.push(format!("({})", formatted.join(", ")));
                }
                _ => {}
            }
        }
        let kw = self.kw("VALUES");
        if self.config.leading_commas {
            let mut lines = vec![format!("{kw} {}", value_groups[0])];
            for group in &value_groups[1..] {
                let padding = " ".repeat(kw.len() + 1 - 2);
                lines.push(format!("{padding}, {group}"));
            }
            lines.join("\n")
        } else {
            let mut lines = Vec::new();
            if self.config.river {
                // River style: VALUES aligned.
                for (i, group) in value_groups.iter().enumerate() {
                    if i == 0 {
                        if value_groups.len() > 1 {
                            lines.push(format!("{kw} {group},"));
                        } else {
                            lines.push(format!("{kw} {group}"));
                        }
                    } else {
                        let padding = " ".repeat(kw.len() + 1);
                        if i < value_groups.len() - 1 {
                            lines.push(format!("{padding}{group},"));
                        } else {
                            lines.push(format!("{padding}{group}"));
                        }
                    }
                }
            } else {
                // Left-aligned: VALUES on its own line.
                lines.push(kw);
                let indent = self.config.indent;
                for (i, group) in value_groups.iter().enumerate() {
                    if i < value_groups.len() - 1 {
                        lines.push(format!("{indent}{group},"));
                    } else {
                        lines.push(format!("{indent}{group}"));
                    }
                }
            }
            lines.join("\n")
        }
    }

    fn collect_value_groups(&self, node: Node<'a>) -> Vec<String> {
        let mut groups = Vec::new();
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            match child.kind() {
                "values_clause" => {
                    groups.extend(self.collect_value_groups(child));
                }
                "expr_list" => {
                    let items = flatten_list(child, "expr_list");
                    let formatted: Vec<_> = items.iter().map(|i| self.format_expr(*i)).collect();
                    groups.push(format!("({})", formatted.join(", ")));
                }
                _ => {}
            }
        }
        groups
    }
}
