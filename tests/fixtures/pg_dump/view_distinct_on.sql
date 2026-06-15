 SELECT DISTINCT ON (country) country,
    id
   FROM app.users
  ORDER BY country, id;
