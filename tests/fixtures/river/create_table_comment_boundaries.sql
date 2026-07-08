CREATE TABLE widgets ( -- table of widgets
  -- primary identifier
  id int PRIMARY KEY,
  name text
  -- last column comment
)
-- storage options below
WITH (fillfactor = 70);
