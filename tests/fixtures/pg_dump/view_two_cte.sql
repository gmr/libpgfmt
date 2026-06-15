 WITH x AS (
         SELECT users.id AS a
           FROM app.users
        ), y AS (
         SELECT orders.id AS b
           FROM app.orders
        )
 SELECT x.a,
    y.b
   FROM x,
    y;
