-- SQL examples extracted from the PostgreSQL documentation by
-- scripts/extract_doc_corpus.py. Records are separated by a line
-- holding `;;`. Each record is preceded by its source file.
--
-- Copyright (c) 1996-2025, PostgreSQL Global Development Group.
-- Used under the PostgreSQL License.
;;
-- advanced.sgml
CREATE VIEW myview AS
    SELECT name, temp_lo, temp_hi, prcp, date, location
        FROM weather, cities
        WHERE city = name;
;;
-- advanced.sgml
CREATE TABLE cities (
        name     varchar(80) PRIMARY KEY,
        location point
);
;;
-- advanced.sgml
INSERT INTO weather VALUES ('Berkeley', 45, 53, 0.0, '1994-11-28');
;;
-- advanced.sgml
UPDATE accounts SET balance = balance - 100.00
    WHERE name = 'Alice';
;;
-- advanced.sgml
BEGIN;
;;
-- advanced.sgml
SELECT depname, empno, salary, avg(salary) OVER (PARTITION BY depname) FROM empsalary;
;;
-- advanced.sgml
SELECT depname, empno, salary,
       row_number() OVER (PARTITION BY depname ORDER BY salary DESC)
FROM empsalary;
;;
-- advanced.sgml
SELECT salary, sum(salary) OVER () FROM empsalary;
;;
-- advanced.sgml
SELECT salary, sum(salary) OVER (ORDER BY salary) FROM empsalary;
;;
-- advanced.sgml
SELECT depname, empno, salary, enroll_date
FROM
  (SELECT depname, empno, salary, enroll_date,
     row_number() OVER (PARTITION BY depname ORDER BY salary DESC, empno) AS pos
     FROM empsalary
  ) AS ss
WHERE pos < 3;
;;
-- advanced.sgml
SELECT sum(salary) OVER w, avg(salary) OVER w
  FROM empsalary
  WINDOW w AS (PARTITION BY depname ORDER BY salary DESC);
;;
-- advanced.sgml
CREATE TABLE capitals (
  name       text,
  population real,
  elevation  int,    -- (in ft)
  state      char(2)
);
;;
-- advanced.sgml
CREATE TABLE cities (
  name       text,
  population real,
  elevation  int     -- (in ft)
);
;;
-- advanced.sgml
SELECT name, elevation
  FROM cities
  WHERE elevation > 500;
;;
-- advanced.sgml
SELECT name, elevation
    FROM ONLY cities
    WHERE elevation > 500;
;;
-- amcheck.sgml
SET client_min_messages = DEBUG1;
;;
-- array.sgml
CREATE TABLE sal_emp (
    name            text,
    pay_by_quarter  integer[],
    schedule        text[][]
);
;;
-- array.sgml
CREATE TABLE tictactoe (
    squares   integer[3][3]
);
;;
-- array.sgml
INSERT INTO sal_emp
    VALUES ('Bill',
    '{10000, 10000, 10000, 10000}',
    '{{"meeting", "lunch"}, {"training", "presentation"}}');
;;
-- array.sgml
SELECT * FROM sal_emp;
;;
-- array.sgml
INSERT INTO sal_emp
    VALUES ('Bill',
    '{10000, 10000, 10000, 10000}',
    '{{"meeting", "lunch"}, {"meeting"}}');
;;
-- array.sgml
INSERT INTO sal_emp
    VALUES ('Bill',
    ARRAY[10000, 10000, 10000, 10000],
    ARRAY[['meeting', 'lunch'], ['training', 'presentation']]);
;;
-- array.sgml
SELECT name FROM sal_emp WHERE pay_by_quarter[1] <> pay_by_quarter[2];
;;
-- array.sgml
SELECT pay_by_quarter[3] FROM sal_emp;
;;
-- array.sgml
SELECT schedule[1:2][1:1] FROM sal_emp WHERE name = 'Bill';
;;
-- array.sgml
SELECT schedule[1:2][2] FROM sal_emp WHERE name = 'Bill';
;;
-- array.sgml
SELECT schedule[:2][2:] FROM sal_emp WHERE name = 'Bill';
;;
-- array.sgml
SELECT array_dims(schedule) FROM sal_emp WHERE name = 'Carol';
;;
-- array.sgml
SELECT array_upper(schedule, 1) FROM sal_emp WHERE name = 'Carol';
;;
-- array.sgml
SELECT array_length(schedule, 1) FROM sal_emp WHERE name = 'Carol';
;;
-- array.sgml
SELECT cardinality(schedule) FROM sal_emp WHERE name = 'Carol';
;;
-- array.sgml
UPDATE sal_emp SET pay_by_quarter = '{25000,25000,27000,27000}'
    WHERE name = 'Carol';
;;
-- array.sgml
UPDATE sal_emp SET pay_by_quarter = ARRAY[25000,25000,27000,27000]
    WHERE name = 'Carol';
;;
-- array.sgml
UPDATE sal_emp SET pay_by_quarter[4] = 15000
    WHERE name = 'Bill';
;;
-- array.sgml
UPDATE sal_emp SET pay_by_quarter[1:2] = '{27000,27000}'
    WHERE name = 'Carol';
;;
-- array.sgml
SELECT ARRAY[1,2] || ARRAY[3,4];
;;
-- array.sgml
SELECT array_dims(1 || '[0:1]={2,3}'::int[]);
;;
-- array.sgml
SELECT array_dims(ARRAY[1,2] || ARRAY[3,4,5]);
;;
-- array.sgml
SELECT array_dims(ARRAY[1,2] || ARRAY[[3,4],[5,6]]);
;;
-- array.sgml
SELECT array_prepend(1, ARRAY[2,3]);
;;
-- array.sgml
SELECT * FROM sal_emp WHERE pay_by_quarter[1] = 10000 OR
                            pay_by_quarter[2] = 10000 OR
                            pay_by_quarter[3] = 10000 OR
                            pay_by_quarter[4] = 10000;
;;
-- array.sgml
SELECT * FROM sal_emp WHERE 10000 = ANY (pay_by_quarter);
;;
-- array.sgml
SELECT * FROM sal_emp WHERE 10000 = ALL (pay_by_quarter);
;;
-- array.sgml
SELECT * FROM
   (SELECT pay_by_quarter,
           generate_subscripts(pay_by_quarter, 1) AS s
      FROM sal_emp) AS foo
 WHERE pay_by_quarter[s] = 10000;
;;
-- array.sgml
SELECT * FROM sal_emp WHERE pay_by_quarter && ARRAY[10000];
;;
-- array.sgml
SELECT array_position(ARRAY['sun','mon','tue','wed','thu','fri','sat'], 'mon');
;;
-- array.sgml
SELECT f1[1][-2][3] AS e1, f1[1][-1][5] AS e2
 FROM (SELECT '[1:1][-2:-1][3:5]={{{1,2,3},{4,5,6}}}'::int[] AS f1) AS ss;
;;
-- auto-explain.sgml
LOAD 'auto_explain';
;;
-- bloom.sgml
CREATE INDEX bloomidx ON tbloom USING bloom (i1,i2,i3)
       WITH (length=80, col1=2, col2=2, col3=4);
;;
-- bloom.sgml
CREATE OPERATOR CLASS text_ops
DEFAULT FOR TYPE text USING bloom AS
    OPERATOR    1   =(text, text),
    FUNCTION    1   hashtext(text);
;;
-- btree-gin.sgml
CREATE TABLE test (a int4);
;;
-- btree-gist.sgml
CREATE TABLE mytable (addr inet);
;;
-- btree-gist.sgml
CREATE INDEX good_index ON mytable USING GIST (addr inet_ops);
;;
-- charset.sgml
CREATE COLLATION mycollation1 (provider = icu, locale = 'ja-JP');
;;
-- charset.sgml
SELECT a < 'foo' FROM test1;
;;
-- charset.sgml
SELECT a < ('foo' COLLATE "fr_FR") FROM test1;
;;
-- charset.sgml
SELECT a < b FROM test1;
;;
-- charset.sgml
SELECT a < b COLLATE "de_DE" FROM test1;
;;
-- charset.sgml
SELECT a COLLATE "de_DE" < b FROM test1;
;;
-- charset.sgml
SELECT a || b FROM test1;
;;
-- charset.sgml
SELECT * FROM test1 ORDER BY a || 'foo';
;;
-- charset.sgml
SELECT * FROM test1 ORDER BY a || b;
;;
-- charset.sgml
SELECT * FROM test1 ORDER BY a || b COLLATE "fr_FR";
;;
-- charset.sgml
SELECT a COLLATE "C" < b COLLATE "POSIX" FROM test1;
;;
-- charset.sgml
CREATE COLLATION german (provider = libc, locale = 'de_DE');
;;
-- charset.sgml
CREATE COLLATION german (provider = icu, locale = 'de-DE');
;;
-- charset.sgml
CREATE COLLATION german FROM "de_DE";
;;
-- charset.sgml
CREATE COLLATION ndcoll (provider = icu, locale = 'und', deterministic = false);
;;
-- charset.sgml
CREATE COLLATION case_insensitive (provider = icu, locale = 'und-u-ks-level2', deterministic = false);
;;
-- charset.sgml
CREATE COLLATION level3 (provider = icu, deterministic = false, locale = 'und-u-ka-shifted-ks-level3');
;;
-- charset.sgml
CREATE DATABASE korean WITH ENCODING 'EUC_KR' LC_COLLATE='ko_KR.euckr' LC_CTYPE='ko_KR.euckr' TEMPLATE=template0;
;;
-- charset.sgml
SHOW client_encoding;
;;
-- charset.sgml
RESET client_encoding;
;;
-- citext.sgml
SELECT * FROM tab WHERE lower(col) = LOWER(?);
;;
-- citext.sgml
CREATE TABLE users (
    nick CITEXT PRIMARY KEY,
    pass TEXT   NOT NULL
);
;;
-- config.sgml
SET configuration_parameter TO DEFAULT;
;;
-- config.sgml
UPDATE pg_settings SET setting = reset_val WHERE name = 'configuration_parameter';
;;
-- config.sgml
SELECT DISTINCT plugin FROM pg_replication_slots WHERE plugin IS NOT NULL;
;;
-- config.sgml
SELECT to_hex(trunc(EXTRACT(EPOCH FROM backend_start))::integer) || '.' ||
       to_hex(pid)
FROM pg_stat_activity;
;;
-- config.sgml
CREATE TABLE postgres_log
(
  log_time timestamp(3) with time zone,
  user_name text,
  database_name text,
  process_id integer,
  connection_from text,
  session_id text,
  session_line_num bigint,
  command_tag text,
  session_start_time timestamp with time zone,
  virtual_transaction_id text,
  transaction_id bigint,
  error_severity text,
  sql_state_code text,
  message text,
  detail text,
  hint text,
  internal_query text,
  internal_query_pos integer,
  context text,
  query text,
  query_pos integer,
  location text,
  application_name text,
  backend_type text,
  leader_pid integer,
  query_id bigint,
  PRIMARY KEY (session_id, session_line_num)
);
;;
-- config.sgml
COPY postgres_log FROM '/full/path/to/logfile.csv' WITH csv;
;;
-- cube.sgml
SELECT c FROM test ORDER BY c <-> cube(ARRAY[0.5, 0.5, 0.5]) LIMIT 1;
;;
-- cube.sgml
SELECT c FROM test ORDER BY c ~> 1 LIMIT 5;
;;
-- cube.sgml
SELECT c FROM test ORDER BY c ~> 3 DESC LIMIT 5;
;;
-- cube.sgml
SELECT cube_union('(0,5,2),(2,3,1)', '0');
;;
-- cube.sgml
SELECT cube_inter('(0,-1),(1,1)', '(-2),(2)');
;;
-- cube.sgml
SELECT cube_contains('(0,0),(1,1)', '0.5,0.5');
;;
-- datatype.sgml
SELECT x,
  round(x::numeric) AS num_round,
  round(x::double precision) AS dbl_round
FROM generate_series(-3.5, 3.5, 1) AS x;
;;
-- datatype.sgml
SELECT '12.34'::float8::numeric::money;
;;
-- datatype.sgml
SELECT '52093.89'::money::numeric::float8;
;;
-- datatype.sgml
CREATE TABLE test1 (a character(4));
;;
-- datatype.sgml
SET bytea_output = 'escape';
;;
-- datatype.sgml
SELECT '2 years 15 months 100 weeks 99 hours 123456789 milliseconds'::interval;
;;
-- datatype.sgml
CREATE TABLE test1 (a boolean, b text);
;;
-- datatype.sgml
CREATE TYPE mood AS ENUM ('sad', 'ok', 'happy');
;;
-- datatype.sgml
INSERT INTO person VALUES ('Larry', 'sad');
;;
-- datatype.sgml
CREATE TYPE happiness AS ENUM ('happy', 'very happy', 'ecstatic');
;;
-- datatype.sgml
SELECT person.name, holidays.num_weeks FROM person, holidays
  WHERE person.current_mood::text = holidays.happiness::text;
;;
-- datatype.sgml
SELECT macaddr8_set7bit('08:00:2b:01:02:03');
;;
-- datatype.sgml
CREATE TABLE test (a BIT(3), b BIT VARYING(5));
;;
-- datatype.sgml
SELECT 'a fat cat sat on a mat and ate a fat rat'::tsvector;
;;
-- datatype.sgml
SELECT $$the lexeme '    ' contains spaces$$::tsvector;
;;
-- datatype.sgml
SELECT $$the lexeme 'Joe''s' contains a quote$$::tsvector;
;;
-- datatype.sgml
SELECT 'a:1 fat:2 cat:3 sat:4 on:5 a:6 mat:7 and:8 ate:9 a:10 fat:11 rat:12'::tsvector;
;;
-- datatype.sgml
SELECT 'a:1A fat:2B,4C cat:5D'::tsvector;
;;
-- datatype.sgml
SELECT 'The Fat Rats'::tsvector;
;;
-- datatype.sgml
SELECT to_tsvector('english', 'The Fat Rats');
;;
-- datatype.sgml
SELECT 'fat & rat'::tsquery;
;;
-- datatype.sgml
SELECT 'fat:ab & cat'::tsquery;
;;
-- datatype.sgml
SELECT 'super:*'::tsquery;
;;
-- datatype.sgml
SELECT to_tsquery('Fat:ab & Cats');
;;
-- datatype.sgml
SELECT to_tsvector( 'postgraduate' ) @@ to_tsquery( 'postgres:*' );
;;
-- datatype.sgml
SELECT to_tsvector( 'postgraduate' ), to_tsquery( 'postgres:*' );
;;
-- datatype.sgml
CREATE DOMAIN posint AS integer CHECK (VALUE > 0);
;;
-- datatype.sgml
SELECT * FROM pg_attribute WHERE attrelid = 'mytable'::regclass;
;;
-- datatype.sgml
SELECT * FROM pg_attribute
  WHERE attrelid = (SELECT oid FROM pg_class WHERE relname = 'mytable');
;;
-- dblink.sgml
SELECT *
    FROM dblink('dbname=mydb options=-csearch_path=',
                'SELECT proname, prosrc FROM pg_proc')
      AS t1(proname name, prosrc text)
    WHERE proname LIKE 'bytea%';
;;
-- dblink.sgml
CREATE VIEW myremote_pg_proc AS
  SELECT *
    FROM dblink('dbname=postgres options=-csearch_path=',
                'SELECT proname, prosrc FROM pg_proc')
    AS t1(proname name, prosrc text);
;;
-- dblink.sgml
SELECT dblink_get_connections();
;;
-- dblink.sgml
SELECT dblink_error_message('dtest1');
;;
-- dblink.sgml
SELECT dblink_send_query('dtest1', 'SELECT * FROM foo WHERE f1 < 3');
;;
-- dblink.sgml
SELECT dblink_is_busy('dtest1');
;;
-- dblink.sgml
SELECT dblink_cancel_query('dtest1');
;;
-- dblink.sgml
CREATE TYPE dblink_pkey_results AS (position int, colname text);
;;
-- ddl.sgml
CREATE TABLE my_first_table (
    first_column text,
    second_column integer
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric
);
;;
-- ddl.sgml
DROP TABLE my_first_table;
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric DEFAULT 9.99
);
;;
-- ddl.sgml
INSERT INTO people (name, address) VALUES ('A', 'foo');
;;
-- ddl.sgml
INSERT INTO people (id, name, address) VALUES (DEFAULT, 'C', 'baz');
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric CHECK (price > 0)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric CONSTRAINT positive_price CHECK (price > 0)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric CHECK (price > 0),
    discounted_price numeric CHECK (discounted_price > 0),
    CHECK (price > discounted_price)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric,
    CHECK (price > 0),
    discounted_price numeric,
    CHECK (discounted_price > 0),
    CHECK (price > discounted_price)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric CHECK (price > 0),
    discounted_price numeric,
    CHECK (discounted_price > 0 AND price > discounted_price)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric,
    CHECK (price > 0),
    discounted_price numeric,
    CHECK (discounted_price > 0),
    CONSTRAINT valid_discount CHECK (price > discounted_price)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer NOT NULL,
    name text NOT NULL,
    price numeric
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer NOT NULL,
    name text CONSTRAINT products_name_not_null NOT NULL,
    price numeric
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric,
    NOT NULL product_no,
    NOT NULL name
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer NOT NULL,
    name text NOT NULL,
    price numeric NOT NULL CHECK (price > 0)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer NULL,
    name text NULL,
    price numeric NULL
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer UNIQUE,
    name text,
    price numeric
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric,
    UNIQUE (product_no)
);
;;
-- ddl.sgml
CREATE TABLE example (
    a integer,
    b integer,
    c integer,
    UNIQUE (a, c)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer CONSTRAINT must_be_different UNIQUE,
    name text,
    price numeric
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer UNIQUE NULLS NOT DISTINCT,
    name text,
    price numeric
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    name text,
    price numeric,
    UNIQUE NULLS NOT DISTINCT (product_no)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer UNIQUE NOT NULL,
    name text,
    price numeric
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer PRIMARY KEY,
    name text,
    price numeric
);
;;
-- ddl.sgml
CREATE TABLE example (
    a integer,
    b integer,
    c integer,
    PRIMARY KEY (a, c)
);
;;
-- ddl.sgml
CREATE TABLE orders (
    order_id integer PRIMARY KEY,
    product_no integer REFERENCES products (product_no),
    quantity integer
);
;;
-- ddl.sgml
CREATE TABLE orders (
    order_id integer PRIMARY KEY,
    product_no integer REFERENCES products,
    quantity integer
);
;;
-- ddl.sgml
CREATE TABLE t1 (
  a integer PRIMARY KEY,
  b integer,
  c integer,
  FOREIGN KEY (b, c) REFERENCES other_table (c1, c2)
);
;;
-- ddl.sgml
CREATE TABLE tenants (
    tenant_id integer PRIMARY KEY
);
;;
-- ddl.sgml
CREATE TABLE circles (
    c circle,
    EXCLUDE USING gist (c WITH &&)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    price numeric,
    valid_at daterange
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    price numeric,
    valid_at daterange,
    PRIMARY KEY (product_no, valid_at WITHOUT OVERLAPS)
);
;;
-- ddl.sgml
CREATE TABLE products (
    product_no integer,
    price numeric,
    valid_at daterange,
    UNIQUE (product_no, valid_at WITHOUT OVERLAPS)
);
;;
-- ddl.sgml
CREATE TABLE variants (
  id         integer,
  product_no integer,
  name       text,
  valid_at   daterange,
  PRIMARY KEY (id, valid_at WITHOUT OVERLAPS)
);
;;
-- ddl.sgml
CREATE TABLE variants (
  id         integer,
  product_no integer,
  name       text,
  valid_at   daterange,
  PRIMARY KEY (id, valid_at WITHOUT OVERLAPS),
  FOREIGN KEY (product_no, PERIOD valid_at) REFERENCES products (product_no, PERIOD valid_at)
);
;;
-- ddl.sgml
ALTER TABLE products ADD COLUMN description text;
;;
-- ddl.sgml
ALTER TABLE products ADD COLUMN description text CHECK (description <> '');
;;
-- ddl.sgml
ALTER TABLE products DROP COLUMN description;
;;
-- ddl.sgml
ALTER TABLE products DROP COLUMN description CASCADE;
;;
-- ddl.sgml
ALTER TABLE products ADD CHECK (name <> '');
;;
-- ddl.sgml
ALTER TABLE products ALTER COLUMN product_no SET NOT NULL;
;;
-- ddl.sgml
ALTER TABLE products DROP CONSTRAINT some_name;
;;
-- ddl.sgml
ALTER TABLE products ALTER COLUMN product_no DROP NOT NULL;
;;
-- ddl.sgml
ALTER TABLE products ALTER COLUMN price SET DEFAULT 7.77;
;;
-- ddl.sgml
ALTER TABLE products ALTER COLUMN price DROP DEFAULT;
;;
-- ddl.sgml
ALTER TABLE products ALTER COLUMN price TYPE numeric(10,2);
;;
-- ddl.sgml
ALTER TABLE products RENAME COLUMN product_no TO product_number;
;;
-- ddl.sgml
ALTER TABLE products RENAME TO items;
;;
-- ddl.sgml
GRANT UPDATE ON accounts TO joe;
;;
-- ddl.sgml
REVOKE ALL ON accounts FROM PUBLIC;
;;
-- ddl.sgml
GRANT SELECT ON mytable TO PUBLIC;
;;
-- ddl.sgml
CREATE TABLE accounts (manager text, company text, contact_email text);
;;
-- ddl.sgml
CREATE POLICY user_policy ON users
    USING (user_name = current_user);
;;
-- ddl.sgml
CREATE POLICY user_sel_policy ON users
    FOR SELECT
    USING (true);
;;
-- ddl.sgml
CREATE POLICY admin_local_only ON passwd AS RESTRICTIVE TO admin
    USING (pg_catalog.inet_client_addr() IS NULL);
;;
-- ddl.sgml
SELECT * FROM information WHERE group_id = 2 FOR UPDATE;
;;
-- ddl.sgml
CREATE SCHEMA myschema;
;;
-- ddl.sgml
DROP SCHEMA myschema;
;;
-- ddl.sgml
DROP SCHEMA myschema CASCADE;
;;
-- ddl.sgml
SHOW search_path;
;;
-- ddl.sgml
SET search_path TO myschema,public;
;;
-- ddl.sgml
DROP TABLE mytable;
;;
-- ddl.sgml
SET search_path TO myschema;
;;
-- ddl.sgml
SELECT 3 OPERATOR(pg_catalog.+) 4;
;;
-- ddl.sgml
REVOKE CREATE ON SCHEMA public FROM PUBLIC;
;;
-- ddl.sgml
CREATE TABLE cities (
    name            text,
    population      float,
    elevation       int     -- in feet
);
;;
-- ddl.sgml
SELECT name, elevation
    FROM cities
    WHERE elevation > 500;
;;
-- ddl.sgml
SELECT name, elevation
    FROM cities*
    WHERE elevation > 500;
;;
-- ddl.sgml
SELECT c.tableoid, c.name, c.elevation
FROM cities c
WHERE c.elevation > 500;
;;
-- ddl.sgml
SELECT p.relname, c.name, c.elevation
FROM cities c, pg_class p
WHERE c.elevation > 500 AND c.tableoid = p.oid;
;;
-- ddl.sgml
SELECT c.tableoid::regclass, c.name, c.elevation
FROM cities c
WHERE c.elevation > 500;
;;
-- ddl.sgml
INSERT INTO cities (name, population, elevation, state)
VALUES ('Albany', NULL, NULL, 'NY');
;;
-- ddl.sgml
CREATE TABLE measurement (
    city_id         int not null,
    logdate         date not null,
    peaktemp        int,
    unitsales       int
);
;;
-- ddl.sgml
CREATE TABLE measurement (
    city_id         int not null,
    logdate         date not null,
    peaktemp        int,
    unitsales       int
) PARTITION BY RANGE (logdate);
;;
-- ddl.sgml
CREATE TABLE measurement_y2006m02 PARTITION OF measurement
    FOR VALUES FROM ('2006-02-01') TO ('2006-03-01');
;;
-- ddl.sgml
CREATE TABLE measurement_y2006m02 PARTITION OF measurement
    FOR VALUES FROM ('2006-02-01') TO ('2006-03-01')
    PARTITION BY RANGE (peaktemp);
;;
-- ddl.sgml
CREATE INDEX ON measurement (logdate);
;;
-- ddl.sgml
DROP TABLE measurement_y2006m02;
;;
-- ddl.sgml
ALTER TABLE measurement DETACH PARTITION measurement_y2006m02;
;;
-- ddl.sgml
CREATE TABLE measurement_y2008m02 PARTITION OF measurement
    FOR VALUES FROM ('2008-02-01') TO ('2008-03-01')
    TABLESPACE fasttablespace;
;;
-- ddl.sgml
CREATE INDEX measurement_usls_idx ON ONLY measurement (unitsales);
;;
-- ddl.sgml
ALTER TABLE ONLY measurement ADD UNIQUE (city_id, logdate);
;;
-- ddl.sgml
CREATE TABLE measurement_y2006m02 () INHERITS (measurement);
;;
-- ddl.sgml
CREATE TABLE measurement_y2006m02 (
    CHECK ( logdate >= DATE '2006-02-01' AND logdate < DATE '2006-03-01' )
) INHERITS (measurement);
;;
-- ddl.sgml
CREATE INDEX measurement_y2006m02_logdate ON measurement_y2006m02 (logdate);
;;
-- ddl.sgml
CREATE OR REPLACE FUNCTION measurement_insert_trigger()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO measurement_y2008m01 VALUES (NEW.*);
    RETURN NULL;
