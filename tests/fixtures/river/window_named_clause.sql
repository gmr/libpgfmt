SELECT id, row_number() OVER w FROM events WINDOW w AS (PARTITION BY id ORDER BY id);
