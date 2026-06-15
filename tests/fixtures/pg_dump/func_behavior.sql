CREATE OR REPLACE FUNCTION app.fn_behavior(x integer)
 RETURNS integer
 LANGUAGE sql
 IMMUTABLE PARALLEL SAFE STRICT
AS $function$ SELECT x $function$

