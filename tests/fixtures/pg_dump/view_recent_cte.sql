 WITH recent AS (
         SELECT orders.user_id,
            orders.total
           FROM app.orders
          WHERE orders.placed_at > (now() - '7 days'::interval)
        )
 SELECT user_id,
    sum(total) AS wk
   FROM recent
  GROUP BY user_id;
