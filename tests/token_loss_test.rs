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

use libpgfmt::{format, format_plpgsql, style::Style};

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
    let chars: Vec<char> = sql.chars().collect();
    let mut out = Vec::new();
    let mut word = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '$'
            && word.is_empty()
            && let Some((tag_end, close)) = dollar_quote(&chars, i)
        {
            // A dollar-quoted body is tokenized on its own, between its
            // delimiters. Quotes inside it are confined to it, and libpgfmt's
            // deliberate re-indentation of a body changes only whitespace.
            let delimiter: String = chars[i..tag_end].iter().collect();
            let body: String = chars[tag_end..close].iter().collect();
            out.push(delimiter.clone());
            out.extend(tokens(&body));
            out.push(delimiter);
            i = (close + tag_end - i).min(chars.len());
            continue;
        }
        if c == '\'' {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            let mut literal = String::from("'");
            i += 1;
            while i < chars.len() {
                literal.push(chars[i]);
                i += 1;
                if chars[i - 1] == '\'' {
                    break;
                }
            }
            out.push(literal);
            continue;
        }
        if c.is_alphanumeric() || c == '_' || c == '.' || c == '$' {
            word.push(c.to_ascii_lowercase());
        } else {
            if !word.is_empty() {
                out.push(std::mem::take(&mut word));
            }
            if !c.is_whitespace() && c != '"' {
                out.push(c.to_string());
            }
        }
        i += 1;
    }
    if !word.is_empty() {
        out.push(word);
    }
    out
}

/// For a dollar quote opening at `i`, the index just past the opening
/// delimiter and the index where the closing one starts. `None` when the `$`
/// is something else, such as a positional parameter `$1`.
fn dollar_quote(chars: &[char], i: usize) -> Option<(usize, usize)> {
    let mut j = i + 1;
    while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
        j += 1;
    }
    if j >= chars.len() || chars[j] != '$' || chars.get(i + 1).is_some_and(char::is_ascii_digit) {
        return None;
    }
    let tag = &chars[i..=j];
    let body = j + 1;
    let close = (body..=chars.len().saturating_sub(tag.len()))
        .find(|&k| &chars[k..k + tag.len()] == tag)
        .unwrap_or(chars.len());
    Some((body, close))
}

/// Rewrites that both sides get, so the deliberate normalizations the
/// formatter performs do not read as content loss.
///
/// Every entry here is a rewrite libpgfmt makes on purpose: it spells a type
/// alias out, writes an operator the way the SQL standard does, or writes a
/// CAST as `::`. Adding to
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

    let tokens = &rewrite_casts(tokens);
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

