CREATE FOREIGN TABLE foo (bar INTEGER OPTIONS (column_name 'Bar') NOT NULL, baz TEXT OPTIONS (column_name 'Baz')) SERVER srv OPTIONS (table_name 'dbo.Foo')
