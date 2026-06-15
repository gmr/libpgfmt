CREATE OR REPLACE FUNCTION app.fn_table(x integer)
 RETURNS TABLE(a integer, b text)
 LANGUAGE sql
AS $function$ SELECT x, 'y'::text $function$

