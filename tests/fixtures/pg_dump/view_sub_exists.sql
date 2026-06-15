 SELECT email
   FROM app.users u
  WHERE (EXISTS ( SELECT 1
           FROM app.orders o
          WHERE o.user_id = u.id));
