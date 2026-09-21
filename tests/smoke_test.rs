use libpgfmt::error::FormatError;
use libpgfmt::{format, style::Style};

#[test]
fn select_simple_river() {
    let sql = "SELECT file_hash FROM file_system WHERE file_name = '.vimrc'";
    let result = format(sql, Style::River).unwrap();
    let expected = "\
SELECT file_hash
  FROM file_system
 WHERE file_name = '.vimrc';";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

#[test]
fn select_multi_col_or_river() {
    let sql = "SELECT a.title, a.released_on, a.recorded_on FROM albums AS a WHERE a.title = 'Charcoal Lane' OR a.title = 'The New Danger'";
    let result = format(sql, Style::River).unwrap();
    let expected = "\
SELECT a.title,
       a.released_on,
       a.recorded_on
  FROM albums AS a
 WHERE a.title = 'Charcoal Lane'
    OR a.title = 'The New Danger';";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

#[test]
fn delete_simple_river() {
    let sql = "DELETE FROM albums WHERE id = 1";
    let result = format(sql, Style::River).unwrap();
    let expected = "\
DELETE
  FROM albums
 WHERE id = 1;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

#[test]
fn update_simple_river() {
    let sql = "UPDATE albums SET release_date = '1990-01-01 01:01:01.00000' WHERE title = 'The New Danger'";
    let result = format(sql, Style::River).unwrap();
    let expected = "\
UPDATE albums
   SET release_date = '1990-01-01 01:01:01.00000'
 WHERE title = 'The New Danger';";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

#[test]
fn select_simple_mozilla() {
    let sql = "SELECT client_id, submission_date FROM main_summary WHERE submission_date > '20180101' AND sample_id = '42' LIMIT 10";
    let result = format(sql, Style::Mozilla).unwrap();
    let expected = "\
SELECT
    client_id,
    submission_date
FROM main_summary
WHERE
    submission_date > '20180101'
    AND sample_id = '42'
LIMIT 10;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

#[test]
fn select_simple_dbt() {
    let sql = "SELECT client_id, submission_date FROM main_summary WHERE submission_date > '20180101' AND sample_id = '42' LIMIT 10";
    let result = format(sql, Style::Dbt).unwrap();
    let expected = "\
select
    client_id,
    submission_date

from main_summary

where
    submission_date > '20180101'
    and sample_id = '42'

limit 10;";
    assert_eq!(result, expected, "\nGot:\n{result}");
}

// Regression for https://github.com/gmr/pgfmt/issues/7: typed string
// literals (INTERVAL/DATE/TIMESTAMP '...') must keep their literal value.
#[test]
fn typed_literal_constants_river() {
    let cases = [
        ("SELECT INTERVAL '2 days'", "SELECT INTERVAL '2 days';"),
        ("select interval '3 days'", "SELECT INTERVAL '3 days';"),
        ("SELECT DATE '2020-01-01'", "SELECT DATE '2020-01-01';"),
        (
            "SELECT TIMESTAMP '2020-01-01 10:00'",
            "SELECT TIMESTAMP '2020-01-01 10:00';",
        ),
        ("SELECT INTERVAL '2' DAY", "SELECT INTERVAL '2' DAY;"),
        (
            "SELECT INTERVAL(6) '2 days'",
            "SELECT INTERVAL(6) '2 days';",
        ),
        (
            "SELECT INTERVAL '1' HOUR TO MINUTE",
            "SELECT INTERVAL '1' HOUR TO MINUTE;",
        ),
    ];
    for (sql, expected) in cases {
        let result = format(sql, Style::River).unwrap();
        assert_eq!(result, expected, "\nInput: {sql}\nGot:\n{result}");
    }
}

// Regression for https://github.com/gmr/libpgfmt/issues/16: a broken
// statement in multi-statement input must surface as a syntax error rather
// than being silently dropped while the parseable statements are returned.
#[test]
fn broken_statement_errors_instead_of_dropping() {
    let result = format("SELECT 1; THIS IS NOT SQL @@@;", Style::River);
    assert!(
        matches!(result, Err(FormatError::Syntax(_))),
        "expected Err(FormatError::Syntax(..)), got {result:?}"
    );
}

// PG19: RESPECT/IGNORE NULLS null-treatment on window functions must be
// preserved rather than silently dropped.
#[test]
fn window_null_treatment_river() {
    let cases = [
        (
            "SELECT first_value(price) respect nulls over (order by ts) FROM t",
            "SELECT FIRST_VALUE(price) RESPECT NULLS OVER (ORDER BY ts)\n  FROM t;",
        ),
        (
            "SELECT lag(price) ignore nulls over (order by ts) FROM t",
            "SELECT LAG(price) IGNORE NULLS OVER (ORDER BY ts)\n  FROM t;",
        ),
    ];
    for (sql, expected) in cases {
        let result = format(sql, Style::River).unwrap();
        assert_eq!(result, expected, "\nInput: {sql}\nGot:\n{result}");
    }
}

// Aggregate FILTER (WHERE ...) must be preserved and keyword-cased.
#[test]
fn aggregate_filter_clause() {
    let sql = "SELECT count(*) filter (where active) FROM users";
    assert_eq!(
        format(sql, Style::River).unwrap(),
        "SELECT COUNT(*) FILTER (WHERE active)\n  FROM users;"
    );
    // Left-aligned, lowercase-keyword style.
    assert_eq!(
        format(sql, Style::Dbt).unwrap(),
        "select count(*) filter (where active)\n\nfrom users;"
    );
}

// Ordered-set aggregate WITHIN GROUP (ORDER BY ...) must be preserved.
#[test]
fn aggregate_within_group_clause() {
    let sql = "SELECT percentile_cont(0.5) within group (order by score desc) FROM t";
    assert_eq!(
        format(sql, Style::River).unwrap(),
        "SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY score DESC)\n  FROM t;"
    );
    // Leading-comma, lowercase-keyword style.
    assert_eq!(
        format(sql, Style::Mattmc3).unwrap(),
        "select percentile_cont(0.5) within group (order by score desc)\n  from t;"
    );
}

// WITHIN GROUP, FILTER, and OVER can co-occur and must render in grammar order.
#[test]
fn aggregate_within_group_filter_over() {
    let sql =
        "SELECT rank() within group (order by a) filter (where b) over (partition by c) FROM t";
    assert_eq!(
        format(sql, Style::River).unwrap(),
        "SELECT RANK() WITHIN GROUP (ORDER BY a) FILTER (WHERE b) OVER (PARTITION BY c)\n  FROM t;"
    );
}

#[test]
fn cast_multiword_type_names() {
    // Regression: multi-word type names in :: casts must not be truncated
    // (e.g. `::character varying` previously dropped `varying`).
    let cases = [
        (
            "SELECT a::character varying",
            "SELECT a::CHARACTER VARYING;",
        ),
        ("SELECT a::varchar(50)", "SELECT a::VARCHAR(50);"),
        ("SELECT a::char(10)", "SELECT a::CHAR(10);"),
        (
            "SELECT a::character varying(50)",
            "SELECT a::CHARACTER VARYING(50);",
        ),
        ("SELECT a::double precision", "SELECT a::DOUBLE PRECISION;"),
        ("SELECT a::bit varying(8)", "SELECT a::BIT VARYING(8);"),
        (
            "SELECT a::timestamp with time zone",
            "SELECT a::TIMESTAMP WITH TIME ZONE;",
        ),
    ];
    for (sql, expected) in cases {
        let result = format(sql, Style::River).unwrap();
        assert_eq!(result, expected, "\nInput: {sql}\nGot:\n{result}");
    }
}

// Column-level constraint clauses were dropped or mangled: COLLATE, the
// ConstraintAttr attributes, REFERENCES key actions, IDENTITY sequence
// options, the generated-expression parentheses, and NULLS NOT DISTINCT.
#[test]
fn column_constraint_clauses_preserved() {
    let cases = [
        (
            "CREATE TABLE t (b text COLLATE \"C\" NOT NULL)",
            "CREATE TABLE t (\n    b TEXT COLLATE \"C\" NOT NULL\n);",
        ),
        (
            "CREATE TABLE t (a int REFERENCES u (id) ON DELETE SET NULL (a) ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED)",
            "CREATE TABLE t (\n    a INTEGER REFERENCES u (id) ON DELETE SET NULL (a) ON UPDATE CASCADE DEFERRABLE INITIALLY DEFERRED\n);",
        ),
        (
            "CREATE TABLE t (a int GENERATED ALWAYS AS IDENTITY (START WITH 1 INCREMENT BY 2))",
            "CREATE TABLE t (\n    a INTEGER GENERATED ALWAYS AS IDENTITY (START WITH 1 INCREMENT BY 2)\n);",
        ),
        (
            "CREATE TABLE t (a int GENERATED ALWAYS AS (b + 1) STORED)",
            "CREATE TABLE t (\n    a INTEGER GENERATED ALWAYS AS (b + 1) STORED\n);",
        ),
        (
            "CREATE TABLE t (a int UNIQUE NULLS NOT DISTINCT)",
            "CREATE TABLE t (\n    a INTEGER UNIQUE NULLS NOT DISTINCT\n);",
        ),
        (
            "CREATE TABLE t (a int CHECK (a > 0) NO INHERIT)",
            "CREATE TABLE t (\n    a INTEGER CHECK (a > 0) NO INHERIT\n);",
        ),
    ];
    for (sql, expected) in cases {
        let result = format(sql, Style::River).unwrap();
        assert_eq!(result, expected, "\nInput: {sql}\nGot:\n{result}");
    }
}

// Table-level constraint bodies were dropped or emitted as invalid SQL:
// EXCLUDE lost its access method, operator list, and parentheses; WITHOUT
// OVERLAPS and PERIOD were truncated or moved outside the key parentheses.
#[test]
fn table_constraint_clauses_preserved() {
    let cases = [
        (
            "CREATE TABLE t (a int, EXCLUDE USING gist (a WITH =) WHERE (a > 0))",
            "CREATE TABLE t (\n    a INTEGER,\n      EXCLUDE USING gist (a WITH =) WHERE (a > 0)\n);",
        ),
        (
            "CREATE TABLE t (a int, b int, UNIQUE NULLS NOT DISTINCT (a, b) INCLUDE (b))",
            "CREATE TABLE t (\n    a INTEGER,\n    b INTEGER,\n      UNIQUE NULLS NOT DISTINCT (a, b) INCLUDE (b)\n);",
        ),
        (
            "CREATE TABLE t (a int, FOREIGN KEY (a) REFERENCES u (id) ON DELETE CASCADE DEFERRABLE)",
            "CREATE TABLE t (\n    a INTEGER,\n      FOREIGN KEY (a) REFERENCES u (id) ON DELETE CASCADE DEFERRABLE\n);",
        ),
    ];
    for (sql, expected) in cases {
        let result = format(sql, Style::River).unwrap();
        assert_eq!(result, expected, "\nInput: {sql}\nGot:\n{result}");
    }

    // WITHOUT OVERLAPS and PERIOD belong inside the key's parentheses. River
    // style hoists PRIMARY KEY above the columns.
    let sql = "CREATE TABLE v (id int, valid_at daterange, product_no int, \
               PRIMARY KEY (id, valid_at WITHOUT OVERLAPS), \
               FOREIGN KEY (product_no, PERIOD valid_at) REFERENCES p (product_no, PERIOD valid_at))";
    let result = format(sql, Style::River).unwrap();
    assert!(
        result.contains("PRIMARY KEY (id, valid_at WITHOUT OVERLAPS)"),
        "\nGot:\n{result}"
    );
    assert!(
        result.contains(
            "FOREIGN KEY (product_no, PERIOD valid_at) REFERENCES p (product_no, PERIOD valid_at)"
        ),
        "\nGot:\n{result}"
    );
}

// Every CREATE FUNCTION option except LANGUAGE and AS was silently dropped,
// along with OR REPLACE, RETURNS TABLE, and SQL-standard routine bodies.
#[test]
fn create_function_options_preserved() {
    let sql = "CREATE FUNCTION f(a int DEFAULT 1) RETURNS TABLE (x int, y text) \
               LANGUAGE sql STRICT SECURITY DEFINER PARALLEL SAFE COST 100 ROWS 10 \
               SET search_path TO 'public' AS $$ SELECT 1 $$";
    assert_eq!(
        format(sql, Style::River).unwrap(),
        "\
CREATE FUNCTION f(a int DEFAULT 1) RETURNS TABLE (x INTEGER, y TEXT)
    LANGUAGE sql
    STRICT
    SECURITY DEFINER
    PARALLEL SAFE
    COST 100
    ROWS 10
    SET search_path TO 'public'
    AS $$
 SELECT 1
$$;"
    );

    let replace = "CREATE OR REPLACE FUNCTION f() RETURNS int LANGUAGE sql \
                   IMMUTABLE LEAKPROOF RETURNS NULL ON NULL INPUT AS $$ SELECT 1 $$";
    let result = format(replace, Style::River).unwrap();
    assert!(
        result.starts_with("CREATE OR REPLACE FUNCTION"),
        "\nGot:\n{result}"
    );
    for opt in ["IMMUTABLE", "LEAKPROOF", "RETURNS NULL ON NULL INPUT"] {
        assert!(result.contains(opt), "missing {opt}\nGot:\n{result}");
    }
}

// SQL-standard routine bodies: BEGIN ATOMIC blocks format their statements
// per style, and RETURN bodies are kept.
#[test]
fn create_function_routine_body_preserved() {
    assert_eq!(
        format(
            "CREATE FUNCTION g() RETURNS text LANGUAGE sql BEGIN ATOMIC SELECT note FROM c WHERE color = 1; END",
            Style::River
        )
        .unwrap(),
        "\
CREATE FUNCTION g() RETURNS TEXT
    LANGUAGE sql
    BEGIN ATOMIC
        SELECT note
          FROM c
         WHERE color = 1;
    END;"
    );

    assert_eq!(
        format(
            "CREATE FUNCTION f(x int) RETURNS int LANGUAGE sql RETURN x + 1",
            Style::River
        )
        .unwrap(),
        "CREATE FUNCTION f(x int) RETURNS INTEGER\n    LANGUAGE sql\n    RETURN x + 1;"
    );
}

// Table-level CREATE TABLE clauses were dropped: UNLOGGED/TEMP, the access
// method, ON COMMIT, and the whole PARTITION OF / OF-type form, which
// collapsed to an empty `CREATE TABLE t ()`.
#[test]
fn create_table_level_options_preserved() {
    let cases = [
        (
            "CREATE UNLOGGED TABLE IF NOT EXISTS t (a int) ON COMMIT DROP",
            "CREATE UNLOGGED TABLE IF NOT EXISTS t (\n    a INTEGER\n)\nON COMMIT DROP;",
        ),
        (
            "CREATE TEMP TABLE t (a int) ON COMMIT DELETE ROWS",
            "CREATE TEMP TABLE t (\n    a INTEGER\n)\nON COMMIT DELETE ROWS;",
        ),
        (
            "CREATE TABLE t (a int) USING heap",
            "CREATE TABLE t (\n    a INTEGER\n)\nUSING heap;",
        ),
        (
            "CREATE TABLE t PARTITION OF u FOR VALUES FROM (1) TO (2)",
            "CREATE TABLE t PARTITION OF u\nFOR VALUES FROM (1) TO (2);",
        ),
        (
            "CREATE TABLE t PARTITION OF u DEFAULT",
            "CREATE TABLE t PARTITION OF u\nDEFAULT;",
        ),
        (
            "CREATE TABLE t PARTITION OF u FOR VALUES WITH (MODULUS 4, REMAINDER 0)",
            "CREATE TABLE t PARTITION OF u\nFOR VALUES WITH (MODULUS 4, REMAINDER 0);",
        ),
        (
            "CREATE TABLE t OF ty (PRIMARY KEY (a))",
            "CREATE TABLE t OF ty (\n    PRIMARY KEY (a)\n);",
        ),
    ];
    for (sql, expected) in cases {
        let result = format(sql, Style::River).unwrap();
        assert_eq!(result, expected, "\nInput: {sql}\nGot:\n{result}");
    }
}

// No style may emit trailing whitespace. LIKE clauses and typed-table
// elements previously rendered as untyped columns and were padded.
#[test]
fn no_trailing_whitespace_on_table_elements() {
    let cases = [
        "CREATE TABLE t (LIKE u INCLUDING ALL) INHERITS (v) TABLESPACE ts",
        "CREATE TABLE t OF ty (PRIMARY KEY (a))",
        "CREATE TABLE t (a int, EXCLUDE USING gist (a WITH =))",
        // dbt separates SELECT clauses with a blank line; indenting it into
        // a routine body must leave it genuinely blank.
        "CREATE FUNCTION g() RETURNS text LANGUAGE sql BEGIN ATOMIC SELECT a FROM c; END",
    ];
    for &style in Style::ALL {
        for sql in cases {
            let result = format(sql, style).unwrap();
            for line in result.lines() {
                assert_eq!(
                    line.trim_end(),
                    line,
                    "\nStyle: {style}\nInput: {sql}\nGot:\n{result}"
                );
            }
        }
    }
}

// Function options and routine bodies must follow the style's indent width
// rather than a hardcoded four spaces, and typed-table elements must be
// keyword-cased rather than echoed as raw source.
#[test]
fn function_and_typed_table_respect_style() {
    // GitLab indents with two spaces.
    assert_eq!(
        format(
            "CREATE FUNCTION g() RETURNS text LANGUAGE sql STRICT BEGIN ATOMIC SELECT a FROM c; END",
            Style::Gitlab
        )
        .unwrap(),
        "\
CREATE FUNCTION g() RETURNS TEXT
  LANGUAGE sql
  STRICT
  BEGIN ATOMIC
    SELECT a
    FROM c;
  END;"
    );

    // dbt lowercases keywords, including inside typed-table elements.
    assert_eq!(
        format(
            "CREATE TABLE t OF ty (PRIMARY KEY (a), b WITH OPTIONS NOT NULL)",
            Style::Dbt
        )
        .unwrap(),
        "\
create table t of ty (
    primary key (a),
    b with options not null
);"
    );
}
// Regression for https://github.com/gmr/libpgfmt/issues/45: a stray token
// outside any statement parses as a short ERROR node at the root, which the
// size threshold tolerated and formatting then discarded. Junk before,
// after, or between statements must error rather than vanish.
#[test]
fn junk_outside_statements_errors_instead_of_dropping() {
    let cases = [
        "1 SELECT random();",
        "x SELECT random();",
        "!!! SELECT 1;",
        "SELECT random() 1;",
        "SELECT 1; 2 SELECT 2;",
    ];
    for sql in cases {
        let result = format(sql, Style::River);
        assert!(
            matches!(result, Err(FormatError::Syntax(_))),
            "expected Err(FormatError::Syntax(..)) for {sql:?}, got {result:?}"
        );
    }

    // Valid input is unaffected, including the decimal literals the size
    // threshold was originally added for.
    let valid = [
        ("SELECT random();", "SELECT random();"),
        ("SELECT 800.00;", "SELECT 800.00;"),
        ("SELECT 1; SELECT 2;", "SELECT 1;\n\nSELECT 2;"),
    ];
    for (sql, expected) in valid {
        assert_eq!(
            format(sql, Style::River).unwrap(),
            expected,
            "\nInput: {sql}"
        );
    }
}

// Regression for https://github.com/gmr/libpgfmt/issues/49: MERGE had no arm
// in format_stmt, so it fell through to the verbatim passthrough — the only
// DML statement that was neither laid out nor keyword-cased.
#[test]
fn merge_statement_river_layout() {
    let sql = "merge into customer_account ca using recent_transactions t \
               on t.customer_id = ca.customer_id \
               when matched then update set balance = balance + transaction_value \
               when not matched then insert (customer_id, balance) \
               values (t.customer_id, t.transaction_value)";
    assert_eq!(
        format(sql, Style::River).unwrap(),
        "\
MERGE INTO customer_account AS ca
     USING recent_transactions AS t
        ON t.customer_id = ca.customer_id
      WHEN MATCHED THEN
           UPDATE SET balance = balance + transaction_value
      WHEN NOT MATCHED THEN
           INSERT (customer_id, balance)
           VALUES (t.customer_id, t.transaction_value);"
    );

    // Left-aligned styles use their own indent for the action.
    assert_eq!(
        format(
            "MERGE INTO t USING u ON t.id = u.id WHEN MATCHED THEN UPDATE SET a = 1",
            Style::Gitlab
        )
        .unwrap(),
        "\
MERGE INTO t
USING u
ON t.id = u.id
WHEN MATCHED THEN
  UPDATE SET a = 1;"
    );
}

// Every merge_when_clause shape, the WHEN qualifiers, and the surrounding
// clauses must survive and be cased.
#[test]
fn merge_statement_all_clause_shapes() {
    let head = "MERGE INTO t USING u ON t.id = u.id ";
    let cases = [
        (
            "WHEN MATCHED THEN DELETE",
            "WHEN MATCHED THEN\n           DELETE",
        ),
        (
            "WHEN MATCHED THEN DO NOTHING",
            "WHEN MATCHED THEN\n           DO NOTHING",
        ),
        (
            "WHEN MATCHED AND u.x > 0 THEN UPDATE SET a = 1",
            "WHEN MATCHED AND u.x > 0 THEN\n           UPDATE SET a = 1",
        ),
        (
            "WHEN NOT MATCHED THEN INSERT VALUES (1)",
            "WHEN NOT MATCHED THEN\n           INSERT VALUES (1)",
        ),
        (
            "WHEN NOT MATCHED THEN INSERT DEFAULT VALUES",
            "WHEN NOT MATCHED THEN\n           INSERT DEFAULT VALUES",
        ),
        (
            "WHEN NOT MATCHED THEN INSERT OVERRIDING SYSTEM VALUE VALUES (1)",
            "WHEN NOT MATCHED THEN\n           INSERT OVERRIDING SYSTEM VALUE\n           VALUES (1)",
        ),
        (
            "WHEN NOT MATCHED BY SOURCE THEN DELETE",
            "WHEN NOT MATCHED BY SOURCE THEN\n           DELETE",
        ),
        // NOT MATCHED [BY TARGET] admits only INSERT or DO NOTHING.
        (
            "WHEN NOT MATCHED BY TARGET THEN INSERT VALUES (1)",
            "WHEN NOT MATCHED BY TARGET THEN\n           INSERT VALUES (1)",
        ),
    ];
    for (when, expected) in cases {
        let result = format(&format!("{head}{when}"), Style::River).unwrap();
        assert!(
            result.contains(expected),
            "\nInput: {when}\nExpected to contain:\n{expected}\nGot:\n{result}"
        );
    }

    // A CTE, a subquery source, a split ON condition, and RETURNING.
    let full = "WITH c AS (SELECT 1 AS id) MERGE INTO t USING (SELECT 1 AS id) s \
                ON t.id = s.id AND t.k = s.k WHEN MATCHED THEN DELETE \
                RETURNING merge_action(), t.id";
    let result = format(full, Style::River).unwrap();
    for fragment in [
        "WITH c AS (",
        "USING (SELECT 1 AS id) AS s",
        "        ON t.id = s.id",
        "       AND t.k = s.k",
        " RETURNING MERGE_ACTION(), t.id",
    ] {
        assert!(
            result.contains(fragment),
            "missing {fragment:?}\nGot:\n{result}"
        );
    }
}

// Formatting must be idempotent for every style. An alias written with an
// explicit AS previously doubled the keyword on each pass — `t AS x` became
// `t AS AS x`, then `t AS AS AS`, which no longer parses.
#[test]
fn relation_alias_is_idempotent() {
    let cases = [
        "UPDATE t AS x SET a = 1",
        "UPDATE t x SET a = 1",
        "DELETE FROM t AS x",
        "MERGE INTO products AS p USING n ON p.id = n.id WHEN MATCHED THEN DELETE",
    ];
    for &style in Style::ALL {
        for sql in cases {
            let once = format(sql, style).unwrap();
            let twice = format(&once, style).unwrap();
            assert_eq!(once, twice, "\nStyle: {style}\nInput: {sql}");
            assert!(
                !once.contains("AS AS"),
                "\nStyle: {style}\nInput: {sql}\nGot:\n{once}"
            );
        }
    }
}

// Zero-argument keyword functions keep their parentheses tight.
#[test]
fn zero_arg_keyword_functions() {
    assert_eq!(
        format("SELECT merge_action()", Style::River).unwrap(),
        "SELECT MERGE_ACTION();"
    );
    assert_eq!(
        format("SELECT current_user", Style::River).unwrap(),
        "SELECT CURRENT_USER;"
    );
}

// Regression for https://github.com/gmr/libpgfmt/issues/51: INSERT, UPDATE
// and DELETE never read opt_with_clause, so a leading WITH was discarded and
// the output referenced a CTE that no longer existed — invalid SQL, returned
// as Ok.
#[test]
fn dml_with_clause_preserved() {
    let cases = [
        "WITH c AS (SELECT 1 AS id) INSERT INTO t SELECT id FROM c",
        "WITH c AS (SELECT 1 AS id) UPDATE t SET a = 1 FROM c WHERE t.id = c.id",
        "WITH c AS (SELECT 1 AS id) DELETE FROM t USING c WHERE t.id = c.id",
        "WITH c AS (SELECT 1 AS id) MERGE INTO t USING c ON t.id = c.id WHEN MATCHED THEN DELETE",
        "WITH c AS (SELECT 1 AS id) SELECT * FROM c",
    ];
    for &style in Style::ALL {
        for sql in cases {
            let result = format(sql, style).unwrap();
            let upper = result.to_uppercase();
            // dbt puts WITH on its own line, so check the parts rather than
            // the contiguous phrase.
            assert!(
                upper.contains("WITH") && upper.contains("C AS ("),
                "CTE dropped\nStyle: {style}\nInput: {sql}\nGot:\n{result}"
            );
            // The formatted statement must still parse.
            format(&result, style).unwrap();
        }
    }

    assert_eq!(
        format(
            "WITH c AS (SELECT 1 AS id) UPDATE t SET a = 1 FROM c WHERE t.id = c.id",
            Style::River
        )
        .unwrap(),
        "  WITH c AS (\n       SELECT 1 AS id\n       )\nUPDATE t\n   SET a = 1\n  FROM c\n WHERE t.id = c.id;"
    );
}

// WITH RECURSIVE dropped its RECURSIVE keyword in every style, including for
// SELECT. A self-referencing CTE is invalid SQL without it.
#[test]
fn with_recursive_keyword_preserved() {
    let cases = [
        "WITH RECURSIVE c AS (SELECT 1 AS id) SELECT * FROM c",
        "WITH RECURSIVE c AS (SELECT 1 AS id) UPDATE t SET a = 1 FROM c",
        "WITH RECURSIVE c AS (SELECT 1 AS id) DELETE FROM t USING c",
        "WITH RECURSIVE c AS (SELECT 1 AS id) INSERT INTO t SELECT id FROM c",
    ];
    for &style in Style::ALL {
        for sql in cases {
            let result = format(sql, style).unwrap();
            assert!(
                result.to_uppercase().contains("WITH RECURSIVE"),
                "RECURSIVE dropped\nStyle: {style}\nInput: {sql}\nGot:\n{result}"
            );
        }
    }

    // A plain WITH must not gain the keyword.
    let plain = format("WITH c AS (SELECT 1 AS id) SELECT * FROM c", Style::River).unwrap();
    assert!(
        !plain.to_uppercase().contains("RECURSIVE"),
        "\nGot:\n{plain}"
    );
}
