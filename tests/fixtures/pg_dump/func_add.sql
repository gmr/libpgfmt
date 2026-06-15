CREATE OR REPLACE FUNCTION app.add(a integer, b integer)
 RETURNS integer
 LANGUAGE sql
 IMMUTABLE
AS $function$ SELECT a + b $function$

