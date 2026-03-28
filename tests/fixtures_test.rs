use libpgfmt::{format, style::Style};
use std::path::Path;

fn run_fixture(style: Style, name: &str) {
    let style_dir = match style {
        Style::River => "river",
        Style::Mozilla => "mozilla",
        Style::Aweber => "aweber",
        Style::Dbt => "dbt",
        Style::Gitlab => "gitlab",
        Style::Kickstarter => "kickstarter",
        Style::Mattmc3 => "mattmc3",
    };
    let base = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join(style_dir);
    let sql_path = base.join(format!("{name}.sql"));
    let expected_path = base.join(format!("{name}.expected"));

    let sql = std::fs::read_to_string(&sql_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", sql_path.display()));
    let expected = std::fs::read_to_string(&expected_path)
        .unwrap_or_else(|e| panic!("Failed to read {}: {e}", expected_path.display()));

    let result = format(sql.trim(), style)
        .unwrap_or_else(|e| panic!("Failed to format {style_dir}/{name}: {e}"));

    pretty_assertions::assert_eq!(
        result.trim(),
        expected.trim(),
        "\n\nStyle: {style_dir}, Fixture: {name}"
    );
}

// ── River fixtures ──────────────────────────────────────────────────────

#[test]
fn river_select_simple() {
    run_fixture(Style::River, "select_simple");
}

#[test]
fn river_select_and() {
    run_fixture(Style::River, "select_and");
}

#[test]
fn river_select_or() {
    run_fixture(Style::River, "select_or");
}

#[test]
fn river_select_alias() {
    run_fixture(Style::River, "select_alias");
}

#[test]
fn river_select_agg_functions() {
    run_fixture(Style::River, "select_agg_functions");
}

#[test]
fn river_select_case() {
    run_fixture(Style::River, "select_case");
}

#[test]
fn river_select_distinct() {
    run_fixture(Style::River, "select_distinct");
}

#[test]
fn river_select_group_by() {
    run_fixture(Style::River, "select_group_by");
}

#[test]
fn river_select_having() {
    run_fixture(Style::River, "select_having");
}

#[test]
fn river_select_join() {
    run_fixture(Style::River, "select_join");
}

#[test]
fn river_select_order_limit() {
    run_fixture(Style::River, "select_order_limit");
}

#[test]
fn river_select_cte() {
    run_fixture(Style::River, "select_cte");
}

#[test]
fn river_select_subquery_exists() {
    run_fixture(Style::River, "select_subquery_exists");
}

#[test]
fn river_select_subquery_in() {
    run_fixture(Style::River, "select_subquery_in");
}

#[test]
fn river_select_subquery_scalar() {
    run_fixture(Style::River, "select_subquery_scalar");
}

#[test]
fn river_select_subquery_nested() {
    run_fixture(Style::River, "select_subquery_nested");
}

#[test]
fn river_select_union() {
    run_fixture(Style::River, "select_union");
}

#[test]
fn river_insert_values() {
    run_fixture(Style::River, "insert_values");
}

#[test]
fn river_update_simple() {
    run_fixture(Style::River, "update_simple");
}

#[test]
fn river_update_multi_set() {
    run_fixture(Style::River, "update_multi_set");
}

#[test]
fn river_delete_simple() {
    run_fixture(Style::River, "delete_simple");
}

#[test]
fn river_create_table() {
    run_fixture(Style::River, "create_table");
}

// ── Mozilla fixtures ────────────────────────────────────────────────────

#[test]
fn mozilla_select_simple() {
    run_fixture(Style::Mozilla, "select_simple");
}

#[test]
fn mozilla_select_single_col() {
    run_fixture(Style::Mozilla, "select_single_col");
}

#[test]
fn mozilla_select_join() {
    run_fixture(Style::Mozilla, "select_join");
}

#[test]
fn mozilla_select_group_order() {
    run_fixture(Style::Mozilla, "select_group_order");
}

#[test]
fn mozilla_select_cte() {
    run_fixture(Style::Mozilla, "select_cte");
}

#[test]
fn mozilla_select_subquery() {
    run_fixture(Style::Mozilla, "select_subquery");
}

#[test]
fn mozilla_select_union() {
    run_fixture(Style::Mozilla, "select_union");
}

#[test]
fn mozilla_insert_multi() {
    run_fixture(Style::Mozilla, "insert_multi");
}

#[test]
fn mozilla_update_multi() {
    run_fixture(Style::Mozilla, "update_multi");
}

#[test]
fn mozilla_delete_and() {
    run_fixture(Style::Mozilla, "delete_and");
}

#[test]
fn mozilla_create_table() {
    run_fixture(Style::Mozilla, "create_table");
}

#[test]
fn mozilla_select_using_join() {
    run_fixture(Style::Mozilla, "select_using_join");
}

// ── AWeber fixtures ─────────────────────────────────────────────────────

#[test]
fn aweber_select_simple() {
    run_fixture(Style::Aweber, "select_simple");
}

#[test]
fn aweber_select_or() {
    run_fixture(Style::Aweber, "select_or");
}

#[test]
fn aweber_select_join() {
    run_fixture(Style::Aweber, "select_join");
}

#[test]
fn aweber_select_left_join() {
    run_fixture(Style::Aweber, "select_left_join");
}

#[test]
fn aweber_select_subquery() {
    run_fixture(Style::Aweber, "select_subquery");
}

// ── dbt fixtures ────────────────────────────────────────────────────────

#[test]
fn dbt_select_simple() {
    run_fixture(Style::Dbt, "select_simple");
}

#[test]
fn dbt_select_join() {
    run_fixture(Style::Dbt, "select_join");
}

#[test]
fn dbt_select_group_order() {
    run_fixture(Style::Dbt, "select_group_order");
}

#[test]
fn dbt_select_cte() {
    run_fixture(Style::Dbt, "select_cte");
}

// ── GitLab fixtures ─────────────────────────────────────────────────────

#[test]
fn gitlab_select_simple() {
    run_fixture(Style::Gitlab, "select_simple");
}

#[test]
fn gitlab_select_join() {
    run_fixture(Style::Gitlab, "select_join");
}

#[test]
fn gitlab_select_group_order() {
    run_fixture(Style::Gitlab, "select_group_order");
}

#[test]
fn gitlab_select_cte() {
    run_fixture(Style::Gitlab, "select_cte");
}

// ── Kickstarter fixtures ────────────────────────────────────────────────

#[test]
fn kickstarter_select_simple() {
    run_fixture(Style::Kickstarter, "select_simple");
}

#[test]
fn kickstarter_select_join() {
    run_fixture(Style::Kickstarter, "select_join");
}

#[test]
fn kickstarter_select_where() {
    run_fixture(Style::Kickstarter, "select_where");
}

#[test]
fn kickstarter_select_cte() {
    run_fixture(Style::Kickstarter, "select_cte");
}

// ── mattmc3 fixtures ────────────────────────────────────────────────────

#[test]
fn mattmc3_select_simple() {
    run_fixture(Style::Mattmc3, "select_simple");
}

#[test]
fn mattmc3_select_or() {
    run_fixture(Style::Mattmc3, "select_or");
}

#[test]
fn mattmc3_select_join() {
    run_fixture(Style::Mattmc3, "select_join");
}

#[test]
fn mattmc3_insert_values() {
    run_fixture(Style::Mattmc3, "insert_values");
}

#[test]
fn mattmc3_update_multi() {
    run_fixture(Style::Mattmc3, "update_multi");
}
