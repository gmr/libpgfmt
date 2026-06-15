//! `Style::PgDump` renderer — reproduces PostgreSQL's `ruleutils.c` deparser
//! layout (the output of `pg_get_viewdef` / `pg_get_functiondef`).
//!
//! Unlike the river / left-aligned engine, this renderer's correctness bar is
//! byte-idempotency: feeding genuine deparser output back through it must
//! return that output unchanged. It therefore reproduces ruleutils' exact
//! indentation rather than reading `StyleConfig` flags, and reproduces each
//! single-line expression verbatim (the deparser has already normalized casts,
//! parens and spacing) rather than re-formatting it.
//!
//! Layout rules observed from `pg_get_viewdef(oid, true)`. At nesting depth `d`
//! (subqueries inside CTEs add one level), with `STEP = 8`:
//!   - `SELECT` / `WITH` keywords start at column `2 + STEP*d` (one leading
//!     space at the top level);
//!   - the other clause keywords (`FROM`/`WHERE`/`GROUP BY`/`HAVING`/
//!     `ORDER BY`) right-align so their first word ends at column `7 + STEP*d`;
//!   - target / comma-separated-FROM continuation lines indent `4 + STEP*d`;
//!   - JOIN steps indent `5 + STEP*d`;
//!   - a multi-line `CASE` target is a block at `8 + STEP*d`, its `WHEN`/`ELSE`
//!     lines at `12 + STEP*d`, `END` back at `8 + STEP*d`;
//!   - a CTE body renders at depth `d+1`; its closing paren sits at
//!     `STEP*(d+1)`; set-operation keywords sit at column 1.
//!
//! Subqueries embedded in WHERE / HAVING / target expressions (`IN (SELECT …)`,
//! `EXISTS (…)`, scalar subqueries) are rendered inline at `d+1` and spliced
//! back in. Not yet handled: subqueries in the FROM list (derived tables),
//! GROUP BY / ORDER BY items or inside CASE arms. Statements the renderer
//! doesn't recognize fall back to verbatim source (still idempotent on
//! deparser output).

use crate::error::FormatError;
use crate::formatter::Formatter;
use crate::formatter::select::SelectClauses;
use crate::node_helpers::{NodeExt, flatten_list};
use tree_sitter::Node;

/// Indentation added per nesting level.
const STEP: usize = 8;
/// Column (1-based) at which a right-aligned clause keyword's first word ends,
/// at depth 0.
const RIVER_END: usize = 7;

impl<'a> Formatter<'a> {
    /// Render a single top-level statement in ruleutils (`pg_dump`) layout.
    pub(crate) fn format_pgdump_stmt(&self, stmt: Node<'a>) -> Result<String, FormatError> {
        let Some(inner) = stmt.named_children_vec().into_iter().next() else {
            return Ok(self.collapse_ws(self.text(stmt)));
        };
        match inner.kind() {
            "SelectStmt" => Ok(format!("{};", self.pgdump_select(inner, 0))),
            "CreateFunctionStmt" => Ok(self.pgdump_create_function(inner)),
            // Out of scope: reproduce verbatim so deparser output still
            // round-trips and nothing is mangled.
            _ => Ok(self.text(inner).trim().to_string()),
        }
    }