END;
$$
LANGUAGE plpgsql;
;;
-- ddl.sgml
CREATE TRIGGER insert_measurement_trigger
    BEFORE INSERT ON measurement
    FOR EACH ROW EXECUTE FUNCTION measurement_insert_trigger();
;;
-- ddl.sgml
CREATE RULE measurement_insert_y2006m02 AS
ON INSERT TO measurement WHERE
    ( logdate >= DATE '2006-02-01' AND logdate < DATE '2006-03-01' )
DO INSTEAD
    INSERT INTO measurement_y2006m02 VALUES (NEW.*);
;;
-- ddl.sgml
ALTER TABLE measurement_y2006m02 NO INHERIT measurement;
;;
-- ddl.sgml
CREATE TABLE measurement_y2008m02 (
    CHECK ( logdate >= DATE '2008-02-01' AND logdate < DATE '2008-03-01' )
) INHERITS (measurement);
;;
-- ddl.sgml
ANALYZE ONLY measurement;
;;
-- ddl.sgml
SET enable_partition_pruning = on;                 -- the default
SELECT count(*) FROM measurement WHERE logdate >= DATE '2008-01-01';
;;
-- ddl.sgml
SET enable_partition_pruning = off;
;;
-- ddl.sgml
SET enable_partition_pruning = on;
;;
-- ddl.sgml
CREATE TYPE rainbow AS ENUM ('red', 'orange', 'yellow',
                             'green', 'blue', 'purple');
;;
-- ddl.sgml
CREATE FUNCTION get_color_note (rainbow) RETURNS text
BEGIN ATOMIC
  SELECT note FROM my_colors WHERE color = $1;
;;
-- dict-int.sgml
ALTER TEXT SEARCH CONFIGURATION english
    ALTER MAPPING FOR int, uint WITH intdict;
;;
-- dict-xsyn.sgml
ALTER TEXT SEARCH CONFIGURATION english
    ALTER MAPPING FOR word, asciiword WITH xsyn, english_stem;
;;
-- dml.sgml
INSERT INTO products VALUES (1, 'Cheese', 9.99);
;;
-- dml.sgml
INSERT INTO products (product_no, name, price) VALUES (1, 'Cheese', 9.99);
;;
-- dml.sgml
INSERT INTO products (product_no, name) VALUES (1, 'Cheese');
;;
-- dml.sgml
INSERT INTO products (product_no, name, price) VALUES (1, 'Cheese', DEFAULT);
;;
-- dml.sgml
INSERT INTO products (product_no, name, price) VALUES
    (1, 'Cheese', 9.99),
    (2, 'Bread', 1.99),
    (3, 'Milk', 2.99);
;;
-- dml.sgml
INSERT INTO products (product_no, name, price)
  SELECT product_no, name, price FROM new_products
    WHERE release_date = 'today';
;;
-- dml.sgml
UPDATE products SET price = 10 WHERE price = 5;
;;
-- dml.sgml
UPDATE products SET price = price * 1.10;
;;
-- dml.sgml
UPDATE mytable SET a = 5, b = 3, c = 1 WHERE a > 0;
;;
-- dml.sgml
DELETE FROM products WHERE price = 10;
;;
-- dml.sgml
DELETE FROM products;
;;
-- dml.sgml
CREATE TABLE users (firstname text, lastname text, id serial PRIMARY KEY);
;;
-- dml.sgml
UPDATE products SET price = price * 1.10
  WHERE price <= 99.99
  RETURNING name, price AS new_price;
;;
-- dml.sgml
DELETE FROM products
  WHERE obsoletion_date = 'today'
  RETURNING *;
;;
-- dml.sgml
MERGE INTO products p USING new_products n ON p.product_no = n.product_no
  WHEN NOT MATCHED THEN INSERT VALUES (n.product_no, n.name, n.price)
  WHEN MATCHED THEN UPDATE SET name = n.name, price = n.price
  RETURNING p.*;
;;
-- dml.sgml
UPDATE products SET price = price * 1.10
  WHERE price <= 99.99
  RETURNING name, old.price AS old_price, new.price AS new_price,
            new.price - old.price AS price_change;
;;
-- ecpg.sgml
CREATE TYPE comp_t AS (intval integer, textval varchar(32));
;;
-- ecpg.sgml
CREATE FUNCTION create_complex(r double, i double) RETURNS complex
LANGUAGE SQL
IMMUTABLE
AS $$ SELECT $1 * complex '(1,0')' + $2 * complex '(0,1)' $$;
;;
-- ecpg.sgml
$CLOSE DATABASE;                /* close the current connection */
EXEC SQL CLOSE DATABASE;
;;
-- event-trigger.sgml
CREATE FUNCTION noddl() RETURNS event_trigger
    AS 'noddl' LANGUAGE C;
;;
-- event-trigger.sgml
CREATE OR REPLACE FUNCTION no_rewrite()
 RETURNS event_trigger
 LANGUAGE plpgsql AS
$$
---
--- Implement local Table Rewriting policy:
---   public.foo is not allowed rewriting, ever
---   other tables are only allowed rewriting between 1am and 6am
---   unless they have more than 100 blocks
---
DECLARE
  table_oid oid := pg_event_trigger_table_rewrite_oid();
  current_hour integer := extract('hour' FROM current_time);
  pages integer;
  max_pages integer := 100;
BEGIN
  IF pg_event_trigger_table_rewrite_oid() = 'public.foo'::regclass
  THEN
        RAISE EXCEPTION 'you''re not allowed to rewrite the table %',
                        table_oid::regclass;
  END IF;

  SELECT INTO pages relpages FROM pg_class WHERE oid = table_oid;
  IF pages > max_pages
  THEN
        RAISE EXCEPTION 'rewrites only allowed for table with less than % pages',
                        max_pages;
  END IF;

  IF current_hour NOT BETWEEN 1 AND 6
  THEN
        RAISE EXCEPTION 'rewrites only allowed between 1am and 6am';
  END IF;
END;
$$;
;;
-- extend.sgml
SET LOCAL search_path TO @extschema@, pg_temp;
;;
-- extend.sgml
CREATE TABLE my_config (key text, value text);
;;
-- extend.sgml
CREATE TABLE my_config (key text, value text, standard_entry boolean);
;;
-- file-fdw.sgml
CREATE EXTENSION file_fdw;
;;
-- file-fdw.sgml
CREATE SERVER pglog FOREIGN DATA WRAPPER file_fdw;
;;
-- file-fdw.sgml
CREATE FOREIGN TABLE pglog (
  log_time timestamp(3) with time zone,
  user_name text,
  database_name text,
  process_id integer,
  connection_from text,
  session_id text,
  session_line_num bigint,
  command_tag text,
  session_start_time timestamp with time zone,
  virtual_transaction_id text,
  transaction_id bigint,
  error_severity text,
  sql_state_code text,
  message text,
  detail text,
  hint text,
  internal_query text,
  internal_query_pos integer,
  context text,
  query text,
  query_pos integer,
  location text,
  application_name text,
  backend_type text,
  leader_pid integer,
  query_id bigint
) SERVER pglog
OPTIONS ( filename 'log/pglog.csv', format 'csv' );
;;
-- file-fdw.sgml
CREATE FOREIGN TABLE films (
 code char(5) NOT NULL,
 title text NOT NULL,
 rating text OPTIONS (force_null 'true')
) SERVER film_server
OPTIONS ( filename 'films/db.csv', format 'csv' );
;;
-- func/func-admin.sgml
SELECT pg_restore_relation_stats(
    'schemaname', 'myschema',
    'relname',    'mytable',
    'relpages',   173::integer,
    'reltuples',  10000::real);
;;
-- func/func-admin.sgml
SELECT pg_restore_attribute_stats(
    'schemaname', 'myschema',
    'relname',    'mytable',
    'attname',    'col1',
    'inherited',  false,
    'avg_width',  125::integer,
    'null_frac',  0.5::real);
;;
-- func/func-admin.sgml
SELECT pg_size_pretty(sum(pg_relation_size(relid))) AS total_size
  FROM pg_partition_tree('measurement');
;;
-- func/func-admin.sgml
SELECT convert_from(pg_read_binary_file('file_in_utf8.txt'), 'UTF8');
;;
-- func/func-aggregate.sgml
SELECT count(*) FROM sometable;
;;
-- func/func-comparison.sgml
SELECT ROW(1,2.5,'this is a test') = ROW(1, 3, 'not the same');
;;
-- func/func-datetime.sgml
SELECT CURRENT_TIMESTAMP;
;;
-- func/func-datetime.sgml
SELECT pg_sleep(1.5);
;;
-- func/func-enum.sgml
CREATE TYPE rainbow AS ENUM ('red', 'orange', 'yellow', 'green', 'blue', 'purple');
;;
-- func/func-event-triggers.sgml
CREATE FUNCTION test_event_trigger_for_drops()
        RETURNS event_trigger LANGUAGE plpgsql AS $$
DECLARE
    obj record;
BEGIN
    FOR obj IN SELECT * FROM pg_event_trigger_dropped_objects()
    LOOP
        RAISE NOTICE '% dropped object: % %.% %',
                     tg_tag,
                     obj.object_type,
                     obj.schema_name,
                     obj.object_name,
                     obj.object_identity;
    END LOOP;
END;
$$;
;;
-- func/func-event-triggers.sgml
CREATE FUNCTION test_event_trigger_table_rewrite_oid()
 RETURNS event_trigger
 LANGUAGE plpgsql AS
$$
BEGIN
  RAISE NOTICE 'rewriting table % for reason %',
                pg_event_trigger_table_rewrite_oid()::regclass,
                pg_event_trigger_table_rewrite_reason();
END;
$$;
;;
-- func/func-info.sgml
SELECT has_table_privilege('myschema.mytable', 'select');
;;
-- func/func-info.sgml
SELECT has_function_privilege('joeuser', 'myfunc(int, text)', 'execute');
;;
-- func/func-info.sgml
SELECT relname FROM pg_class WHERE pg_table_is_visible(oid);
;;
-- func/func-info.sgml
SELECT pg_type_is_visible('myschema.widget'::regtype);
;;
-- func/func-info.sgml
SELECT currval(pg_get_serial_sequence('sometable', 'id'));
;;
-- func/func-json.sgml
SELECT js,
  js IS JSON "json?",
  js IS JSON SCALAR "scalar?",
  js IS JSON OBJECT "object?",
  js IS JSON ARRAY "array?"
FROM (VALUES
      ('123'), ('"abc"'), ('{"a": "b"}'), ('[1,2]'),('abc')) foo(js);
;;
-- func/func-json.sgml
SELECT js,
  js IS JSON OBJECT "object?",
  js IS JSON ARRAY "array?",
  js IS JSON ARRAY WITH UNIQUE KEYS "array w. UK?",
  js IS JSON ARRAY WITHOUT UNIQUE KEYS "array w/o UK?"
FROM (VALUES ('[{"a":"1"},
 {"b":"2","b":"3"}]')) foo(js);
;;
-- func/func-json.sgml
CREATE TABLE my_films ( js jsonb );
;;
-- func/func-json.sgml
SELECT jt.* FROM
 my_films,
 JSON_TABLE (js, '$.favorites[*]' COLUMNS (
   id FOR ORDINALITY,
   kind text PATH '$.kind',
   title text PATH '$.films[*].title' WITH WRAPPER,
   director text PATH '$.films[*].director' WITH WRAPPER)) AS jt;
;;
-- func/func-json.sgml
SELECT jt.* FROM
 my_films,
 JSON_TABLE (js, '$.favorites[*] ? (@.films[*].director == $filter)'
   PASSING 'Alfred Hitchcock' AS filter
     COLUMNS (
     id FOR ORDINALITY,
     kind text PATH '$.kind',
     title text FORMAT JSON PATH '$.films[*].title' OMIT QUOTES,
     director text PATH '$.films[*].director' KEEP QUOTES)) AS jt;
;;
-- func/func-json.sgml
SELECT jt.* FROM
 my_films,
 JSON_TABLE ( js, '$.favorites[*] ? (@.films[*].director == $filter)'
   PASSING 'Alfred Hitchcock' AS filter
   COLUMNS (
    id FOR ORDINALITY,
    kind text PATH '$.kind',
    NESTED PATH '$.films[*]' COLUMNS (
      title text FORMAT JSON PATH '$.title' OMIT QUOTES,
      director text PATH '$.director' KEEP QUOTES))) AS jt;
;;
-- func/func-json.sgml
SELECT jt.* FROM
 my_films,
 JSON_TABLE ( js, '$.favorites[*]'
   COLUMNS (
    id FOR ORDINALITY,
    kind text PATH '$.kind',
    NESTED PATH '$.films[*]' COLUMNS (
      title text FORMAT JSON PATH '$.title' OMIT QUOTES,
      director text PATH '$.director' KEEP QUOTES))) AS jt;
;;
-- func/func-json.sgml
SELECT * FROM JSON_TABLE (
'{"favorites":
    [{"movies":
      [{"name": "One", "director": "John Doe"},
       {"name": "Two", "director": "Don Joe"}],
     "books":
      [{"name": "Mystery", "authors": [{"name": "Brown Dan"}]},
       {"name": "Wonder", "authors": [{"name": "Jun Murakami"}, {"name":"Craig Doe"}]}]
}]}'::json, '$.favorites[*]'
COLUMNS (
  user_id FOR ORDINALITY,
  NESTED '$.movies[*]'
    COLUMNS (
    movie_id FOR ORDINALITY,
    mname text PATH '$.name',
    director text),
  NESTED '$.books[*]'
    COLUMNS (
      book_id FOR ORDINALITY,
      bname text PATH '$.name',
      NESTED '$.authors[*]'
        COLUMNS (
          author_id FOR ORDINALITY,
          author_name text PATH '$.name'))));
;;
-- func/func-matching.sgml
SELECT regexp_match('foobarbequebaz', 'bar.*que');
;;
-- func/func-matching.sgml
SELECT (regexp_match('foobarbequebaz', 'bar.*que'))[1];
;;
-- func/func-matching.sgml
SELECT regexp_matches('foo', 'not there');
;;
-- func/func-matching.sgml
SELECT col1, (SELECT regexp_matches(col2, '(bar)(beque)')) FROM tab;
;;
-- func/func-matching.sgml
SELECT foo FROM regexp_split_to_table('the quick brown fox jumps over the lazy dog', '\s+') AS foo;
;;
-- func/func-matching.sgml
SELECT regexp_split_to_array('the quick brown fox jumps over the lazy dog', '\s+');
;;
-- func/func-srf.sgml
SELECT * FROM generate_series(2,4);
;;
-- func/func-statistics.sgml
SELECT m.* FROM pg_statistic_ext join pg_statistic_ext_data on (oid = stxoid),
                pg_mcv_list_items(stxdmcv) m WHERE stxname = 'stts';
;;
-- func/func-trigger.sgml
CREATE TRIGGER z_min_update
BEFORE UPDATE ON tablename
FOR EACH ROW EXECUTE FUNCTION suppress_redundant_updates_trigger();
;;
-- fuzzystrmatch.sgml
SELECT soundex('hello world!');
;;
-- fuzzystrmatch.sgml
SELECT daitch_mokotoff('George');
;;
-- fuzzystrmatch.sgml
CREATE TABLE s (nm text);
;;
-- fuzzystrmatch.sgml
CREATE FUNCTION soundex_tsvector(v_name text) RETURNS tsvector
BEGIN ATOMIC
  SELECT to_tsvector('simple',
                     string_agg(array_to_string(daitch_mokotoff(n), ' '), ' '))
  FROM regexp_split_to_table(v_name, '\s+') AS n;
;;
-- gist.sgml
CREATE INDEX ON my_table USING GIST (my_inet_column inet_ops);
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_consistent(internal, data_type, smallint, oid, internal)
RETURNS bool
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_union(internal, internal)
RETURNS storage_type
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_compress(internal)
RETURNS internal
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_decompress(internal)
RETURNS internal
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_picksplit(internal, internal)
RETURNS internal
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_same(storage_type, storage_type, internal)
RETURNS internal
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_distance(internal, data_type, smallint, oid, internal)
RETURNS float8
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_fetch(internal)
RETURNS internal
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_options(internal)
RETURNS void
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_sortsupport(internal)
RETURNS void
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
CREATE OR REPLACE FUNCTION my_translate_cmptype(integer)
RETURNS smallint
AS 'MODULE_PATHNAME'
LANGUAGE C STRICT;
;;
-- gist.sgml
ALTER OPERATOR FAMILY my_opfamily USING gist ADD
    FUNCTION 12 ("any", "any") my_translate_cmptype(int);
;;
-- hstore.sgml
CREATE INDEX hidx ON testhstore USING GIST (h);
;;
-- hstore.sgml
CREATE INDEX hidx ON testhstore USING GIST (h gist_hstore_ops(siglen=32));
;;
-- hstore.sgml
CREATE INDEX hidx ON testhstore USING BTREE (h);
;;
-- hstore.sgml
UPDATE tab SET h['c'] = '3';
;;
-- hstore.sgml
UPDATE tab SET h = h || hstore('c', '3');
;;
-- hstore.sgml
UPDATE tab SET h = h || hstore(ARRAY['q', 'w'], ARRAY['11', '12']);
;;
-- hstore.sgml
UPDATE tab SET h = delete(h, 'k1');
;;
-- hstore.sgml
CREATE TABLE stat AS SELECT (each(h)).key, (each(h)).value FROM testhstore;
;;
-- hstore.sgml
SELECT key, count(*) FROM
  (SELECT (each(h)).key FROM testhstore) AS stat
  GROUP BY key
  ORDER BY count DESC, key;
;;
-- hstore.sgml
UPDATE tablename SET hstorecol = hstorecol || '';
;;
-- hstore.sgml
ALTER TABLE tablename ALTER hstorecol TYPE hstore USING hstorecol || '';
;;
-- indices.sgml
CREATE TABLE test1 (
    id integer,
    content varchar
);
;;
-- indices.sgml
CREATE INDEX test1_id_index ON test1 (id);
;;
-- indices.sgml
CREATE TABLE test2 (
  major int,
  minor int,
  name varchar
);
;;
-- indices.sgml
CREATE INDEX test2_mm_idx ON test2 (major, minor);
;;
-- indices.sgml
CREATE INDEX test2_info_nulls_low ON test2 (info NULLS FIRST);
;;
-- indices.sgml
SELECT * FROM test1 WHERE lower(col1) = 'value';
;;
-- indices.sgml
CREATE INDEX test1_lower_col1_idx ON test1 (lower(col1));
;;
-- indices.sgml
SELECT * FROM people WHERE (first_name || ' ' || last_name) = 'John Smith';
;;
-- indices.sgml
CREATE INDEX people_names ON people ((first_name || ' ' || last_name));
;;
-- indices.sgml
CREATE INDEX access_log_client_ip_ix ON access_log (client_ip)
WHERE NOT (client_ip > inet '192.168.100.0' AND
           client_ip < inet '192.168.100.255');
;;
-- indices.sgml
SELECT *
FROM access_log
WHERE url = '/index.html' AND client_ip = inet '212.78.10.32';
;;
-- indices.sgml
SELECT *
FROM access_log
WHERE url = '/index.html' AND client_ip = inet '192.168.100.23';
;;
-- indices.sgml
CREATE INDEX orders_unbilled_index ON orders (order_nr)
    WHERE billed IS NOT TRUE;
;;
-- indices.sgml
SELECT * FROM orders WHERE billed IS NOT TRUE AND order_nr < 10000;
;;
-- indices.sgml
SELECT * FROM orders WHERE billed IS NOT TRUE AND amount > 5000.00;
;;
-- indices.sgml
SELECT * FROM orders WHERE order_nr = 3501;
;;
-- indices.sgml
CREATE INDEX mytable_cat_data ON mytable (category, data);
;;
-- indices.sgml
SELECT x, y FROM tab WHERE x = 'key';
;;
-- indices.sgml
SELECT x, z FROM tab WHERE x = 'key';
;;
-- indices.sgml
SELECT y FROM tab WHERE x = 'key';
;;
-- indices.sgml
CREATE INDEX tab_x_y ON tab(x) INCLUDE (y);
;;
-- indices.sgml
CREATE UNIQUE INDEX tab_x_y ON tab(x) INCLUDE (y);
;;
-- indices.sgml
CREATE INDEX tab_x_y ON tab(x, y);
;;
-- indices.sgml
SELECT f(x) FROM tab WHERE f(x) < 1;
;;
-- indices.sgml
CREATE INDEX tab_f_x ON tab (f(x)) INCLUDE (x);
;;
-- indices.sgml
CREATE UNIQUE INDEX tests_success_constraint ON tests (subject, target)
    WHERE success;
;;
-- indices.sgml
SELECT target FROM tests WHERE subject = 'some-subject' AND success;
;;
-- indices.sgml
CREATE INDEX test_index ON test_table (col varchar_pattern_ops);
;;
-- indices.sgml
SELECT am.amname AS index_method,
       opc.opcname AS opclass_name,
       opc.opcintype::regtype AS indexed_type,
       opc.opcdefault AS is_default
    FROM pg_am am, pg_opclass opc
    WHERE opc.opcmethod = am.oid
    ORDER BY index_method, opclass_name;
;;
-- indices.sgml
SELECT am.amname AS index_method,
       opc.opcname AS opclass_name,
       opf.opfname AS opfamily_name,
       opc.opcintype::regtype AS indexed_type,
       opc.opcdefault AS is_default
    FROM pg_am am, pg_opclass opc, pg_opfamily opf
    WHERE opc.opcmethod = am.oid AND
          opc.opcfamily = opf.oid
    ORDER BY index_method, opclass_name;
;;
-- indices.sgml
SELECT am.amname AS index_method,
       opf.opfname AS opfamily_name,
       amop.amopopr::regoperator AS opfamily_operator
    FROM pg_am am, pg_opfamily opf, pg_amop amop
    WHERE opf.opfmethod = am.oid AND
          amop.amopfamily = opf.oid
    ORDER BY index_method, opfamily_name, opfamily_operator;
;;
-- indices.sgml
CREATE TABLE test1c (
    id integer,
    content varchar COLLATE "x"
);
;;
-- indices.sgml
CREATE INDEX test1c_content_y_index ON test1c (content COLLATE "y");
;;
-- intagg.sgml
CREATE TABLE summary AS
  SELECT id_left, int_array_aggregate(id_right) AS rights
  FROM many_to_many
  GROUP BY id_left;
;;
-- json.sgml
SELECT '{"bar": "baz", "balance": 7.77, "active":false}'::json;
;;
-- json.sgml
SELECT '{"reading": 1.230e-5}'::json, '{"reading": 1.230e-5}'::jsonb;
;;
-- json.sgml
SELECT doc->'site_name' FROM websites
  WHERE doc @> '{"tags":[{"term":"paris"}, {"term":"food"}]}';
;;
-- json.sgml
SELECT doc->'site_name' FROM websites
  WHERE doc->'tags' @> '[{"term":"paris"}, {"term":"food"}]';
;;
-- json.sgml
CREATE INDEX idxgin ON api USING GIN (jdoc);
;;
-- json.sgml
CREATE INDEX idxginp ON api USING GIN (jdoc jsonb_path_ops);
;;
-- json.sgml
CREATE INDEX idxgintags ON api USING GIN ((jdoc -> 'tags'));
;;
-- json.sgml
SELECT jdoc->'guid', jdoc->'name' FROM api WHERE jdoc @? '$.tags[*] ? (@ == "qui")';
;;
-- json.sgml
SELECT jdoc->'guid', jdoc->'name' FROM api WHERE jdoc @@ '$.tags[*] == "qui"';
;;
-- libpq.sgml
SELECT * FROM mytable WHERE x = $1::bigint;
;;
-- libpq.sgml
SELECT 1 AS FOO, 2 AS "BAR";
;;
-- libpq.sgml
UPDATE mytable SET x = x + 1 WHERE id = 42;
;;
-- lo.sgml
CREATE TABLE image (title text, raster lo);
;;
-- lobj.sgml
CREATE TABLE image (
    name            text,
    raster          oid
);
;;
-- logical-replication.sgml
CREATE PUBLICATION mypub FOR TABLE users, departments;
;;
-- logical-replication.sgml
CREATE SUBSCRIPTION mysub CONNECTION 'dbname=foo host=bar user=repuser' PUBLICATION mypub;
;;
-- logicaldecoding.sgml
ALTER TABLE user_catalog_table SET (user_catalog_table = true);
;;
-- ltree.sgml
CREATE INDEX path_gist_idx ON test USING GIST (path);
;;
-- ltree.sgml
CREATE INDEX path_gist_idx ON test USING GIST (path gist_ltree_ops(siglen=100));
;;
-- ltree.sgml
CREATE INDEX path_gist_idx ON test USING GIST (array_path);
;;
-- ltree.sgml
CREATE INDEX path_gist_idx ON test USING GIST (array_path gist__ltree_ops(siglen=100));
;;
-- ltree.sgml
CREATE TABLE test (path ltree);
;;
-- maintenance.sgml
SELECT c.oid::regclass AS table_name,
       greatest(age(c.relfrozenxid), age(t.relfrozenxid)) AS age
