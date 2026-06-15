//! `Style::PgDump` renderer — reproduces PostgreSQL's `ruleutils.c` deparser
//! layout (the output of `pg_get_viewdef` / `pg_get_functiondef`).
//!
//! Unlike the river / left-aligned engine, this renderer's correctness bar is
//! byte-idempotency: feeding genuine deparser output back through it must
//! return that output unchanged. It therefore reproduces ruleutils' exact
//! indentation rather than reading `StyleConfig` flags, and reproduces each
//! expression verbatim (the deparser has already normalized casts, parens and
//! spacing) rather than re-formatting it.
//!
//! Layout rules observed from `pg_get_viewdef(oid, true)` at the top level:
//! clause keywords right-align so their first word ends at column 7
//! (`SELECT`/`FROM`/`WHERE`/`HAVING`/`GROUP BY`/`ORDER BY`); continuation
//! targets indent 4; JOIN steps indent 5; set-operation keywords sit at
//! column 1.
//!
//! Scope (first increment): flat, single-level views (target list, FROM with
//! joins, WHERE, GROUP BY, HAVING, ORDER BY, set operations) and functions
//! (`CREATE FUNCTION` header + attributes + verbatim body). CTEs, scalar
//! subqueries, multi-line CASE and DISTINCT-target indentation are not yet
//! handled; statements the renderer doesn't recognize fall back to verbatim
//! source (still idempotent on deparser output).

use crate::error::FormatError;
use crate::formatter::Formatter;
use crate::formatter::select::SelectClauses;
use crate::node_helpers::{NodeExt, flatten_list};
use tree_sitter::Node;

/// Column at which a clause keyword's first word ends (1 leading space before
/// `SELECT`). Leading spaces for a keyword = `RIVER_END - first_word_len`.
const RIVER_END: usize = 7;
/// Indentation for target-list continuation lines.
const TARGET_INDENT: usize = 4;
/// Indentation for JOIN continuation lines.
const JOIN_INDENT: usize = 5;

impl<'a> Formatter<'a> {
    /// Render a single top-level statement in ruleutils (`pg_dump`) layout.
    pub(crate) fn format_pgdump_stmt(&self, stmt: Node<'a>) -> Result<String, FormatError> {
        let Some(inner) = stmt.named_children_vec().into_iter().next() else {
            return Ok(self.collapse_ws(self.text(stmt)));
        };
        match inner.kind() {
            "SelectStmt" => Ok(format!("{};", self.pgdump_select(inner))),
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

    /// Leading spaces so that `word` (a clause keyword's first token) ends at
    /// [`RIVER_END`].
    fn river_pad(&self, word_len: usize) -> String {
        " ".repeat(RIVER_END.saturating_sub(word_len))
    }

    fn pgdump_select(&self, node: Node<'a>) -> String {
        let snp = node.find_child("select_no_parens").unwrap_or(node);
        let clauses = self.collect_select_clauses(snp);
        self.pgdump_render_clauses(&clauses)
    }

    /// Render collected SELECT clauses as a ruleutils body (no trailing `;`).
    fn pgdump_render_clauses(&self, c: &SelectClauses<'a>) -> String {
        let mut s = String::new();

        // SELECT [DISTINCT] target, target, ...
        s.push(' ');
        s.push_str("SELECT ");
        if let Some(d) = c.distinct {
            s.push_str(&self.collapse_ws(self.text(d)));
            s.push(' ');
        }
        let targets: Vec<String> = if c.targets.is_empty() {
            vec!["*".to_string()]
        } else {
            c.targets
                .iter()
                .map(|t| self.collapse_ws(self.text(*t)))
                .collect()
        };
        let cont = format!(",\n{}", " ".repeat(TARGET_INDENT));
        s.push_str(&targets.join(&cont));

        // FROM
        if let Some(from) = c.from {
            let items = self.pgdump_from_items(from);
            if let Some((first, joins)) = items.split_first() {
                s.push('\n');
                s.push_str(&self.river_pad(4)); // FROM
                s.push_str("FROM ");
                s.push_str(first);
                let join_pad = " ".repeat(JOIN_INDENT);
                for j in joins {
                    s.push('\n');
                    s.push_str(&join_pad);
                    s.push_str(j);
                }
            }
        }

        // WHERE
        if let Some(w) = c.where_clause
            && let Some(expr) = w.find_child_any(&["a_expr", "c_expr"])
        {
            s.push('\n');
            s.push_str(&self.river_pad(5)); // WHERE
            s.push_str("WHERE ");
            s.push_str(&self.collapse_ws(self.text(expr)));
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
            s.push_str(&self.river_pad(5)); // GROUP
            s.push_str("GROUP BY ");
            s.push_str(&items.join(", "));
        }

        // HAVING
        if let Some(h) = c.having_clause
            && let Some(expr) = h.find_child_any(&["a_expr", "c_expr"])
        {
            s.push('\n');
            s.push_str(&self.river_pad(6)); // HAVING
            s.push_str("HAVING ");
            s.push_str(&self.collapse_ws(self.text(expr)));
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
            s.push_str(&self.river_pad(5)); // ORDER
            s.push_str("ORDER BY ");
            s.push_str(&items.join(", "));
        }

        // Set operation (UNION / INTERSECT / EXCEPT): keyword at column 1,
        // then the right-hand select rendered the same way.
        if let Some(so) = &c.set_op {
            s.push('\n');
            s.push_str(&so.keyword);
            if let Some(q) = &so.quantifier {
                s.push(' ');
                s.push_str(q);
            }
            s.push('\n');
            let right = if let Some(rc) = &so.right_clauses {
                self.pgdump_render_clauses(rc)
            } else {
                let rc = self.collect_select_clauses(so.right);
                self.pgdump_render_clauses(&rc)
            };
            s.push_str(&right);
        }

        s
    }

    /// Flatten a `from_clause` into ruleutils FROM items: the base relation
    /// followed by one entry per JOIN step (`JOIN tbl ON ...`).
    fn pgdump_from_items(&self, from_clause: Node<'a>) -> Vec<String> {
        let mut items = Vec::new();
        if let Some(list) = from_clause.find_child("from_list") {
            for r in flatten_list(list, "from_list") {
                self.pgdump_table_ref_items(r, &mut items);
            }
        }
        items
    }

    fn pgdump_table_ref_items(&self, node: Node<'a>, items: &mut Vec<String>) {
        let joined = if node.kind() == "joined_table" {
            Some(node)
        } else {
            node.find_child("joined_table")
        };
        if let Some(jt) = joined {
            // Left side first (may itself be a join), then this join step:
            // everything from the end of the left table_ref to the end of the
            // joined_table (the JOIN keyword, right table and qualifier),
            // reproduced verbatim.
            if let Some(left) = jt.find_child("table_ref") {
                self.pgdump_table_ref_items(left, items);
                let step = self.collapse_ws(&self.source[left.end_byte()..jt.end_byte()]);
                items.push(step);
            } else {
                items.push(self.collapse_ws(self.text(jt)));
            }
        } else {
            items.push(self.collapse_ws(self.text(node)));
        }
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
