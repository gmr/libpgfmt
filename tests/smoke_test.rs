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