FROM pg_class c
LEFT JOIN pg_class t ON c.reltoastrelid = t.oid
WHERE c.relkind IN ('r', 'm');
;;
-- manage-ag.sgml
ALTER DATABASE mydb SET geqo TO off;
;;
-- manage-ag.sgml
CREATE TABLESPACE fastspace LOCATION '/ssd1/postgresql/data';
;;
-- manage-ag.sgml
CREATE TABLE foo(i int) TABLESPACE space1;
;;
-- manage-ag.sgml
SET default_tablespace = space1;
;;
-- monitoring.sgml
SELECT pid, wait_event_type, wait_event FROM pg_stat_activity WHERE wait_event IS NOT NULL;
;;
-- monitoring.sgml
SELECT a.pid, a.wait_event, w.description
  FROM pg_stat_activity a JOIN
       pg_wait_events w ON (a.wait_event_type = w.type AND
                            a.wait_event = w.name)
  WHERE a.wait_event IS NOT NULL AND a.state = 'active';
;;
-- monitoring.sgml
SELECT pg_stat_get_backend_pid(backendid) AS pid,
       pg_stat_get_backend_activity(backendid) AS query
FROM pg_stat_get_backend_idset() AS backendid;
;;
-- monitoring.sgml
SELECT pg_relation_filepath(oid), relpages FROM pg_class WHERE relname = 'customer';
;;
-- monitoring.sgml
SELECT relname, relpages
FROM pg_class,
     (SELECT reltoastrelid
      FROM pg_class
      WHERE relname = 'customer') AS ss
WHERE oid = ss.reltoastrelid OR
      oid = (SELECT indexrelid
             FROM pg_index
             WHERE indrelid = ss.reltoastrelid)
ORDER BY relname;
;;
-- monitoring.sgml
SELECT c2.relname, c2.relpages
FROM pg_class c, pg_class c2, pg_index i
WHERE c.relname = 'customer' AND
      c.oid = i.indrelid AND
      c2.oid = i.indexrelid
ORDER BY c2.relname;
;;
-- monitoring.sgml
SELECT relname, relpages
FROM pg_class
ORDER BY relpages DESC;
;;
-- perform.sgml
SELECT relpages, reltuples FROM pg_class WHERE relname = 'tenk1';
;;
-- perform.sgml
CREATE STATISTICS stts (dependencies) ON city, zip FROM zipcodes;
;;
-- perform.sgml
SELECT * FROM zipcodes WHERE city = 'San Francisco' AND zip = '94105';
;;
-- perform.sgml
SELECT * FROM zipcodes WHERE city = 'San Francisco' AND zip = '90210';
;;
-- perform.sgml
CREATE STATISTICS stts2 (ndistinct) ON city, state, zip FROM zipcodes;
;;
-- perform.sgml
CREATE STATISTICS stts3 (mcv) ON city, state FROM zipcodes;
;;
-- perform.sgml
SELECT * FROM a, b, c WHERE a.id = b.id AND b.ref = c.id;
;;
-- perform.sgml
SELECT * FROM a LEFT JOIN (b JOIN c ON (b.ref = c.id)) ON (a.id = b.id);
;;
-- perform.sgml
SELECT * FROM a LEFT JOIN b ON (a.bid = b.id) LEFT JOIN c ON (a.cid = c.id);
;;
-- perform.sgml
SELECT *
FROM x, y,
    (SELECT * FROM a, b, c WHERE something) AS ss
WHERE somethingelse;
;;
-- perform.sgml
SELECT * FROM x, y, a, b, c WHERE something AND somethingelse;
;;
-- pgcrypto.sgml
CREATE OR REPLACE FUNCTION sha1(bytea) RETURNS text AS $$
    SELECT encode(digest($1, 'sha1'), 'hex')
$$ LANGUAGE SQL STRICT IMMUTABLE;
;;
-- pgoverexplain.sgml
LOAD 'pg_overexplain';
;;
-- pgplanadvice.sgml
EXPLAIN (COSTS OFF, PLAN_ADVICE)
        SELECT * FROM join_fact f JOIN join_dim d ON f.dim_id = d.id;
;;
-- pgplanadvice.sgml
SET pg_plan_advice.advice = 'JOIN_ORDER(f d)';
;;
-- pgplanadvice.sgml
SET pg_plan_advice.advice = 'JOIN_ORDER(x f d)';
;;
-- pgplanadvice.sgml
SET pg_plan_advice.advice = 'hash_join(f g) join_order(f g) index_scan(f no_such_index)';
;;
-- pgrowlocks.sgml
SELECT * FROM accounts AS a, pgrowlocks('accounts') AS p
  WHERE p.locked_row = a.ctid;
;;
-- pgtrgm.sgml
CREATE TABLE test_trgm (t text);
;;
-- pgtrgm.sgml
CREATE INDEX trgm_idx ON test_trgm USING GIN (t gin_trgm_ops);
;;
-- pgtrgm.sgml
CREATE INDEX trgm_idx ON test_trgm USING GIST (t gist_trgm_ops(siglen=32));
;;
-- pgtrgm.sgml
SELECT * FROM test_trgm WHERE t LIKE '%foo%bar';
;;
-- pgtrgm.sgml
SELECT * FROM test_trgm WHERE t ~ '(foo|bar)';
;;
-- pgtrgm.sgml
CREATE TABLE words AS SELECT word FROM
        ts_stat('SELECT to_tsvector(''simple'', bodytext) FROM documents');
;;
-- pgtrgm.sgml
CREATE INDEX words_idx ON words USING GIN (word gin_trgm_ops);
;;
-- planstats.sgml
EXPLAIN SELECT * FROM tenk1;
;;
-- planstats.sgml
EXPLAIN SELECT * FROM tenk1 WHERE unique1 < 1000;
;;
-- planstats.sgml
SELECT histogram_bounds FROM pg_stats
WHERE tablename='tenk1' AND attname='unique1';
;;
-- planstats.sgml
EXPLAIN SELECT * FROM tenk1 WHERE stringu1 = 'CRAAAA';
;;
-- planstats.sgml
SELECT null_frac, n_distinct, most_common_vals, most_common_freqs FROM pg_stats
WHERE tablename='tenk1' AND attname='stringu1';
;;
-- planstats.sgml
EXPLAIN SELECT * FROM tenk1 WHERE stringu1 = 'xxx';
;;
-- planstats.sgml
EXPLAIN SELECT * FROM tenk1 WHERE stringu1 < 'IAAAAA';
;;
-- planstats.sgml
SELECT histogram_bounds FROM pg_stats
WHERE tablename='tenk1' AND attname='stringu1';
;;
-- planstats.sgml
EXPLAIN SELECT * FROM tenk1 WHERE unique1 < 1000 AND stringu1 = 'xxx';
;;
-- planstats.sgml
EXPLAIN SELECT * FROM tenk1 t1, tenk2 t2
WHERE t1.unique1 < 50 AND t1.unique2 = t2.unique2;
;;
-- planstats.sgml
SELECT tablename, null_frac,n_distinct, most_common_vals FROM pg_stats
WHERE tablename IN ('tenk1', 'tenk2') AND attname='unique2';
;;
-- planstats.sgml
CREATE TABLE t (a INT, b INT);
;;
-- planstats.sgml
SELECT relpages, reltuples FROM pg_class WHERE relname = 't';
;;
-- planstats.sgml
EXPLAIN (ANALYZE, TIMING OFF, BUFFERS OFF) SELECT * FROM t WHERE a = 1;
;;
-- planstats.sgml
EXPLAIN (ANALYZE, TIMING OFF, BUFFERS OFF) SELECT * FROM t WHERE a = 1 AND b = 1;
;;
-- planstats.sgml
CREATE STATISTICS stts (dependencies) ON a, b FROM t;
;;
-- planstats.sgml
EXPLAIN (ANALYZE, TIMING OFF, BUFFERS OFF) SELECT COUNT(*) FROM t GROUP BY a;
;;
-- planstats.sgml
EXPLAIN (ANALYZE, TIMING OFF, BUFFERS OFF) SELECT COUNT(*) FROM t GROUP BY a, b;
;;
-- planstats.sgml
DROP STATISTICS stts;
;;
-- planstats.sgml
SELECT m.* FROM pg_statistic_ext JOIN pg_statistic_ext_data ON (oid = stxoid),
                pg_mcv_list_items(stxdmcv) m WHERE stxname = 'stts2';