    /// Collapse all runs of whitespace to a single space and trim. A no-op on
    /// canonical (single-line, single-spaced) deparser expressions; it folds
    /// the deparser's line breaks so the layout can be re-imposed.
    fn collapse_ws(&self, text: &str) -> String {
        text.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    /// Like [`collapse_ws`] but keeps a single boundary space when the original
    /// began or ended with whitespace, so collapsed fragments concatenate with
    /// correct spacing around a spliced-in subquery.
    fn collapse_preserve_edges(&self, text: &str) -> String {
        if text.is_empty() {
            return String::new();
        }
        let lead = text.starts_with(char::is_whitespace);
        let trail = text.ends_with(char::is_whitespace);
        let core = self.collapse_ws(text);
        if core.is_empty() {
            return if lead || trail {
                " ".into()
            } else {
                String::new()
            };
        }
        format!(
            "{}{core}{}",
            if lead { " " } else { "" },
            if trail { " " } else { "" }
        )
    }

    /// Render an expression that may embed sub-`SELECT`s. Parts outside a
    /// subquery are reproduced verbatim (whitespace collapsed); each
    /// `select_with_parens` is rendered at `depth + 1` and spliced inline, the
    /// way ruleutils lays out `IN (SELECT ...)`, `EXISTS (SELECT ...)` and
    /// scalar subqueries. With no embedded subquery this is just `collapse_ws`.
    fn render_expr_text(&self, node: Node<'a>, depth: usize) -> String {
        let mut subs = Vec::new();
        self.collect_select_with_parens(node, &mut subs);
        if subs.is_empty() {
            return self.collapse_ws(self.text(node));
        }
        let full = self.source;
        let mut out = String::new();
        let mut pos = node.start_byte();
        for swp in subs {
            out.push_str(&self.collapse_preserve_edges(&full[pos..swp.start_byte()]));
            out.push_str(&self.render_select_with_parens(swp, depth));
            pos = swp.end_byte();
        }
        out.push_str(&self.collapse_preserve_edges(&full[pos..node.end_byte()]));
        out
    }

    /// Collect top-level `select_with_parens` nodes (not recursing into a
    /// subquery once found), in source order.
    fn collect_select_with_parens(&self, node: Node<'a>, out: &mut Vec<Node<'a>>) {
        let mut cursor = node.walk();
        for child in node.named_children(&mut cursor) {
            if child.kind() == "select_with_parens" {
                out.push(child);
            } else {
                self.collect_select_with_parens(child, out);
            }
        }
    }

    /// Render a parenthesized subquery inline: `( <body at depth+1> )` with the
    /// body's first line de-indented so `SELECT` follows the open paren and
    /// subsequent clauses align at the deeper river column.
    fn render_select_with_parens(&self, swp: Node<'a>, depth: usize) -> String {
        let body = swp.find_child("select_no_parens").or_else(|| {
            swp.find_child("SelectStmt")
                .and_then(|s| s.find_child("select_no_parens"))
        });
        let Some(body) = body else {
            return self.collapse_ws(self.text(swp));
        };
        let clauses = self.collect_select_clauses(body);
        let inner = self.pgdump_render_clauses(&clauses, depth + 1);
        let dedented = match inner.split_once('\n') {
            Some((first, rest)) => format!("{}\n{rest}", first.trim_start()),
            None => inner.trim_start().to_string(),
        };
        format!("( {dedented})")
    }

    /// Leading spaces so `word` (a right-aligned clause keyword's first token)
    /// ends at the river column for `depth`.
    fn river_pad(&self, word_len: usize, depth: usize) -> String {
        " ".repeat((RIVER_END + STEP * depth).saturating_sub(word_len))
    }

    fn pgdump_select(&self, node: Node<'a>, depth: usize) -> String {
        let snp = node.find_child("select_no_parens").unwrap_or(node);
        let clauses = self.collect_select_clauses(snp);
        self.pgdump_render_clauses(&clauses, depth)
    }

    /// Render collected SELECT clauses as a ruleutils body (no trailing `;`).
    fn pgdump_render_clauses(&self, c: &SelectClauses<'a>, depth: usize) -> String {
        let mut s = String::new();

        // WITH ... CTE list.
        if let Some(w) = c.with_clause {
            s.push_str(&self.pgdump_with(w, depth));
            s.push('\n');
        }

        // SELECT [DISTINCT] target, target, ... (CASE targets render as blocks).
        s.push_str(&self.pgdump_targets(c, depth));

        // FROM
        if let Some(from) = c.from {
            s.push_str(&self.pgdump_from(from, depth));
        }

        // WHERE
        if let Some(w) = c.where_clause
            && let Some(expr) = w.find_child_any(&["a_expr", "c_expr"])
        {
            s.push('\n');
            s.push_str(&self.river_pad(5, depth));
            s.push_str("WHERE ");
            s.push_str(&self.render_expr_text(expr, depth));
        }

        // GROUP BY
        if let Some(g) = c.group_clause
            && let Some(list) = g.find_child("group_by_list")
        {
            let items: Vec<String> = flatten_list(list, "group_by_list")
                .iter()
                .map(|i| self.collapse_ws(self.text(*i)))
                .collect();
            s.push('\n');
            s.push_str(&self.river_pad(5, depth));
            s.push_str("GROUP BY ");
            s.push_str(&items.join(", "));
        }

        // HAVING
        if let Some(h) = c.having_clause
            && let Some(expr) = h.find_child_any(&["a_expr", "c_expr"])
        {
            s.push('\n');
            s.push_str(&self.river_pad(6, depth));
            s.push_str("HAVING ");
            s.push_str(&self.render_expr_text(expr, depth));
        }

        // ORDER BY
        if let Some(sort) = c.sort_clause
            && let Some(list) = sort.find_child("sortby_list")
        {
            let items: Vec<String> = flatten_list(list, "sortby_list")
                .iter()
                .map(|i| self.collapse_ws(self.text(*i)))
                .collect();
            s.push('\n');
            s.push_str(&self.river_pad(5, depth));
            s.push_str("ORDER BY ");
            s.push_str(&items.join(", "));
        }

        // Set operation (UNION / INTERSECT / EXCEPT): keyword at column 1, then
        // the right-hand select rendered the same way.
        if let Some(so) = &c.set_op {
            s.push('\n');
            s.push_str(&so.keyword);
            if let Some(q) = &so.quantifier {
                s.push(' ');
                s.push_str(q);
            }
            s.push('\n');
            let right = if let Some(rc) = &so.right_clauses {
                self.pgdump_render_clauses(rc, depth)
            } else {
                let rc = self.collect_select_clauses(so.right);
                self.pgdump_render_clauses(&rc, depth)
            };
            s.push_str(&right);
        }

        s
    }

    /// Render `SELECT [DISTINCT]` and the target list. Simple targets render
    /// inline at indent `4 + STEP*depth`; `CASE` targets render as blocks.
    fn pgdump_targets(&self, c: &SelectClauses<'a>, depth: usize) -> String {
        let lead = STEP * depth + 1; // SELECT keyword leading spaces
        let cont = " ".repeat(STEP * depth + 4);
        let mut s = format!("{}SELECT", " ".repeat(lead));
        if let Some(d) = c.distinct {
            s.push(' ');
            s.push_str(&self.collapse_ws(self.text(d)));
        }

        let targets = &c.targets;
        if targets.is_empty() {
            s.push_str(" *");
            return s;
        }

        let last = targets.len() - 1;
        for (i, t) in targets.iter().enumerate() {
            if let Some(case_node) = self.target_case_expr(*t) {
                // CASE block always starts on its own line.
                s.push('\n');
                s.push_str(&self.render_case_block(*t, case_node, depth));
            } else if i == 0 {
                s.push(' ');
                s.push_str(&self.render_expr_text(*t, depth));
            } else {
                s.push('\n');
                s.push_str(&cont);
                s.push_str(&self.render_expr_text(*t, depth));
            }
            if i != last {
                s.push(',');
            }
        }
        s
    }

    /// If a target's expression is (only) a `CASE`, return its `case_expr`
    /// node. Descends single-child `a_expr`/`c_expr` wrappers; returns `None`
    /// when the CASE is part of a larger expression.
    fn target_case_expr(&self, target_el: Node<'a>) -> Option<Node<'a>> {
        let mut node = target_el.find_child_any(&["a_expr", "c_expr", "case_expr"])?;
        loop {
            match node.kind() {
                "case_expr" => return Some(node),
                "a_expr" | "c_expr" => {
                    let kids = node.named_children_vec();
                    if kids.len() == 1 {
                        node = kids[0];
                    } else {
                        return None;
                    }
                }
                _ => return None,
            }
        }
    }

    /// Render a `CASE` target as a ruleutils block, including the trailing
    /// `AS alias` (taken verbatim from after the `case_expr`).
    fn render_case_block(&self, target_el: Node<'a>, case_node: Node<'a>, depth: usize) -> String {
        let ci = " ".repeat(STEP * depth + 8); // CASE / END
        let wi = " ".repeat(STEP * depth + 12); // WHEN / ELSE
        let mut s = format!("{ci}CASE");

        // Optional simple-CASE argument: CASE <expr> WHEN ...
        let mut cursor = case_node.walk();
        for child in case_node.named_children(&mut cursor) {
            match child.kind() {
                "kw_case" | "when_clause_list" | "case_default" | "kw_end" => {}
                _ => {
                    s.push(' ');
                    s.push_str(&self.collapse_ws(self.text(child)));
                    break;
                }
            }
        }

        if let Some(wcl) = case_node.find_child("when_clause_list") {
            for wc in flatten_list(wcl, "when_clause_list") {
                s.push('\n');
                s.push_str(&wi);
                s.push_str(&self.collapse_ws(self.text(wc)));
            }
        }
        if let Some(def) = case_node.find_child("case_default") {
            s.push('\n');
            s.push_str(&wi);
            s.push_str(&self.collapse_ws(self.text(def)));
        }
        s.push('\n');
        s.push_str(&ci);
        s.push_str("END");

        // Trailing `AS alias` (everything in the target after the CASE).
        let tail = self.collapse_ws(&self.source[case_node.end_byte()..target_el.end_byte()]);
        if !tail.is_empty() {
            s.push(' ');
            s.push_str(&tail);
        }
        s
    }

    /// Render the FROM clause: comma-separated items at indent `4 + STEP*depth`,
    /// each item's JOIN steps at indent `5 + STEP*depth`.
    fn pgdump_from(&self, from_clause: Node<'a>, depth: usize) -> String {
        let Some(list) = from_clause.find_child("from_list") else {
            return String::new();
        };
        let items: Vec<String> = flatten_list(list, "from_list")
            .iter()
            .map(|tr| self.render_table_ref(*tr, depth))
            .collect();
        let Some((first, rest)) = items.split_first() else {
            return String::new();
        };
        let mut s = format!("\n{}FROM {first}", self.river_pad(4, depth));
        let cont = " ".repeat(STEP * depth + 4);
        for item in rest {
            s.push_str(",\n");
            s.push_str(&cont);
            s.push_str(item);
        }
        s
    }

    /// Render one FROM item (a relation, possibly with JOINs) as a (possibly
    /// multi-line) string: the base relation followed by one line per JOIN step.
    fn render_table_ref(&self, node: Node<'a>, depth: usize) -> String {
        let joined = if node.kind() == "joined_table" {
            Some(node)
        } else {
            node.find_child("joined_table")
        };
        let Some(jt) = joined else {
            return self.collapse_ws(self.text(node));
        };
        let Some(left) = jt.find_child("table_ref") else {
            return self.collapse_ws(self.text(jt));
        };
        let mut s = self.render_table_ref(left, depth);
        let step = self.collapse_ws(&self.source[left.end_byte()..jt.end_byte()]);
        s.push('\n');
        s.push_str(&" ".repeat(STEP * depth + 5));
        s.push_str(&step);
        s
    }

    /// Render a `WITH` clause: each CTE as `name AS ( <body at depth+1> )`,
    /// with subsequent CTEs continuing on the closing-paren line.
    fn pgdump_with(&self, w: Node<'a>, depth: usize) -> String {
        let mut s = format!("{}WITH ", " ".repeat(STEP * depth + 1));
        let close = " ".repeat(STEP * (depth + 1));
        let Some(list) = w.find_child("cte_list") else {
            return s;
        };
        let ctes = flatten_list(list, "cte_list");
        let last = ctes.len().saturating_sub(1);
        for (i, cte) in ctes.iter().enumerate() {
            let name = cte
                .find_child("name")
                .map(|n| self.collapse_ws(self.text(n)))
                .unwrap_or_default();
            s.push_str(&name);
            s.push_str(" AS (");
            let body = cte
                .find_child("PreparableStmt")
                .and_then(|p| p.find_child("SelectStmt"))
                .and_then(|s| s.find_child("select_no_parens"))
                .or_else(|| cte.find_child("select_no_parens"));
            if let Some(body) = body {
                let clauses = self.collect_select_clauses(body);
                s.push('\n');
                s.push_str(&self.pgdump_render_clauses(&clauses, depth + 1));
            }
            s.push('\n');
            s.push_str(&close);
            s.push(')');
            if i != last {
                s.push_str(", ");
            }
        }
        s
    }

    /// Render a `CREATE FUNCTION` statement in `pg_get_functiondef` layout:
    /// signature on the first line, `RETURNS` and each option on its own
    /// space-prefixed line, and the `AS` clause (body verbatim) last.
    fn pgdump_create_function(&self, node: Node<'a>) -> String {
        let mut s = String::new();

        // Signature: CREATE [OR REPLACE] FUNCTION name(args) — verbatim up to
        // and including the argument list's closing paren.
        let sig_end = node
            .find_child("func_args_with_defaults")
            .or_else(|| node.find_child("func_args"))
            .map(|n| n.end_byte())
            .unwrap_or_else(|| {
                node.find_child("func_name")
                    .map(|n| n.end_byte())
                    .unwrap_or(node.end_byte())
            });
        s.push_str(&self.collapse_ws(&self.source[node.start_byte()..sig_end]));

        // RETURNS <type>
        if let Some(ret) = node.find_child("func_return") {
            s.push_str("\n RETURNS ");
            s.push_str(&self.collapse_ws(self.text(ret)));
        }

        // Options (LANGUAGE, volatility, STRICT, AS body, ...) in source order,
        // which matches the deparser's canonical order on genuine input.
        if let Some(opts) = node
            .find_child("opt_createfunc_opt_list")
            .and_then(|n| n.find_child("createfunc_opt_list"))
            .or_else(|| node.find_child("createfunc_opt_list"))
        {
            for item in flatten_list(opts, "createfunc_opt_list") {
                if let Some(as_kw) = item.find_child("kw_as") {
                    // AS clause: body reproduced verbatim (dollar-quoted text
                    // may span multiple lines and must not be collapsed).
                    let body = self.source[as_kw.end_byte()..item.end_byte()].trim_start();
                    s.push_str("\nAS ");
                    s.push_str(body);
                } else {
                    s.push_str("\n ");
                    s.push_str(&self.collapse_ws(self.text(item)));
                }
            }
        }

        s
    }
}
