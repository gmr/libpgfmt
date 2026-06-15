#!/usr/bin/env bash
#
# Regenerate the Style::PgDump idempotency fixtures from genuine PostgreSQL
# deparser output.
#
# Stands up a throwaway local cluster (nothing global is touched), creates a
# set of representative objects, captures pg_get_viewdef / pg_get_functiondef
# output into this directory, then tears the cluster down. The captured files
# are committed so the test suite needs no PostgreSQL to run.
#
# Requires PostgreSQL client+server binaries on PATH (initdb, pg_ctl, psql).
# Usage: bash tests/fixtures/pg_dump/generate.sh
set -euo pipefail

FIXDIR="$(cd "$(dirname "$0")" && pwd)"
PORT=5599
WORK="$(mktemp -d "${TMPDIR:-/tmp}/pgdump_fix.XXXXXX")"
export PGDATA="$WORK/data"
SOCK="$WORK/sock"
mkdir -p "$SOCK"

cleanup() {
    pg_ctl -D "$PGDATA" -w stop >/dev/null 2>&1 || true
    rm -rf "$WORK"
}
trap cleanup EXIT

echo "initdb ($WORK) ..."
initdb -D "$PGDATA" --no-locale -E UTF8 -U postgres >/dev/null
pg_ctl -D "$PGDATA" -o "-k $SOCK -p $PORT -h ''" -l "$WORK/log" -w start >/dev/null

psql() { command psql -h "$SOCK" -p "$PORT" -U postgres -d postgres "$@"; }
cap() { psql -At -c "$1"; }

echo "creating objects ..."
psql -v ON_ERROR_STOP=1 -q >/dev/null <<'SQL'
CREATE SCHEMA app;
CREATE TABLE app.users (id bigint PRIMARY KEY, email text NOT NULL, country text, created_at timestamptz, active boolean DEFAULT true);
CREATE TABLE app.orders (id bigint PRIMARY KEY, user_id bigint, total numeric(10,2), placed_at timestamptz);

CREATE VIEW app.us_users AS
  SELECT id, email, created_at AT TIME ZONE 'UTC' AS created_utc
  FROM app.users WHERE country = 'US' AND active;

CREATE VIEW app.order_totals AS
  SELECT u.email, count(*) AS n, sum(o.total) AS revenue
  FROM app.users u JOIN app.orders o ON o.user_id = u.id
  WHERE o.placed_at > now() - interval '30 days'
  GROUP BY u.email HAVING sum(o.total) > 100 ORDER BY revenue DESC;

CREATE VIEW app.win AS
  SELECT email, row_number() OVER (PARTITION BY country ORDER BY created_at) AS rn FROM app.users;

CREATE VIEW app.uni AS
  SELECT id FROM app.users UNION SELECT user_id FROM app.orders;

-- Nested shapes: CTEs, CASE blocks, comma-separated FROM.
CREATE VIEW app.recent_cte AS
  WITH recent AS (SELECT user_id, total FROM app.orders WHERE placed_at > now() - interval '7 days')
  SELECT user_id, sum(total) AS wk FROM recent GROUP BY user_id;
CREATE VIEW app.two_cte AS
  WITH x AS (SELECT id AS a FROM app.users), y AS (SELECT id AS b FROM app.orders)
  SELECT x.a, y.b FROM x, y;
CREATE VIEW app.distinct_case AS
  SELECT DISTINCT country, CASE WHEN active THEN 'on' ELSE 'off' END AS st FROM app.users;
CREATE VIEW app.case_plain AS
  SELECT id, CASE WHEN active THEN 'on' ELSE 'off' END AS st FROM app.users;
CREATE VIEW app.case_first AS
  SELECT CASE WHEN active THEN 1 ELSE 0 END AS x, id FROM app.users;

-- Subqueries embedded in expressions (IN, EXISTS, scalar in target list).
CREATE VIEW app.sub AS
  SELECT u.email FROM app.users u WHERE u.id IN (SELECT user_id FROM app.orders);
CREATE VIEW app.sub_exists AS
  SELECT u.email FROM app.users u
  WHERE EXISTS (SELECT 1 FROM app.orders o WHERE o.user_id = u.id);
CREATE VIEW app.sub_scalar AS
  SELECT u.email, (SELECT count(*) FROM app.orders o WHERE o.user_id = u.id) AS n FROM app.users u;

CREATE FUNCTION app.add(a integer, b integer) RETURNS integer LANGUAGE sql IMMUTABLE AS $$ SELECT a + b $$;
CREATE FUNCTION app.bump(p_id bigint) RETURNS void LANGUAGE plpgsql AS $fn$
BEGIN
  UPDATE app.users SET active = true WHERE id = p_id;
END;
$fn$;
SQL

echo "capturing fixtures ..."
for v in us_users order_totals win uni recent_cte two_cte distinct_case case_plain case_first sub sub_exists sub_scalar; do
    cap "SELECT pg_get_viewdef('app.$v'::regclass, true);" > "$FIXDIR/view_$v.sql"
done
cap "SELECT pg_get_functiondef('app.add(integer,integer)'::regprocedure);" > "$FIXDIR/func_add.sql"
cap "SELECT pg_get_functiondef('app.bump(bigint)'::regprocedure);" > "$FIXDIR/func_bump.sql"

echo "done — fixtures written to $FIXDIR"