;;
-- planstats.sgml
EXPLAIN (ANALYZE, TIMING OFF, BUFFERS OFF) SELECT * FROM t WHERE a = 1 AND b = 10;
;;
-- planstats.sgml
EXPLAIN (ANALYZE, TIMING OFF, BUFFERS OFF) SELECT * FROM t WHERE a <= 49 AND b > 49;
;;
-- plperl.sgml
DO $$
    # PL/Perl code
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE FUNCTION perl_max (integer, integer) RETURNS integer AS $$
    if ($_[0] > $_[1]) { return $_[0]; }
    return $_[1];
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE FUNCTION perl_max (integer, integer) RETURNS integer AS $$
    my ($x, $y) = @_;
    if (not defined $x) {
        return undef if not defined $y;
        return $y;
    }
    return $x if not defined $y;
    return $x if $x > $y;
    return $y;
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE FUNCTION perl_and(bool, bool) RETURNS bool
TRANSFORM FOR TYPE bool
AS $$
  my ($a, $b) = @_;
  return $a && $b;
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE OR REPLACE FUNCTION returns_array()
RETURNS text[][] AS $$
    return [['a"b','c,d'],['e\\f','g']];
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE OR REPLACE FUNCTION concat_array_elements(text[]) RETURNS TEXT AS $$
    my $arg = shift;
    my $result = "";
    return undef if (!defined $arg);

    # as an array reference
    for (@$arg) {
        $result .= $_;
    }

    # also works as a string
    $result .= $arg;

    return $result;
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE TABLE employee (
    name text,
    basesalary integer,
    bonus integer
);
;;
-- plperl.sgml
CREATE TABLE test (
    i int,
    v varchar
);
;;
-- plperl.sgml
CREATE OR REPLACE FUNCTION init() RETURNS VOID AS $$
        $_SHARED{my_plan} = spi_prepare('SELECT (now() + $1)::date AS now',
                                        'INTERVAL');
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE PROCEDURE transaction_test1()
LANGUAGE plperl
AS $$
foreach my $i (0..9) {
    spi_exec_query("INSERT INTO test1 (a) VALUES ($i)");
    if ($i % 2 == 0) {
        spi_commit();
    } else {
        spi_rollback();
    }
}
$$;
;;
-- plperl.sgml
CREATE OR REPLACE FUNCTION set_var(name text, val text) RETURNS text AS $$
    if ($_SHARED{$_[0]} = $_[1]) {
        return 'ok';
    } else {
        return "cannot set shared variable $_[0] to $_[1]";
    }
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE OR REPLACE FUNCTION myfuncs() RETURNS void AS $$
    $_SHARED{myquote} = sub {
        my $arg = shift;
        $arg =~ s/(['\\])/\\$1/g;
        return "'$arg'";
    };
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE FUNCTION badfunc() RETURNS integer AS $$
    my $tmpfile = "/tmp/badfile";
    open my $fh, '>', $tmpfile
        or elog(ERROR, qq{could not open the file "$tmpfile": $!});
    print $fh "Testing writing to a file\n";
    close $fh or elog(ERROR, qq{could not close the file "$tmpfile": $!});
    return 1;
$$ LANGUAGE plperl;
;;
-- plperl.sgml
CREATE OR REPLACE FUNCTION perlsnitch() RETURNS event_trigger AS $$
  elog(NOTICE, "perlsnitch: " . $_TD->{event} . " " . $_TD->{tag} . " ");
$$ LANGUAGE plperl;
;;
-- plperl.sgml
DO 'elog(WARNING, join ", ", sort keys %INC)' LANGUAGE plperl;
;;
-- plpgsql.sgml
CREATE FUNCTION somefunc() RETURNS integer AS $$
<< outerblock >>
DECLARE
    quantity integer := 30;
BEGIN
    RAISE NOTICE 'Quantity here is %', quantity;  -- Prints 30
    quantity := 50;
    --
    -- Create a subblock
    --
    DECLARE
        quantity integer := 80;
    BEGIN
        RAISE NOTICE 'Quantity here is %', quantity;  -- Prints 80
        RAISE NOTICE 'Outer quantity here is %', outerblock.quantity;  -- Prints 50
    END;

    RAISE NOTICE 'Quantity here is %', quantity;  -- Prints 50

    RETURN quantity;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
DECLARE
  x integer := 1;
;;
-- plpgsql.sgml
CREATE FUNCTION sales_tax(subtotal real) RETURNS real AS $$
BEGIN
    RETURN subtotal * 0.06;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION sales_tax(real) RETURNS real AS $$
DECLARE
    subtotal ALIAS FOR $1;
BEGIN
    RETURN subtotal * 0.06;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION instr(varchar, integer) RETURNS integer AS $$
DECLARE
    v_string ALIAS FOR $1;
    index ALIAS FOR $2;
BEGIN
    -- some computations using v_string and index here
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION sales_tax(subtotal real, OUT tax real) AS $$
BEGIN
    tax := subtotal * 0.06;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
SELECT sales_tax(100.00);
;;
-- plpgsql.sgml
CREATE FUNCTION sum_n_product(x int, y int, OUT sum int, OUT prod int) AS $$
BEGIN
    sum := x + y;
    prod := x * y;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE PROCEDURE sum_n_product(x int, y int, OUT sum int, OUT prod int) AS $$
BEGIN
    sum := x + y;
    prod := x * y;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CALL sum_n_product(2, 4, NULL, NULL);
;;
-- plpgsql.sgml
CREATE FUNCTION extended_sales(p_itemno int)
RETURNS TABLE(quantity int, total numeric) AS $$
BEGIN
    RETURN QUERY SELECT s.quantity, s.quantity * s.price FROM sales AS s
                 WHERE s.itemno = p_itemno;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION add_three_values(v1 anyelement, v2 anyelement, v3 anyelement)
RETURNS anyelement AS $$
DECLARE
    result ALIAS FOR $0;
BEGIN
    result := v1 + v2 + v3;
    RETURN result;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION add_three_values(v1 anyelement, v2 anyelement, v3 anyelement,
                                 OUT sum anyelement)
AS $$
BEGIN
    sum := v1 + v2 + v3;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION add_three_values(v1 anycompatible, v2 anycompatible, v3 anycompatible)
RETURNS anycompatible AS $$
BEGIN
    RETURN v1 + v2 + v3;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
SELECT add_three_values(1, 2, 4.7);
;;
-- plpgsql.sgml
DECLARE
  prior ALIAS FOR old;
;;
-- plpgsql.sgml
CREATE FUNCTION less_than(a text, b text) RETURNS boolean AS $$
BEGIN
    RETURN a < b;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION less_than(a text, b text) RETURNS boolean AS $$
DECLARE
    local_a text := a;
    local_b text := b;
BEGIN
    RETURN local_a < local_b;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
DECLARE
    local_a text COLLATE "en_US";
;;
-- plpgsql.sgml
CREATE FUNCTION less_than_c(a text, b text) RETURNS boolean AS $$
BEGIN
    RETURN a < b COLLATE "C";
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE TABLE mytable (id int PRIMARY KEY, data text);
;;
-- plpgsql.sgml
SELECT * INTO myrec FROM emp WHERE empname = myname;
;;
-- plpgsql.sgml
BEGIN
    SELECT * INTO STRICT myrec FROM emp WHERE empname = myname;
;;
-- plpgsql.sgml
CREATE FUNCTION get_userid(username text) RETURNS int
AS $$
#print_strict_params on
DECLARE
userid int;
BEGIN
    SELECT users.userid INTO STRICT userid
        FROM users WHERE users.username = get_userid.username;
    RETURN userid;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
EXECUTE 'SELECT count(*) FROM mytable WHERE inserted_by = $1 AND inserted <= $2'
   INTO c
   USING checked_user, checked_date;
;;
-- plpgsql.sgml
EXECUTE 'SELECT count(*) FROM '
    || quote_ident(tabname)
    || ' WHERE inserted_by = $1 AND inserted <= $2'
   INTO c
   USING checked_user, checked_date;
;;
-- plpgsql.sgml
EXECUTE format('SELECT count(*) FROM %I '
   'WHERE inserted_by = $1 AND inserted <= $2', tabname)
   INTO c
   USING checked_user, checked_date;
;;
-- plpgsql.sgml
EXECUTE format('UPDATE tbl SET %I = $1 '
   'WHERE key = $2', colname) USING newvalue, keyvalue;
;;
-- plpgsql.sgml
EXECUTE 'UPDATE tbl SET '
        || quote_ident(colname)
        || ' = '
        || quote_literal(newvalue)
        || ' WHERE key = '
        || quote_literal(keyvalue);
;;
-- plpgsql.sgml
EXECUTE 'UPDATE tbl SET '
        || quote_ident(colname)
        || ' = '
        || quote_nullable(newvalue)
        || ' WHERE key = '
        || quote_nullable(keyvalue);
;;
-- plpgsql.sgml
EXECUTE 'UPDATE tbl SET '
        || quote_ident(colname)
        || ' = $$'
        || newvalue
        || '$$ WHERE key = '
        || quote_literal(keyvalue);
;;
-- plpgsql.sgml
EXECUTE format('UPDATE tbl SET %I = %L '
   'WHERE key = %L', colname, newvalue, keyvalue);
;;
-- plpgsql.sgml
EXECUTE format('UPDATE tbl SET %I = $1 WHERE key = $2', colname)
   USING newvalue, keyvalue;
;;
-- plpgsql.sgml
BEGIN
    y := x / 0;
;;
-- plpgsql.sgml
CREATE TABLE foo (fooid INT, foosubid INT, fooname TEXT);
;;
-- plpgsql.sgml
CREATE FUNCTION get_available_flightid(date) RETURNS SETOF integer AS
$BODY$
BEGIN
    RETURN QUERY SELECT flightid
                   FROM flight
                  WHERE flightdate >= $1
                    AND flightdate < ($1 + 1);

    -- Since execution is not finished, we can check whether rows were returned
    -- and raise exception if not.
    IF NOT FOUND THEN
        RAISE EXCEPTION 'No flight at %.', $1;
    END IF;

    RETURN;
 END;
$BODY$
LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE PROCEDURE triple(INOUT x int)
LANGUAGE plpgsql
AS $$
BEGIN
    x := x * 3;
END;
$$;
;;
-- plpgsql.sgml
CREATE FUNCTION sum(int[]) RETURNS int8 AS $$
DECLARE
  s int8 := 0;
  x int;
BEGIN
  FOREACH x IN ARRAY $1
  LOOP
    s := s + x;
  END LOOP;
  RETURN s;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION scan_rows(int[]) RETURNS void AS $$
DECLARE
  x int[];
BEGIN
  FOREACH x SLICE 1 IN ARRAY $1
  LOOP
    RAISE NOTICE 'row = %', x;
  END LOOP;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
INSERT INTO mytab(firstname, lastname) VALUES('Tom', 'Jones');
;;
-- plpgsql.sgml
CREATE TABLE db (a INT PRIMARY KEY, b TEXT);
;;
-- plpgsql.sgml
DECLARE
  text_var1 text;
;;
-- plpgsql.sgml
CREATE OR REPLACE FUNCTION outer_func() RETURNS integer AS $$
BEGIN
  RETURN inner_func();
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
DECLARE
    curs1 refcursor;
;;
-- plpgsql.sgml
DECLARE
    key integer;
;;
-- plpgsql.sgml
FETCH curs1 INTO rowvar;
;;
-- plpgsql.sgml
MOVE curs1;
;;
-- plpgsql.sgml
UPDATE foo SET dataval = myval WHERE CURRENT OF curs1;
;;
-- plpgsql.sgml
CLOSE curs1;
;;
-- plpgsql.sgml
CREATE TABLE test (col text);
;;
-- plpgsql.sgml
CREATE FUNCTION reffunc2() RETURNS refcursor AS '
DECLARE
    ref refcursor;
BEGIN
    OPEN ref FOR SELECT col FROM test;
    RETURN ref;
END;
' LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION myfunc(refcursor, refcursor) RETURNS SETOF refcursor AS $$
BEGIN
    OPEN $1 FOR SELECT * FROM table_1;
    RETURN NEXT $1;
    OPEN $2 FOR SELECT * FROM table_2;
    RETURN NEXT $2;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE PROCEDURE transaction_test1()
LANGUAGE plpgsql
AS $$
BEGIN
    FOR i IN 0..9 LOOP
        INSERT INTO test1 (a) VALUES (i);
        IF i % 2 = 0 THEN
            COMMIT;
        ELSE
            ROLLBACK;
        END IF;
    END LOOP;
END;
$$;
;;
-- plpgsql.sgml
CREATE PROCEDURE transaction_test2()
LANGUAGE plpgsql
AS $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN SELECT * FROM test2 ORDER BY x LOOP
        INSERT INTO test1 (a) VALUES (r.x);
        COMMIT;
    END LOOP;
END;
$$;
;;
-- plpgsql.sgml
CREATE TABLE emp (
    empname           text,
    salary            integer,
    last_date         timestamp,
    last_user         text
);
;;
-- plpgsql.sgml
CREATE TABLE emp (
    empname           text NOT NULL,
    salary            integer
);
;;
-- plpgsql.sgml
CREATE TABLE emp (
    empname           text PRIMARY KEY,
    salary            integer
);
;;
-- plpgsql.sgml
CREATE OR REPLACE FUNCTION snitch() RETURNS event_trigger AS $$
BEGIN
    RAISE NOTICE 'snitch: % %', tg_event, tg_tag;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
INSERT INTO foo (foo) VALUES (foo(foo));
;;
-- plpgsql.sgml
INSERT INTO dest (col) SELECT foo + bar FROM src;
;;
-- plpgsql.sgml
CREATE FUNCTION stamp_user(id int, comment text) RETURNS void AS $$
    #variable_conflict use_variable
    DECLARE
        curtime timestamp := now();
    BEGIN
        UPDATE users SET last_modified = curtime, comment = comment
          WHERE users.id = id;
    END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION stamp_user(id int, comment text) RETURNS void AS $$
    <<fn>>
    DECLARE
        curtime timestamp := now();
    BEGIN
        UPDATE users SET last_modified = fn.curtime, comment = stamp_user.comment
          WHERE users.id = stamp_user.id;
    END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION logfunc1(logtxt text) RETURNS void AS $$
    BEGIN
        INSERT INTO logtable VALUES (logtxt, 'now');
    END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE FUNCTION logfunc2(logtxt text) RETURNS void AS $$
    DECLARE
        curtime timestamp;
    BEGIN
        curtime := 'now';
        INSERT INTO logtable VALUES (logtxt, curtime);
    END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
SET plpgsql.extra_warnings TO 'shadowed_variables';
;;
-- plpgsql.sgml
SET plpgsql.extra_warnings TO 'strict_multi_assignment';
;;
-- plpgsql.sgml
CREATE OR REPLACE FUNCTION cs_fmt_browser_version(v_name varchar2,
                                                  v_version varchar2)
RETURN varchar2 IS
BEGIN
    IF v_version IS NULL THEN
        RETURN v_name;
;;
-- plpgsql.sgml
CREATE OR REPLACE FUNCTION cs_fmt_browser_version(v_name varchar,
                                                  v_version varchar)
RETURNS varchar AS $$
BEGIN
    IF v_version IS NULL THEN
        RETURN v_name;
    END IF;
    RETURN v_name || '/' || v_version;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE OR REPLACE PROCEDURE cs_update_referrer_type_proc IS
    CURSOR referrer_keys IS
        SELECT * FROM cs_referrer_keys
        ORDER BY try_order;
;;
-- plpgsql.sgml
CREATE OR REPLACE PROCEDURE cs_update_referrer_type_proc() AS $func$
DECLARE
    referrer_keys CURSOR IS
        SELECT * FROM cs_referrer_keys
        ORDER BY try_order;
    func_body text;
    func_cmd text;
BEGIN
    func_body := 'BEGIN';

    FOR referrer_key IN referrer_keys LOOP
        func_body := func_body ||
          ' IF v_' || referrer_key.kind
          || ' LIKE ' || quote_literal(referrer_key.key_string)
          || ' THEN RETURN ' || quote_literal(referrer_key.referrer_type)
          || '; END IF;' ;
    END LOOP;

    func_body := func_body || ' RETURN NULL; END;';

    func_cmd :=
      'CREATE OR REPLACE FUNCTION cs_find_referrer_type(v_host varchar,
                                                        v_domain varchar,
                                                        v_url varchar)
        RETURNS varchar AS '
      || quote_literal(func_body)
      || ' LANGUAGE plpgsql;' ;

    EXECUTE func_cmd;
END;
$func$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
CREATE OR REPLACE PROCEDURE cs_parse_url(
    v_url IN VARCHAR2,
    v_host OUT VARCHAR2,  -- This will be passed back
    v_path OUT VARCHAR2,  -- This one too
    v_query OUT VARCHAR2) -- And this one
IS
    a_pos1 INTEGER;
;;
-- plpgsql.sgml
CREATE OR REPLACE FUNCTION cs_parse_url(
    v_url IN VARCHAR,
    v_host OUT VARCHAR,  -- This will be passed back
    v_path OUT VARCHAR,  -- This one too
    v_query OUT VARCHAR) -- And this one
AS $$
DECLARE
    a_pos1 INTEGER;
    a_pos2 INTEGER;
BEGIN
    v_host := NULL;
    v_path := NULL;
    v_query := NULL;
    a_pos1 := instr(v_url, '//');

    IF a_pos1 = 0 THEN
        RETURN;
    END IF;
    a_pos2 := instr(v_url, '/', a_pos1 + 2);
    IF a_pos2 = 0 THEN
        v_host := substr(v_url, a_pos1 + 2);
        v_path := '/';
        RETURN;
    END IF;

    v_host := substr(v_url, a_pos1 + 2, a_pos2 - a_pos1 - 2);
    a_pos1 := instr(v_url, '?', a_pos2 + 1);

    IF a_pos1 = 0 THEN
        v_path := substr(v_url, a_pos2);
        RETURN;
    END IF;

    v_path := substr(v_url, a_pos2, a_pos1 - a_pos2);
    v_query := substr(v_url, a_pos1 + 1);
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
SELECT * FROM cs_parse_url('http://foobar.com/query.cgi?baz');
;;
-- plpgsql.sgml
CREATE OR REPLACE PROCEDURE cs_create_job(v_job_id IN INTEGER) IS
    a_running_job_count INTEGER;
;;
-- plpgsql.sgml
CREATE OR REPLACE PROCEDURE cs_create_job(v_job_id integer) AS $$
DECLARE
    a_running_job_count integer;
BEGIN
    LOCK TABLE cs_jobs IN EXCLUSIVE MODE;

    SELECT count(*) INTO a_running_job_count FROM cs_jobs WHERE end_stamp IS NULL;

    IF a_running_job_count > 0 THEN
        COMMIT; -- free lock
        RAISE EXCEPTION 'Unable to create a new job: a job is currently running'; -- 
    END IF;

    DELETE FROM cs_active_job;
    INSERT INTO cs_active_job(job_id) VALUES (v_job_id);

    BEGIN
        INSERT INTO cs_jobs (job_id, start_stamp) VALUES (v_job_id, now());
    EXCEPTION
        WHEN unique_violation THEN -- 
            -- don't worry if it already exists
    END;
    COMMIT;
END;
$$ LANGUAGE plpgsql;
;;
-- plpgsql.sgml
BEGIN
    SAVEPOINT s1;
;;
-- plpython.sgml
CREATE FUNCTION pymax (a integer, b integer)
  RETURNS integer
AS $$
  if a > b:
    return a
  return b
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION pystrip(x text)
  RETURNS text
AS $$
  x = x.strip()  # error
  return x
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION pystrip(x text)
  RETURNS text
AS $$
  global x
  x = x.strip()  # ok now
  return x
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION pymax (a integer, b integer)
  RETURNS integer
AS $$
  if (a is None) or (b is None):
    return None
  if a > b:
    return a
  return b
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION return_arr()
  RETURNS int[]
AS $$
return [1, 2, 3, 4, 5]
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION test_type_conversion_array_int4(x int4[]) RETURNS int4[] AS $$
plpy.info(x, type(x))
return x
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION return_str_arr()
  RETURNS varchar[]
AS $$
return "hello"
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE TABLE employee (
  name text,
  salary integer,
  age integer
);
;;
-- plpython.sgml
CREATE TYPE named_value AS (
  name   text,
  value  integer
);
;;
-- plpython.sgml
CREATE FUNCTION make_pair (name text, value integer)
  RETURNS named_value
AS $$
  return ( name, value )
  # or alternatively, as list: return [ name, value ]
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION make_pair (name text, value integer)
  RETURNS named_value
AS $$
  return { "name": name, "value": value }
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION make_pair (name text, value integer)
  RETURNS named_value
AS $$
  class named_value:
    def __init__ (self, n, v):
      self.name = n
      self.value = v
  return named_value(name, value)

  # or simply
  class nv: pass
  nv.name = name
  nv.value = value
  return nv
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION multiout_simple(OUT i integer, OUT j integer) AS $$
return (1, 2)
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE PROCEDURE python_triple(INOUT a integer, INOUT b integer) AS $$
return (a * 3, b * 3)
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE TYPE greeting AS (
  how text,
  who text
);
;;
-- plpython.sgml
CREATE FUNCTION greet (how text)
  RETURNS SETOF greeting
AS $$
  # return tuple containing lists as composite types
  # all other combinations work also
  return ( [ how, "World" ], [ how, "PostgreSQL" ], [ how, "PL/Python" ] )
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION greet (how text)
  RETURNS SETOF greeting
AS $$
  class producer:
    def __init__ (self, how, who):
      self.how = how
      self.who = who
      self.ndx = -1

    def __iter__ (self):
      return self

    def __next__(self):
      self.ndx += 1
      if self.ndx == len(self.who):
        raise StopIteration
      return ( self.how, self.who[self.ndx] )

  return producer(how, [ "World", "PostgreSQL", "PL/Python" ])
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION greet (how text)
  RETURNS SETOF greeting
AS $$
  for who in [ "World", "PostgreSQL", "PL/Python" ]:
    yield ( how, who )
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION multiout_simple_setof(n integer, OUT integer, OUT integer) RETURNS SETOF record AS $$
return [(1, 2)] * n
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
DO $$
    # PL/Python code
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION usesavedplan() RETURNS trigger AS $$
    if "plan" in SD:
        plan = SD["plan"]
    else:
        plan = plpy.prepare("SELECT 1")
        SD["plan"] = plan
    # rest of function
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION count_odd_iterator() RETURNS integer AS $$
odd = 0
for row in plpy.cursor("SELECT num FROM largetable"):
    if row['num'] % 2:
         odd += 1
return odd
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION try_adding_joe() RETURNS text AS $$
    try:
        plpy.execute("INSERT INTO users(username) VALUES ('joe')")
    except plpy.SPIError:
        return "something went wrong"
    else:
        return "Joe added"
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION insert_fraction(numerator int, denominator int) RETURNS text AS $$
from plpy import spiexceptions
try:
    plan = plpy.prepare("INSERT INTO fractions (frac) VALUES ($1 / $2)", ["int", "int"])
    plpy.execute(plan, [numerator, denominator])
except spiexceptions.DivisionByZero:
    return "denominator cannot equal zero"
except spiexceptions.UniqueViolation:
    return "already have that fraction"
except plpy.SPIError as e:
    return "other error, SQLSTATE %s" % e.sqlstate
else:
    return "fraction inserted"
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION transfer_funds() RETURNS void AS $$
try:
    plpy.execute("UPDATE accounts SET balance = balance - 100 WHERE account_name = 'joe'")
    plpy.execute("UPDATE accounts SET balance = balance + 100 WHERE account_name = 'mary'")
except plpy.SPIError as e:
    result = "error transferring funds: %s" % e.args
else:
    result = "funds transferred correctly"
plan = plpy.prepare("INSERT INTO operations (result) VALUES ($1)", ["text"])
plpy.execute(plan, [result])
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE FUNCTION transfer_funds2() RETURNS void AS $$
try:
    with plpy.subtransaction():
        plpy.execute("UPDATE accounts SET balance = balance - 100 WHERE account_name = 'joe'")
        plpy.execute("UPDATE accounts SET balance = balance + 100 WHERE account_name = 'mary'")
except plpy.SPIError as e:
    result = "error transferring funds: %s" % e.args
else:
    result = "funds transferred correctly"
plan = plpy.prepare("INSERT INTO operations (result) VALUES ($1)", ["text"])
plpy.execute(plan, [result])
$$ LANGUAGE plpython3u;
;;
-- plpython.sgml
CREATE PROCEDURE transaction_test1()
LANGUAGE plpython3u
AS $$
for i in range(0, 10):
    plpy.execute("INSERT INTO test1 (a) VALUES (%d)" % i)
    if i % 2 == 0:
        plpy.commit()
    else:
        plpy.rollback()
$$;
;;
-- pltcl.sgml
CREATE FUNCTION tcl_max(integer, integer) RETURNS integer AS $$
    if {$1 > $2} {return $1}
    return $2
$$ LANGUAGE pltcl STRICT;
;;
-- pltcl.sgml
CREATE FUNCTION tcl_max(integer, integer) RETURNS integer AS $$
    if {[argisnull 1]} {
        if {[argisnull 2]} { return_null }
        return $2
    }
    if {[argisnull 2]} { return $1 }
    if {$1 > $2} {return $1}
    return $2
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE TABLE employee (
    name text,
    salary integer,
    age integer
);
;;
-- pltcl.sgml
CREATE FUNCTION square_cube(IN int, OUT squared int, OUT cubed int) AS $$
    return [list squared [expr {$1 * $1}] cubed [expr {$1 * $1 * $1}]]
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE PROCEDURE tcl_triple(INOUT a integer, INOUT b integer) AS $$
    return [list a [expr {$1 * 3}] b [expr {$2 * 3}]]
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE FUNCTION raise_pay(employee, delta int) RETURNS employee AS $$
    set 1(salary) [expr {$1(salary) + $2}]
    return [array get 1]
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE FUNCTION sequence(int, int) RETURNS SETOF int AS $$
    for {set i $1} {$i < $2} {incr i} {
        return_next $i
    }
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE FUNCTION table_of_squares(int, int) RETURNS TABLE (x int, x2 int) AS $$
    for {set i $1} {$i < $2} {incr i} {
        return_next [list x $i x2 [expr {$i * $i}]]
    }
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE FUNCTION t1_count(integer, integer) RETURNS integer AS $$
    if {![ info exists GD(plan) ]} {
        # prepare the saved plan on the first call
        set GD(plan) [ spi_prepare \
                "SELECT count(*) AS cnt FROM t1 WHERE num >= \$1 AND num <= \$2" \
                [ list int4 int4 ] ]
    }
    spi_execp -count 1 $GD(plan) [ list $1 $2 ]
    return $cnt
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE FUNCTION trigfunc_modcount() RETURNS trigger AS $$
    switch $TG_op {
        INSERT {
            set NEW($1) 0
        }
        UPDATE {
            set NEW($1) $OLD($1)
            incr NEW($1)
        }
        default {
            return OK
        }
    }
    return [array get NEW]
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE OR REPLACE FUNCTION tclsnitch() RETURNS event_trigger AS $$
  elog NOTICE "tclsnitch: $TG_event $TG_tag"
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE FUNCTION transfer_funds() RETURNS void AS $$
    if [catch {
        spi_exec "UPDATE accounts SET balance = balance - 100 WHERE account_name = 'joe'"
        spi_exec "UPDATE accounts SET balance = balance + 100 WHERE account_name = 'mary'"
    } errormsg] {
        set result [format "error transferring funds: %s" $errormsg]
    } else {
        set result "funds transferred successfully"
    }
    spi_exec "INSERT INTO operations (result) VALUES ('[quote $result]')"
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE FUNCTION transfer_funds2() RETURNS void AS $$
    if [catch {
        subtransaction {
            spi_exec "UPDATE accounts SET balance = balance - 100 WHERE account_name = 'joe'"
            spi_exec "UPDATE accounts SET balance = balance + 100 WHERE account_name = 'mary'"
        }
    } errormsg] {
        set result [format "error transferring funds: %s" $errormsg]
    } else {
        set result "funds transferred successfully"
    }
    spi_exec "INSERT INTO operations (result) VALUES ('[quote $result]')"
$$ LANGUAGE pltcl;
;;
-- pltcl.sgml
CREATE PROCEDURE transaction_test1()
LANGUAGE pltcl
AS $$
for {set i 0} {$i < 10} {incr i} {
    spi_exec "INSERT INTO test1 (a) VALUES ($i)"
    if {$i % 2 == 0} {
        commit
    } else {
        rollback
    }
}
$$;
;;
-- postgres-fdw.sgml
ALTER USER MAPPING FOR some_non_superuser SERVER loopback_nopw
OPTIONS (ADD password_required 'false');
;;
-- postgres-fdw.sgml
CREATE SERVER subscription_server FOREIGN DATA WRAPPER postgres_fdw OPTIONS (host 'foreign-host', dbname 'foreign_db');
;;
-- postgres-fdw.sgml
CREATE EXTENSION postgres_fdw;
;;
-- postgres-fdw.sgml
CREATE SERVER foreign_server
        FOREIGN DATA WRAPPER postgres_fdw
        OPTIONS (host '192.83.123.89', port '5432', dbname 'foreign_db');
;;
-- postgres-fdw.sgml
CREATE USER MAPPING FOR local_user
        SERVER foreign_server
        OPTIONS (user 'foreign_user', password 'password');
;;
-- postgres-fdw.sgml
CREATE FOREIGN TABLE foreign_table (
        id integer NOT NULL,
        data text
)
        SERVER foreign_server
        OPTIONS (schema_name 'some_schema', table_name 'some_table');
;;
-- protocol.sgml
INSERT INTO mytable VALUES(1);
;;
-- queries.sgml
SELECT * FROM table1;
;;
-- queries.sgml
SELECT a, b + c FROM table1;
;;
-- queries.sgml
SELECT 3 * 4;
;;
-- queries.sgml
SELECT random();
;;
-- queries.sgml
SELECT * FROM some_very_long_table_name s JOIN another_fairly_long_name a ON s.id = a.num;
;;
-- queries.sgml
SELECT * FROM people AS mother JOIN people AS child ON mother.id = child.mother_id;
;;
-- queries.sgml
CREATE TABLE foo (fooid int, foosubid int, fooname text);
;;
-- queries.sgml
SELECT *
    FROM dblink('dbname=mydb', 'SELECT proname, prosrc FROM pg_proc')
      AS t1(proname name, prosrc text)
    WHERE proname LIKE 'bytea%';
;;
-- queries.sgml
SELECT *
FROM ROWS FROM
    (
        json_to_recordset('[{"a":40,"b":"foo"},{"a":"100","b":"bar"}]')
            AS (a INTEGER, b TEXT),
        generate_series(1, 3)
    ) AS x (p, q, s)
ORDER BY p;
;;
-- queries.sgml
SELECT * FROM foo, LATERAL (SELECT * FROM bar WHERE bar.id = foo.bar_id) ss;
;;
-- queries.sgml
SELECT * FROM foo, bar WHERE bar.id = foo.bar_id;
;;
-- queries.sgml
SELECT p1.id, p2.id, v1, v2
FROM polygons p1, polygons p2,
     LATERAL vertices(p1.poly) v1,
     LATERAL vertices(p2.poly) v2
WHERE (v1 <-> v2) < 10 AND p1.id != p2.id;
;;
-- queries.sgml
SELECT p1.id, p2.id, v1, v2
FROM polygons p1 CROSS JOIN LATERAL vertices(p1.poly) v1,
     polygons p2 CROSS JOIN LATERAL vertices(p2.poly) v2
WHERE (v1 <-> v2) < 10 AND p1.id != p2.id;
;;
-- queries.sgml
SELECT m.name
FROM manufacturers m LEFT JOIN LATERAL get_product_names(m.id) pname ON true
WHERE pname IS NULL;
;;
-- queries.sgml
SELECT product_id, p.name, (sum(s.units) * p.price) AS sales
    FROM products p LEFT JOIN sales s USING (product_id)
    GROUP BY product_id, p.name, p.price;
;;
-- queries.sgml
SELECT product_id, p.name, (sum(s.units) * (p.price - p.cost)) AS profit
    FROM products p LEFT JOIN sales s USING (product_id)
    WHERE s.date > CURRENT_DATE - INTERVAL '4 weeks'
    GROUP BY product_id, p.name, p.price, p.cost
    HAVING sum(p.price * s.units) > 5000;
;;
-- queries.sgml
SELECT a, b FROM table1 ORDER BY a + b, c;
;;
-- queries.sgml
SELECT a + b AS sum, c FROM table1 ORDER BY sum;
;;
-- queries.sgml
VALUES (1, 'one'), (2, 'two'), (3, 'three');
;;
-- queries.sgml
SELECT 1 AS column1, 'one' AS column2
UNION ALL
SELECT 2, 'two'
UNION ALL
SELECT 3, 'three';
;;
-- queries.sgml
WITH regional_sales AS (
    SELECT region, SUM(amount) AS total_sales
    FROM orders
    GROUP BY region
), top_regions AS (
    SELECT region
    FROM regional_sales
    WHERE total_sales > (SELECT SUM(total_sales)/10 FROM regional_sales)
)
SELECT region,
       product,
       SUM(quantity) AS product_units,
       SUM(amount) AS product_sales
FROM orders
WHERE region IN (SELECT region FROM top_regions)
GROUP BY region, product;
;;
-- queries.sgml
WITH RECURSIVE t(n) AS (
    VALUES (1)
  UNION ALL
    SELECT n+1 FROM t WHERE n < 100
)
SELECT sum(n) FROM t;
;;
-- queries.sgml
WITH RECURSIVE search_tree(id, link, data) AS (
    SELECT t.id, t.link, t.data
    FROM tree t
  UNION ALL
    SELECT t.id, t.link, t.data
    FROM tree t, search_tree st
    WHERE t.id = st.link
)
SELECT * FROM search_tree;
;;
-- queries.sgml
WITH RECURSIVE search_tree(id, link, data, path) AS (
    SELECT t.id, t.link, t.data, ARRAY[t.id]
    FROM tree t
  UNION ALL
    SELECT t.id, t.link, t.data, path || t.id
    FROM tree t, search_tree st
    WHERE t.id = st.link
)
SELECT * FROM search_tree ORDER BY path;
;;
-- queries.sgml
WITH RECURSIVE search_tree(id, link, data, path) AS (
    SELECT t.id, t.link, t.data, ARRAY[ROW(t.f1, t.f2)]
    FROM tree t
  UNION ALL
    SELECT t.id, t.link, t.data, path || ROW(t.f1, t.f2)
    FROM tree t, search_tree st
    WHERE t.id = st.link
)
SELECT * FROM search_tree ORDER BY path;
;;
-- queries.sgml
WITH RECURSIVE search_tree(id, link, data, depth) AS (
    SELECT t.id, t.link, t.data, 0
    FROM tree t
  UNION ALL
    SELECT t.id, t.link, t.data, depth + 1
    FROM tree t, search_tree st
    WHERE t.id = st.link
)
SELECT * FROM search_tree ORDER BY depth;
;;
-- queries.sgml
WITH RECURSIVE search_tree(id, link, data) AS (
    SELECT t.id, t.link, t.data
    FROM tree t
  UNION ALL
    SELECT t.id, t.link, t.data
    FROM tree t, search_tree st
    WHERE t.id = st.link
) SEARCH DEPTH FIRST BY id SET ordercol
SELECT * FROM search_tree ORDER BY ordercol;
;;
-- queries.sgml
WITH RECURSIVE search_graph(id, link, data, depth) AS (
    SELECT g.id, g.link, g.data, 0
    FROM graph g
  UNION ALL
    SELECT g.id, g.link, g.data, sg.depth + 1
    FROM graph g, search_graph sg
    WHERE g.id = sg.link
)
SELECT * FROM search_graph;
;;
-- queries.sgml
WITH RECURSIVE search_graph(id, link, data, depth, is_cycle, path) AS (
    SELECT g.id, g.link, g.data, 0,
      false,
      ARRAY[g.id]
    FROM graph g
  UNION ALL
    SELECT g.id, g.link, g.data, sg.depth + 1,
      g.id = ANY(path),
      path || g.id
    FROM graph g, search_graph sg
    WHERE g.id = sg.link AND NOT is_cycle
)
SELECT * FROM search_graph;
;;
-- queries.sgml
WITH RECURSIVE search_graph(id, link, data, depth, is_cycle, path) AS (
    SELECT g.id, g.link, g.data, 0,
      false,
      ARRAY[ROW(g.f1, g.f2)]
    FROM graph g
  UNION ALL
    SELECT g.id, g.link, g.data, sg.depth + 1,
      ROW(g.f1, g.f2) = ANY(path),
      path || ROW(g.f1, g.f2)
    FROM graph g, search_graph sg
    WHERE g.id = sg.link AND NOT is_cycle
)
SELECT * FROM search_graph;
;;
-- queries.sgml
WITH RECURSIVE search_graph(id, link, data, depth) AS (
    SELECT g.id, g.link, g.data, 1
    FROM graph g
  UNION ALL
    SELECT g.id, g.link, g.data, sg.depth + 1
    FROM graph g, search_graph sg
    WHERE g.id = sg.link
) CYCLE id SET is_cycle USING path
SELECT * FROM search_graph;
;;
-- queries.sgml
WITH RECURSIVE t(n) AS (
    SELECT 1
  UNION ALL
    SELECT n+1 FROM t
)
SELECT n FROM t LIMIT 100;
;;
-- queries.sgml
WITH w AS (
    SELECT * FROM big_table
)
SELECT * FROM w WHERE key = 123;
;;
-- queries.sgml
SELECT * FROM big_table WHERE key = 123;
;;
-- queries.sgml
WITH w AS (
    SELECT * FROM big_table
)
SELECT * FROM w AS w1 JOIN w AS w2 ON w1.key = w2.ref
WHERE w2.key = 123;
;;
-- queries.sgml
WITH w AS NOT MATERIALIZED (
    SELECT * FROM big_table
)
SELECT * FROM w AS w1 JOIN w AS w2 ON w1.key = w2.ref
WHERE w2.key = 123;
;;
-- queries.sgml
WITH w AS (
    SELECT key, very_expensive_function(val) AS f FROM some_table
)
SELECT * FROM w AS w1 JOIN w AS w2 ON w1.f = w2.f;
;;
-- queries.sgml
WITH moved_rows AS (
    DELETE FROM products
    WHERE
        "date" >= '2010-10-01' AND
        "date" < '2010-11-01'
    RETURNING *
)
INSERT INTO products_log
SELECT * FROM moved_rows;
;;
-- queries.sgml
WITH t AS (
    DELETE FROM foo
)
DELETE FROM bar;
;;
-- queries.sgml
WITH RECURSIVE included_parts(sub_part, part) AS (
    SELECT sub_part, part FROM parts WHERE part = 'our_product'
  UNION ALL
    SELECT p.sub_part, p.part
    FROM included_parts pr, parts p
    WHERE p.part = pr.sub_part
)
DELETE FROM parts
  WHERE part IN (SELECT part FROM included_parts);
;;
-- queries.sgml
WITH t AS (
    UPDATE products SET price = price * 1.05
    RETURNING *
)
SELECT * FROM products;
;;
-- queries.sgml
WITH t AS (
    UPDATE products SET price = price * 1.05
    RETURNING *
)
SELECT * FROM t;
;;
-- query.sgml
CREATE TABLE weather (
    city            varchar(80),
    temp_lo         int,           -- low temperature
    temp_hi         int,           -- high temperature
    prcp            real,          -- precipitation
    date            date
);
;;
-- query.sgml
CREATE TABLE cities (
    name            varchar(80),
    location        point
);
;;
-- query.sgml
INSERT INTO weather VALUES ('San Francisco', 46, 50, 0.25, '1994-11-27');
;;
-- query.sgml
INSERT INTO cities VALUES ('San Francisco', '(-194.0, 53.0)');
;;
-- query.sgml
INSERT INTO weather (city, temp_lo, temp_hi, prcp, date)
    VALUES ('San Francisco', 43, 57, 0.0, '1994-11-29');
;;
-- query.sgml
INSERT INTO weather (date, city, temp_hi, temp_lo)
    VALUES ('1994-11-29', 'Hayward', 54, 37);
;;
-- query.sgml
COPY weather FROM '/home/user/weather.txt';
;;
-- query.sgml
SELECT * FROM weather;
;;
-- query.sgml
SELECT city, temp_lo, temp_hi, prcp, date FROM weather;
;;
-- query.sgml
SELECT city, (temp_hi+temp_lo)/2 AS temp_avg, date FROM weather;
;;
-- query.sgml
SELECT * FROM weather
    WHERE city = 'San Francisco' AND prcp > 0.0;
;;
-- query.sgml
SELECT * FROM weather
    ORDER BY city;
;;
-- query.sgml
SELECT * FROM weather
    ORDER BY city, temp_lo;
;;
-- query.sgml
SELECT DISTINCT city
    FROM weather;
;;
-- query.sgml
SELECT DISTINCT city
    FROM weather
    ORDER BY city;
;;
-- query.sgml
SELECT * FROM weather JOIN cities ON city = name;
;;
-- query.sgml
SELECT city, temp_lo, temp_hi, prcp, date, location
    FROM weather JOIN cities ON city = name;
;;
-- query.sgml
SELECT weather.city, weather.temp_lo, weather.temp_hi,
       weather.prcp, weather.date, cities.location
    FROM weather JOIN cities ON weather.city = cities.name;
;;
-- query.sgml
SELECT *
    FROM weather, cities
    WHERE city = name;
;;
-- query.sgml
SELECT *
    FROM weather LEFT OUTER JOIN cities ON weather.city = cities.name;
;;
-- query.sgml
SELECT w1.city, w1.temp_lo AS low, w1.temp_hi AS high,
       w2.city, w2.temp_lo AS low, w2.temp_hi AS high
    FROM weather w1 JOIN weather w2
        ON w1.temp_lo < w2.temp_lo AND w1.temp_hi > w2.temp_hi;
;;
-- query.sgml
SELECT *
    FROM weather w JOIN cities c ON w.city = c.name;
;;
-- query.sgml
SELECT max(temp_lo) FROM weather;
;;
-- query.sgml
SELECT city FROM weather
    WHERE temp_lo = (SELECT max(temp_lo) FROM weather);
;;
-- query.sgml
SELECT city, count(*), max(temp_lo)
    FROM weather
    GROUP BY city;
;;
-- query.sgml
SELECT city, count(*), max(temp_lo)
    FROM weather
    GROUP BY city
    HAVING max(temp_lo) < 40;
;;
-- query.sgml
SELECT city, count(*), max(temp_lo)
    FROM weather
    WHERE city LIKE 'S%'            -- 
    GROUP BY city;
;;
-- query.sgml
SELECT city, count(*) FILTER (WHERE temp_lo < 45), max(temp_lo)
    FROM weather
    GROUP BY city;
;;
-- query.sgml
UPDATE weather
    SET temp_hi = temp_hi - 2,  temp_lo = temp_lo - 2
    WHERE date > '1994-11-28';
;;
-- query.sgml
DELETE FROM weather WHERE city = 'Hayward';
;;
-- rangetypes.sgml
CREATE TABLE reservation (room int, during tsrange);
;;
-- rangetypes.sgml
SELECT '{}'::int4multirange;
;;
-- rangetypes.sgml
SELECT nummultirange();
;;
-- rangetypes.sgml
CREATE TYPE floatrange AS RANGE (
    subtype = float8,
    subtype_diff = float8mi
);
;;
-- rangetypes.sgml
CREATE FUNCTION time_subtype_diff(x time, y time) RETURNS float8 AS
'SELECT EXTRACT(EPOCH FROM (x - y))' LANGUAGE sql STRICT IMMUTABLE;
;;
-- rangetypes.sgml
CREATE INDEX reservation_idx ON reservation USING GIST (during);
;;
-- rangetypes.sgml
CREATE TABLE reservation (
    during tsrange,
    EXCLUDE USING GIST (during WITH &&)
);
;;
-- rangetypes.sgml
INSERT INTO reservation VALUES
    ('[2010-01-01 11:30, 2010-01-01 15:00)');
;;
-- rangetypes.sgml
CREATE EXTENSION btree_gist;
;;
-- ref/alter_aggregate.sgml
ALTER AGGREGATE myavg(integer) RENAME TO my_average;
;;
-- ref/alter_aggregate.sgml
ALTER AGGREGATE myavg(integer) OWNER TO joe;
;;
-- ref/alter_aggregate.sgml
ALTER AGGREGATE mypercentile(float8 ORDER BY integer) SET SCHEMA myschema;
;;
-- ref/alter_aggregate.sgml
ALTER AGGREGATE mypercentile(float8, integer) SET SCHEMA myschema;
;;
-- ref/alter_collation.sgml
ALTER COLLATION "de_DE" RENAME TO german;
;;
-- ref/alter_collation.sgml
ALTER COLLATION "en_US" OWNER TO joe;
;;
-- ref/alter_conversion.sgml
ALTER CONVERSION iso_8859_1_to_utf8 RENAME TO latin1_to_unicode;
;;
-- ref/alter_conversion.sgml
ALTER CONVERSION iso_8859_1_to_utf8 OWNER TO joe;
;;
-- ref/alter_database.sgml
ALTER DATABASE test SET enable_indexscan TO off;
;;
-- ref/alter_default_privileges.sgml
ALTER DEFAULT PRIVILEGES IN SCHEMA myschema GRANT SELECT ON TABLES TO PUBLIC;
;;
-- ref/alter_default_privileges.sgml
ALTER DEFAULT PRIVILEGES IN SCHEMA myschema REVOKE SELECT ON TABLES FROM PUBLIC;
;;
-- ref/alter_default_privileges.sgml
ALTER DEFAULT PRIVILEGES FOR ROLE admin REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;
;;
-- ref/alter_default_privileges.sgml
ALTER DEFAULT PRIVILEGES IN SCHEMA public REVOKE EXECUTE ON FUNCTIONS FROM PUBLIC;
;;
-- ref/alter_domain.sgml
ALTER DOMAIN zipcode SET NOT NULL;
;;
-- ref/alter_domain.sgml
ALTER DOMAIN zipcode DROP NOT NULL;
;;
-- ref/alter_domain.sgml
ALTER DOMAIN zipcode ADD CONSTRAINT zipchk CHECK (char_length(VALUE) = 5);
;;
-- ref/alter_domain.sgml
ALTER DOMAIN zipcode DROP CONSTRAINT zipchk;
;;
-- ref/alter_domain.sgml
ALTER DOMAIN zipcode RENAME CONSTRAINT zipchk TO zip_check;
;;
-- ref/alter_domain.sgml
ALTER DOMAIN zipcode SET SCHEMA customers;
;;
-- ref/alter_extension.sgml
ALTER EXTENSION hstore UPDATE TO '2.0';
;;
-- ref/alter_extension.sgml
ALTER EXTENSION hstore SET SCHEMA utils;
;;
-- ref/alter_extension.sgml
ALTER EXTENSION hstore ADD FUNCTION populate_record(anyelement, hstore);
;;
-- ref/alter_foreign_data_wrapper.sgml
ALTER FOREIGN DATA WRAPPER dbi OPTIONS (ADD foo '1', DROP bar);
;;
-- ref/alter_foreign_data_wrapper.sgml
ALTER FOREIGN DATA WRAPPER dbi VALIDATOR bob.myvalidator;
;;
-- ref/alter_foreign_table.sgml
ALTER FOREIGN TABLE distributors ALTER COLUMN street SET NOT NULL;
;;
-- ref/alter_foreign_table.sgml
ALTER FOREIGN TABLE myschema.distributors OPTIONS (ADD opt1 'value', SET opt2 'value2', DROP opt3);
;;
-- ref/alter_function.sgml
ALTER FUNCTION sqrt(integer) RENAME TO square_root;
;;
-- ref/alter_function.sgml
ALTER FUNCTION sqrt(integer) OWNER TO joe;
;;
-- ref/alter_function.sgml
ALTER FUNCTION sqrt(integer) SET SCHEMA maths;
;;
-- ref/alter_function.sgml
ALTER FUNCTION sqrt(integer) DEPENDS ON EXTENSION mathlib;
;;
-- ref/alter_function.sgml
ALTER FUNCTION check_password(text) SET search_path = admin, pg_temp;
;;
-- ref/alter_function.sgml
ALTER FUNCTION check_password(text) RESET search_path;
;;
-- ref/alter_group.sgml
ALTER GROUP staff ADD USER karl, john;
;;
-- ref/alter_group.sgml
ALTER GROUP workers DROP USER beth;
;;
-- ref/alter_index.sgml
ALTER INDEX distributors RENAME TO suppliers;
;;
-- ref/alter_index.sgml
ALTER INDEX distributors SET TABLESPACE fasttablespace;
;;
-- ref/alter_index.sgml
ALTER INDEX distributors SET (fillfactor = 75);
;;
-- ref/alter_index.sgml
CREATE INDEX coord_idx ON measured (x, y, (z + t));
;;
-- ref/alter_materialized_view.sgml
ALTER MATERIALIZED VIEW foo RENAME TO bar;
;;
-- ref/alter_operator.sgml
ALTER OPERATOR @@ (text, text) OWNER TO joe;
;;
-- ref/alter_operator.sgml
ALTER OPERATOR && (int[], int[]) SET (RESTRICT = _int_contsel, JOIN = _int_contjoinsel);
;;
-- ref/alter_operator.sgml
ALTER OPERATOR && (int[], int[]) SET (COMMUTATOR = &&);
;;
-- ref/alter_opfamily.sgml
ALTER OPERATOR FAMILY integer_ops USING btree ADD

  -- int4 vs int2
  OPERATOR 1 < (int4, int2) ,
  OPERATOR 2 <= (int4, int2) ,
  OPERATOR 3 = (int4, int2) ,
  OPERATOR 4 >= (int4, int2) ,
  OPERATOR 5 > (int4, int2) ,
  FUNCTION 1 btint42cmp(int4, int2) ,

  -- int2 vs int4
  OPERATOR 1 < (int2, int4) ,
  OPERATOR 2 <= (int2, int4) ,
  OPERATOR 3 = (int2, int4) ,
  OPERATOR 4 >= (int2, int4) ,
  OPERATOR 5 > (int2, int4) ,
  FUNCTION 1 btint24cmp(int2, int4) ;
;;
-- ref/alter_opfamily.sgml
ALTER OPERATOR FAMILY integer_ops USING btree DROP

  -- int4 vs int2
  OPERATOR 1 (int4, int2) ,
  OPERATOR 2 (int4, int2) ,
  OPERATOR 3 (int4, int2) ,
  OPERATOR 4 (int4, int2) ,
  OPERATOR 5 (int4, int2) ,
  FUNCTION 1 (int4, int2) ,

  -- int2 vs int4
  OPERATOR 1 (int2, int4) ,
  OPERATOR 2 (int2, int4) ,
  OPERATOR 3 (int2, int4) ,
  OPERATOR 4 (int2, int4) ,
  OPERATOR 5 (int2, int4) ,
  FUNCTION 1 (int2, int4) ;
;;
-- ref/alter_procedure.sgml
ALTER PROCEDURE insert_data(integer, integer) RENAME TO insert_record;
;;
-- ref/alter_procedure.sgml
ALTER PROCEDURE insert_data(integer, integer) OWNER TO joe;
;;
-- ref/alter_procedure.sgml
ALTER PROCEDURE insert_data(integer, integer) SET SCHEMA accounting;
;;
-- ref/alter_procedure.sgml
ALTER PROCEDURE insert_data(integer, integer) DEPENDS ON EXTENSION myext;
;;
-- ref/alter_procedure.sgml
ALTER PROCEDURE check_password(text) SET search_path = admin, pg_temp;
;;
-- ref/alter_procedure.sgml
ALTER PROCEDURE check_password(text) RESET search_path;
;;
-- ref/alter_publication.sgml
ALTER PUBLICATION noinsert SET (publish = 'update, delete');
;;
-- ref/alter_publication.sgml
ALTER PUBLICATION mypublication ADD TABLE users (user_id, firstname), departments;
;;
-- ref/alter_publication.sgml
ALTER PUBLICATION mypublication SET TABLE users (user_id, firstname, lastname), TABLE departments;
;;
-- ref/alter_publication.sgml
ALTER PUBLICATION mypublication SET ALL TABLES EXCEPT (TABLE users, departments);
;;
-- ref/alter_publication.sgml
ALTER PUBLICATION mypublication SET ALL TABLES;
;;
-- ref/alter_publication.sgml
ALTER PUBLICATION sales_publication ADD TABLES IN SCHEMA marketing, sales;
;;
-- ref/alter_publication.sgml
ALTER PUBLICATION production_publication ADD TABLE users, departments, TABLES IN SCHEMA production;
;;
-- ref/alter_role.sgml
ALTER ROLE davide WITH PASSWORD 'hu8jmn3';
;;
-- ref/alter_role.sgml
ALTER ROLE davide WITH PASSWORD NULL;
;;
-- ref/alter_role.sgml
ALTER ROLE chris VALID UNTIL 'May 4 12:00:00 2015 +1';
;;
-- ref/alter_role.sgml
ALTER ROLE fred VALID UNTIL 'infinity';
;;
-- ref/alter_role.sgml
ALTER ROLE miriam CREATEROLE CREATEDB;
;;
-- ref/alter_role.sgml
ALTER ROLE worker_bee SET maintenance_work_mem = 100000;
;;
-- ref/alter_role.sgml
ALTER ROLE fred IN DATABASE devel SET client_min_messages = DEBUG;
;;
-- ref/alter_routine.sgml
ALTER ROUTINE foo(integer) RENAME TO foobar;
;;
-- ref/alter_rule.sgml
ALTER RULE notify_all ON emp RENAME TO notify_me;
;;
-- ref/alter_sequence.sgml
ALTER SEQUENCE serial RESTART WITH 105;
;;
-- ref/alter_server.sgml
ALTER SERVER foo OPTIONS (host 'foo', dbname 'foodb');
;;
-- ref/alter_server.sgml
ALTER SERVER foo VERSION '8.4' OPTIONS (SET host 'baz');
;;
-- ref/alter_subscription.sgml
ALTER SUBSCRIPTION mysub SET PUBLICATION insert_only;
;;
-- ref/alter_subscription.sgml
ALTER SUBSCRIPTION mysub DISABLE;
;;
-- ref/alter_system.sgml
ALTER SYSTEM SET wal_level = replica;
;;
-- ref/alter_system.sgml
ALTER SYSTEM RESET wal_level;
;;
-- ref/alter_system.sgml
ALTER SYSTEM SET shared_preload_libraries TO NULL;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ADD COLUMN address varchar(30);
;;
-- ref/alter_table.sgml
ALTER TABLE measurements
  ADD COLUMN mtime timestamp with time zone DEFAULT now();
;;
-- ref/alter_table.sgml
ALTER TABLE transactions
  ADD COLUMN status varchar(30) DEFAULT 'old',
  ALTER COLUMN status SET DEFAULT 'current';
;;
-- ref/alter_table.sgml
ALTER TABLE distributors DROP COLUMN address RESTRICT;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors
    ALTER COLUMN address TYPE varchar(80),
    ALTER COLUMN name TYPE varchar(100);
;;
-- ref/alter_table.sgml
ALTER TABLE foo
    ALTER COLUMN foo_timestamp SET DATA TYPE timestamp with time zone
    USING
        timestamp with time zone 'epoch' + foo_timestamp * interval '1 second';
;;
-- ref/alter_table.sgml
ALTER TABLE foo
    ALTER COLUMN foo_timestamp DROP DEFAULT,
    ALTER COLUMN foo_timestamp TYPE timestamp with time zone
    USING
        timestamp with time zone 'epoch' + foo_timestamp * interval '1 second',
    ALTER COLUMN foo_timestamp SET DEFAULT now();
;;
-- ref/alter_table.sgml
ALTER TABLE distributors RENAME COLUMN address TO city;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors RENAME TO suppliers;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors RENAME CONSTRAINT zipchk TO zip_check;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ALTER COLUMN street SET NOT NULL;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ALTER COLUMN street DROP NOT NULL;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ADD CONSTRAINT zipchk CHECK (char_length(zipcode) = 5);
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ADD CONSTRAINT zipchk CHECK (char_length(zipcode) = 5) NO INHERIT;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors DROP CONSTRAINT zipchk;
;;
-- ref/alter_table.sgml
ALTER TABLE ONLY distributors DROP CONSTRAINT zipchk;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address);
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ADD CONSTRAINT distfk FOREIGN KEY (address) REFERENCES addresses (address) NOT VALID;
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ADD CONSTRAINT dist_id_zipcode_key UNIQUE (dist_id, zipcode);
;;
-- ref/alter_table.sgml
ALTER TABLE distributors ADD PRIMARY KEY (dist_id);
;;
-- ref/alter_table.sgml
ALTER TABLE distributors SET TABLESPACE fasttablespace;
;;
-- ref/alter_table.sgml
ALTER TABLE myschema.distributors SET SCHEMA yourschema;
;;
-- ref/alter_table.sgml
CREATE UNIQUE INDEX CONCURRENTLY dist_id_temp_idx ON distributors (dist_id);
;;
-- ref/alter_table.sgml
ALTER TABLE measurement
    ATTACH PARTITION measurement_y2016m07 FOR VALUES FROM ('2016-07-01') TO ('2016-08-01');
