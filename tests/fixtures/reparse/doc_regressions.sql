-- Statements from the PostgreSQL documentation that once formatted into
-- SQL libpgfmt could no longer parse (issue #54). Each must format, and
-- the formatted result must parse again, in every style.
-- One statement per record, separated by a line containing only `;;`.

SELECT to_hex(trunc(EXTRACT(EPOCH FROM backend_start))::integer) || '.' ||
       to_hex(pid)
FROM pg_stat_activity
;;
CREATE TRIGGER mytrigger
AFTER INSERT OR UPDATE ON referencing_table
FOR EACH ROW EXECUTE PROCEDURE
check_primary_key (
    'column A', 'column B',         -- referencing table columns
    'myschema."referenced table"',  -- referenced table
    '"column A"', '"column B"'      -- referenced table columns
)
;;
CREATE TRIGGER mytrigger
AFTER DELETE OR UPDATE ON referenced_table
FOR EACH ROW EXECUTE PROCEDURE
check_foreign_key (
    1,                              -- number of referencing tables
    'cascade',                      -- action
    'column A', 'column B',         -- referenced table columns
    'myschema."referencing table"', -- referencing table
    '"column A"', '"column B"'      -- referencing table columns
)
;;
SELECT *
    FROM dblink('dbname=mydb options=-csearch_path=',
                'SELECT proname, prosrc FROM pg_proc')
      AS t1(proname name, prosrc text)
    WHERE proname LIKE 'bytea%'
;;
CREATE VIEW myremote_pg_proc AS
  SELECT *
    FROM dblink('dbname=postgres options=-csearch_path=',
                'SELECT proname, prosrc FROM pg_proc')
    AS t1(proname name, prosrc text)
;;
SELECT js,
  js IS JSON "json?",
  js IS JSON SCALAR "scalar?",
  js IS JSON OBJECT "object?",
  js IS JSON ARRAY "array?"
FROM (VALUES
      ('123'), ('"abc"'), ('{"a": "b"}'), ('[1,2]'),('abc')) foo(js)
;;
SELECT js,
  js IS JSON OBJECT "object?",
  js IS JSON ARRAY "array?",
  js IS JSON ARRAY WITH UNIQUE KEYS "array w. UK?",
  js IS JSON ARRAY WITHOUT UNIQUE KEYS "array w/o UK?"
FROM (VALUES ('[{"a":"1"},
 {"b":"2","b":"3"}]')) foo(js)
;;
SELECT * FROM a CROSS JOIN b CROSS JOIN c WHERE a.id = b.id AND b.ref = c.id
;;
INSERT INTO t SELECT i % 100, i % 100 FROM generate_series(1, 10000) s(i)
;;
SELECT *
    FROM dblink('dbname=mydb', 'SELECT proname, prosrc FROM pg_proc')
      AS t1(proname name, prosrc text)
    WHERE proname LIKE 'bytea%'
;;
SELECT p1.id, p2.id, v1, v2
FROM polygons p1 CROSS JOIN LATERAL vertices(p1.poly) v1,
     polygons p2 CROSS JOIN LATERAL vertices(p2.poly) v2
WHERE (v1 <-> v2) < 10 AND p1.id != p2.id
;;
INSERT INTO t1 SELECT i/100, i/500
                 FROM generate_series(1,1000000) s(i)
;;
INSERT INTO t2 SELECT mod(i,100), mod(i,100)
                 FROM generate_series(1,1000000) s(i)
;;
INSERT INTO t3 SELECT i FROM generate_series('2020-01-01'::timestamp,
                                             '2020-12-31'::timestamp,
                                             '1 minute'::interval) s(i)
;;
INSERT INTO distributors (did, dname) VALUES (11, 'Global Electronics')
    ON CONFLICT (did) DO SELECT
    RETURNING *
;;
INSERT INTO distributors AS d (did, dname) VALUES (12, 'Micro Devices Inc')
    ON CONFLICT (did) DO SELECT WHERE d.dname != EXCLUDED.dname
    RETURNING *
;;
INSERT INTO distributors (did, dname) VALUES (13, 'Advanced Systems')
    ON CONFLICT (did) DO SELECT FOR UPDATE
    RETURNING *
;;
SELECT * FROM distributors_2(111) AS (f1 int, f2 text)
;;
UPDATE weather SET (temp_lo, temp_hi, prcp) = (temp_lo+1, temp_lo+15, DEFAULT)
  WHERE city = 'San Francisco' AND date = '2003-07-03'
;;
UPDATE accounts SET (contact_first_name, contact_last_name) =
    (SELECT first_name, last_name FROM employees
     WHERE employees.id = accounts.sales_person)
;;
UPDATE summary s SET (sum_x, sum_y, avg_x, avg_y) =
    (SELECT sum(x), sum(y), avg(x), avg(y) FROM data d
     WHERE d.group_id = s.group_id)
;;
SELECT
    count(*) AS unfiltered,
    count(*) FILTER (WHERE i < 5) AS filtered
FROM generate_series(1,10) AS s(i)
;;
SELECT *
FROM crosstab(
  'SELECT rowid, attribute, value
   FROM ct
   WHERE attribute = ''att2'' OR attribute = ''att3''
   ORDER BY 1, 2')
AS ct(row_name text, category_1 text, category_2 text, category_3 text)
;;
SELECT * FROM crosstab(
  'SELECT year, month, qty FROM sales ORDER BY 1',
  'SELECT m FROM generate_series(1, 12) m'
) AS (
  year int,
  "Jan" int,
  "Feb" int,
  "Mar" int,
  "Apr" int,
  "May" int,
  "Jun" int,
  "Jul" int,
  "Aug" int,
  "Sep" int,
  "Oct" int,
  "Nov" int,
  "Dec" int
)
;;
SELECT * FROM crosstab
(
  'SELECT rowid, rowdt, attribute, val FROM cth ORDER BY 1',
  'SELECT DISTINCT attribute FROM cth ORDER BY 1'
)
AS
(
       rowid text,
       rowdt timestamp,
       temperature int4,
       test_result text,
       test_startdate timestamp,
       volts float8
)
;;
SELECT * FROM connectby('connectby_tree', 'keyid', 'parent_keyid', 'pos', 'row2', 0, '~')
    AS t(keyid text, parent_keyid text, level int, branch text, pos int)
;;
SELECT * FROM my_table TABLESAMPLE SYSTEM_ROWS(100)
;;
SELECT * FROM my_table TABLESAMPLE SYSTEM_TIME(1000)
;;
SELECT
  unsafe_sum(x) OVER (ORDER BY n ROWS BETWEEN CURRENT ROW AND 1 FOLLOWING)
FROM (VALUES (1, 1.0e20::float8),
             (2, 1.0::float8)) AS v (n,x)
;;
SELECT sum(x) OVER (ORDER BY x RANGE BETWEEN 5 PRECEDING AND 10 FOLLOWING)
  FROM mytable
;;
SELECT * FROM
xpath_table('article_id',
            'article_xml',
            'articles',
            '/article/author|/article/pages|/article/title',
            'date_entered > ''2003-01-01'' ')
AS t(article_id integer, author text, page_count integer, title text)
;;
SELECT * FROM
  xpath_table('id','xml','test',
              '/doc/@num|/doc/line/@num|/doc/line/a|/doc/line/b|/doc/line/c',
              'true')
  AS t(id int, doc_num varchar(10), line_num varchar(10), val1 int, val2 int, val3 int)
WHERE id = 1 ORDER BY doc_num, line_num
;;
SELECT t.*,i.doc_num FROM
  xpath_table('id', 'xml', 'test',
              '/doc/line/@num|/doc/line/a|/doc/line/b|/doc/line/c',
              'true')
    AS t(id int, line_num varchar(10), val1 int, val2 int, val3 int),
  xpath_table('id', 'xml', 'test', '/doc/@num', 'true')
    AS i(id int, doc_num varchar(10))
WHERE i.id=t.id AND i.id=1
ORDER BY doc_num, line_num
;;
