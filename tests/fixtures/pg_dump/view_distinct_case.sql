 SELECT DISTINCT country,
        CASE
            WHEN active THEN 'on'::text
            ELSE 'off'::text
        END AS st
   FROM app.users;