;;
-- ref/alter_table.sgml
ALTER TABLE cities
    ATTACH PARTITION cities_ab FOR VALUES IN ('a', 'b');
;;
-- ref/alter_table.sgml
ALTER TABLE orders
    ATTACH PARTITION orders_p4 FOR VALUES WITH (MODULUS 4, REMAINDER 3);
;;
-- ref/alter_table.sgml
ALTER TABLE cities
    ATTACH PARTITION cities_partdef DEFAULT;
;;
-- ref/alter_table.sgml
ALTER TABLE measurement
    DETACH PARTITION measurement_y2015m12;
;;
-- ref/alter_tablespace.sgml
ALTER TABLESPACE index_space RENAME TO fast_raid;
;;
-- ref/alter_tablespace.sgml
ALTER TABLESPACE index_space OWNER TO mary;
;;
-- ref/alter_trigger.sgml
ALTER TRIGGER emp_stamp ON emp RENAME TO emp_track_chgs;
;;
-- ref/alter_trigger.sgml
ALTER TRIGGER emp_stamp ON emp DEPENDS ON EXTENSION emplib;
;;
-- ref/alter_tsconfig.sgml
ALTER TEXT SEARCH CONFIGURATION my_config
  ALTER MAPPING REPLACE english WITH swedish;
;;
-- ref/alter_tsdictionary.sgml
ALTER TEXT SEARCH DICTIONARY my_dict ( StopWords = newrussian );
;;
-- ref/alter_tsdictionary.sgml
ALTER TEXT SEARCH DICTIONARY my_dict ( language = dutch, StopWords );
;;
-- ref/alter_tsdictionary.sgml
ALTER TEXT SEARCH DICTIONARY my_dict ( dummy );
;;
-- ref/alter_type.sgml
ALTER TYPE electronic_mail RENAME TO email;
;;
-- ref/alter_type.sgml
ALTER TYPE email OWNER TO joe;
;;
-- ref/alter_type.sgml
ALTER TYPE email SET SCHEMA customers;
;;
-- ref/alter_type.sgml
ALTER TYPE compfoo ADD ATTRIBUTE f3 int;
;;
-- ref/alter_type.sgml
ALTER TYPE colors ADD VALUE 'orange' AFTER 'red';
;;
-- ref/alter_type.sgml
ALTER TYPE colors RENAME VALUE 'purple' TO 'mauve';
;;
-- ref/alter_user_mapping.sgml
ALTER USER MAPPING FOR bob SERVER foo OPTIONS (SET password 'public');
;;
-- ref/alter_view.sgml
ALTER VIEW foo RENAME TO bar;
;;
-- ref/alter_view.sgml
CREATE TABLE base_table (id int, ts timestamptz);
;;
-- ref/call.sgml
CALL do_db_maintenance();
;;
-- ref/close.sgml
CLOSE liahona;
;;
-- ref/cluster.sgml
CLUSTER employees USING employees_ind;
;;
-- ref/cluster.sgml
CLUSTER employees;
;;
-- ref/cluster.sgml
CLUSTER;
;;
-- ref/comment.sgml
COMMENT ON TABLE mytable IS 'This is my table.';
;;
-- ref/comment.sgml
COMMENT ON TABLE mytable IS NULL;
;;
-- ref/comment.sgml
COMMENT ON ACCESS METHOD gin IS 'GIN index access method';
;;
-- ref/commit.sgml
COMMIT;
;;
-- ref/commit_prepared.sgml
COMMIT PREPARED 'foobar';
;;
-- ref/copy.sgml
COPY (SELECT j FROM (VALUES ('null'::json), (NULL::json)) v(j))
     TO stdout (FORMAT JSON);
;;
-- ref/copy.sgml
COPY country TO STDOUT (DELIMITER '|');
;;
-- ref/copy.sgml
COPY (SELECT * FROM (VALUES(1),(2)) val(id)) TO STDOUT  (FORMAT JSON, FORCE_ARRAY);
;;
-- ref/copy.sgml
COPY country FROM '/usr1/proj/bray/sql/country_data';
;;
-- ref/copy.sgml
COPY (SELECT * FROM country WHERE country_name LIKE 'A%') TO '/usr1/proj/bray/sql/a_list_countries.copy';
;;
-- ref/copy.sgml
COPY country TO PROGRAM 'gzip > /usr1/proj/bray/sql/country_data.gz';
;;
-- ref/create_access_method.sgml
CREATE ACCESS METHOD heptree TYPE INDEX HANDLER heptree_handler;
;;
-- ref/create_aggregate.sgml
SELECT agg(col) FROM tab;
;;
-- ref/create_aggregate.sgml
SELECT col FROM tab ORDER BY col USING sortop LIMIT 1;
;;
-- ref/create_cast.sgml
SELECT CAST(42 AS float8);
;;
-- ref/create_cast.sgml
INSERT INTO foo (f1) VALUES (42);
;;
-- ref/create_cast.sgml
SELECT 2 + 4.0;
;;
-- ref/create_cast.sgml
SELECT CAST ( 2 AS numeric ) + 4.0;
;;
-- ref/create_cast.sgml
CREATE CAST (bigint AS int4) WITH FUNCTION int4(bigint) AS ASSIGNMENT;
;;
-- ref/create_collation.sgml
CREATE COLLATION french (locale = 'fr_FR.utf8');
;;
-- ref/create_collation.sgml
CREATE COLLATION german_phonebook (provider = icu, locale = 'de-u-co-phonebk');
;;
-- ref/create_conversion.sgml
CREATE CONVERSION myconv FOR 'UTF8' TO 'LATIN1' FROM myfunc;
;;
-- ref/create_database.sgml
CREATE DATABASE lusiadas;
;;
-- ref/create_database.sgml
CREATE DATABASE sales OWNER salesapp TABLESPACE salesspace;
;;
-- ref/create_database.sgml
CREATE DATABASE music
    LOCALE 'sv_SE.utf8'
    TEMPLATE template0;
;;
-- ref/create_database.sgml
CREATE DATABASE music2
    LOCALE 'sv_SE.iso885915'
    ENCODING LATIN9
    TEMPLATE template0;
;;
-- ref/create_domain.sgml
INSERT INTO tab (domcol) VALUES ((SELECT domcol FROM tab WHERE false));
;;
-- ref/create_domain.sgml
CREATE DOMAIN us_postal_code AS TEXT
CHECK(
   VALUE ~ '^\d{5}$'
OR VALUE ~ '^\d{5}-\d{4}$'
);
;;
-- ref/create_event_trigger.sgml
CREATE OR REPLACE FUNCTION abort_any_command()
  RETURNS event_trigger
 LANGUAGE plpgsql
  AS $$
BEGIN
  RAISE EXCEPTION 'command % is disabled', tg_tag;
END;
$$;
;;
-- ref/create_extension.sgml
CREATE EXTENSION hstore SCHEMA addons;
;;
-- ref/create_extension.sgml
SET search_path = addons;
;;
-- ref/create_foreign_data_wrapper.sgml
CREATE FOREIGN DATA WRAPPER dummy;
;;
-- ref/create_foreign_data_wrapper.sgml
CREATE FOREIGN DATA WRAPPER file HANDLER file_fdw_handler;
;;
-- ref/create_foreign_data_wrapper.sgml
CREATE FOREIGN DATA WRAPPER mywrapper
    OPTIONS (debug 'true');
;;
-- ref/create_foreign_table.sgml
CREATE FOREIGN TABLE films (
    code        char(5) NOT NULL,
    title       varchar(40) NOT NULL,
    did         integer NOT NULL,
    date_prod   date,
    kind        varchar(10),
    len         interval hour to minute
)
SERVER film_server;
;;
-- ref/create_foreign_table.sgml
CREATE FOREIGN TABLE measurement_y2016m07
    PARTITION OF measurement FOR VALUES FROM ('2016-07-01') TO ('2016-08-01')
    SERVER server_07;
;;
-- ref/create_function.sgml
CREATE FUNCTION add(integer, integer) RETURNS integer
    AS 'SELECT $1 + $2;'
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT;
;;
-- ref/create_function.sgml
CREATE FUNCTION add(a integer, b integer) RETURNS integer
    LANGUAGE SQL
    IMMUTABLE
    RETURNS NULL ON NULL INPUT
    RETURN a + b;
;;
-- ref/create_function.sgml
CREATE OR REPLACE FUNCTION increment(i integer) RETURNS integer AS $$
        BEGIN
                RETURN i + 1;
        END;
$$ LANGUAGE plpgsql;
;;
-- ref/create_function.sgml
CREATE FUNCTION dup(IN int, OUT f1 int, OUT f2 text)
    AS $$ SELECT $1, CAST($1 AS text) || ' is text' $$
    LANGUAGE SQL;
;;
-- ref/create_function.sgml
CREATE TYPE dup_result AS (f1 int, f2 text);
;;
-- ref/create_function.sgml
CREATE FUNCTION dup(int) RETURNS TABLE(f1 int, f2 text)
    AS $$ SELECT $1, CAST($1 AS text) || ' is text' $$
    LANGUAGE SQL;
;;
-- ref/create_function.sgml
CREATE FUNCTION check_password(uname TEXT, pass TEXT)
RETURNS BOOLEAN AS $$
DECLARE passed BOOLEAN;
BEGIN
        SELECT  (pwd = $2) INTO passed
        FROM    pwds
        WHERE   username = $1;

        RETURN passed;
END;
$$  LANGUAGE plpgsql
    SECURITY DEFINER
    -- Set a secure search_path: trusted schema(s), then 'pg_temp'.
    SET search_path = admin, pg_temp;
;;
-- ref/create_index.sgml
CREATE UNIQUE INDEX title_idx ON films (title);
;;
-- ref/create_index.sgml
CREATE UNIQUE INDEX title_idx ON films (title) INCLUDE (director, rating);
;;
-- ref/create_index.sgml
CREATE INDEX title_idx ON films (title) WITH (deduplicate_items = off);
;;
-- ref/create_index.sgml
CREATE INDEX ON films ((lower(title)));
;;
-- ref/create_index.sgml
CREATE INDEX title_idx_german ON films (title COLLATE "de_DE");
;;
-- ref/create_index.sgml
CREATE INDEX title_idx_nulls_low ON films (title NULLS FIRST);
;;
-- ref/create_index.sgml
CREATE UNIQUE INDEX title_idx ON films (title) WITH (fillfactor = 70);
;;
-- ref/create_index.sgml
CREATE INDEX gin_idx ON documents_table USING GIN (locations) WITH (fastupdate = off);
;;
-- ref/create_index.sgml
CREATE INDEX code_idx ON films (code) TABLESPACE indexspace;
;;
-- ref/create_index.sgml
CREATE INDEX pointloc
    ON points USING gist (box(location,location));
;;
-- ref/create_index.sgml
CREATE INDEX CONCURRENTLY sales_quantity_index ON sales_table (quantity);
;;
-- ref/create_language.sgml
CREATE FUNCTION plsample_call_handler() RETURNS language_handler
    AS '$libdir/plsample'
    LANGUAGE C;
