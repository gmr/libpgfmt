 SELECT count(*) FILTER (WHERE active) AS act,
    count(*) AS cnt
   FROM app.users;
