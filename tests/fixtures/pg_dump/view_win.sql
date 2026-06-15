 SELECT email,
    row_number() OVER (PARTITION BY country ORDER BY created_at) AS rn
   FROM app.users;
