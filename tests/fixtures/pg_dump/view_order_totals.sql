 SELECT u.email,
    count(*) AS n,
    sum(o.total) AS revenue
   FROM app.users u
     JOIN app.orders o ON o.user_id = u.id
  WHERE o.placed_at > (now() - '30 days'::interval)
  GROUP BY u.email
 HAVING sum(o.total) > 100::numeric
  ORDER BY (sum(o.total)) DESC;
