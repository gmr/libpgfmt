 SELECT id,
    email,
    (created_at AT TIME ZONE 'UTC'::text) AS created_utc
   FROM app.users
  WHERE country = 'US'::text AND active;
