CREATE OR REPLACE FUNCTION app.fn_variadic(VARIADIC arr integer[])
 RETURNS integer
 LANGUAGE sql
AS $function$ SELECT array_length(arr, 1) $function$

