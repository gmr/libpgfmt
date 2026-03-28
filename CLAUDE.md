# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

libpgfmt is a Rust library for formatting PostgreSQL SQL and PL/pgSQL. It uses
[tree-sitter-postgres](https://github.com/gmr/tree-sitter-postgres) for parsing
(both `LANGUAGE` for SQL and `LANGUAGE_PLPGSQL` for PL/pgSQL function bodies)
and supports 7 formatting styles: river, mozilla, aweber, dbt, gitlab,
kickstarter, mattmc3.

## Build Commands

```sh
cargo build
cargo test
cargo test <test_name>           # run a single test
cargo clippy -- -D warnings      # lint (warnings are errors)
cargo fmt --check                # check formatting
cargo fmt                        # auto-format
just check                       # run fmt-check + lint + test
just set-version 1.2.0           # update version in Cargo.toml
just release 1.2.0               # set version, commit, tag, push
```

## Architecture

**Public API** (`src/lib.rs`): Two entry points — `format(sql, style)` for SQL
and `format_plpgsql(code, style)` for PL/pgSQL bodies. Both parse with
tree-sitter, then delegate to the `Formatter` struct.

**Formatter** (`src/formatter/mod.rs`): Holds source text, `Style`, and
`StyleConfig`. The `StyleConfig` is a set of boolean/string flags derived from
the style enum (river vs left-aligned, keyword casing, indent width, leading
commas, etc.). Formatting logic reads these flags rather than matching on
`Style` variants.

**Expression formatting** (`src/formatter/expr.rs`): Recursive `format_expr()`
handles all expression node types — column refs, constants, operators, function
calls, CASE, typecasts, subqueries. Returns inline SQL text. The
`join_with_multiline_indent()` helper handles multi-line subquery alignment.

**SELECT formatting** (`src/formatter/select.rs`): The most complex module.
`collect_select_clauses()` walks the CST and collects all clauses into a
`SelectClauses` struct. Then either `format_select_river()` or
`format_select_left_aligned()` renders them based on the style config. Handles
CTEs, JOINs, UNION/INTERSECT/EXCEPT, subqueries.

**Statement formatting** (`src/formatter/stmt.rs`): INSERT, UPDATE, DELETE,
CREATE TABLE/VIEW/FUNCTION/DOMAIN. The `format_stmt()` method dispatches by
node kind.

**PL/pgSQL** (`src/formatter/plpgsql.rs`): Formats blocks, declarations,
IF/ELSIF/ELSE, loops, CASE, RAISE, RETURN, exception handling.

**Node helpers** (`src/node_helpers.rs`): `NodeExt` trait adds `find_child()`,
`find_child_any()`, `has_child()`, `named_children_vec()` to tree-sitter
`Node`. `flatten_list()` converts left-recursive grammar lists into flat vectors.

## Key Design Decisions

**tree-sitter requires semicolons**: The grammar won't parse SQL without
trailing semicolons. `lib.rs` adds them if missing.

**AND/OR splitting uses text scanning**: The tree-sitter grammar nests AND/OR
inside comparison operators due to precedence rules, making AST-based condition
splitting unreliable. `split_top_level_conditions()` formats the expression
first, then splits the resulting text on AND/OR keywords while tracking quote
state and paren depth.

**Function name casing**: Only known SQL built-in functions (in
`SQL_BUILTIN_FUNCTIONS`) follow keyword casing. User-defined function names are
preserved as-is.

**Error tolerance**: Small tree-sitter ERROR nodes (< 5 bytes, e.g., decimal
fractions) are tolerated; only structural errors reject the input.

## Test Fixtures

Test fixtures in `tests/fixtures/<style>/` are pairs of `.sql` (input) and
`.expected` (expected output) files, copied from the Python
[pgfmt](https://github.com/gmr/pgfmt) project. The `fixtures_test.rs` harness
runs each pair through `format()` and compares with `pretty_assertions`.
