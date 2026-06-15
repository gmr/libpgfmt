//! Idempotency harness for `Style::PgDump`.
//!
//! Each fixture under `tests/fixtures/pg_dump/` is genuine PostgreSQL deparser
//! output (`pg_get_viewdef` / `pg_get_functiondef`). The correctness bar is
//! byte-idempotency: formatting that output with `Style::PgDump` must return it
//! unchanged. Fixtures under `_deferred/` exercise constructs not yet supported
//! and are intentionally excluded.

use std::fs;
use std::path::Path;

use libpgfmt::format;
use libpgfmt::style::Style;
use pretty_assertions::assert_eq;

#[test]
fn pgdump_fixtures_are_idempotent() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/pg_dump");
    let mut checked = 0;
    let mut entries: Vec<_> = fs::read_dir(&dir)
        .expect("read pg_dump fixtures dir")
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|e| e == "sql"))
        .collect();
    entries.sort();

    for path in entries {
        let raw = fs::read_to_string(&path).expect("read fixture");
        // Files keep a trailing newline for git friendliness; the deparser
        // output itself has none, so compare against the trimmed-end form.
        let expected = raw.trim_end_matches('\n');
        let got = format(expected, Style::PgDump)
            .unwrap_or_else(|e| panic!("{}: format error: {e}", path.display()));
        assert_eq!(
            got,
            expected,
            "\n{} is not idempotent under Style::PgDump",
            path.display()
        );
        checked += 1;
    }

    assert!(
        checked > 0,
        "no pg_dump fixtures found in {}",
        dir.display()
    );
}
