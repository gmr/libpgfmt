 SELECT country,
    percentile_cont(0.5::double precision) WITHIN GROUP (ORDER BY (id::double precision)) AS med
   FROM app.users
  GROUP BY country;
