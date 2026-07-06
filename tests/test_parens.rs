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
