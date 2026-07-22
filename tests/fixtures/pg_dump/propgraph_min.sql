CREATE PROPERTY GRAPH app.graph_min
    VERTEX TABLES (
        app.users KEY (id) PROPERTIES (active, country, created_at, email, id)
    );
