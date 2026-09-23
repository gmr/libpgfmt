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

// Regression (#26): the EXCEPTION handler body was dropped, leaving a bare
// EXCEPTION keyword, because the code looked for proc_conditions/proc_sect as
// direct children of exception_sect instead of inside each proc_exception node.
#[test]
fn exception_handler_body_preserved() {
    let body = "BEGIN NULL; EXCEPTION WHEN OTHERS THEN RAISE; END";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    let expected = "\
BEGIN
  NULL;
EXCEPTION
  WHEN OTHERS THEN
    RAISE;
END;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

// Regression (#32): RAISE ... USING options (ERRCODE, MESSAGE, ...) were
// dropped because only raise_level/string_literal/sql_expression were matched.
#[test]
fn raise_using_options_preserved() {
    let body = "BEGIN RAISE EXCEPTION 'bad %', x USING ERRCODE = '22000', MESSAGE = 'boom'; END";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    let expected = "\
BEGIN
  RAISE EXCEPTION 'bad %', x USING ERRCODE = '22000', MESSAGE = 'boom';
END;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

// Regression (#32): additional RAISE ... USING option keywords (DETAIL, HINT)
// are preserved alongside ERRCODE.
#[test]
fn raise_using_detail_hint_options_preserved() {
    let body = "BEGIN RAISE EXCEPTION 'bad' USING DETAIL = 'd', HINT = 'h', ERRCODE = '22000'; END";
    let result = format_plpgsql(body, Style::Aweber).unwrap();
    let expected = "\
BEGIN
  RAISE EXCEPTION 'bad' USING DETAIL = 'd', HINT = 'h', ERRCODE = '22000';
END;";
    assert_eq!(result, expected, "\nGot:\n{result}");
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

// https://github.com/gmr/libpgfmt/issues/69 and #70: block labels, end labels,
// compiler directives, a bound cursor's query and comments are kept, and a
// nested block ends with `;`. The output parses and is a fixed point.
#[test]
fn plpgsql_keeps_labels_directives_cursors_comments_and_nested_terminators() {
    let body = "#variable_conflict use_variable\n<<outerblock>>\nDECLARE\n    c CURSOR IS SELECT * FROM t ORDER BY a;\nBEGIN\n    BEGIN\n        -- standalone\n        NULL; -- inner\n    END;\n    COMMIT;\nEND outerblock";
    for &style in Style::ALL {
        let once = format_plpgsql(body, style).unwrap();
        for piece in [
            "#variable_conflict use_variable",
            "<<outerblock>>",
            "CURSOR IS SELECT * FROM t ORDER BY a;",
            "-- inner",
            "-- standalone",
            " outerblock;",
        ] {
            assert!(
                once.contains(piece) || once.to_lowercase().contains(&piece.to_lowercase()),
                "\nStyle: {style}\nmissing {piece:?} in:\n{once}"
            );
        }
        let twice = format_plpgsql(&once, style).unwrap_or_else(|e| {
            panic!("\nStyle: {style}\nFormatted to:\n{once}\nWhich fails: {e}")
        });
        assert_eq!(once, twice, "\nStyle: {style}");
    }
}

// A bound cursor's query that ends in a `--` comment keeps its `;` on the next
// line, out of the comment, so the output still parses.
#[test]
fn plpgsql_cursor_query_ending_in_line_comment_keeps_terminator() {
    let body = "DECLARE\n    c CURSOR FOR SELECT 1 -- note\n;\nBEGIN\n    OPEN c;\nEND";
    for &style in Style::ALL {
        let once = format_plpgsql(body, style).unwrap();
        assert!(
            once.contains("-- note\n;"),
            "\nStyle: {style}\nGot:\n{once}"
        );
        let twice = format_plpgsql(&once, style).unwrap_or_else(|e| {
            panic!("\nStyle: {style}\nFormatted to:\n{once}\nWhich fails: {e}")
        });
        assert_eq!(once, twice, "\nStyle: {style}");
    }
}
