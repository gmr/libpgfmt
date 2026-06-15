 SELECT
        CASE
            WHEN active THEN 1
            ELSE 0
        END AS x,
    id
   FROM app.users;
