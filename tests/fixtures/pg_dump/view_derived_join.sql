 SELECT u.email,
    t.n
   FROM app.users u
     JOIN ( SELECT orders.user_id,
            count(*) AS n
           FROM app.orders
          GROUP BY orders.user_id) t ON t.user_id = u.id;
