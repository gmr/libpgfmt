 SELECT id,
    email
   FROM app.users u1
  WHERE (EXISTS ( SELECT 1
           FROM app.orders o
          WHERE o.user_id = u1.id AND o.total > (( SELECT avg(orders.total) AS avg
                   FROM app.orders))));
