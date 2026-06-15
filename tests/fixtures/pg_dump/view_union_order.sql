 SELECT users.id
   FROM app.users
UNION
 SELECT orders.user_id AS id
   FROM app.orders
  ORDER BY 1
 LIMIT 3;
