CREATE OR REPLACE FUNCTION app.bump(p_id bigint)
 RETURNS void
 LANGUAGE plpgsql
AS $function$
BEGIN
  UPDATE app.users SET active = true WHERE id = p_id;
END;
$function$