;;
-- ref/create_language.sgml
CREATE EXTENSION plsample;
;;
-- ref/create_opclass.sgml
CREATE OPERATOR CLASS gist__int_ops
    DEFAULT FOR TYPE _int4 USING gist AS
        OPERATOR        3       &&,
        OPERATOR        6       = (anyarray, anyarray),
        OPERATOR        7       @>,
        OPERATOR        8       <@,
        OPERATOR        20      @@ (_int4, query_int),
        FUNCTION        1       g_int_consistent (internal, _int4, smallint, oid, internal),
        FUNCTION        2       g_int_union (internal, internal),
        FUNCTION        3       g_int_compress (internal),
        FUNCTION        4       g_int_decompress (internal),
        FUNCTION        5       g_int_penalty (internal, internal, internal),
        FUNCTION        6       g_int_picksplit (internal, internal),
        FUNCTION        7       g_int_same (_int4, _int4, internal);
;;
-- ref/create_operator.sgml
CREATE OPERATOR === (
    LEFTARG = box,
    RIGHTARG = box,
    FUNCTION = area_equal_function,
    COMMUTATOR = ===,
    NEGATOR = !==,
    RESTRICT = area_restriction_function,
    JOIN = area_join_function,
    HASHES, MERGES
);
;;
-- ref/create_procedure.sgml
CREATE PROCEDURE insert_data(a integer, b integer)
LANGUAGE SQL
AS $$
INSERT INTO tbl VALUES (a);
INSERT INTO tbl VALUES (b);
$$;
;;
-- ref/create_procedure.sgml
CREATE PROCEDURE insert_data(a integer, b integer)
LANGUAGE SQL
BEGIN ATOMIC
  INSERT INTO tbl VALUES (a);
;;
-- ref/create_procedure.sgml
CALL insert_data(1, 2);
;;
-- ref/create_publication.sgml
CREATE PUBLICATION mypublication FOR TABLE users, departments;
;;
-- ref/create_publication.sgml
CREATE PUBLICATION active_departments FOR TABLE departments WHERE (active IS TRUE);
;;
-- ref/create_publication.sgml
CREATE PUBLICATION alltables FOR ALL TABLES;
;;
-- ref/create_publication.sgml
CREATE PUBLICATION insert_only FOR TABLE mydata
    WITH (publish = 'insert');
;;
-- ref/create_publication.sgml
CREATE PUBLICATION production_publication FOR TABLE users, departments, TABLES IN SCHEMA production;
;;
-- ref/create_publication.sgml
CREATE PUBLICATION sales_publication FOR TABLES IN SCHEMA marketing, sales;
;;
-- ref/create_publication.sgml
CREATE PUBLICATION users_filtered FOR TABLE users (user_id, firstname);
;;
-- ref/create_publication.sgml
CREATE PUBLICATION all_sequences FOR ALL SEQUENCES;
;;
-- ref/create_publication.sgml
CREATE PUBLICATION all_tables_sequences FOR ALL TABLES, ALL SEQUENCES;
;;
-- ref/create_publication.sgml
CREATE PUBLICATION all_tables_except FOR ALL TABLES EXCEPT (TABLE users, departments);
;;
-- ref/create_publication.sgml
CREATE PUBLICATION all_sequences_tables_except FOR ALL SEQUENCES, ALL TABLES EXCEPT (TABLE users, departments);
;;
-- ref/create_role.sgml
CREATE ROLE jonathan LOGIN;
;;
-- ref/create_role.sgml
CREATE USER davide WITH PASSWORD 'jw8s0F4';
;;
-- ref/create_role.sgml
CREATE ROLE miriam WITH LOGIN PASSWORD 'jw8s0F4' VALID UNTIL '2005-01-01';
;;
-- ref/create_role.sgml
CREATE ROLE admin WITH CREATEDB CREATEROLE;
;;
-- ref/create_rule.sgml
CREATE RULE "_RETURN" AS
    ON SELECT TO t1
    DO INSTEAD
        SELECT * FROM t2;
;;
-- ref/create_rule.sgml
CREATE RULE notify_me AS ON UPDATE TO mytable DO ALSO NOTIFY mytable;
;;
-- ref/create_schema.sgml
CREATE SCHEMA AUTHORIZATION joe;
;;
-- ref/create_schema.sgml
CREATE SCHEMA IF NOT EXISTS test AUTHORIZATION joe;
;;
-- ref/create_schema.sgml
CREATE SCHEMA hollywood
    CREATE TABLE films (title text, release date, awards text[])
    CREATE VIEW winners AS
        SELECT title, release FROM films WHERE awards IS NOT NULL;
;;
-- ref/create_schema.sgml
CREATE SCHEMA hollywood;
;;
-- ref/create_sequence.sgml
CREATE SEQUENCE serial START 101;
;;
-- ref/create_sequence.sgml
SELECT nextval('serial');
;;
-- ref/create_sequence.sgml
INSERT INTO distributors VALUES (nextval('serial'), 'nothing');
;;
-- ref/create_server.sgml
CREATE SERVER myserver FOREIGN DATA WRAPPER postgres_fdw OPTIONS (host 'foo', dbname 'foodb', port '5432');
;;
-- ref/create_statistics.sgml
CREATE TABLE t1 (
    a   int,
    b   int
);
;;
-- ref/create_statistics.sgml
CREATE TABLE t2 (
    a   int,
    b   int
);
;;
-- ref/create_statistics.sgml
CREATE TABLE t3 (
    a   timestamp
);
;;
-- ref/create_subscription.sgml
CREATE SUBSCRIPTION mysub
         CONNECTION 'host=192.168.1.50 port=5432 user=foo dbname=foodb'
        PUBLICATION mypublication, insert_only;
;;
-- ref/create_subscription.sgml
CREATE SUBSCRIPTION mysub
         CONNECTION 'host=192.168.1.50 port=5432 user=foo dbname=foodb'
        PUBLICATION insert_only
               WITH (enabled = false);
;;
-- ref/create_table.sgml
CREATE TABLE films (
    code        char(5) CONSTRAINT firstkey PRIMARY KEY,
    title       varchar(40) NOT NULL,
    did         integer NOT NULL,
    date_prod   date,
    kind        varchar(10),
    len         interval hour to minute
);
;;
-- ref/create_table.sgml
CREATE TABLE array_int (
    vector  int[][]
);
;;
-- ref/create_table.sgml
CREATE TABLE films (
    code        char(5),
    title       varchar(40),
    did         integer,
    date_prod   date,
    kind        varchar(10),
    len         interval hour to minute,
    CONSTRAINT production UNIQUE(date_prod)
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    did     integer CHECK (did > 100),
    name    varchar(40)
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    did     integer,
    name    varchar(40),
    CONSTRAINT con1 CHECK (did > 100 AND name <> '')
);
;;
-- ref/create_table.sgml
CREATE TABLE films (
    code        char(5),
    title       varchar(40),
    did         integer,
    date_prod   date,
    kind        varchar(10),
    len         interval hour to minute,
    CONSTRAINT code_title PRIMARY KEY(code,title)
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    did     integer,
    name    varchar(40),
    PRIMARY KEY(did)
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    name      varchar(40) DEFAULT 'Luso Films',
    did       integer DEFAULT nextval('distributors_serial'),
    modtime   timestamp DEFAULT current_timestamp
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    did     integer CONSTRAINT no_null NOT NULL,
    name    varchar(40) NOT NULL
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    did     integer,
    name    varchar(40) UNIQUE
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    did     integer,
    name    varchar(40),
    UNIQUE(name)
);
;;
-- ref/create_table.sgml
CREATE TABLE distributors (
    did     integer,
    name    varchar(40),
    UNIQUE(name) WITH (fillfactor=70)
)
WITH (fillfactor=70);
;;
-- ref/create_table.sgml
CREATE TABLE cinemas (
        id serial,
        name text,
        location text
) TABLESPACE diskvol1;
;;
-- ref/create_table.sgml
CREATE TYPE employee_type AS (name text, salary numeric);
;;
-- ref/create_table.sgml
CREATE TABLE measurement (
    logdate         date NOT NULL,
    peaktemp        int,
    unitsales       int
) PARTITION BY RANGE (logdate);
;;
-- ref/create_table.sgml
CREATE TABLE measurement_year_month (
    logdate         date NOT NULL,
    peaktemp        int,
    unitsales       int
) PARTITION BY RANGE (EXTRACT(YEAR FROM logdate), EXTRACT(MONTH FROM logdate));
;;
-- ref/create_table.sgml
CREATE TABLE cities (
    city_id      bigserial NOT NULL,
    name         text NOT NULL,
    population   bigint
) PARTITION BY LIST (left(lower(name), 1));
;;
-- ref/create_table.sgml
CREATE TABLE orders (
    order_id     bigint NOT NULL,
    cust_id      bigint NOT NULL,
    status       text
) PARTITION BY HASH (order_id);
;;
-- ref/create_table.sgml
CREATE TABLE measurement_y2016m07
    PARTITION OF measurement (
    unitsales DEFAULT 0
) FOR VALUES FROM ('2016-07-01') TO ('2016-08-01');
;;
-- ref/create_table.sgml
CREATE TABLE measurement_ym_older
    PARTITION OF measurement_year_month
    FOR VALUES FROM (MINVALUE, MINVALUE) TO (2016, 11);
;;
-- ref/create_table.sgml
CREATE TABLE cities_ab
    PARTITION OF cities (
    CONSTRAINT city_id_nonzero CHECK (city_id != 0)
) FOR VALUES IN ('a', 'b');
;;
-- ref/create_table.sgml
CREATE TABLE cities_ab
    PARTITION OF cities (
    CONSTRAINT city_id_nonzero CHECK (city_id != 0)
) FOR VALUES IN ('a', 'b') PARTITION BY RANGE (population);
;;
-- ref/create_table.sgml
CREATE TABLE orders_p1 PARTITION OF orders
    FOR VALUES WITH (MODULUS 4, REMAINDER 0);
;;
-- ref/create_table.sgml
CREATE TABLE cities_partdef
    PARTITION OF cities DEFAULT;
;;
-- ref/create_table_as.sgml
CREATE TABLE films_recent AS
  SELECT * FROM films WHERE date_prod >= '2002-01-01';
;;
-- ref/create_table_as.sgml
CREATE TABLE films2 AS
  TABLE films;
;;
-- ref/create_table_as.sgml
PREPARE recentfilms(date) AS
  SELECT * FROM films WHERE date_prod > $1;
;;
-- ref/create_tablespace.sgml
CREATE TABLESPACE dbspace LOCATION '/data/dbs';
;;
-- ref/create_tablespace.sgml
CREATE TABLESPACE indexspace OWNER genevieve LOCATION '/data/indexes';
;;
-- ref/create_transform.sgml
CREATE TRANSFORM FOR hstore LANGUAGE plpython3u (
    FROM SQL WITH FUNCTION hstore_to_plpython(internal),
    TO SQL WITH FUNCTION plpython_to_hstore(internal)
);
;;
-- ref/create_trigger.sgml
CREATE TRIGGER check_update
    BEFORE UPDATE ON accounts
    FOR EACH ROW
    EXECUTE FUNCTION check_account_update();
;;
-- ref/create_trigger.sgml
CREATE OR REPLACE TRIGGER check_update
    BEFORE UPDATE OF balance ON accounts
    FOR EACH ROW
    EXECUTE FUNCTION check_account_update();
;;
-- ref/create_trigger.sgml
CREATE TRIGGER check_update
    BEFORE UPDATE ON accounts
    FOR EACH ROW
    WHEN (OLD.balance IS DISTINCT FROM NEW.balance)
    EXECUTE FUNCTION check_account_update();
;;
-- ref/create_trigger.sgml
CREATE TRIGGER log_update
    AFTER UPDATE ON accounts
    FOR EACH ROW
    WHEN (OLD.* IS DISTINCT FROM NEW.*)
    EXECUTE FUNCTION log_account_update();
;;
-- ref/create_trigger.sgml
CREATE TRIGGER view_insert
    INSTEAD OF INSERT ON my_view
    FOR EACH ROW
    EXECUTE FUNCTION view_insert_row();
;;
-- ref/create_trigger.sgml
CREATE TRIGGER transfer_insert
    AFTER INSERT ON transfer
    REFERENCING NEW TABLE AS inserted
    FOR EACH STATEMENT
    EXECUTE FUNCTION check_transfer_balances_to_zero();
;;
-- ref/create_trigger.sgml
CREATE TRIGGER paired_items_update
    AFTER UPDATE ON paired_items
    REFERENCING NEW TABLE AS newtab OLD TABLE AS oldtab
    FOR EACH ROW
    EXECUTE FUNCTION check_matching_pairs();
;;
-- ref/create_tsdictionary.sgml
CREATE TEXT SEARCH DICTIONARY my_russian (
    template = snowball,
    language = russian,
    stopwords = myrussian
);
;;
-- ref/create_type.sgml
CREATE TYPE compfoo AS (f1 int, f2 text);
;;
-- ref/create_type.sgml
CREATE TYPE bug_status AS ENUM ('new', 'open', 'closed');
;;
-- ref/create_type.sgml
CREATE TYPE float8_range AS RANGE (subtype = float8, subtype_diff = float8mi);
;;
-- ref/create_type.sgml
CREATE TYPE box;
;;
-- ref/create_type.sgml
CREATE TYPE box (
    INTERNALLENGTH = 16,
    INPUT = my_box_in_function,
    OUTPUT = my_box_out_function,
    ELEMENT = float4
);
;;
-- ref/create_type.sgml
CREATE TYPE bigobj (
    INPUT = lo_filein, OUTPUT = lo_fileout,
    INTERNALLENGTH = VARIABLE
);
;;
-- ref/create_user_mapping.sgml
CREATE USER MAPPING FOR bob SERVER foo OPTIONS (user 'bob', password 'secret');
;;
-- ref/create_view.sgml
CREATE VIEW vista AS SELECT 'Hello World';
;;
-- ref/create_view.sgml
CREATE VIEW vista AS SELECT text 'Hello World' AS hello;
;;
-- ref/create_view.sgml
CREATE VIEW comedies AS
    SELECT *
    FROM films
    WHERE kind = 'Comedy';
;;
-- ref/create_view.sgml
CREATE VIEW universal_comedies AS
    SELECT *
    FROM comedies
    WHERE classification = 'U'
    WITH LOCAL CHECK OPTION;
;;
-- ref/create_view.sgml
CREATE VIEW pg_comedies AS
    SELECT *
    FROM comedies
    WHERE classification = 'PG'
    WITH CASCADED CHECK OPTION;
;;
-- ref/create_view.sgml
CREATE VIEW comedies AS
    SELECT f.*,
           country_code_to_name(f.country_code) AS country,
           (SELECT avg(r.rating)
            FROM user_ratings r
            WHERE r.film_id = f.id) AS avg_rating
    FROM films f
    WHERE f.kind = 'Comedy';
;;
-- ref/create_view.sgml
CREATE RECURSIVE VIEW public.nums_1_100 (n) AS
    VALUES (1)
UNION ALL
    SELECT n+1 FROM nums_1_100 WHERE n < 100;
;;
-- ref/declare.sgml
DECLARE liahona CURSOR FOR SELECT * FROM films;
;;
-- ref/delete.sgml
DELETE FROM films USING producers
  WHERE producer_id = producers.id AND producers.name = 'foo';
;;
-- ref/delete.sgml
DELETE FROM films
  WHERE producer_id IN (SELECT id FROM producers WHERE name = 'foo');
;;
-- ref/delete.sgml
DELETE FROM films WHERE kind <> 'Musical';
;;
-- ref/delete.sgml
DELETE FROM films;
;;
-- ref/delete.sgml
DELETE FROM tasks WHERE status = 'DONE' RETURNING *;
;;
-- ref/delete.sgml
DELETE FROM tasks WHERE CURRENT OF c_tasks;
;;
-- ref/delete.sgml
WITH delete_batch AS (
  SELECT l.ctid FROM user_logs AS l
    WHERE l.status = 'archived'
    ORDER BY l.creation_date
    FOR UPDATE
    LIMIT 10000
)
DELETE FROM user_logs AS dl
  USING delete_batch AS del
  WHERE dl.ctid = del.ctid;
;;
-- ref/discard.sgml
CLOSE ALL;
;;
-- ref/do.sgml
DO $$DECLARE r record;
BEGIN
    FOR r IN SELECT table_schema, table_name FROM information_schema.tables
             WHERE table_type = 'VIEW' AND table_schema = 'public'
    LOOP
        EXECUTE 'GRANT ALL ON ' || quote_ident(r.table_schema) || '.' || quote_ident(r.table_name) || ' TO webuser';
    END LOOP;
END$$;
;;
-- ref/drop_access_method.sgml
DROP ACCESS METHOD heptree;
;;
-- ref/drop_aggregate.sgml
DROP AGGREGATE myavg(integer);
;;
-- ref/drop_aggregate.sgml
DROP AGGREGATE myrank(VARIADIC "any" ORDER BY VARIADIC "any");
;;
-- ref/drop_aggregate.sgml
DROP AGGREGATE myavg(integer), myavg(bigint);
;;
-- ref/drop_cast.sgml
DROP CAST (text AS int);
;;
-- ref/drop_collation.sgml
DROP COLLATION german;
;;
-- ref/drop_conversion.sgml
DROP CONVERSION myname;
;;
-- ref/drop_domain.sgml
DROP DOMAIN box;
;;
-- ref/drop_event_trigger.sgml
DROP EVENT TRIGGER snitch;
;;
-- ref/drop_extension.sgml
DROP EXTENSION hstore;
;;
-- ref/drop_foreign_data_wrapper.sgml
DROP FOREIGN DATA WRAPPER dbi;
;;
-- ref/drop_foreign_table.sgml
DROP FOREIGN TABLE films, distributors;
;;
-- ref/drop_function.sgml
DROP FUNCTION sqrt(integer);
;;
-- ref/drop_function.sgml
DROP FUNCTION sqrt(integer), sqrt(bigint);
;;
-- ref/drop_function.sgml
DROP FUNCTION update_employee_salaries;
;;
-- ref/drop_function.sgml
DROP FUNCTION update_employee_salaries();
;;
-- ref/drop_index.sgml
DROP INDEX title_idx;
;;
-- ref/drop_language.sgml
DROP LANGUAGE plsample;
;;
-- ref/drop_materialized_view.sgml
DROP MATERIALIZED VIEW order_summary;
;;
-- ref/drop_opclass.sgml
DROP OPERATOR CLASS widget_ops USING btree;
;;
-- ref/drop_operator.sgml
DROP OPERATOR ^ (integer, integer);
;;
-- ref/drop_operator.sgml
DROP OPERATOR ~ (none, bit);
;;
-- ref/drop_operator.sgml
DROP OPERATOR ~ (none, bit), ^ (integer, integer);
;;
-- ref/drop_opfamily.sgml
DROP OPERATOR FAMILY float_ops USING btree;
;;
-- ref/drop_policy.sgml
DROP POLICY p1 ON my_table;
;;
-- ref/drop_procedure.sgml
DROP PROCEDURE do_db_maintenance;
;;
-- ref/drop_procedure.sgml
DROP PROCEDURE do_db_maintenance(IN target_schema text, OUT results text);
;;
-- ref/drop_publication.sgml
DROP PUBLICATION mypublication;
;;
-- ref/drop_role.sgml
DROP ROLE jonathan;
;;
-- ref/drop_routine.sgml
DROP ROUTINE foo(integer);
;;
-- ref/drop_rule.sgml
DROP RULE newrule ON mytable;
;;
-- ref/drop_schema.sgml
DROP SCHEMA mystuff CASCADE;
;;
-- ref/drop_sequence.sgml
DROP SEQUENCE serial;
;;
-- ref/drop_server.sgml
DROP SERVER IF EXISTS foo;
;;
-- ref/drop_statistics.sgml
DROP STATISTICS IF EXISTS
    accounting.users_uid_creation,
    public.grants_user_role;
;;
-- ref/drop_subscription.sgml
DROP SUBSCRIPTION mysub;
;;
-- ref/drop_table.sgml
DROP TABLE films, distributors;
;;
-- ref/drop_tablespace.sgml
DROP TABLESPACE mystuff;
;;
-- ref/drop_transform.sgml
DROP TRANSFORM FOR hstore LANGUAGE plpython3u;
;;
-- ref/drop_trigger.sgml
DROP TRIGGER if_dist_exists ON films;
;;
-- ref/drop_tsconfig.sgml
DROP TEXT SEARCH CONFIGURATION my_english;
;;
-- ref/drop_tsdictionary.sgml
DROP TEXT SEARCH DICTIONARY english;
;;
-- ref/drop_tsparser.sgml
DROP TEXT SEARCH PARSER my_parser;
;;
-- ref/drop_tstemplate.sgml
DROP TEXT SEARCH TEMPLATE thesaurus;
;;
-- ref/drop_type.sgml
DROP TYPE box;
;;
-- ref/drop_user_mapping.sgml
DROP USER MAPPING IF EXISTS FOR bob SERVER foo;
;;
-- ref/drop_view.sgml
DROP VIEW kinds;
;;
-- ref/explain.sgml
EXPLAIN SELECT * FROM foo;
;;
-- ref/explain.sgml
EXPLAIN (FORMAT JSON) SELECT * FROM foo;
;;
-- ref/explain.sgml
EXPLAIN SELECT * FROM foo WHERE i = 4;
;;
-- ref/explain.sgml
EXPLAIN (FORMAT YAML) SELECT * FROM foo WHERE i='4';
;;
-- ref/explain.sgml
EXPLAIN (COSTS FALSE) SELECT * FROM foo WHERE i = 4;
;;
-- ref/explain.sgml
EXPLAIN SELECT sum(i) FROM foo WHERE i < 10;
;;
-- ref/explain.sgml
PREPARE query(int, int) AS SELECT sum(bar) FROM test
    WHERE id > $1 AND id < $2
    GROUP BY foo;
;;
-- ref/explain.sgml
EXPLAIN (GENERIC_PLAN)
  SELECT sum(bar) FROM test
    WHERE id > $1 AND id < $2
    GROUP BY foo;
;;
-- ref/explain.sgml
EXPLAIN (GENERIC_PLAN)
  SELECT sum(bar) FROM test
    WHERE id > $1::integer AND id < $2::integer
    GROUP BY foo;
;;
-- ref/fetch.sgml
BEGIN WORK;
;;
-- ref/grant.sgml
GRANT INSERT ON films TO PUBLIC;
;;
-- ref/grant.sgml
GRANT ALL PRIVILEGES ON kinds TO manuel;
;;
-- ref/grant.sgml
GRANT admins TO joe;
;;
-- ref/import_foreign_schema.sgml
IMPORT FOREIGN SCHEMA foreign_films
    FROM SERVER film_server INTO films;
;;
-- ref/import_foreign_schema.sgml
IMPORT FOREIGN SCHEMA foreign_films LIMIT TO (actors, directors)
    FROM SERVER film_server INTO films;
;;
-- ref/insert.sgml
INSERT INTO films VALUES
    ('UA502', 'Bananas', 105, '1971-07-13', 'Comedy', '82 minutes');
;;
-- ref/insert.sgml
INSERT INTO films (code, title, did, date_prod, kind)
    VALUES ('T_601', 'Yojimbo', 106, '1961-06-16', 'Drama');
;;
-- ref/insert.sgml
INSERT INTO films VALUES
    ('UA502', 'Bananas', 105, DEFAULT, 'Comedy', '82 minutes');
;;
-- ref/insert.sgml
INSERT INTO films DEFAULT VALUES;
;;
-- ref/insert.sgml
INSERT INTO films (code, title, did, date_prod, kind) VALUES
    ('B6717', 'Tampopo', 110, '1985-02-10', 'Comedy'),
    ('HG120', 'The Dinner Game', 140, DEFAULT, 'Comedy');
;;
-- ref/insert.sgml
INSERT INTO films SELECT * FROM tmp_films WHERE date_prod < '2004-05-07';
;;
-- ref/insert.sgml
INSERT INTO distributors (did, dname) VALUES (DEFAULT, 'XYZ Widgets')
   RETURNING did;
;;
-- ref/insert.sgml
WITH upd AS (
  UPDATE employees SET sales_count = sales_count + 1 WHERE id =
    (SELECT sales_person FROM accounts WHERE name = 'Acme Corporation')
    RETURNING *
)
INSERT INTO employees_log SELECT *, current_timestamp FROM upd;
;;
-- ref/insert.sgml
INSERT INTO distributors (did, dname)
    VALUES (5, 'Gizmo Transglobal'), (6, 'Associated Computing, Inc')
    ON CONFLICT (did) DO UPDATE SET dname = EXCLUDED.dname;
;;
-- ref/insert.sgml
INSERT INTO distributors (did, dname)
    VALUES (5, 'Gizmo Transglobal'), (6, 'Associated Computing, Inc')
    ON CONFLICT (did) DO UPDATE SET dname = EXCLUDED.dname
    RETURNING old.did AS old_did, old.dname AS old_dname,
              new.did AS new_did, new.dname AS new_dname;
;;
-- ref/insert.sgml
INSERT INTO distributors (did, dname) VALUES (7, 'Redline GmbH')
    ON CONFLICT (did) DO NOTHING;
;;
-- ref/insert.sgml
INSERT INTO distributors (did, dname) VALUES (11, 'Global Electronics')
    ON CONFLICT (did) DO SELECT
    RETURNING *;
;;
-- ref/insert.sgml
INSERT INTO distributors AS d (did, dname) VALUES (12, 'Micro Devices Inc')
    ON CONFLICT (did) DO SELECT WHERE d.dname != EXCLUDED.dname
    RETURNING *;
;;
-- ref/insert.sgml
INSERT INTO distributors (did, dname) VALUES (13, 'Advanced Systems')
    ON CONFLICT (did) DO SELECT FOR UPDATE
    RETURNING *;
