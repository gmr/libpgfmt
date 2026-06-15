 SELECT email,
    ( SELECT count(*) AS count
           FROM app.orders o
          WHERE o.user_id = u.id) AS n
   FROM app.users u;
