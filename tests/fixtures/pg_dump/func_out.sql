CREATE OR REPLACE FUNCTION app.fn_out(a integer, OUT q integer, OUT r integer)
 RETURNS record
 LANGUAGE sql
AS $function$ SELECT a / 2, a % 2 $function$

