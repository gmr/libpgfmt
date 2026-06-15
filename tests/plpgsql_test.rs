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

// Regression: declarations using multi-word type names, DEFAULT, and ALIAS FOR
// a positional parameter previously failed to parse (grammar gaps), and the
// ALIAS form dropped its target when formatting.
#[test]
fn declarations_types_default_alias() {
    let body = "DECLARE\n  a character varying(50);\n  b double precision;\n  c timestamp with time zone;\n  d integer DEFAULT 0;\n  username ALIAS FOR $1;\nBEGIN\n  NULL;\nEND";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    let expected = "\
DECLARE
  a character varying(50);
  b double precision;
  c timestamp with time zone;
  d integer DEFAULT 0;
  username ALIAS FOR $1;
BEGIN
  NULL;
END;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

// Regression: `RETURN NEXT` (bare) failed to parse, and the formatter dropped
// the NEXT keyword.
#[test]
fn return_next_bare() {
    let body = "BEGIN\n  RETURN NEXT;\nEND";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    assert_eq!(result, "BEGIN\n  RETURN NEXT;\nEND;", "\nGot:\n{result}");
}

// Regression: a FOR loop over a query dropped the query text after IN.
#[test]
fn for_over_query_keeps_query() {
    let body = "BEGIN\n  FOR r IN SELECT id FROM t LOOP\n    RETURN NEXT r;\n  END LOOP;\nEND";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    assert!(
        result.contains("FOR r IN SELECT id FROM t LOOP"),
        "query dropped from FOR clause:\n{result}"
    );
    assert!(result.contains("RETURN NEXT r;"), "\nGot:\n{result}");
}

// Regression: format_plpgsql falls back to SQL formatting when the body is not
// PL/pgSQL (e.g. a LANGUAGE sql function body), rather than erroring.
#[test]
fn sql_body_fallback() {
    let body = "WITH t AS (SELECT 1 AS n) SELECT n FROM t";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    assert!(result.contains("SELECT"), "\nGot:\n{result}");
    assert!(result.trim_end().ends_with(';'), "\nGot:\n{result}");
}
