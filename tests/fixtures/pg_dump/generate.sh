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

-- LIMIT / OFFSET, derived tables in FROM, set-op with trailing ORDER BY/LIMIT.
CREATE VIEW app.lim AS
  SELECT id FROM app.users ORDER BY id LIMIT 10 OFFSET 5;
CREATE VIEW app.derived AS
  SELECT s.email FROM (SELECT email FROM app.users WHERE active) s;
CREATE VIEW app.derived_join AS
  SELECT u.email, t.n FROM app.users u
  JOIN (SELECT user_id, count(*) AS n FROM app.orders GROUP BY user_id) t ON t.user_id = u.id;
CREATE VIEW app.union_order AS
  SELECT id FROM app.users UNION SELECT user_id FROM app.orders ORDER BY 1 LIMIT 3;

-- Deeper / varied real-world shapes (validation sweep).
CREATE VIEW app.nested_sub AS
  SELECT id, email FROM app.users u1
  WHERE EXISTS (SELECT 1 FROM app.orders o
                WHERE o.user_id = u1.id AND o.total > (SELECT avg(total) FROM app.orders));
CREATE VIEW app.lateral AS
  SELECT t.uid, t.s FROM app.users u,
  LATERAL (SELECT u.id AS uid, sum(total) AS s FROM app.orders o WHERE o.user_id = u.id GROUP BY u.id) t;
CREATE VIEW app.window_frame AS
  SELECT id, sum(total) OVER (PARTITION BY user_id ORDER BY placed_at
    ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW) AS run FROM app.orders;
CREATE VIEW app.distinct_on AS
  SELECT DISTINCT ON (country) country, id FROM app.users ORDER BY country, id;
CREATE VIEW app.filter_agg AS
  SELECT count(*) FILTER (WHERE active) AS act, count(*) AS cnt FROM app.users;

CREATE FUNCTION app.add(a integer, b integer) RETURNS integer LANGUAGE sql IMMUTABLE AS $$ SELECT a + b $$;
CREATE FUNCTION app.bump(p_id bigint) RETURNS void LANGUAGE plpgsql AS $fn$
BEGIN
  UPDATE app.users SET active = true WHERE id = p_id;
END;
$fn$;
-- Function variety: DEFAULT args, OUT params, VARIADIC, RETURNS TABLE/SETOF,
-- grouped behavior attributes, SET, SQL-standard RETURN body.
CREATE FUNCTION app.fn_default(a integer, b integer DEFAULT 0) RETURNS integer LANGUAGE sql AS $$ SELECT a + b $$;
CREATE FUNCTION app.fn_out(IN a integer, OUT q integer, OUT r integer) LANGUAGE sql AS $$ SELECT a / 2, a % 2 $$;
CREATE FUNCTION app.fn_variadic(VARIADIC arr integer[]) RETURNS integer LANGUAGE sql AS $$ SELECT array_length(arr, 1) $$;
CREATE FUNCTION app.fn_table(x integer) RETURNS TABLE(a integer, b text) LANGUAGE sql AS $$ SELECT x, 'y'::text $$;
CREATE FUNCTION app.fn_setof(x integer) RETURNS SETOF integer LANGUAGE sql AS $$ SELECT generate_series(1, x) $$;
CREATE FUNCTION app.fn_behavior(x integer) RETURNS integer LANGUAGE sql STRICT IMMUTABLE PARALLEL SAFE AS $$ SELECT x $$;
CREATE FUNCTION app.fn_set(x integer) RETURNS integer LANGUAGE sql SET search_path TO 'public' AS $$ SELECT x $$;
CREATE FUNCTION app.fn_return(x integer) RETURNS integer LANGUAGE sql RETURN x + 1;
SQL

echo "capturing fixtures ..."
for v in us_users order_totals win uni recent_cte two_cte distinct_case case_plain case_first \
         sub sub_exists sub_scalar lim derived derived_join union_order \
         nested_sub lateral window_frame distinct_on filter_agg; do
    cap "SELECT pg_get_viewdef('app.$v'::regclass, true);" > "$FIXDIR/view_$v.sql"
done
cap "SELECT pg_get_functiondef('app.add(integer,integer)'::regprocedure);" > "$FIXDIR/func_add.sql"
cap "SELECT pg_get_functiondef('app.bump(bigint)'::regprocedure);" > "$FIXDIR/func_bump.sql"
for f in fn_default fn_out fn_variadic fn_table fn_setof fn_behavior fn_set fn_return; do
    cap "SELECT pg_get_functiondef(p.oid) FROM pg_proc p WHERE p.proname = '$f';" > "$FIXDIR/func_${f#fn_}.sql"
done

echo "done — fixtures written to $FIXDIR"
