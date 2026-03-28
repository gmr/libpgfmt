# libpgfmt

A Rust library for formatting PostgreSQL SQL and PL/pgSQL, powered by
[tree-sitter-postgres](https://github.com/gmr/tree-sitter-postgres).

Supports 7 formatting styles based on popular SQL style guides:

| Style | Description |
|-------|-------------|
| **river** (default) | Keywords right-aligned to form a visual "river" ([sqlstyle.guide](https://www.sqlstyle.guide/)) |
| **mozilla** | Keywords left-aligned, content indented 4 spaces |
| **aweber** | River style with JOINs participating in keyword alignment |
| **dbt** | Lowercase keywords, blank lines between clauses |
| **gitlab** | 2-space indent, uppercase keywords |
| **kickstarter** | 2-space indent, compact JOIN...ON on same line |
| **mattmc3** | Lowercase river with leading commas |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
libpgfmt = "1"
```

### Format SQL

```rust
use libpgfmt::{format, style::Style};

let sql = "SELECT a.id, a.name, COUNT(o.id) AS order_count \
           FROM accounts AS a \
           LEFT JOIN orders AS o ON a.id = o.account_id \
           WHERE a.active = TRUE \
           GROUP BY a.id, a.name";

let formatted = format(sql, Style::River).unwrap();
assert_eq!(formatted, "\
SELECT a.id,
       a.name,
       COUNT(o.id) AS order_count
  FROM accounts AS a
       LEFT JOIN orders AS o
       ON a.id = o.account_id
 WHERE a.active = TRUE
GROUP BY a.id, a.name;");
```

### Choose a style

```rust
use libpgfmt::{format, style::Style};

let sql = "SELECT id, name FROM users WHERE active = TRUE ORDER BY name";

// dbt style: lowercase keywords, blank lines between clauses
let dbt = format(sql, Style::Dbt).unwrap();
assert_eq!(dbt, "\
select
    id,
    name

from users

where
    active = true

order by name;");

// Parse style name from string
let style: Style = "mozilla".parse().unwrap();
let mozilla = format(sql, style).unwrap();
```

### Format PL/pgSQL

```rust
use libpgfmt::{format_plpgsql, style::Style};

let body = "DECLARE x integer := 0; BEGIN IF x > 0 THEN RETURN x; END IF; RETURN 0; END";
let formatted = format_plpgsql(body, Style::River).unwrap();
```

### Error handling

```rust
use libpgfmt::{format, style::Style, error::FormatError};

match format("SELECT * FORM broken", Style::River) {
    Ok(formatted) => println!("{formatted}"),
    Err(FormatError::Syntax(msg)) => eprintln!("Bad SQL: {msg}"),
    Err(FormatError::Parser(msg)) => eprintln!("Parser init failed: {msg}"),
}
```

## Style examples

Given: `SELECT file_hash FROM file_system WHERE file_name = '.vimrc'`

**River** (default):
```sql
SELECT file_hash
  FROM file_system
 WHERE file_name = '.vimrc';
```

**Mozilla**:
```sql
SELECT file_hash
FROM file_system
WHERE
    file_name = '.vimrc';
```

**dbt**:
```sql
select file_hash

from file_system

where
    file_name = '.vimrc';
```

**mattmc3** (leading commas):
```sql
select file_hash
  from file_system
 where file_name = '.vimrc';
```

## Supported statements

- `SELECT` (with CTEs, JOINs, subqueries, UNION/INTERSECT/EXCEPT, DISTINCT, GROUP BY, HAVING, ORDER BY, LIMIT/OFFSET, window functions)
- `INSERT` (VALUES and SELECT variants)
- `UPDATE` (with SET and WHERE)
- `DELETE` (with WHERE)
- `CREATE TABLE` (columns, constraints, PRIMARY KEY, WITH options)
- `CREATE VIEW` / `CREATE MATERIALIZED VIEW`
- `CREATE FUNCTION` / `CREATE PROCEDURE`
- `CREATE DOMAIN`
- PL/pgSQL blocks (DECLARE, BEGIN/END, IF/ELSIF/ELSE, FOR/WHILE/LOOP, CASE, RAISE, RETURN, exception handling)

Unsupported statements are passed through with normalized whitespace.

## Minimum Rust version

Rust 1.88 or later (edition 2024, let-chains).

## License

BSD-3-Clause
