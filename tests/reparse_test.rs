//! Regression guard for https://github.com/gmr/libpgfmt/issues/54.
//!
//! 33 statements from the PostgreSQL documentation formatted into SQL that
//! libpgfmt itself could no longer parse — clauses were dropped or mangled and
//! `format()` still returned `Ok`. Formatting must never produce output that
//! fails to parse, so each statement is formatted and the result re-formatted,
//! in every style.

use libpgfmt::{format, style::Style};

fn statements() -> Vec<String> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/reparse/doc_regressions.sql");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    body.split("\n;;")
        .map(|s| {
            s.lines()
                .filter(|l| {
                    !l.trim_start().starts_with("-- Statements")
                        && !l.trim_start().starts_with("-- SQL libpgfmt")
                        && !l.trim_start().starts_with("-- the formatted")
                        && !l.trim_start().starts_with("-- One statement")
                })
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string()
        })
        .filter(|s| !s.is_empty())
        .collect()
}

#[test]
fn formatted_output_parses_in_every_style() {
    let statements = statements();
    assert_eq!(statements.len(), 33, "fixture should hold 33 statements");

    for &style in Style::ALL {
        for sql in &statements {
            let once = format(sql, style).unwrap_or_else(|e| {
                panic!("\nStyle: {style}\nInput:\n{sql}\nFailed to format: {e}")
            });
            format(&once, style).unwrap_or_else(|e| {
                panic!("\nStyle: {style}\nInput:\n{sql}\nFormatted to:\n{once}\nWhich no longer parses: {e}")
            });
        }
    }
}

/// Formatting is a fixed point for every statement whose output is stable.
#[test]
fn formatting_is_idempotent_in_every_style() {
    for &style in Style::ALL {
        for sql in &statements() {
            let once = format(sql, style).unwrap();
            let twice = format(&once, style).unwrap();
            assert_eq!(once, twice, "\nStyle: {style}\nInput:\n{sql}");
        }
    }
}

/// Reparsing and idempotence both pass on output that is stable, valid and
/// missing a clause. These modifiers were the ones silently dropped, so assert
/// each survives verbatim wherever the fixture uses it.
#[test]
fn clause_modifiers_survive_in_every_style() {
    const CLAUSES: &[&str] = &[
        "IS JSON SCALAR",
        "IS JSON OBJECT",
        "IS JSON ARRAY",
        "WITH UNIQUE KEYS",
        "WITHOUT UNIQUE KEYS",
        "TABLESAMPLE SYSTEM_ROWS(100)",
        "TABLESAMPLE SYSTEM_TIME(1000)",
    ];
    // Whitespace, spacing before an argument list and keyword casing are all
    // layout, so compare on a single-spaced upper-case form.
    fn squeeze(s: &str) -> String {
        s.split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_uppercase()
            .replace(" (", "(")
    }

    let mut seen = vec![false; CLAUSES.len()];
    for sql in &statements() {
        let input = squeeze(sql);
        for (i, clause) in CLAUSES.iter().enumerate() {
            if !input.contains(clause) {
                continue;
            }
            seen[i] = true;
            for &style in Style::ALL {
                let out = squeeze(&format(sql, style).unwrap());
                assert!(
                    out.contains(clause),
                    "\nStyle: {style}\nInput:\n{sql}\nDropped `{clause}` from:\n{out}"
                );
            }
        }
    }
    for (i, clause) in CLAUSES.iter().enumerate() {
        assert!(seen[i], "no fixture statement exercises `{clause}`");
    }
}