;;
-- ref/listen.sgml
LISTEN virtual;
;;
-- ref/merge.sgml
MERGE INTO customer_account ca
USING recent_transactions t
ON t.customer_id = ca.customer_id
WHEN MATCHED THEN
  UPDATE SET balance = balance + transaction_value
WHEN NOT MATCHED THEN
  INSERT (customer_id, balance)
  VALUES (t.customer_id, t.transaction_value);
;;
-- ref/merge.sgml
MERGE INTO wines w
USING wine_stock_changes s
ON s.winename = w.winename
WHEN NOT MATCHED AND s.stock_delta > 0 THEN
  INSERT VALUES(s.winename, s.stock_delta)
WHEN MATCHED AND w.stock + s.stock_delta > 0 THEN
  UPDATE SET stock = w.stock + s.stock_delta
WHEN MATCHED THEN
  DELETE
RETURNING merge_action(), w.winename, old.stock AS old_stock, new.stock AS new_stock;
;;
-- ref/merge.sgml
MERGE INTO wines w
USING new_wine_list s
ON s.winename = w.winename
WHEN NOT MATCHED BY TARGET THEN
  INSERT VALUES(s.winename, s.stock)
WHEN MATCHED AND w.stock != s.stock THEN
  UPDATE SET stock = s.stock
WHEN NOT MATCHED BY SOURCE THEN
  DELETE;
;;
-- ref/pg_dump.sgml
CREATE DATABASE foo WITH TEMPLATE template0;
;;
-- ref/pg_rewind.sgml
CREATE USER rewind_user LOGIN;
;;
-- ref/pgbench.sgml
UPDATE pgbench_accounts
  SET abalance = abalance + :delta
  WHERE aid = :aid
  RETURNING abalance \gset
-- compound of two queries
SELECT 1 \;
;;
-- ref/prepare.sgml
PREPARE fooplan (int, text, bool, numeric) AS
    INSERT INTO foo VALUES($1, $2, $3, $4);
;;
-- ref/prepare.sgml
PREPARE usrrptplan (int) AS
    SELECT * FROM users u, logs l WHERE u.usrid=$1 AND u.usrid=l.usrid
    AND l.date = $2;
;;
-- ref/prepare_transaction.sgml
PREPARE TRANSACTION 'foobar';
;;
-- ref/psql-ref.sgml
SELECT 1; SELECT 2; SELECT 3;
;;
-- ref/psql-ref.sgml
SELECT 1\; SELECT 2\; SELECT 3;
;;
-- ref/refresh_materialized_view.sgml
REFRESH MATERIALIZED VIEW order_summary;
;;
-- ref/refresh_materialized_view.sgml
REFRESH MATERIALIZED VIEW annual_statistics_basis WITH NO DATA;
;;
-- ref/reindex.sgml
REINDEX INDEX my_index;
;;
-- ref/reindex.sgml
REINDEX TABLE my_table;
;;
-- ref/reindex.sgml
REINDEX TABLE CONCURRENTLY my_broken_table;
;;
-- ref/release_savepoint.sgml
ROLLBACK;
;;
-- ref/revoke.sgml
REVOKE INSERT ON films FROM PUBLIC;
;;
-- ref/revoke.sgml
REVOKE ALL PRIVILEGES ON kinds FROM manuel;
;;
-- ref/revoke.sgml
REVOKE admins FROM joe;
;;
-- ref/rollback_prepared.sgml
ROLLBACK PREPARED 'foobar';
;;
-- ref/rollback_to.sgml
ROLLBACK TO SAVEPOINT my_savepoint;
;;
-- ref/security_label.sgml
SECURITY LABEL FOR selinux ON TABLE mytable IS 'system_u:object_r:sepgsql_table_t:s0';
;;
-- ref/security_label.sgml
SECURITY LABEL FOR selinux ON TABLE mytable IS NULL;
;;
-- ref/select.sgml
SELECT DISTINCT ON (location) location, time, report
    FROM weather_reports
    ORDER BY location, time DESC;
;;
-- ref/select.sgml
SELECT name FROM distributors ORDER BY code;
;;
-- ref/select.sgml
SELECT * FROM (SELECT * FROM mytable FOR UPDATE) ss WHERE col1 = 5;
;;
-- ref/select.sgml
SELECT * FROM (SELECT * FROM mytable FOR UPDATE) ss ORDER BY column1;
;;
-- ref/select.sgml
SELECT f.title, f.did, d.name, f.date_prod, f.kind
    FROM distributors d JOIN films f USING (did);
;;
-- ref/select.sgml
SELECT kind, sum(len) AS total FROM films GROUP BY kind;
;;
-- ref/select.sgml
SELECT kind, sum(len) AS total
    FROM films
    GROUP BY kind
    HAVING sum(len) < interval '5 hours';
;;
-- ref/select.sgml
SELECT * FROM distributors ORDER BY name;
;;
-- ref/select.sgml
CREATE FUNCTION distributors(int) RETURNS SETOF distributors AS $$
    SELECT * FROM distributors WHERE did = $1;
$$ LANGUAGE SQL;
;;
-- ref/select.sgml
SELECT * FROM unnest(ARRAY['a','b','c','d','e','f']) WITH ORDINALITY;
;;
-- ref/select.sgml
WITH t AS (
    SELECT random() AS x FROM generate_series(1, 3)
  )
SELECT * FROM t
UNION ALL
SELECT * FROM t;
;;
-- ref/select.sgml
WITH RECURSIVE employee_recursive(distance, employee_name, manager_name) AS (
    SELECT 1, employee_name, manager_name
    FROM employee
    WHERE manager_name = 'Mary'
  UNION ALL
    SELECT er.distance + 1, e.employee_name, e.manager_name
    FROM employee_recursive er, employee e
    WHERE er.employee_name = e.manager_name
  )
SELECT distance, employee_name FROM employee_recursive;
;;
-- ref/select.sgml
SELECT m.name AS mname, pname
FROM manufacturers m, LATERAL get_product_names(m.id) pname;
;;
-- ref/select.sgml
SELECT m.name AS mname, pname
FROM manufacturers m LEFT JOIN LATERAL get_product_names(m.id) pname ON true;
;;
-- ref/select.sgml
SELECT 2+2;
;;
-- ref/select_into.sgml
SELECT * INTO films_recent FROM films WHERE date_prod >= '2002-01-01';
;;
-- ref/set.sgml
SET search_path TO my_schema, public;
;;
-- ref/set.sgml
SET search_path TO 'my_schema, public';
;;
-- ref/set.sgml
SET temp_tablespaces TO NULL;
;;
-- ref/set_role.sgml
SELECT SESSION_USER, CURRENT_USER;
;;
-- ref/set_transaction.sgml
BEGIN TRANSACTION ISOLATION LEVEL REPEATABLE READ;
;;
-- ref/show.sgml
SHOW DateStyle;
;;
-- ref/show.sgml
SHOW geqo;
;;
-- ref/show.sgml
SHOW ALL;
;;
-- ref/truncate.sgml
TRUNCATE bigtable, fattable;
;;
-- ref/truncate.sgml
TRUNCATE bigtable, fattable RESTART IDENTITY;
;;
-- ref/truncate.sgml
TRUNCATE othertable CASCADE;
;;
-- ref/unlisten.sgml
UNLISTEN virtual;
;;
-- ref/update.sgml
UPDATE films SET kind = 'Dramatic' WHERE kind = 'Drama';
;;
-- ref/update.sgml
UPDATE weather SET temp_lo = temp_lo+1, temp_hi = temp_lo+15, prcp = DEFAULT
  WHERE city = 'San Francisco' AND date = '2003-07-03';
;;
-- ref/update.sgml
UPDATE weather SET temp_lo = temp_lo+1, temp_hi = temp_lo+15, prcp = DEFAULT
  WHERE city = 'San Francisco' AND date = '2003-07-03'
  RETURNING temp_lo, temp_hi, prcp, old.prcp AS old_prcp;
;;
-- ref/update.sgml
UPDATE weather SET (temp_lo, temp_hi, prcp) = (temp_lo+1, temp_lo+15, DEFAULT)
  WHERE city = 'San Francisco' AND date = '2003-07-03';
;;
-- ref/update.sgml
UPDATE employees SET sales_count = sales_count + 1 FROM accounts
  WHERE accounts.name = 'Acme Corporation'
  AND employees.id = accounts.sales_person;
;;
-- ref/update.sgml
UPDATE employees SET sales_count = sales_count + 1 WHERE id =
  (SELECT sales_person FROM accounts WHERE name = 'Acme Corporation');
;;
-- ref/update.sgml
UPDATE accounts SET (contact_first_name, contact_last_name) =
    (SELECT first_name, last_name FROM employees
     WHERE employees.id = accounts.sales_person);
;;
-- ref/update.sgml
UPDATE accounts SET contact_first_name = first_name,
                    contact_last_name = last_name
  FROM employees WHERE employees.id = accounts.sales_person;
;;
-- ref/update.sgml
UPDATE summary s SET (sum_x, sum_y, avg_x, avg_y) =
    (SELECT sum(x), sum(y), avg(x), avg(y) FROM data d
     WHERE d.group_id = s.group_id);
;;
-- ref/update.sgml
UPDATE films SET kind = 'Dramatic' WHERE CURRENT OF c_films;
;;
-- ref/update.sgml
WITH exceeded_max_retries AS (
  SELECT w.ctid FROM work_item AS w
    WHERE w.status = 'active' AND w.num_retries > 10
    ORDER BY w.retry_timestamp
    FOR UPDATE
    LIMIT 5000
)
UPDATE work_item SET status = 'failed'
  FROM exceeded_max_retries AS emr
  WHERE work_item.ctid = emr.ctid;
;;
-- ref/vacuum.sgml
VACUUM (VERBOSE, ANALYZE) onek;
;;
-- ref/values.sgml
INSERT INTO films VALUES
    ('UA502', 'Bananas', 105, DEFAULT, 'Comedy', '82 minutes'),
    ('T_601', 'Yojimbo', 106, DEFAULT, 'Drama', DEFAULT);
;;
-- ref/values.sgml
SELECT f.*
  FROM films f, (VALUES('MGM', 'Horror'), ('UA', 'Sci-Fi')) AS t (studio, kind)
  WHERE f.studio = t.studio AND f.kind = t.kind;
;;
-- ref/values.sgml
SELECT * FROM machines
WHERE ip_address IN (VALUES('192.168.0.1'::inet), ('192.168.0.10'), ('192.168.1.43'));
;;
-- rowtypes.sgml
CREATE TYPE complex AS (
    r       double precision,
    i       double precision
);
;;
-- rowtypes.sgml
CREATE TABLE on_hand (
    item      inventory_item,
    count     integer
);
;;
-- rowtypes.sgml
CREATE FUNCTION price_extension(inventory_item, integer) RETURNS numeric
AS 'SELECT $1.price * $2' LANGUAGE SQL;
;;
-- rowtypes.sgml
CREATE TABLE inventory_item (
    name            text,
    supplier_id     integer REFERENCES suppliers,
    price           numeric CHECK (price > 0)
);
;;
-- rowtypes.sgml
SELECT item.name FROM on_hand WHERE item.price > 9.99;
;;
-- rowtypes.sgml
SELECT (item).name FROM on_hand WHERE (item).price > 9.99;
;;
-- rowtypes.sgml
SELECT (on_hand.item).name FROM on_hand WHERE (on_hand.item).price > 9.99;
;;
-- rowtypes.sgml
INSERT INTO mytab (complex_col) VALUES((1.1,2.2));
;;
-- rowtypes.sgml
INSERT INTO mytab (complex_col.r, complex_col.i) VALUES(1.1, 2.2);
;;
-- rowtypes.sgml
SELECT c FROM inventory_item c;
;;
-- rowtypes.sgml
SELECT c.* FROM inventory_item c;
;;
-- rowtypes.sgml
SELECT c.name, c.supplier_id, c.price FROM inventory_item c;
;;
-- rowtypes.sgml
SELECT (myfunc(x)).* FROM some_table;
;;
-- rowtypes.sgml
SELECT m.* FROM some_table, LATERAL myfunc(x) AS m;
;;
-- rowtypes.sgml
SELECT somefunc(c.*) FROM inventory_item c;
;;
-- rowtypes.sgml
SELECT * FROM inventory_item c ORDER BY c;
;;
-- rowtypes.sgml
SELECT * FROM inventory_item c ORDER BY ROW(c.name, c.supplier_id, c.price);
;;
-- rowtypes.sgml
SELECT c.name FROM inventory_item c WHERE c.price > 1000;
;;
-- rowtypes.sgml
SELECT somefunc(c) FROM inventory_item c;
;;
-- rules.sgml
CREATE VIEW myview AS SELECT * FROM mytab;
;;
-- rules.sgml
CREATE TABLE shoe_data (
    shoename   text,          -- primary key
    sh_avail   integer,       -- available number of pairs
    slcolor    text,          -- preferred shoelace color
    slminlen   real,          -- minimum shoelace length
    slmaxlen   real,          -- maximum shoelace length
    slunit     text           -- length unit
);
;;
-- rules.sgml
CREATE VIEW shoe AS
    SELECT sh.shoename,
           sh.sh_avail,
           sh.slcolor,
           sh.slminlen,
           sh.slminlen * un.un_fact AS slminlen_cm,
           sh.slmaxlen,
           sh.slmaxlen * un.un_fact AS slmaxlen_cm,
           sh.slunit
      FROM shoe_data sh, unit un
     WHERE sh.slunit = un.un_name;
;;
-- rules.sgml
INSERT INTO unit VALUES ('cm', 1.0);
;;
-- rules.sgml
SELECT shoelace.sl_name, shoelace.sl_avail,
       shoelace.sl_color, shoelace.sl_len,
       shoelace.sl_unit, shoelace.sl_len_cm
  FROM shoelace shoelace;
;;
-- rules.sgml
SELECT s.sl_name, s.sl_avail,
       s.sl_color, s.sl_len, s.sl_unit,
       s.sl_len * u.un_fact AS sl_len_cm
  FROM shoelace old, shoelace new,
       shoelace_data s, unit u
 WHERE s.sl_unit = u.un_name;
;;
-- rules.sgml
SELECT shoelace.sl_name, shoelace.sl_avail,
       shoelace.sl_color, shoelace.sl_len,
       shoelace.sl_unit, shoelace.sl_len_cm
  FROM (SELECT s.sl_name,
               s.sl_avail,
               s.sl_color,
               s.sl_len,
               s.sl_unit,
               s.sl_len * u.un_fact AS sl_len_cm
          FROM shoelace_data s, unit u
         WHERE s.sl_unit = u.un_name) shoelace;
;;
-- rules.sgml
SELECT * FROM shoe_ready WHERE total_avail >= 2;
;;
-- rules.sgml
SELECT shoe_ready.shoename, shoe_ready.sh_avail,
       shoe_ready.sl_name, shoe_ready.sl_avail,
       shoe_ready.total_avail
  FROM shoe_ready shoe_ready
 WHERE shoe_ready.total_avail >= 2;
;;
-- rules.sgml
SELECT shoe_ready.shoename, shoe_ready.sh_avail,
       shoe_ready.sl_name, shoe_ready.sl_avail,
       shoe_ready.total_avail
  FROM (SELECT rsh.shoename,
               rsh.sh_avail,
               rsl.sl_name,
               rsl.sl_avail,
               least(rsh.sh_avail, rsl.sl_avail) AS total_avail
          FROM shoe rsh, shoelace rsl
         WHERE rsl.sl_color = rsh.slcolor
           AND rsl.sl_len_cm >= rsh.slminlen_cm
           AND rsl.sl_len_cm <= rsh.slmaxlen_cm) shoe_ready
 WHERE shoe_ready.total_avail >= 2;
;;
-- rules.sgml
SELECT shoe_ready.shoename, shoe_ready.sh_avail,
       shoe_ready.sl_name, shoe_ready.sl_avail,
       shoe_ready.total_avail
  FROM (SELECT rsh.shoename,
               rsh.sh_avail,
               rsl.sl_name,
               rsl.sl_avail,
               least(rsh.sh_avail, rsl.sl_avail) AS total_avail
          FROM (SELECT sh.shoename,
                       sh.sh_avail,
                       sh.slcolor,
                       sh.slminlen,
                       sh.slminlen * un.un_fact AS slminlen_cm,
                       sh.slmaxlen,
                       sh.slmaxlen * un.un_fact AS slmaxlen_cm,
                       sh.slunit
                  FROM shoe_data sh, unit un
                 WHERE sh.slunit = un.un_name) rsh,
               (SELECT s.sl_name,
                       s.sl_avail,
                       s.sl_color,
                       s.sl_len,
                       s.sl_unit,
                       s.sl_len * u.un_fact AS sl_len_cm
                  FROM shoelace_data s, unit u
                 WHERE s.sl_unit = u.un_name) rsl
         WHERE rsl.sl_color = rsh.slcolor
           AND rsl.sl_len_cm >= rsh.slminlen_cm
           AND rsl.sl_len_cm <= rsh.slmaxlen_cm) shoe_ready
 WHERE shoe_ready.total_avail >= 2;
;;
-- rules.sgml
SELECT t2.b FROM t1, t2 WHERE t1.a = t2.a;
;;
-- rules.sgml
UPDATE t1 SET a = t1.a, b = t2.b FROM t2 WHERE t1.a = t2.a;
;;
-- rules.sgml
SELECT t1.a, t2.b FROM t1, t2 WHERE t1.a = t2.a;
;;
-- rules.sgml
SELECT t1.a, t2.b, t1.ctid FROM t1, t2 WHERE t1.a = t2.a;
;;
-- rules.sgml
CREATE MATERIALIZED VIEW mymatview AS SELECT * FROM mytab;
;;
-- rules.sgml
CREATE TABLE mymatview AS SELECT * FROM mytab;
;;
-- rules.sgml
REFRESH MATERIALIZED VIEW mymatview;
;;
-- rules.sgml
CREATE TABLE invoice (
    invoice_no    integer        PRIMARY KEY,
    seller_no     integer,       -- ID of salesperson
    invoice_date  date,          -- date of sale
    invoice_amt   numeric(13,2)  -- amount of sale
);
;;
-- rules.sgml
CREATE MATERIALIZED VIEW sales_summary AS
  SELECT
      seller_no,
      invoice_date,
      sum(invoice_amt)::numeric(13,2) AS sales_amt
    FROM invoice
    WHERE invoice_date < CURRENT_DATE
    GROUP BY
      seller_no,
      invoice_date;
;;
-- rules.sgml
REFRESH MATERIALIZED VIEW sales_summary;
;;
-- rules.sgml
SELECT count(*) FROM words WHERE word = 'caterpiler';
;;
-- rules.sgml
SELECT word FROM words ORDER BY word <-> 'caterpiler' LIMIT 10;
;;
-- rules.sgml
CREATE TABLE shoelace_log (
    sl_name    text,          -- shoelace changed
    sl_avail   integer,       -- new available value
    log_who    text,          -- who did it
    log_when   timestamp      -- when
);
;;
-- rules.sgml
UPDATE shoelace_data SET sl_avail = 6 WHERE sl_name = 'sl7';
;;
-- rules.sgml
SELECT * FROM shoelace_log;
;;
-- rules.sgml
UPDATE shoelace_data SET sl_avail = 6
  FROM shoelace_data shoelace_data
 WHERE shoelace_data.sl_name = 'sl7';
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       new.sl_name, new.sl_avail,
       current_user, current_timestamp )
  FROM shoelace_data new, shoelace_data old;
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       new.sl_name, new.sl_avail,
       current_user, current_timestamp )
  FROM shoelace_data new, shoelace_data old,
       shoelace_data shoelace_data;
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       new.sl_name, new.sl_avail,
       current_user, current_timestamp )
  FROM shoelace_data new, shoelace_data old,
       shoelace_data shoelace_data
 WHERE new.sl_avail <> old.sl_avail;
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       new.sl_name, new.sl_avail,
       current_user, current_timestamp )
  FROM shoelace_data new, shoelace_data old,
       shoelace_data shoelace_data
 WHERE new.sl_avail <> old.sl_avail
   AND shoelace_data.sl_name = 'sl7';
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       shoelace_data.sl_name, 6,
       current_user, current_timestamp )
  FROM shoelace_data new, shoelace_data old,
       shoelace_data shoelace_data
 WHERE 6 <> old.sl_avail
   AND shoelace_data.sl_name = 'sl7';
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       shoelace_data.sl_name, 6,
       current_user, current_timestamp )
  FROM shoelace_data new, shoelace_data old,
       shoelace_data shoelace_data
 WHERE 6 <> shoelace_data.sl_avail
   AND shoelace_data.sl_name = 'sl7';
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       shoelace_data.sl_name, 6,
       current_user, current_timestamp )
  FROM shoelace_data
 WHERE 6 <> shoelace_data.sl_avail
   AND shoelace_data.sl_name = 'sl7';
;;
-- rules.sgml
UPDATE shoelace_data SET sl_color = 'green'
 WHERE sl_name = 'sl7';
;;
-- rules.sgml
INSERT INTO shoelace_log VALUES (
       shoelace_data.sl_name, shoelace_data.sl_avail,
       current_user, current_timestamp )
  FROM shoelace_data
 WHERE shoelace_data.sl_avail <> shoelace_data.sl_avail
   AND shoelace_data.sl_name = 'sl7';
;;
-- rules.sgml
UPDATE shoelace_data SET sl_avail = 0
 WHERE sl_color = 'black';
;;
-- rules.sgml
INSERT INTO shoelace_log
SELECT shoelace_data.sl_name, 0,
       current_user, current_timestamp
  FROM shoelace_data
 WHERE 0 <> shoelace_data.sl_avail
   AND shoelace_data.sl_color = 'black';
;;
-- rules.sgml
CREATE RULE shoe_ins_protect AS ON INSERT TO shoe
    DO INSTEAD NOTHING;
;;
-- rules.sgml
CREATE RULE shoelace_ins AS ON INSERT TO shoelace
    DO INSTEAD
    INSERT INTO shoelace_data VALUES (
           NEW.sl_name,
           NEW.sl_avail,
           NEW.sl_color,
           NEW.sl_len,
           NEW.sl_unit
    );
;;
-- rules.sgml
CREATE RULE shoelace_ins AS ON INSERT TO shoelace
    DO INSTEAD
    INSERT INTO shoelace_data VALUES (
           NEW.sl_name,
           NEW.sl_avail,
           NEW.sl_color,
           NEW.sl_len,
           NEW.sl_unit
    )
    RETURNING
           shoelace_data.*,
           (SELECT shoelace_data.sl_len * u.un_fact
            FROM unit u WHERE shoelace_data.sl_unit = u.un_name);
;;
-- rules.sgml
CREATE TABLE shoelace_arrive (
    arr_name    text,
    arr_quant   integer
);
;;
-- rules.sgml
SELECT * FROM shoelace_arrive;
;;
-- rules.sgml
SELECT * FROM shoelace;
;;
-- rules.sgml
INSERT INTO shoelace_ok SELECT * FROM shoelace_arrive;
;;
-- rules.sgml
SELECT * FROM shoelace ORDER BY sl_name;
;;
-- rules.sgml
INSERT INTO shoelace_ok
SELECT shoelace_arrive.arr_name, shoelace_arrive.arr_quant
  FROM shoelace_arrive shoelace_arrive, shoelace_ok shoelace_ok;
;;
-- rules.sgml
UPDATE shoelace
   SET sl_avail = shoelace.sl_avail + shoelace_arrive.arr_quant
  FROM shoelace_arrive shoelace_arrive, shoelace_ok shoelace_ok,
       shoelace_ok old, shoelace_ok new,
       shoelace shoelace
 WHERE shoelace.sl_name = shoelace_arrive.arr_name;
;;
-- rules.sgml
UPDATE shoelace_data
   SET sl_name = shoelace.sl_name,
       sl_avail = shoelace.sl_avail + shoelace_arrive.arr_quant,
       sl_color = shoelace.sl_color,
       sl_len = shoelace.sl_len,
       sl_unit = shoelace.sl_unit
  FROM shoelace_arrive shoelace_arrive, shoelace_ok shoelace_ok,
       shoelace_ok old, shoelace_ok new,
       shoelace shoelace, shoelace old,
       shoelace new, shoelace_data shoelace_data
 WHERE shoelace.sl_name = shoelace_arrive.arr_name
   AND shoelace_data.sl_name = shoelace.sl_name;
;;
-- rules.sgml
UPDATE shoelace_data
   SET sl_name = s.sl_name,
       sl_avail = s.sl_avail + shoelace_arrive.arr_quant,
       sl_color = s.sl_color,
       sl_len = s.sl_len,
       sl_unit = s.sl_unit
  FROM shoelace_arrive shoelace_arrive, shoelace_ok shoelace_ok,
       shoelace_ok old, shoelace_ok new,
       shoelace shoelace, shoelace old,
       shoelace new, shoelace_data shoelace_data,
       shoelace old, shoelace new,
       shoelace_data s, unit u
 WHERE s.sl_name = shoelace_arrive.arr_name
   AND shoelace_data.sl_name = s.sl_name;
;;
-- rules.sgml
INSERT INTO shoelace_log
SELECT s.sl_name,
       s.sl_avail + shoelace_arrive.arr_quant,
       current_user,
       current_timestamp
  FROM shoelace_arrive shoelace_arrive, shoelace_ok shoelace_ok,
       shoelace_ok old, shoelace_ok new,
       shoelace shoelace, shoelace old,
       shoelace new, shoelace_data shoelace_data,
       shoelace old, shoelace new,
       shoelace_data s, unit u,
       shoelace_data old, shoelace_data new
       shoelace_log shoelace_log
 WHERE s.sl_name = shoelace_arrive.arr_name
   AND shoelace_data.sl_name = s.sl_name
   AND (s.sl_avail + shoelace_arrive.arr_quant) <> s.sl_avail;
