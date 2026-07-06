use libpgfmt::{format, style::Style};

#[test]
fn preserve_parens_around_or_in_and() {
    let sql = "SELECT 1 FROM t WHERE (a IS NULL OR b > 1) AND c = 'x'";
    let result = format(sql, Style::River).unwrap();
    assert!(
        result.contains("(a IS NULL OR b > 1)"),
        "Parentheses around OR were dropped:\n{result}"
    );
}

#[test]
fn no_unnecessary_parens() {
    let sql = "SELECT 1 FROM t WHERE a = 1 AND b = 2";
    let result = format(sql, Style::River).unwrap();
    assert!(!result.contains('('), "Unexpected parens added:\n{result}");
}

#[test]
fn preserve_adjacent_parens() {
    let sql = "SELECT 1 FROM t WHERE (a = 1) AND (b = 2)";
    let result = format(sql, Style::River).unwrap();
    assert!(
        result.contains("(a = 1)") && result.contains("(b = 2)"),
        "Adjacent parens corrupted:\n{result}"
    );
}

// https://github.com/gmr/pgfmt/issues/24 — rewriting CAST(x AS t) to x::t must
// keep parens around a compound operand, because :: binds tighter than
// arithmetic. Dropping them changes what gets cast.
#[test]
fn cast_wraps_compound_operand_with_function_call() {
    let sql = "SELECT CAST(foo(x) + 1 AS integer);";
    let result = format(sql, Style::River).unwrap();
    assert!(
        result.contains("(foo(x) + 1)::INTEGER"),
        "CAST dropped required parens:\n{result}"
    );
}

#[test]
fn cast_leaves_bare_function_call_unparenthesized() {
    let sql = "SELECT CAST(foo(x) AS integer);";
    let result = format(sql, Style::River).unwrap();
    assert!(
        result.contains("foo(x)::INTEGER") && !result.contains("(foo(x))::INTEGER"),
        "CAST over-parenthesized a simple operand:\n{result}"
    );
}

// A field selection on a function-call result needs the call parenthesized:
// `(foo(x)).bar` selects field `bar` from the composite row, whereas
// `foo(x).bar` is invalid. The base already contains `(`, so the re-wrap must
// key off outer enclosure, not the mere presence of a paren.
#[test]
fn field_selection_wraps_function_call_result() {
    let sql = "SELECT (foo(x)).bar;";
    let result = format(sql, Style::River).unwrap();
    assert!(
        result.contains("(foo(x)).bar"),
        "Function-call field selection lost its parens:\n{result}"
    );
}

// An E'...' escape string carries backslash escapes; a \' inside it is not a
// terminator. The CAST-to-:: rewrite must not misread the trailing bytes as a
// top-level compound operand and wrap the literal in needless parens.
#[test]
fn cast_leaves_escape_string_unparenthesized() {
    let sql = "SELECT CAST(E'a\\'s value' AS text);";
    let result = format(sql, Style::River).unwrap();
    assert!(
        !result.contains("(E'"),
        "CAST over-parenthesized an escape-string operand:\n{result}"
    );
}

// Formats `sql` in the Aweber style and asserts that (1) `expected_substr`
// survives, (2) reformatting the output is idempotent, and (3) no line carries
// trailing whitespace. Shared by the column-level CHECK regression tests.
fn assert_idempotent_and_clean(sql: &str, expected_substr: &str) {
    let once = format(sql, Style::Aweber).unwrap();
    assert!(
        once.contains(expected_substr),
        "Expected substring {expected_substr:?} missing:\n{once}"
    );
    let twice = format(&once, Style::Aweber).unwrap();
    assert_eq!(
        once.trim(),
        twice.trim(),
        "Re-formatting was not idempotent:\n{twice}"
    );
    for line in once.lines() {
        assert_eq!(
            line.trim_end(),
            line,
            "Line has trailing whitespace: {line:?}\n{once}"
        );
    }
}

// https://github.com/gmr/pgfmt/issues/11 — a column-level CHECK constraint
// requires its expression to stay parenthesized. Dropping the parens emits
// invalid SQL that mis-parses on the next pass and corrupts the table.
#[test]
fn preserve_col_check_parens_and_idempotent() {
    let sql = "CREATE TABLE etudiant (
    email_etudiant VARCHAR(50) PRIMARY KEY CHECK (email_etudiant LIKE '_%@_%._%'),
    nom_etudiant VARCHAR(50) NOT NULL,
    date_naissance DATE NOT NULL,
    code_postal CHAR(5)
);";
    assert_idempotent_and_clean(sql, "CHECK (email_etudiant LIKE '_%@_%._%')");
}

// render_aligned_column also drives CREATE FOREIGN TABLE, so the same
// column-level CHECK parenthesization must hold there. See gmr/pgfmt#11.
#[test]
fn preserve_foreign_col_check_parens_and_idempotent() {
    let sql = "CREATE FOREIGN TABLE etudiant (
    email_etudiant VARCHAR(50) CHECK (email_etudiant LIKE '_%@_%._%'),
    nom_etudiant VARCHAR(50) NOT NULL,
    date_naissance DATE NOT NULL,
    code_postal CHAR(5)
) SERVER remote_server;";
    assert_idempotent_and_clean(sql, "CHECK (email_etudiant LIKE '_%@_%._%')");
}
