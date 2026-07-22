 SELECT id,
    lag(email) IGNORE NULLS OVER (ORDER BY id) AS prev_email
   FROM app.users;
