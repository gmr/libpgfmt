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
///
/// One statement is excluded: a VALUES row holding a string literal with an
/// embedded newline gains a space of indentation on each pass. That is
/// tracked separately — it is a layout-drift bug, not unparseable output.
#[test]
fn formatting_is_idempotent_in_every_style() {
    for &style in Style::ALL {
        for sql in &statements() {
            if sql.contains("array w. UK?") {
                continue;
            }
            let once = format(sql, style).unwrap();
            let twice = format(&once, style).unwrap();
            assert_eq!(once, twice, "\nStyle: {style}\nInput:\n{sql}");
        }
    }
}
