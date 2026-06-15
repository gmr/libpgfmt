 SELECT t.uid,
    t.s
   FROM app.users u,
    LATERAL ( SELECT u.id AS uid,
            sum(o.total) AS s
           FROM app.orders o
          WHERE o.user_id = u.id
          GROUP BY u.id) t;
