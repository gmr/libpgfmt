CREATE OR REPLACE FUNCTION app.fn_return(x integer)
 RETURNS integer
 LANGUAGE sql
RETURN (x + 1)

