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

// PG19: UPDATE/DELETE ... FOR PORTION OF (temporal tables). Both the table
// name and the clause must be preserved.
#[test]
fn for_portion_of_river() {
    let update = format(
        "UPDATE employees FOR PORTION OF valid_period FROM '2020-01-01' TO '2021-01-01' SET salary = salary * 1.1 WHERE id = 5",
        Style::River,
    )
    .unwrap();
    assert_eq!(
        update,
        "\
UPDATE employees FOR PORTION OF valid_period FROM '2020-01-01' TO '2021-01-01'
   SET salary = salary * 1.1
 WHERE id = 5;",
        "\nGot:\n{update}"
    );

    let delete = format(
        "DELETE FROM employees FOR PORTION OF valid_period FROM '2020-01-01' TO '2021-01-01' WHERE id = 5",
        Style::River,
    )
    .unwrap();
    assert_eq!(
        delete,
        "\
DELETE
  FROM employees FOR PORTION OF valid_period FROM '2020-01-01' TO '2021-01-01'
 WHERE id = 5;",
        "\nGot:\n{delete}"
    );
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

// PG19: GRAPH_TABLE (SQL/PGQ) lays out as an indented block — graph name,
// MATCH and COLUMNS each on their own line; vertices/edges glue without
// separators while their inner contents are space-separated. River-family
// styles anchor the block continuation lines to the FROM content column;
// left-aligned styles indent at their own width with the paren de-dented to
// column 0; pg_dump keeps the query verbatim on one line.
#[test]
fn graph_table_all_styles() {
    let sql = "SELECT * FROM graph_table (my_graph match (a)-[e]->(b) columns (a.id, b.id))";
    let river = "\
SELECT *
  FROM GRAPH_TABLE (
           my_graph
           MATCH (a)-[e]->(b)
           COLUMNS (a.id, b.id)
       );";
    let cases = [
        (Style::River, river),
        (Style::Aweber, river),
        (
            Style::Mattmc3,
            "\
select *
  from graph_table (
           my_graph
           match (a)-[e]->(b)
           columns (a.id, b.id)
       );",
        ),
        (
            Style::Dbt,
            "\
select *

from graph_table (
    my_graph
    match (a)-[e]->(b)
    columns (a.id, b.id)
);",
        ),
        (
            Style::Mozilla,
            "\
SELECT *
FROM GRAPH_TABLE (
    my_graph
    MATCH (a)-[e]->(b)
    COLUMNS (a.id, b.id)
);",
        ),
        (
            Style::Gitlab,
            "\
SELECT *
FROM GRAPH_TABLE (
  my_graph
  MATCH (a)-[e]->(b)
  COLUMNS (a.id, b.id)
);",
        ),
        (
            Style::Kickstarter,
            "\
SELECT *
FROM GRAPH_TABLE (
  my_graph
  MATCH (a)-[e]->(b)
  COLUMNS (a.id, b.id)
);",
        ),
        (
            Style::PgDump,
            " SELECT *\n   FROM graph_table (my_graph match (a)-[e]->(b) columns (a.id, b.id));",
        ),
    ];
    for (style, expected) in cases {
        let got = format(sql, style).unwrap();
        assert_eq!(got, expected, "\nstyle: {style:?}\ngot:\n{got}");
    }
}

// GRAPH_TABLE with element labels, inline element WHERE filters, a top-level
// pattern WHERE, aliased output columns, and a table alias.
#[test]
fn graph_table_complex() {
    let sql = "SELECT an, bn FROM graph_table (g match (a is person where a.age>20)-[e is knows]->(b) where a.id<>b.id columns (a.name as an, b.name as bn)) as t WHERE an IS NOT NULL";
    assert_eq!(
        format(sql, Style::River).unwrap(),
        "\
SELECT an,
       bn
  FROM GRAPH_TABLE (
           g
           MATCH (a IS person WHERE a.age > 20)-[e IS knows]->(b) WHERE a.id <> b.id
           COLUMNS (a.name AS an, b.name AS bn)
       ) AS t
 WHERE an IS NOT NULL;"
    );
}

// GRAPH_TABLE label alternation (`IS A|B`) and a path quantifier (`{2}`).
#[test]
fn graph_table_label_disjunction_and_quantifier() {
    let sql = "SELECT * FROM graph_table (g match (a is person|employee){2} columns (a.id))";
    assert_eq!(
        format(sql, Style::River).unwrap(),
        "\
SELECT *
  FROM GRAPH_TABLE (
           g
           MATCH (a IS person | employee){2}
           COLUMNS (a.id)
       );"
    );
}

// GRAPH_TABLE as the left side of a JOIN still aligns its block, and remains
// idempotent when re-formatted.
#[test]
fn graph_table_joined_and_idempotent() {
    let sql = "SELECT * FROM graph_table (g match (a)-[e]->(b) columns (a.id)) t JOIN other o ON o.id = t.id";
    let expected = "\
SELECT *
  FROM GRAPH_TABLE (
           g
           MATCH (a)-[e]->(b)
           COLUMNS (a.id)
       ) AS t
  JOIN other AS o
    ON o.id = t.id;";
    assert_eq!(format(sql, Style::River).unwrap(), expected);
    assert_eq!(
        format(expected, Style::River).unwrap(),
        expected,
        "\nnot idempotent"
    );
}

// PG19: CREATE PROPERTY GRAPH. River-family styles (river/aweber/mattmc3)
// align each element's fields into columns, using the same pad-to-max +
// single-space semantics as CREATE TABLE's column/type alignment.
#[test]
fn create_property_graph_river_aligned() {
    let sql = "CREATE PROPERTY GRAPH myshop VERTEX TABLES ( products LABEL product, customers LABEL customer, orders LABEL \"order\" ) EDGE TABLES ( order_items SOURCE orders DESTINATION products LABEL contains, customer_orders SOURCE customers DESTINATION orders LABEL has_placed )";
    let expected = "\
CREATE PROPERTY GRAPH myshop
    VERTEX TABLES (
        products  LABEL product,
        customers LABEL customer,
        orders    LABEL \"order\"
    )
    EDGE TABLES (
        order_items     SOURCE orders    DESTINATION products LABEL contains,
        customer_orders SOURCE customers DESTINATION orders   LABEL has_placed
    );";
    assert_eq!(format(sql, Style::Aweber).unwrap(), expected, "\naweber");
    assert_eq!(format(sql, Style::River).unwrap(), expected, "\nriver");
    // mattmc3 shares the alignment but lowercases keywords.
    assert_eq!(
        format(sql, Style::Mattmc3).unwrap(),
        "\
create property graph myshop
    vertex tables (
        products  label product,
        customers label customer,
        orders    label \"order\"
    )
    edge tables (
        order_items     source orders    destination products label contains,
        customer_orders source customers destination orders   label has_placed
    );",
        "\nmattmc3"
    );
}

// PG19: the pg_dump renderer reproduces pg_get_propgraphdef's layout — each
// vertex/edge element on one line — terminated with a semicolon as pg_dump
// emits it. See the byte-for-byte fixtures in tests/fixtures/pg_dump/propgraph_*.sql.
#[test]
fn create_property_graph_pg_dump() {
    let sql = "CREATE PROPERTY GRAPH myshop VERTEX TABLES ( products KEY (product_no), customers KEY (customer_id), orders KEY (order_id) ) EDGE TABLES ( order_items KEY (order_items_id) SOURCE KEY (order_id) REFERENCES orders (order_id) DESTINATION KEY (product_no) REFERENCES products (product_no), customer_orders KEY (customer_orders_id) SOURCE KEY (customer_id) REFERENCES customers (customer_id) DESTINATION KEY (order_id) REFERENCES orders (order_id) )";
    let expected = "\
CREATE PROPERTY GRAPH myshop
    VERTEX TABLES (
        products KEY (product_no),
        customers KEY (customer_id),
        orders KEY (order_id)
    )
    EDGE TABLES (
        order_items KEY (order_items_id) SOURCE KEY (order_id) REFERENCES orders (order_id) DESTINATION KEY (product_no) REFERENCES products (product_no),
        customer_orders KEY (customer_orders_id) SOURCE KEY (customer_id) REFERENCES customers (customer_id) DESTINATION KEY (order_id) REFERENCES orders (order_id)
    );";
    assert_eq!(format(sql, Style::PgDump).unwrap(), expected);
    // Idempotent: feeding the layout back through reproduces it.
    assert_eq!(
        format(expected, Style::PgDump).unwrap(),
        expected,
        "\nnot idempotent"
    );
}

// Non-river left-aligned styles (dbt/mozilla/gitlab/kickstarter) render one
// element per line without column alignment, honoring their case and indent
// (4-space for dbt/mozilla, 2-space for gitlab/kickstarter). Also exercises
// NODE/RELATIONSHIP synonyms, AS aliases, PROPERTIES lists, NO PROPERTIES, and
// PROPERTIES ALL COLUMNS.
#[test]
fn create_property_graph_left_aligned() {
    let sql = "CREATE PROPERTY GRAPH g NODE TABLES ( persons AS p KEY (id) LABEL person PROPERTIES (p.name, p.age AS years), accounts KEY (acct_id) LABEL account NO PROPERTIES ) RELATIONSHIP TABLES ( owns SOURCE KEY (person_id) REFERENCES persons (id) DESTINATION KEY (acct_id) REFERENCES accounts (acct_id) LABEL owns PROPERTIES ALL COLUMNS )";
    let cases = [
        (
            Style::Dbt,
            "\
create property graph g
    node tables (
        persons as p key (id) label person properties (p.name, p.age as years),
        accounts key (acct_id) label account no properties
    )
    relationship tables (
        owns source key (person_id) references persons (id) destination key (acct_id) references accounts (acct_id) label owns properties all columns
    );",
        ),
        (
            Style::Mozilla,
            "\
CREATE PROPERTY GRAPH g
    NODE TABLES (
        persons AS p KEY (id) LABEL person PROPERTIES (p.name, p.age AS years),
        accounts KEY (acct_id) LABEL account NO PROPERTIES
    )
    RELATIONSHIP TABLES (
        owns SOURCE KEY (person_id) REFERENCES persons (id) DESTINATION KEY (acct_id) REFERENCES accounts (acct_id) LABEL owns PROPERTIES ALL COLUMNS
    );",
        ),
        (
            Style::Gitlab,
            "\
CREATE PROPERTY GRAPH g
  NODE TABLES (
    persons AS p KEY (id) LABEL person PROPERTIES (p.name, p.age AS years),
    accounts KEY (acct_id) LABEL account NO PROPERTIES
  )
  RELATIONSHIP TABLES (
    owns SOURCE KEY (person_id) REFERENCES persons (id) DESTINATION KEY (acct_id) REFERENCES accounts (acct_id) LABEL owns PROPERTIES ALL COLUMNS
  );",
        ),
        (
            Style::Kickstarter,
            "\
CREATE PROPERTY GRAPH g
  NODE TABLES (
    persons AS p KEY (id) LABEL person PROPERTIES (p.name, p.age AS years),
    accounts KEY (acct_id) LABEL account NO PROPERTIES
  )
  RELATIONSHIP TABLES (
    owns SOURCE KEY (person_id) REFERENCES persons (id) DESTINATION KEY (acct_id) REFERENCES accounts (acct_id) LABEL owns PROPERTIES ALL COLUMNS
  );",
        ),
    ];
    for (style, expected) in cases {
        let got = format(sql, style).unwrap();
        assert_eq!(got, expected, "\nstyle: {style:?}\ngot:\n{got}");
    }
}

// PG19: ALTER PROPERTY GRAPH ... ADD VERTEX/EDGE TABLES reuses the CREATE block
// layout (each clause prefixed with ADD), across all styles.
#[test]
fn alter_property_graph_add_tables() {
    let sql = "ALTER PROPERTY GRAPH myshop ADD VERTEX TABLES (products KEY (id) LABEL product, customers KEY (cid)) ADD EDGE TABLES (rel SOURCE KEY (a) REFERENCES products (id) DESTINATION KEY (b) REFERENCES customers (cid) LABEL r)";
    // River-family aligns element fields into columns.
    let river = "\
ALTER PROPERTY GRAPH myshop
    ADD VERTEX TABLES (
        products  KEY (id)  LABEL product,
        customers KEY (cid)
    )
    ADD EDGE TABLES (
        rel SOURCE KEY (a) REFERENCES products (id) DESTINATION KEY (b) REFERENCES customers (cid) LABEL r
    );";
    let cases = [
        (Style::River, river),
        (Style::Aweber, river),
        (
            Style::Mattmc3,
            "\
alter property graph myshop
    add vertex tables (
        products  key (id)  label product,
        customers key (cid)
    )
    add edge tables (
        rel source key (a) references products (id) destination key (b) references customers (cid) label r
    );",
        ),
        (
            Style::Dbt,
            "\
alter property graph myshop
    add vertex tables (
        products key (id) label product,
        customers key (cid)
    )
    add edge tables (
        rel source key (a) references products (id) destination key (b) references customers (cid) label r
    );",
        ),
        (
            Style::Mozilla,
            "\
ALTER PROPERTY GRAPH myshop
    ADD VERTEX TABLES (
        products KEY (id) LABEL product,
        customers KEY (cid)
    )
    ADD EDGE TABLES (
        rel SOURCE KEY (a) REFERENCES products (id) DESTINATION KEY (b) REFERENCES customers (cid) LABEL r
    );",
        ),
        (
            Style::Gitlab,
            "\
ALTER PROPERTY GRAPH myshop
  ADD VERTEX TABLES (
    products KEY (id) LABEL product,
    customers KEY (cid)
  )
  ADD EDGE TABLES (
    rel SOURCE KEY (a) REFERENCES products (id) DESTINATION KEY (b) REFERENCES customers (cid) LABEL r
  );",
        ),
        (
            Style::Kickstarter,
            "\
ALTER PROPERTY GRAPH myshop
  ADD VERTEX TABLES (
    products KEY (id) LABEL product,
    customers KEY (cid)
  )
  ADD EDGE TABLES (
    rel SOURCE KEY (a) REFERENCES products (id) DESTINATION KEY (b) REFERENCES customers (cid) LABEL r
  );",
        ),
    ];
    for (style, expected) in cases {
        let got = format(sql, style).unwrap();
        assert_eq!(got, expected, "\nstyle: {style:?}\ngot:\n{got}");
        // Re-formatting the multi-line block reproduces it.
        assert_eq!(
            format(expected, style).unwrap(),
            expected,
            "\nstyle: {style:?} not idempotent"
        );
    }
    // pg_dump uses the same structured block with a terminating semicolon.
    assert_eq!(
        format(sql, Style::PgDump).unwrap(),
        "\
ALTER PROPERTY GRAPH myshop
    ADD VERTEX TABLES (
        products KEY (id) LABEL product,
        customers KEY (cid)
    )
    ADD EDGE TABLES (
        rel SOURCE KEY (a) REFERENCES products (id) DESTINATION KEY (b) REFERENCES customers (cid) LABEL r
    );"
    );
}

// The short ALTER PROPERTY GRAPH forms (DROP TABLES, ALTER ... TABLE ... LABEL)
// have no multi-element block, so they stay on one line, keyword-cased.
#[test]
fn alter_property_graph_single_line_forms() {
    let drop = "ALTER PROPERTY GRAPH myshop DROP VERTEX TABLES (products, customers) CASCADE";
    assert_eq!(
        format(drop, Style::River).unwrap(),
        "ALTER PROPERTY GRAPH myshop DROP VERTEX TABLES (products, customers) CASCADE;"
    );
    assert_eq!(
        format(drop, Style::Dbt).unwrap(),
        "alter property graph myshop drop vertex tables (products, customers) cascade;"
    );
    let label = "ALTER PROPERTY GRAPH g ALTER VERTEX TABLE persons ADD LABEL vip PROPERTIES (tier)";
    assert_eq!(
        format(label, Style::River).unwrap(),
        "ALTER PROPERTY GRAPH g ALTER VERTEX TABLE persons ADD LABEL vip PROPERTIES (tier);"
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
