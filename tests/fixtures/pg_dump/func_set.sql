CREATE OR REPLACE FUNCTION app.fn_set(x integer)
 RETURNS integer
 LANGUAGE sql
 SET search_path TO 'public'
AS $function$ SELECT x $function$

