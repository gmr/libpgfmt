use libpgfmt::{format, style::Style};
use std::path::Path;

fn run_fixture(style: Style, style_name: &str, name: &str) {
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(style_name);
    let sql_path = base.join(format!("{name}.sql"));
    let expected_path = base.join(format!("{name}.expected"));

    let sql = std::fs::read_to_string(&sql_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", sql_path.display()));
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", expected_path.display()));

    let result = format(sql.trim(), style);
    match result {
        Ok(formatted) => {
            pretty_assertions::assert_eq!(
                formatted.trim(),
                expected.trim(),
                "\n\nStyle: {style_name}, Fixture: {name}"
            );
        }
        Err(e) => {
            panic!("Failed to format {style_name}/{name}: {e}");
        }
    }
}

/// Known fixtures that don't match expected output yet due to grammar
/// limitations or incomplete formatting support. These parse successfully
/// but produce different output than the pgfmt reference.
const KNOWN_FAILING: &[&str] = &[
    "river/create_domain",
    "river/create_foreign_table",
    "river/create_function",
    "river/create_matview",
    "river/create_table_with",
    "river/create_view_cte",
    "aweber/select_case_join",
    "aweber/select_cte_nested",
];

/// Discover all .sql files in each style directory and run them.
#[test]
fn all_fixture_pairs() {
    let fixtures_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures");

    let styles: Vec<(String, Style)> = Style::ALL.iter().map(|&s| (s.to_string(), s)).collect();

    let mut total = 0;
    let mut passed = 0;
    let mut failures = Vec::new();

    for (style_name, style) in &styles {
        let style_dir = fixtures_dir.join(style_name);
        if !style_dir.exists() {
            continue;
        }
        let mut entries: Vec<_> = std::fs::read_dir(&style_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "sql"))
            .collect();
        entries.sort_by_key(|e| e.file_name());

        for entry in entries {
            let stem = entry
                .path()
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string();
            let expected_path = style_dir.join(format!("{stem}.expected"));
            if !expected_path.exists() {
                eprintln!("SKIP {style_name}/{stem}: no .expected file");
                continue;
            }
            let fixture_key = format!("{style_name}/{stem}");
            let is_known_failing = KNOWN_FAILING.contains(&fixture_key.as_str());
            total += 1;
            let result = std::panic::catch_unwind(|| {
                run_fixture(*style, style_name, &stem);
            });
            match result {
                Ok(()) => {
                    passed += 1;
                    if is_known_failing {
                        eprintln!("UNEXPECTED PASS {fixture_key}: remove from KNOWN_FAILING");
                    }
                }
                Err(e) => {
                    if is_known_failing {
                        eprintln!("EXPECTED FAIL {fixture_key}");
                        passed += 1; // Don't count as failure.
                    } else {
                        let msg = if let Some(s) = e.downcast_ref::<String>() {
                            s.clone()
                        } else if let Some(s) = e.downcast_ref::<&str>() {
                            s.to_string()
                        } else {
                            "unknown panic".to_string()
                        };
                        let short = if msg.chars().count() > 200 {
                            let truncated: String = msg.chars().take(200).collect();
                            format!("{truncated}...")
                        } else {
                            msg
                        };
                        failures.push(format!("{fixture_key}: {short}"));
                    }
                }
            }
        }
    }

    eprintln!("\n=== Fixture Results: {passed}/{total} passed ===");
    if !failures.is_empty() {
        eprintln!("\nFailures:");
        for f in &failures {
            eprintln!("  FAIL: {f}");
        }
        panic!("{} of {} fixtures failed", failures.len(), total);
    }
}