;;
-- rules.sgml
INSERT INTO shoelace_log
SELECT s.sl_name,
       s.sl_avail + shoelace_arrive.arr_quant,
       current_user,
       current_timestamp
  FROM shoelace_arrive shoelace_arrive, shoelace_data shoelace_data,
       shoelace_data s
 WHERE s.sl_name = shoelace_arrive.arr_name
   AND shoelace_data.sl_name = s.sl_name
   AND s.sl_avail + shoelace_arrive.arr_quant <> s.sl_avail;
;;
-- rules.sgml
INSERT INTO shoelace VALUES ('sl9', 0, 'pink', 35.0, 'inch', 0.0);
;;
-- rules.sgml
CREATE VIEW shoelace_mismatch AS
    SELECT * FROM shoelace WHERE NOT EXISTS
        (SELECT shoename FROM shoe WHERE slcolor = sl_color);
;;
-- rules.sgml
SELECT * FROM shoelace_mismatch;
;;
-- rules.sgml
CREATE VIEW shoelace_can_delete AS
    SELECT * FROM shoelace_mismatch WHERE sl_avail = 0;
;;
-- rules.sgml
DELETE FROM shoelace WHERE EXISTS
    (SELECT * FROM shoelace_can_delete
             WHERE sl_name = shoelace.sl_name);
;;
-- rules.sgml
CREATE TABLE phone_data (person text, phone text, private boolean);
;;
-- rules.sgml
CREATE VIEW phone_number AS
    SELECT person, phone FROM phone_data WHERE phone NOT LIKE '412%';
;;
-- rules.sgml
CREATE VIEW phone_number WITH (security_barrier) AS
    SELECT person, phone FROM phone_data WHERE phone NOT LIKE '412%';
;;
-- rules.sgml
CREATE TABLE computer (
    hostname        text,    -- indexed
    manufacturer    text     -- indexed
);
;;
-- rules.sgml
DELETE FROM software WHERE hostname = $1;
;;
-- rules.sgml
CREATE RULE computer_del AS ON DELETE TO computer
    DO DELETE FROM software WHERE hostname = OLD.hostname;
;;
-- rules.sgml
DELETE FROM computer WHERE hostname = 'mypc.local.net';
;;
-- rules.sgml
DELETE FROM software WHERE computer.hostname = 'mypc.local.net'
                       AND software.hostname = computer.hostname;
;;
-- rules.sgml
DELETE FROM software WHERE computer.hostname >= 'old' AND computer.hostname < 'ole'
                       AND software.hostname = computer.hostname;
;;
-- rules.sgml
DELETE FROM computer WHERE hostname ~ '^old';
;;
-- rules.sgml
DELETE FROM computer WHERE manufacturer = 'bim';
;;
-- rules.sgml
DELETE FROM software WHERE computer.manufacturer = 'bim'
                       AND software.hostname = computer.hostname;
;;
-- spi.sgml
INSERT INTO a SELECT * FROM a;
;;
-- syntax.sgml
SELECT * FROM MY_TABLE;
;;
-- syntax.sgml
UPDATE MY_TABLE SET A = 5;
;;
-- syntax.sgml
uPDaTE my_TabLE SeT a = 5;
;;
-- syntax.sgml
UPDATE my_table SET a = 5;
;;
-- syntax.sgml
UPDATE "my_table" SET "a" = 5;
;;
-- syntax.sgml
SELECT 'foo'
'bar';
;;
-- syntax.sgml
SELECT 'foobar';
;;
-- syntax.sgml
SELECT 'foo'      'bar';
;;
-- syntax.sgml
CREATE FUNCTION dept(text) RETURNS dept
    AS $$ SELECT * FROM dept WHERE name = $1 $$
    LANGUAGE SQL;
;;
-- syntax.sgml
WITH vals (v) AS ( VALUES (1),(3),(4),(3),(2) )
SELECT array_agg(v ORDER BY v DESC) FROM vals;
;;
-- syntax.sgml
WITH vals (k, v) AS ( VALUES ('key0','1'), ('key1','3'), ('key1','2') )
SELECT jsonb_object_agg(k, v ORDER BY v) FROM vals;
;;
-- syntax.sgml
SELECT string_agg(a, ',' ORDER BY a) FROM table;
;;
-- syntax.sgml
WITH vals (v) AS ( VALUES (1),(3),(4),(3),(2) )
SELECT array_agg(DISTINCT v ORDER BY v DESC) FROM vals;
;;
-- syntax.sgml
SELECT percentile_cont(0.5) WITHIN GROUP (ORDER BY income) FROM households;
;;
-- syntax.sgml
SELECT
    count(*) AS unfiltered,
    count(*) FILTER (WHERE i < 5) AS filtered
FROM generate_series(1,10) AS s(i);
;;
-- syntax.sgml
SELECT * FROM tbl WHERE a > 'foo' COLLATE "C";
;;
-- syntax.sgml
SELECT * FROM tbl WHERE a COLLATE "C" > 'foo';
;;
-- syntax.sgml
SELECT * FROM tbl WHERE (a > 'foo') COLLATE "C";
;;
-- syntax.sgml
SELECT name, (SELECT max(pop) FROM cities WHERE cities.state = states.name)
    FROM states;
;;
-- syntax.sgml
SELECT ARRAY[1,2,3+4];
;;
-- syntax.sgml
SELECT ARRAY[1,2,22.7]::integer[];
;;
-- syntax.sgml
SELECT ARRAY[ARRAY[1,2], ARRAY[3,4]];
;;
-- syntax.sgml
CREATE TABLE arr(f1 int[], f2 int[]);
;;
-- syntax.sgml
SELECT ARRAY[]::integer[];
;;
-- syntax.sgml
SELECT ARRAY(SELECT oid FROM pg_proc WHERE proname LIKE 'bytea%');
;;
-- syntax.sgml
SELECT ROW(1,2.5,'this is a test');
;;
-- syntax.sgml
SELECT ROW(t.*, 42) FROM t;
;;
-- syntax.sgml
CREATE TABLE mytable(f1 int, f2 float, f3 text);
;;
-- syntax.sgml
SELECT true OR somefunc();
;;
-- syntax.sgml
SELECT somefunc() OR true;
;;
-- syntax.sgml
SELECT CASE WHEN x > 0 THEN x ELSE 1/0 END FROM tab;
;;
-- syntax.sgml
SELECT CASE WHEN min(employees) > 0
            THEN avg(expenses / employees)
       END
    FROM departments;
;;
-- syntax.sgml
CREATE FUNCTION concat_lower_or_upper(a text, b text, uppercase boolean DEFAULT false)
RETURNS text
AS
$$
 SELECT CASE
        WHEN $3 THEN UPPER($1 || ' ' || $2)
        ELSE LOWER($1 || ' ' || $2)
        END;
$$
LANGUAGE SQL IMMUTABLE STRICT;
;;
-- system-views.sgml
WITH memory_contexts AS (
    SELECT * FROM pg_backend_memory_contexts
)
SELECT sum(c1.total_bytes)
FROM memory_contexts c1, memory_contexts c2
WHERE c2.name = 'CacheMemoryContext'
AND c1.path[c2.level] = c2.path[c2.level];
;;
-- system-views.sgml
SELECT * FROM pg_locks pl LEFT JOIN pg_stat_activity psa
    ON pl.pid = psa.pid;
;;
-- system-views.sgml
SELECT * FROM pg_locks pl LEFT JOIN pg_prepared_xacts ppx
    ON pl.virtualtransaction = '-1/' || ppx.transaction;
;;
-- tableam.sgml
CREATE OR REPLACE FUNCTION my_tableam_handler(internal)
  RETURNS table_am_handler AS 'my_extension', 'my_tableam_handler'
  LANGUAGE C STRICT;
;;
-- tablefunc.sgml
CREATE TABLE ct(id SERIAL, rowid TEXT, attribute TEXT, value TEXT);
;;
-- tablefunc.sgml
CREATE TYPE tablefunc_crosstab_N AS (
    row_name TEXT,
    category_1 TEXT,
    category_2 TEXT,
        .
        .
        .
    category_N TEXT
);
;;
-- tablefunc.sgml
SELECT *
FROM crosstab3(
  'SELECT rowid, attribute, value
   FROM ct
   WHERE attribute = ''att2'' OR attribute = ''att3''
   ORDER BY 1, 2');
;;
-- tablefunc.sgml
CREATE TYPE my_crosstab_float8_5_cols AS (
    my_row_name text,
    my_category_1 float8,
    my_category_2 float8,
    my_category_3 float8,
    my_category_4 float8,
    my_category_5 float8
);
;;
-- tablefunc.sgml
CREATE OR REPLACE FUNCTION crosstab_float8_5_cols(
    IN text,
    OUT my_row_name text,
    OUT my_category_1 float8,
    OUT my_category_2 float8,
    OUT my_category_3 float8,
    OUT my_category_4 float8,
    OUT my_category_5 float8)
  RETURNS SETOF record
  AS '$libdir/tablefunc','crosstab' LANGUAGE C STABLE STRICT;
;;
-- tablefunc.sgml
SELECT row_name, extra_col, cat, value FROM foo ORDER BY 1;
;;
-- tablefunc.sgml
SELECT DISTINCT cat FROM foo ORDER BY 1;
;;
-- tablefunc.sgml
CREATE TABLE sales (year int, month int, qty int);
;;
-- tablefunc.sgml
CREATE TABLE cth(rowid text, rowdt timestamp, attribute text, val text);
;;
-- tablefunc.sgml
SELECT * FROM connectby('connectby_tree', 'keyid', 'parent_keyid', 'pos', 'row2', 0, '~')
    AS t(keyid text, parent_keyid text, level int, branch text, pos int);
;;
-- tablefunc.sgml
CREATE TABLE connectby_tree(keyid text, parent_keyid text, pos int);
;;
-- textsearch.sgml
SELECT title || ' ' ||  author || ' ' ||  abstract || ' ' || body AS document
FROM messages
WHERE mid = 12;
;;
-- textsearch.sgml
SELECT 'a fat cat sat on a mat and ate a fat rat'::tsvector @@ 'cat & rat'::tsquery;
;;
-- textsearch.sgml
SELECT to_tsvector('fat cats ate fat rats') @@ to_tsquery('fat & rat');
;;
-- textsearch.sgml
SELECT 'fat cats ate fat rats'::tsvector @@ to_tsquery('fat & rat');
;;
-- textsearch.sgml
SELECT to_tsvector('fatal error') @@ to_tsquery('fatal <-> error');
;;
-- textsearch.sgml
SELECT phraseto_tsquery('cats ate rats');
;;
-- textsearch.sgml
SELECT title
FROM pgweb
WHERE to_tsvector('english', body) @@ to_tsquery('english', 'friend');
;;
-- textsearch.sgml
SELECT title
FROM pgweb
WHERE to_tsvector(body) @@ to_tsquery('friend');
;;
-- textsearch.sgml
SELECT title
FROM pgweb
WHERE to_tsvector(title || ' ' || body) @@ to_tsquery('create & table')
ORDER BY last_mod_date DESC
LIMIT 10;
;;
-- textsearch.sgml
CREATE INDEX pgweb_idx ON pgweb USING GIN (to_tsvector('english', body));
;;
-- textsearch.sgml
CREATE INDEX pgweb_idx ON pgweb USING GIN (to_tsvector(config_name, body));
;;
-- textsearch.sgml
CREATE INDEX pgweb_idx ON pgweb USING GIN (to_tsvector('english', title || ' ' || body));
;;
-- textsearch.sgml
ALTER TABLE pgweb
    ADD COLUMN textsearchable_index_col tsvector
               GENERATED ALWAYS AS (to_tsvector('english', coalesce(title, '') || ' ' || coalesce(body, ''))) STORED;
;;
-- textsearch.sgml
CREATE INDEX textsearch_idx ON pgweb USING GIN (textsearchable_index_col);
;;
-- textsearch.sgml
SELECT title
FROM pgweb
WHERE textsearchable_index_col @@ to_tsquery('create & table')
ORDER BY last_mod_date DESC
LIMIT 10;
;;
-- textsearch.sgml
UPDATE tt SET ti =
    setweight(to_tsvector(coalesce(title,'')), 'A')    ||
    setweight(to_tsvector(coalesce(keyword,'')), 'B')  ||
    setweight(to_tsvector(coalesce(abstract,'')), 'C') ||
    setweight(to_tsvector(coalesce(body,'')), 'D');
;;
-- textsearch.sgml
CREATE FUNCTION messages_trigger() RETURNS trigger AS $$
BEGIN
  new.tsv :=
     setweight(to_tsvector('pg_catalog.english', coalesce(new.title,'')), 'A') ||
     setweight(to_tsvector('pg_catalog.english', coalesce(new.body,'')), 'D');
  RETURN new;
END
$$ LANGUAGE plpgsql;
;;
-- textsearch.sgml
SELECT * FROM ts_stat('SELECT vector FROM apod')
ORDER BY nentry DESC, ndoc DESC, word
LIMIT 10;
;;
-- textsearch.sgml
SELECT * FROM ts_stat('SELECT vector FROM apod', 'ab')
ORDER BY nentry DESC, ndoc DESC, word
LIMIT 10;
;;
-- textsearch.sgml
ALTER TEXT SEARCH CONFIGURATION astro_en
    ADD MAPPING FOR asciiword WITH astrosyn, english_ispell, english_stem;
;;
-- textsearch.sgml
CREATE TEXT SEARCH DICTIONARY public.simple_dict (
    TEMPLATE = pg_catalog.simple,
    STOPWORDS = english
);
;;
-- textsearch.sgml
CREATE TEXT SEARCH DICTIONARY thesaurus_simple (
    TEMPLATE = thesaurus,
    DictFile = mythesaurus,
    Dictionary = pg_catalog.english_stem
);
;;
-- textsearch.sgml
ALTER TEXT SEARCH CONFIGURATION russian
    ALTER MAPPING FOR asciiword, asciihword, hword_asciipart
    WITH thesaurus_simple;
;;
-- textsearch.sgml
CREATE TEXT SEARCH DICTIONARY thesaurus_astro (
    TEMPLATE = thesaurus,
    DictFile = thesaurus_astro,
    Dictionary = english_stem
);
;;
-- textsearch.sgml
CREATE TEXT SEARCH DICTIONARY english_hunspell (
    TEMPLATE = ispell,
    DictFile = en_us,
    AffFile = en_us,
    Stopwords = english);
;;
-- textsearch.sgml
SELECT ts_lexize('norwegian_ispell', 'overbuljongterningpakkmesterassistent');
;;
-- textsearch.sgml
CREATE TEXT SEARCH DICTIONARY english_stem (
    TEMPLATE = snowball,
    Language = english,
    StopWords = english
);
;;
-- textsearch.sgml
CREATE TEXT SEARCH CONFIGURATION public.pg ( COPY = pg_catalog.english );
;;
-- textsearch.sgml
CREATE TEXT SEARCH DICTIONARY pg_dict (
    TEMPLATE = synonym,
    SYNONYMS = pg_dict
);
;;
-- textsearch.sgml
CREATE TEXT SEARCH DICTIONARY english_ispell (
    TEMPLATE = ispell,
    DictFile = english,
    AffFile = english,
    StopWords = english
);
;;
-- textsearch.sgml
ALTER TEXT SEARCH CONFIGURATION pg
    ALTER MAPPING FOR asciiword, asciihword, hword_asciipart,
                      word, hword, hword_part
    WITH pg_dict, english_ispell, english_stem;
;;
-- textsearch.sgml
ALTER TEXT SEARCH CONFIGURATION pg
    DROP MAPPING FOR email, url, url_path, sfloat, float;
;;
-- textsearch.sgml
SELECT * FROM ts_debug('public.pg', '
PostgreSQL, the highly scalable, SQL compliant, open source object-relational
database management system, is now undergoing beta testing of the next
version of our software.
');
;;
-- textsearch.sgml
CREATE TEXT SEARCH CONFIGURATION public.english ( COPY = pg_catalog.english );
;;
-- trigger.sgml
CREATE TABLE ttest (
    x integer
);
;;
-- tsm-system-rows.sgml
CREATE EXTENSION tsm_system_rows;
;;
-- tsm-system-rows.sgml
SELECT * FROM my_table TABLESAMPLE SYSTEM_ROWS(100);
;;
-- tsm-system-time.sgml
CREATE EXTENSION tsm_system_time;
;;
-- tsm-system-time.sgml
SELECT * FROM my_table TABLESAMPLE SYSTEM_TIME(1000);
;;
-- unaccent.sgml
SELECT unaccent('unaccent', 'H&ocirc;tel');
;;
-- user-manag.sgml
ALTER ROLE myname SET enable_indexscan TO off;
;;
-- user-manag.sgml
CREATE ROLE joe LOGIN;
;;
-- user-manag.sgml
SET ROLE admin;
;;
-- user-manag.sgml
SET ROLE wheel;
;;
-- user-manag.sgml
SET ROLE joe;
;;
-- user-manag.sgml
ALTER TABLE bobs_table OWNER TO alice;
;;
-- user-manag.sgml
REASSIGN OWNED BY doomed_role TO successor_role;
;;
-- user-manag.sgml
GRANT pg_signal_backend TO admin_user;
;;
-- uuid-ossp.sgml
SELECT uuid_generate_v3(uuid_ns_url(), 'http://www.postgresql.org');
;;
-- xaggr.sgml
CREATE AGGREGATE sum (complex)
(
    sfunc = complex_add,
    stype = complex,
    initcond = '(0,0)'
);
;;
-- xaggr.sgml
SELECT sum(a) FROM test_complex;
;;
-- xaggr.sgml
CREATE AGGREGATE avg (float8)
(
    sfunc = float8_accum,
    stype = float8[],
    finalfunc = float8_avg,
    initcond = '{0,0,0}'
);
;;
-- xaggr.sgml
CREATE AGGREGATE sum (complex)
(
    sfunc = complex_add,
    stype = complex,
    initcond = '(0,0)',
    msfunc = complex_add,
    minvfunc = complex_sub,
    mstype = complex,
    minitcond = '(0,0)'
);
;;
-- xaggr.sgml
CREATE AGGREGATE unsafe_sum (float8)
(
    stype = float8,
    sfunc = float8pl,
    mstype = float8,
    msfunc = float8pl,
    minvfunc = float8mi
);
;;
-- xaggr.sgml
SELECT
  unsafe_sum(x) OVER (ORDER BY n ROWS BETWEEN CURRENT ROW AND 1 FOLLOWING)
FROM (VALUES (1, 1.0e20::float8),
             (2, 1.0::float8)) AS v (n,x);
;;
-- xaggr.sgml
CREATE AGGREGATE array_accum (anycompatible)
(
    sfunc = array_append,
    stype = anycompatiblearray,
    initcond = '{}'
);
;;
-- xaggr.sgml
SELECT attrelid::regclass, array_accum(attname)
    FROM pg_attribute
    WHERE attnum > 0 AND attrelid = 'pg_tablespace'::regclass
    GROUP BY attrelid;
;;
-- xaggr.sgml
SELECT percentile_disc(0.5) WITHIN GROUP (ORDER BY income) FROM households;
;;
-- xfunc.sgml
INSERT INTO mytable VALUES ($1);
;;
-- xfunc.sgml
INSERT INTO $1 VALUES (42);
;;
-- xfunc.sgml
CREATE FUNCTION tf1 (accountno integer, debit numeric) RETURNS numeric AS $$
    UPDATE bank
        SET balance = balance - debit
        WHERE accountno = tf1.accountno;
    SELECT 1;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
SELECT tf1(17, 100.0);
;;
-- xfunc.sgml
CREATE FUNCTION tf1 (accountno integer, debit numeric) RETURNS numeric AS $$
    UPDATE bank
        SET balance = balance - debit
        WHERE accountno = tf1.accountno;
    SELECT balance FROM bank WHERE accountno = tf1.accountno;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CREATE FUNCTION tf1 (accountno integer, debit numeric) RETURNS numeric AS $$
    UPDATE bank
        SET balance = balance - debit
        WHERE accountno = tf1.accountno
    RETURNING balance;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CREATE FUNCTION add_em(integer, integer) RETURNS float8 AS $$
    SELECT $1 + $2;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CREATE FUNCTION new_emp() RETURNS emp AS $$
    SELECT text 'None' AS name,
        1000.0 AS salary,
        25 AS age,
        point '(2,2)' AS cubicle;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CREATE FUNCTION new_emp() RETURNS emp AS $$
    SELECT ROW('None', 1000.0, 25, '(2,2)')::emp;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CREATE PROCEDURE tp1 (accountno integer, debit numeric, OUT new_balance numeric) AS $$
    UPDATE bank
        SET balance = balance - debit
        WHERE accountno = tp1.accountno
    RETURNING balance;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CALL tp1(17, 100.0, NULL);
;;
-- xfunc.sgml
CREATE FUNCTION getfoo(int) RETURNS SETOF foo AS $$
    SELECT * FROM foo WHERE fooid = $1;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CREATE TABLE tab (y int, z int);
;;
-- xfunc.sgml
SELECT x, generate_series(1,5) AS g FROM tab;
;;
-- xfunc.sgml
SELECT x, g FROM tab, LATERAL generate_series(1,5) AS g;
;;
-- xfunc.sgml
SELECT srf1(srf2(x), srf3(y)), srf4(srf5(z)) FROM tab;
;;
-- xfunc.sgml
SELECT x, CASE WHEN x > 0 THEN generate_series(1, 5) ELSE 0 END FROM tab;
;;
-- xfunc.sgml
SELECT x, CASE WHEN y > 0 THEN generate_series(1, z) ELSE 5 END FROM tab;
;;
-- xfunc.sgml
CREATE FUNCTION case_generate_series(cond bool, start int, fin int, els int)
  RETURNS SETOF int AS $$
BEGIN
  IF cond THEN
    RETURN QUERY SELECT generate_series(start, fin);
  ELSE
    RETURN QUERY SELECT els;
  END IF;
END$$ LANGUAGE plpgsql;
;;
-- xfunc.sgml
CREATE FUNCTION sum_n_product_with_tab (x int)
RETURNS TABLE(sum int, product int) AS $$
    SELECT $1 + tab.y, $1 * tab.y FROM tab;
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
SELECT anyleast('abc'::text, 'ABC');
;;
-- xfunc.sgml
SELECT anyleast('abc'::text, 'ABC' COLLATE "C");
;;
-- xfunc.sgml
CREATE FUNCTION anyleast (VARIADIC anyarray) RETURNS anyelement AS $$
    SELECT min($1[i] COLLATE "en_US") FROM generate_subscripts($1, 1) g(i);
$$ LANGUAGE SQL;
;;
-- xfunc.sgml
CREATE FUNCTION square_root(double precision) RETURNS double precision
    AS 'dsqrt'
    LANGUAGE internal
    STRICT;
;;
-- xfunc.sgml
SELECT name, c_overpaid(emp, 1500) AS overpaid
    FROM emp
    WHERE name = 'Bill' OR name = 'Sam';
;;
-- xindex.sgml
SELECT * FROM mytable ORDER BY somecol USING ~<~;
;;
-- xindex.sgml
SELECT sum(x) OVER (ORDER BY x RANGE BETWEEN 5 PRECEDING AND 10 FOLLOWING)
  FROM mytable;
;;
-- xindex.sgml
SELECT * FROM table WHERE integer_column < 4;
;;
-- xml2.sgml
SELECT * FROM
xpath_table('article_id',
            'article_xml',
            'articles',
            '/article/author|/article/pages|/article/title',
            'date_entered > ''2003-01-01'' ')
AS t(article_id integer, author text, page_count integer, title text);
;;
-- xml2.sgml
SELECT t.title, p.fullname, p.email
FROM xpath_table('article_id', 'article_xml', 'articles',
                 '/article/title|/article/author/@id',
                 'xpath_string(article_xml,''/article/@date'') > ''2003-03-20'' ')
       AS t(article_id integer, title text, author_id integer),
     tblPeopleInfo AS p
WHERE t.author_id = p.person_id;
;;
-- xml2.sgml
CREATE TABLE test (
    id int PRIMARY KEY,
    xml text
);
;;
-- xml2.sgml
SELECT t.*,i.doc_num FROM
  xpath_table('id', 'xml', 'test',
              '/doc/line/@num|/doc/line/a|/doc/line/b|/doc/line/c',
              'true')
    AS t(id int, line_num varchar(10), val1 int, val2 int, val3 int),
  xpath_table('id', 'xml', 'test', '/doc/@num', 'true')
    AS i(id int, doc_num varchar(10))
WHERE i.id=t.id AND i.id=1
ORDER BY doc_num, line_num;
;;
-- xplang.sgml
CREATE FUNCTION plperl_call_handler() RETURNS language_handler AS
    '$libdir/plperl' LANGUAGE C;
;;
-- xplang.sgml
CREATE FUNCTION plperl_inline_handler(internal) RETURNS void AS
    '$libdir/plperl' LANGUAGE C STRICT;
;;
-- xplang.sgml
CREATE TRUSTED LANGUAGE plperl
    HANDLER plperl_call_handler
    INLINE plperl_inline_handler
    VALIDATOR plperl_validator;
;;
-- xtypes.sgml
CREATE TYPE complex;
;;
-- xtypes.sgml
CREATE TYPE complex (
   internallength = 16,
   input = complex_in,
   output = complex_out,
   receive = complex_recv,
   send = complex_send,
   alignment = double
);
