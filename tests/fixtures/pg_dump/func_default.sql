CREATE OR REPLACE FUNCTION app.fn_default(a integer, b integer DEFAULT 0)
 RETURNS integer
 LANGUAGE sql
AS $function$ SELECT a + b $function$

