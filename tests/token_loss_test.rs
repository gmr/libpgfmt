//! Regression guard for https://github.com/gmr/libpgfmt/issues/58.
//!
//! Formatting must not discard content. Every formatter here walks the CST and
//! renders the node kinds it knows; a kind it does not know falls through and
//! its text never reaches the output, so `format()` returns `Ok` with clauses
//! silently missing. Re-parsing does not catch it — the shorter statement is
//! usually still valid SQL.
//!
//! This test formats every SQL example in the PostgreSQL documentation and
//! compares the tokens of the input against the tokens of the output. Anything
//! in the input that is missing from the output is content loss, except for the
//! deliberate rewrites listed in `canonicalize`.
//!
//! Statements that still lose content are listed in `known_token_loss.txt` so
//! the suite passes while they are worked off. The test fails both when a new
//! statement starts losing content and when a listed one stops, so the list can
//! only shrink and cannot go stale.
//!
//! To see every statement that still loses content, with its input and output:
//!
//! ```sh
//! TOKEN_LOSS_REPORT=1 cargo test --test token_loss_test -- --nocapture
//! ```

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use libpgfmt::{format, style::Style};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/corpus")
        .join(name)
}

/// The corpus as `(id, sql)` pairs, where `id` identifies the statement by its
/// text and so survives regenerating the corpus from a newer PostgreSQL.
fn corpus() -> Vec<(String, String)> {
    let path = fixture("postgres_doc.sql");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    body.split("\n;;\n")
        .map(|record| {
            record
                .lines()
                .filter(|line| !line.trim_start().starts_with("--"))
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string()
        })
        .filter(|sql| !sql.is_empty())
        .map(|sql| (id(&sql), sql))
        .collect()
}

/// A short stable identifier for a statement: FNV-1a over its text.
fn id(sql: &str) -> String {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in sql.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100_0000_01b3);
    }
    format!("{hash:016x}")
}

/// The statements known to still lose content, as `id:style -> note`.
fn known_token_loss() -> HashMap<String, String> {
    let path = fixture("known_token_loss.txt");
    let body = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    body.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(|line| {
            let (id, note) = line.split_once(char::is_whitespace).unwrap_or((line, ""));
            (id.to_string(), note.trim().to_string())
        })
        .collect()
}

/// The tokens of `sql`, lowercased, with whitespace and identifier quoting
/// dropped. String literals stay whole so their contents are compared exactly.
fn tokens(sql: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut word = String::new();
    let mut chars = sql.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\'' {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            let mut literal = String::from("'");
            for c in chars.by_ref() {
                literal.push(c);
                if c == '\'' {
                    break;
                }
            }
            out.push(literal);
        } else if c.is_alphanumeric() || c == '_' || c == '.' || c == '$' {
            word.push(c.to_ascii_lowercase());
        } else {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            if !c.is_whitespace() && c != '"' {
                out.push(c.to_string());
            }
        }
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// Rewrites that both sides get, so the deliberate normalizations the
/// formatter performs do not read as content loss.
///
/// Every entry here is a rewrite libpgfmt makes on purpose: it spells a type
/// alias out, or writes an operator the way the SQL standard does. Adding to
/// this list hides a difference, so an entry belongs here only when the two
/// spellings mean exactly the same thing to PostgreSQL.
fn canonicalize(tokens: &[String]) -> Vec<String> {
    // Multi-token spellings, longest first.
    const SEQUENCES: &[(&[&str], &str)] = &[
        (&["timestamp", "with", "time", "zone"], "timestamptz"),
        (&["double", "precision"], "float8"),
        (&["character", "varying"], "varchar"),
        (&["!", "="], "<>"),
        // The tokenizer splits every operator character, so `<>` in the
        // output arrives as two tokens and must meet the rewritten `!=`.
        (&["<", ">"], "<>"),
    ];
    // Type aliases PostgreSQL treats as the same type.
    const ALIASES: &[(&str, &str)] = &[
        ("int", "int4"),
        ("integer", "int4"),
        ("smallint", "int2"),
        ("bigint", "int8"),
        ("real", "float4"),
        ("decimal", "numeric"),
        ("bool", "boolean"),
        ("char", "bpchar"),
    ];

    let mut out: Vec<String> = Vec::with_capacity(tokens.len());
    let mut i = 0;
    'outer: while i < tokens.len() {
        for (from, to) in SEQUENCES {
            if tokens[i..]
                .starts_with(&from.iter().map(|s| (*s).to_string()).collect::<Vec<_>>()[..])
            {
                out.push((*to).to_string());
                i += from.len();
                continue 'outer;
            }
        }
        let token = &tokens[i];
        let replacement = ALIASES
            .iter()
            .find(|(from, _)| from == token)
            .map_or(token.as_str(), |(_, to)| to);
        out.push(replacement.to_string());
        i += 1;
    }
    out
}

/// The tokens present in `input` but missing from `output`.
fn dropped(input: &str, output: &str) -> Vec<String> {
    let mut missing = canonicalize(&tokens(input));
    for token in canonicalize(&tokens(output)) {
        if let Some(i) = missing.iter().position(|m| *m == token) {
            missing.remove(i);
        }
    }
    // libpgfmt adds the statement terminator the grammar requires.
    missing.retain(|t| t != ";");
    missing
}

#[test]
fn formatting_does_not_drop_content() {
    let known = known_token_loss();
    let mut seen = Vec::new();
    let mut regressions = Vec::new();

    for (statement, sql) in corpus() {
        for &style in Style::ALL {
            let Ok(formatted) = format(&sql, style) else {
                // Statements the grammar cannot parse are a separate concern;
                // the corpus holds doc examples that are not valid alone.
                continue;
            };
            let missing = dropped(&sql, &formatted);
            if missing.is_empty() {
                continue;
            }
            let id = format!("{statement}:{style}");
            seen.push(id.clone());
            if std::env::var_os("TOKEN_LOSS_REPORT").is_some() {
                println!("{id}\nInput:\n{sql}\nFormatted to:\n{formatted}\nDropped: {missing:?}\n");
            }
            if !known.contains_key(&id) {
                regressions.push(format!(
                    "\n{id}\nInput:\n{sql}\nFormatted to:\n{formatted}\nDropped: {missing:?}"
                ));
            }
        }
    }

    let fixed: Vec<_> = known
        .keys()
        .filter(|id| !seen.contains(id))
        .map(|id| format!("  {id}  {}", known[id]))
        .collect();

    assert!(
        regressions.is_empty(),
        "{} statement(s) newly lose content when formatted. Each dropped token \
         was in the input and is not in the output.\n{}",
        regressions.len(),
        regressions.join("\n")
    );
    assert!(
        fixed.is_empty(),
        "{} statement(s) in known_token_loss.txt no longer lose content. Remove \
         them so the list keeps meaning something:\n{}",
        fixed.len(),
        fixed.join("\n")
    );
}
