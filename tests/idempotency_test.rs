//! Idempotency checks: formatting the output again must not change it.

use libpgfmt::format;
use libpgfmt::style::Style;
use pretty_assertions::assert_eq;

fn assert_idempotent(sql: &str) {
    for &style in Style::ALL {
        let first = format(sql, style).unwrap_or_else(|e| panic!("{style}: first pass: {e}"));
        let second = format(&first, style).unwrap_or_else(|e| panic!("{style}: second pass: {e}"));
        assert_eq!(second, first, "{style}: second pass changed the output");
    }
}

/// gmr/pgfmt#15: the JOIN after UPDATE ... FROM moved right on each pass.
#[test]
fn update_from_join_is_idempotent() {
    assert_idempotent(
        "UPDATE _foo
   SET banana = c.banana
  FROM local.fruit AS c
  JOIN local.berry ON berry.bar = c.bar
 WHERE berry.x = _foo.x;",
    );
}

#[test]
fn delete_using_join_is_idempotent() {
    assert_idempotent(
        "DELETE FROM _foo
 USING local.fruit AS c
  JOIN local.berry ON berry.bar = c.bar
 WHERE berry.x = _foo.x;",
    );
}

/// A wide JOIN keyword sets the river width for UPDATE FROM and DELETE USING.
#[test]
fn dml_join_keyword_sets_river_width() {
    let update = format(
        "UPDATE t SET x = b.x FROM a LEFT JOIN b ON a.id = b.id WHERE a.y = 1;",
        Style::Aweber,
    )
    .unwrap();
    assert_eq!(
        update,
        "   UPDATE t
      SET x = b.x
     FROM a
LEFT JOIN b
       ON a.id = b.id
    WHERE a.y = 1;"
    );
    let delete = format(
        "DELETE FROM t USING a LEFT JOIN b ON a.id = b.id WHERE a.y = 1;",
        Style::Aweber,
    )
    .unwrap();
    assert_eq!(
        delete,
        "DELETE
     FROM t
    USING a
LEFT JOIN b
       ON a.id = b.id
    WHERE a.y = 1;"
    );
}