/// `CAST ( x AS t )` as `x :: t`, the spelling libpgfmt writes it in.
fn rewrite_casts(tokens: &[String]) -> Vec<String> {
    let mut out = Vec::with_capacity(tokens.len());
    let mut i = 0;
    while i < tokens.len() {
        if tokens[i] == "cast" && tokens.get(i + 1).is_some_and(|t| t == "(") {
            let mut depth = 0;
            let mut as_at = None;
            let mut close = None;
            for (j, token) in tokens.iter().enumerate().skip(i + 1) {
                match token.as_str() {
                    "(" => depth += 1,
                    ")" => {
                        depth -= 1;
                        if depth == 0 {
                            close = Some(j);
                            break;
                        }
                    }
                    "as" if depth == 1 && as_at.is_none() => as_at = Some(j),
                    _ => {}
                }
            }
            if let (Some(as_at), Some(close)) = (as_at, close) {
                out.extend(rewrite_casts(&tokens[i + 2..as_at]));
                out.push(":".to_string());
                out.push(":".to_string());
                out.extend(rewrite_casts(&tokens[as_at + 1..close]));
                i = close + 1;
                continue;
            }
        }
        out.push(tokens[i].clone());
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
    let mut changed = Vec::new();

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
            match known.get(&id) {
                None => regressions.push(format!(
                    "\n{id}\nInput:\n{sql}\nFormatted to:\n{formatted}\nDropped: {missing:?}"
                )),
                // A listed statement must lose exactly what its note names, so
                // a further loss in it is not hidden by the entry.
                Some(note) => {
                    let drops = format!("{missing:?}");
                    if note.split_once("drops: ").map(|(_, d)| d) != Some(drops.as_str()) {
                        changed.push(format!("  {id}  listed: {note}\n    now drops: {drops}"));
                    }
                }
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
        changed.is_empty(),
        "{} statement(s) in known_token_loss.txt now lose different content. \
         Fix the new loss, or update the `drops:` note if the change is \
         intended:\n{}",
        changed.len(),
        changed.join("\n")
    );
    assert!(
        fixed.is_empty(),
        "{} statement(s) in known_token_loss.txt no longer lose content. Remove \
         them so the list keeps meaning something:\n{}",
        fixed.len(),
        fixed.join("\n")
    );
}

/// Formatted output must parse. The token comparison cannot see a clause that
/// survives as text but no longer as SQL -- a newline dropped after a `--`
/// comment, so the comment swallows what follows, or a missing `;` between
/// two statements. Re-formatting the output catches both.
#[test]
fn formatted_corpus_reparses() {
    let mut failures = Vec::new();
    for (statement, sql) in corpus() {
        for &style in Style::ALL {
            let Ok(formatted) = format(&sql, style) else {
                continue;
            };
            if let Err(e) = format(&formatted, style) {
                failures.push(format!(
                    "\n{statement}:{style}\nInput:\n{sql}\nFormatted to:\n{formatted}\nWhich fails to parse: {e}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} formatted statement(s) no longer parse:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// Formatting is a fixed point. Content can survive the first pass and be lost
/// on the second: `FOR UPDATE LIMIT n` was rendered as `LIMIT n FOR UPDATE`,
/// which the next pass dropped the lock from (#67). Comparing the input with
/// one pass cannot see that.
#[test]
fn formatted_corpus_is_a_fixed_point() {
    let mut failures = Vec::new();
    for (statement, sql) in corpus() {
        for &style in Style::ALL {
            let Ok(once) = format(&sql, style) else {
                continue;
            };
            let Ok(twice) = format(&once, style) else {
                continue; // formatted_corpus_reparses reports this.
            };
            if once != twice {
                failures.push(format!(
                    "\n{statement}:{style}\nInput:\n{sql}\nFirst pass:\n{once}\nSecond pass:\n{twice}"
                ));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} formatted statement(s) change when formatted again:\n{}",
        failures.len(),
        failures.join("\n")
    );
}

/// The PL/pgSQL body of every corpus statement written in PL/pgSQL, as
/// `(id, body)`: the dollar-quoted body of a CREATE FUNCTION or PROCEDURE
/// that names `plpgsql`, or of a DO block. `format_plpgsql` formats these;
/// `format` only re-lays them out, so the statement tests above never reach
/// it.
fn plpgsql_bodies() -> Vec<(String, String)> {
    corpus()
        .into_iter()
        .filter_map(|(_, sql)| {
            let lower = sql.to_lowercase();
            if !lower.contains("plpgsql") && !lower.trim_start().starts_with("do") {
                return None;
            }
            let chars: Vec<char> = sql.chars().collect();
            let open = (0..chars.len()).find_map(|i| {
                (chars[i] == '$' && (i == 0 || !chars[i - 1].is_alphanumeric()))
                    .then(|| dollar_quote(&chars, i))
                    .flatten()
                    .map(|(body, close)| (i, body, close))
            })?;
            let (_, body_start, close) = open;
            let body: String = chars[body_start..close].iter().collect();
            let body = body.trim().to_string();
            body.to_lowercase()
                .contains("begin")
                .then(|| (id(&body), body))
        })
        .collect()
}

/// `format_plpgsql` keeps every token of a body, its output parses, and a
/// second pass changes nothing. See https://github.com/gmr/libpgfmt/issues/69.
#[test]
fn plpgsql_formatting_keeps_content_reparses_and_is_a_fixed_point() {
    let mut failures = Vec::new();
    for (body_id, body) in plpgsql_bodies() {
        for &style in Style::ALL {
            let Ok(once) = format_plpgsql(&body, style) else {
                continue;
            };
            let missing = dropped(&body, &once);
            if !missing.is_empty() {
                failures.push(format!(
                    "\n{body_id}:{style} drops {missing:?}\nInput:\n{body}\nFormatted to:\n{once}"
                ));
                continue;
            }
            match format_plpgsql(&once, style) {
                Err(e) => failures.push(format!(
                    "\n{body_id}:{style} no longer parses: {e}\nFormatted to:\n{once}"
                )),
                Ok(twice) if twice != once => failures.push(format!(
                    "\n{body_id}:{style} changes on a second pass\nFirst:\n{once}\nSecond:\n{twice}"
                )),
                Ok(_) => {}
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} PL/pgSQL formatting failure(s):\n{}",
        failures.len(),
        failures.join("\n")
    );
}
