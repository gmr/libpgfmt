#!/usr/bin/env bash
#
# Regenerate the Style::PgDump idempotency fixtures from genuine PostgreSQL
# deparser output.
#
# Stands up a throwaway PostgreSQL container (compose.yaml, nothing global is
# touched), which loads schema.sql on boot to create a set of representative
# objects. Captures pg_get_viewdef / pg_get_functiondef output into this
# directory, then tears the container down. The captured files are committed so
# the test suite needs no PostgreSQL to run.
#
# Requires Docker with the compose plugin. Run from anywhere:
#   bash tests/fixtures/pg_dump/generate.sh   (or: just gen-pgdump-fixtures)
set -euo pipefail

FIXDIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$FIXDIR/../../.." && pwd)"

compose() { docker compose -f "$ROOT/compose.yaml" "$@"; }

cleanup() {
    compose down -v >/dev/null 2>&1 || true
}
trap cleanup EXIT

echo "starting postgres container ..."
compose up -d --wait

cap() { compose exec -T postgres psql -At -U postgres -d postgres -c "$1"; }

echo "capturing fixtures ..."
for v in us_users order_totals win uni recent_cte two_cte distinct_case case_plain case_first \
         sub sub_exists sub_scalar lim derived derived_join union_order \
         nested_sub lateral window_frame distinct_on filter_agg \
         nulls_win within_group; do
    cap "SELECT pg_get_viewdef('app.$v'::regclass, true);" > "$FIXDIR/view_$v.sql"
done
cap "SELECT pg_get_functiondef('app.add(integer,integer)'::regprocedure);" > "$FIXDIR/func_add.sql"
cap "SELECT pg_get_functiondef('app.bump(bigint)'::regprocedure);" > "$FIXDIR/func_bump.sql"
for f in fn_default fn_out fn_variadic fn_table fn_setof fn_behavior fn_set fn_return; do
    cap "SELECT pg_get_functiondef(p.oid) FROM pg_proc p WHERE p.proname = '$f';" > "$FIXDIR/func_${f#fn_}.sql"
done
# PostgreSQL 19 SQL/PGQ property graphs. pg_dump terminates the deparser body
# with a semicolon, so the fixture carries one to match what a dump emits.
for g in graph_min graph_shop; do
    cap "SELECT pg_get_propgraphdef('app.$g'::regclass) || ';';" > "$FIXDIR/propgraph_${g#graph_}.sql"
done

echo "done — fixtures written to $FIXDIR"
