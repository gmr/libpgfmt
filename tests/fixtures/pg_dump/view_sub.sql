 SELECT email
   FROM app.users u
  WHERE (id IN ( SELECT orders.user_id
           FROM app.orders));
