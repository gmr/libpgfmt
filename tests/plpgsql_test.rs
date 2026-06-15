use libpgfmt::{format_plpgsql, style::Style};

// Regression: a plain `IF ... THEN ... END IF` previously hung forever because
// format_stmt_if never advanced past the closing `kw_if` of `END IF`.
#[test]
fn if_then_end_if_terminates() {
    let body = "BEGIN\n  IF x = 1\n  THEN\n    v := y;\n  END IF;\nEND";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    let expected = "\
BEGIN
  IF x = 1 THEN
    v := y;
  END IF;
END;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

#[test]
fn if_elsif_else_terminates() {
    let body = "BEGIN\n  IF a THEN\n    v := 1;\n  ELSIF b THEN\n    v := 2;\n  ELSE\n    v := 3;\n  END IF;\nEND";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    let expected = "\
BEGIN
  IF a THEN
    v := 1;
  ELSIF b THEN
    v := 2;
  ELSE
    v := 3;
  END IF;
END;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}
