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
