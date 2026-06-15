CREATE OR REPLACE FUNCTION app.fn_setof(x integer)
 RETURNS SETOF integer
 LANGUAGE sql
AS $function$ SELECT generate_series(1, x) $function$

