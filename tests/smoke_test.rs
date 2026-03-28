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
