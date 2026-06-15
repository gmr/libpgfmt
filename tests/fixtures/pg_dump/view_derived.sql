 SELECT email
   FROM ( SELECT users.email
           FROM app.users
          WHERE users.active) s;
